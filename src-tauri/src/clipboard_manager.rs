//! Clipboard Manager Module
//! Handles clipboard monitoring, history storage, and paste injection

use arboard::{Clipboard, ImageData};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use image::{DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

// --- Constants ---

const MAX_HISTORY_SIZE: usize = 50;
const PREVIEW_TEXT_MAX_LEN: usize = 100;
const GIF_CACHE_MARKER: &str = "win11-clipboard-history/gifs/";
const FILE_URI_PREFIX: &str = "file://";

// --- Helper Functions ---

/// Calculates a stable hash for any hashable data.
fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// Helper to get a fresh clipboard instance.
fn get_system_clipboard() -> Result<Clipboard, String> {
    Clipboard::new().map_err(|e| e.to_string())
}

// --- Data Structures ---

/// Content type for clipboard items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum ClipboardContent {
    /// Plain text content
    Text(String),
    /// Image as base64 encoded PNG
    Image {
        base64: String,
        width: u32,
        height: u32,
    },
}

/// A single clipboard history item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    /// Unique identifier
    pub id: String,
    /// The content
    pub content: ClipboardContent,
    /// When it was copied
    pub timestamp: DateTime<Utc>,
    /// Whether this item is pinned
    pub pinned: bool,
    /// Preview text (for display)
    pub preview: String,
}

impl ClipboardItem {
    pub fn new_text(text: String) -> Self {
        let preview = if text.chars().count() > PREVIEW_TEXT_MAX_LEN {
            format!(
                "{}...",
                &text.chars().take(PREVIEW_TEXT_MAX_LEN).collect::<String>()
            )
        } else {
            text.clone()
        };

        Self::create(ClipboardContent::Text(text), preview)
    }

    pub fn new_image(base64: String, width: u32, height: u32, hash: u64) -> Self {
        // We store the hash in the preview string to persist it across sessions
        // without breaking the serialization schema of existing data.
        let preview = format!("Image ({}x{}) #{}", width, height, hash);

        Self::create(
            ClipboardContent::Image {
                base64,
                width,
                height,
            },
            preview,
        )
    }

    fn create(content: ClipboardContent, preview: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            timestamp: Utc::now(),
            pinned: false,
            preview,
        }
    }

    /// Attempts to extract the image hash from the preview string.
    /// Returns None if content is not an image or hash is missing.
    pub fn extract_image_hash(&self) -> Option<u64> {
        if !matches!(self.content, ClipboardContent::Image { .. }) {
            return None;
        }
        self.preview
            .split('#')
            .nth(1)
            .and_then(|h| h.parse::<u64>().ok())
    }
}

// --- Manager Logic ---

/// Manages clipboard operations and history
pub struct ClipboardManager {
    history: Vec<ClipboardItem>,
    /// Track the last pasted content to avoid re-adding it to history
    last_pasted_text: Option<String>,
    last_pasted_image_hash: Option<u64>,
    /// Track last added text hash to prevent duplicates from rapid copies
    last_added_text_hash: Option<u64>,
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            history: Vec::with_capacity(MAX_HISTORY_SIZE),
            last_pasted_text: None,
            last_pasted_image_hash: None,
            last_added_text_hash: None,
        }
    }

    // --- Monitoring / Reading ---

    pub fn get_current_text(&mut self) -> Result<String, arboard::Error> {
        // We unwrap internal map error because arboard::Error is the expected return type here
        // for the monitoring loop in main.rs
        Clipboard::new()?.get_text()
    }

    pub fn get_current_image(
        &mut self,
    ) -> Result<Option<(ImageData<'static>, u64)>, arboard::Error> {
        let mut clipboard = Clipboard::new()?;

        match clipboard.get_image() {
            Ok(image) => {
                let hash = calculate_hash(&image.bytes);
                let owned = ImageData {
                    width: image.width,
                    height: image.height,
                    bytes: image.bytes.into_owned().into(),
                };
                Ok(Some((owned, hash)))
            }
            Err(arboard::Error::ContentNotAvailable) => Ok(None),
            Err(e) => Err(e),
        }
    }

    // --- Adding Items ---

    pub fn add_text(&mut self, text: String) -> Option<ClipboardItem> {
        if self.should_skip_text(&text) {
            return None;
        }

        let text_hash = calculate_hash(&text);

        // Rapid copy detection
        if Some(text_hash) == self.last_added_text_hash {
            return None;
        }

        // Check if this exact text is already the most recent non-pinned item
        // If so, skip entirely - no need to add or move
        if self.is_duplicate_text(&text) {
            self.last_added_text_hash = Some(text_hash);
            return None;
        }

        // Check if this text exists elsewhere in history (not at top)
        // If so, remove the old entry so we can add fresh at top
        self.remove_duplicate_text_from_history(&text);

        // Create new item and add to history
        let item = ClipboardItem::new_text(text);
        self.insert_item(item.clone());

        self.last_added_text_hash = Some(text_hash);

        Some(item)
    }

    pub fn add_image(&mut self, image_data: ImageData<'_>, hash: u64) -> Option<ClipboardItem> {
        if self.should_skip_image(hash) {
            return None;
        }

        let base64_image = self.convert_image_to_base64(&image_data)?;

        let item = ClipboardItem::new_image(
            base64_image,
            image_data.width as u32,
            image_data.height as u32,
            hash,
        );

        self.insert_item(item.clone());
        Some(item)
    }

    // --- State Management Helpers ---

    fn should_skip_text(&mut self, text: &str) -> bool {
        if text.trim().is_empty() {
            return true;
        }

        // Skip internal GIF cache URIs
        if text.contains(FILE_URI_PREFIX) && text.contains(GIF_CACHE_MARKER) {
            eprintln!("[ClipboardManager] Skipping GIF cache URI");
            return true;
        }

        // Skip self-pasted content
        if let Some(ref pasted) = self.last_pasted_text {
            if pasted == text || text.contains(pasted) {
                // Clear the lock so future copies allow this text
                self.last_pasted_text = None;
                return true;
            }
        }

        false
    }

    fn should_skip_image(&mut self, hash: u64) -> bool {
        // Check if just pasted
        if let Some(pasted_hash) = self.last_pasted_image_hash {
            if pasted_hash == hash {
                self.last_pasted_image_hash = None;
                return true;
            }
        }

        // Check if it's the exact same image as the most recent non-pinned item
        if let Some(item) = self.history.iter().find(|item| !item.pinned) {
            if let Some(item_hash) = item.extract_image_hash() {
                if item_hash == hash {
                    return true;
                }
            }
        }

        false
    }

    fn is_duplicate_text(&self, text: &str) -> bool {
        // Check only the very first non-pinned item for exact match logic
        // used in rapid detection
        if let Some(item) = self.history.iter().find(|item| !item.pinned) {
            if matches!(&item.content, ClipboardContent::Text(t) if t == text) {
                return true;
            }
        }
        false
    }

    fn remove_duplicate_text_from_history(&mut self, text: &str) {
        if let Some(pos) = self.history.iter().position(|item| {
            !item.pinned && matches!(&item.content, ClipboardContent::Text(t) if t == text)
        }) {
            self.history.remove(pos);
        }
    }

    fn convert_image_to_base64(&self, image_data: &ImageData<'_>) -> Option<String> {
        let img = DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(
                image_data.width as u32,
                image_data.height as u32,
                image_data.bytes.to_vec(),
            )?, // Returns None if dimensions don't match bytes
        );

        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageFormat::Png).ok()?;
        Some(BASE64.encode(buffer.get_ref()))
    }

    fn insert_item(&mut self, item: ClipboardItem) {
        // Insert after pinned items (first non-pinned slot)
        let insert_pos = self.history.iter().position(|i| !i.pinned).unwrap_or(0);
        self.history.insert(insert_pos, item);

        // Trim history
        self.enforce_history_limit();
    }

    fn enforce_history_limit(&mut self) {
        while self.history.len() > MAX_HISTORY_SIZE {
            // Remove from the end, skipping pinned items if possible
            if let Some(pos) = self.history.iter().rposition(|i| !i.pinned) {
                self.history.remove(pos);
            } else {
                // All items are pinned. We stop removing.
                break;
            }
        }
    }

    // --- Accessors ---

    pub fn get_history(&self) -> Vec<ClipboardItem> {
        self.history.clone()
    }

    pub fn get_item(&self, id: &str) -> Option<&ClipboardItem> {
        self.history.iter().find(|item| item.id == id)
    }

    pub fn clear(&mut self) {
        self.history.retain(|item| item.pinned);
    }

    pub fn remove_item(&mut self, id: &str) {
        self.history.retain(|item| item.id != id);
    }

    pub fn toggle_pin(&mut self, id: &str) -> Option<ClipboardItem> {
        let item = self.history.iter_mut().find(|i| i.id == id)?;
        item.pinned = !item.pinned;
        Some(item.clone())
    }

    // --- Paste Logic ---

    pub fn mark_as_pasted(&mut self, item: &ClipboardItem) {
        match &item.content {
            ClipboardContent::Text(text) => {
                self.last_pasted_text = Some(text.clone());
                self.last_pasted_image_hash = None;
            }
            ClipboardContent::Image { .. } => {
                if let Some(hash) = item.extract_image_hash() {
                    self.last_pasted_image_hash = Some(hash);
                }
                self.last_pasted_text = None;
            }
        }
    }

    /// Mark a specific text as pasted (to prevent it from appearing in history)
    /// Used for emojis/special insertions
    pub fn mark_text_as_pasted(&mut self, text: &str) {
        self.last_pasted_text = Some(text.to_string());
        self.last_added_text_hash = Some(calculate_hash(&text));
    }

    pub fn paste_item(&mut self, item: &ClipboardItem) -> Result<(), String> {
        // 1. Prevent loop: Mark as pasted before OS action
        self.mark_as_pasted(item);

        // 2. Write content to OS clipboard
        let mut clipboard = get_system_clipboard()?;

        match &item.content {
            ClipboardContent::Text(text) => {
                clipboard.set_text(text).map_err(|e| e.to_string())?;
            }
            ClipboardContent::Image {
                base64,
                width,
                height,
            } => {
                self.write_image_to_clipboard(&mut clipboard, base64, *width, *height)?;
            }
        }

        // 3. Simulate User Input
        self.simulate_paste_action()?;

        Ok(())
    }

    fn write_image_to_clipboard(
        &self,
        clipboard: &mut Clipboard,
        base64_str: &str,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        let bytes = BASE64
            .decode(base64_str)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;
        let img =
            image::load_from_memory(&bytes).map_err(|e| format!("Image load failed: {}", e))?;
        let rgba = img.to_rgba8();

        let image_data = ImageData {
            width: width as usize,
            height: height as usize,
            bytes: rgba.into_raw().into(),
        };

        clipboard.set_image(image_data).map_err(|e| e.to_string())
    }

    fn simulate_paste_action(&self) -> Result<(), String> {
        // Wait for clipboard write to settle
        thread::sleep(Duration::from_millis(60));

        // Trigger keystroke
        crate::input_simulator::simulate_paste_keystroke()?;

        // Linux X11/Wayland often needs a moment to process the paste
        // before the clipboard ownership changes or the app reads it.
        #[cfg(target_os = "linux")]
        thread::sleep(Duration::from_millis(250));

        Ok(())
    }
}
