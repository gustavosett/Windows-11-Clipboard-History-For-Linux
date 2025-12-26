//! Clipboard Manager Module
//! Handles clipboard monitoring, history storage, and paste injection

use arboard::{Clipboard, ImageData};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use image::{DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

// --- Constants ---

pub const DEFAULT_MAX_HISTORY_SIZE: usize = 50;
const PREVIEW_TEXT_MAX_LEN: usize = 100;
const GIF_CACHE_MARKER: &str = "win11-clipboard-history/gifs/";
const FILE_URI_PREFIX: &str = "file://";

// --- Helper Functions ---

// Simple FNV-1a implementation for stable hashing across restarts
// This avoids the randomization of DefaultHasher which causes duplicates on restart
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

struct FnvHasher(u64);

impl Default for FnvHasher {
    fn default() -> Self {
        FnvHasher(FNV_OFFSET_BASIS)
    }
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 ^= byte as u64;
            self.0 = self.0.wrapping_mul(FNV_PRIME);
        }
    }
}

/// Calculates a stable hash for any hashable data.
pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = FnvHasher::default();
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
    /// Rich text with HTML formatting (plain text + optional HTML)
    RichText { plain: String, html: String },
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

    pub fn new_rich_text(plain: String, html: String) -> Self {
        let preview = if plain.chars().count() > PREVIEW_TEXT_MAX_LEN {
            format!(
                "{}...",
                &plain.chars().take(PREVIEW_TEXT_MAX_LEN).collect::<String>()
            )
        } else {
            plain.clone()
        };

        Self::create(ClipboardContent::RichText { plain, html }, preview)
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
    /// Path to save the history file
    persistence_path: PathBuf,
    /// Maximum number of history items to keep
    max_history_size: usize,
}

impl ClipboardManager {
    fn clamp_max_history_size(size: usize) -> usize {
        match size {
            0 => DEFAULT_MAX_HISTORY_SIZE,
            1..=100_000 => size,
            _ => 100_000,
        }
    }

    pub fn new(persistence_path: PathBuf, max_history_size: usize) -> Self {
        // Normalize the requested max size and avoid huge allocations
        let max_size = Self::clamp_max_history_size(max_history_size);
        let mut manager = Self {
            history: Vec::with_capacity(max_size),
            last_pasted_text: None,
            last_pasted_image_hash: None,
            last_added_text_hash: None,
            persistence_path,
            max_history_size: max_size,
        };
        manager.load_history();
        manager
    }

    /// Updates the maximum history size and enforces the new limit
    pub fn set_max_history_size(&mut self, new_size: usize) {
        let mut clamped = Self::clamp_max_history_size(new_size);
        // Do not set max less than number of pinned items; we won't delete pins automatically
        let pinned_count = self.history.iter().filter(|i| i.pinned).count();
        if clamped < pinned_count {
            eprintln!(
                "clipboard_manager: requested max history size ({}) is less than the number of pinned items ({}); increasing limit to preserve pinned items.",
                clamped,
                pinned_count
            );
            clamped = pinned_count;
        }
        self.max_history_size = clamped;
        let trimmed = self.enforce_history_limit();
        if trimmed {
            self.save_history();
        }
    }

    /// Gets the current maximum history size
    pub fn get_max_history_size(&self) -> usize {
        self.max_history_size
    }

    fn load_history(&mut self) {
        if !self.persistence_path.exists() {
            return;
        }

        match fs::read_to_string(&self.persistence_path) {
            Ok(content) => {
                match serde_json::from_str::<Vec<ClipboardItem>>(&content) {
                    Ok(items) => {
                        // Reorder items so pinned come first while preserving order within each group
                        let mut pinned_items = Vec::new();
                        let mut unpinned_items = Vec::new();

                        for item in items {
                            if item.pinned {
                                pinned_items.push(item);
                            } else {
                                unpinned_items.push(item);
                            }
                        }

                        pinned_items.extend(unpinned_items);
                        self.history = pinned_items;
                        // Ensure loaded history respects configured limit immediately
                        let history_trimmed = self.enforce_history_limit();
                        // If the loaded history was trimmed, persist it so disk stays in sync.
                        // Avoid saving when nothing changed.
                        if history_trimmed {
                            self.save_history();
                        }
                        // Initialize last_added_text_hash from the most recent item (even if pinned)
                        // This prevents duplication on startup if the clipboard content matches the top item
                        if let Some(first) = self.history.first() {
                            match &first.content {
                                ClipboardContent::Text(text) => {
                                    self.last_added_text_hash = Some(calculate_hash(text));
                                }
                                ClipboardContent::RichText { plain, .. } => {
                                    self.last_added_text_hash = Some(calculate_hash(plain));
                                }
                                ClipboardContent::Image { .. } => {
                                    if let Some(_hash) = first.extract_image_hash() {
                                        // We don't have a separate last_added_image_hash,
                                        // but we can at least avoid text hash collision
                                        self.last_added_text_hash = None;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to parse history: {}", e),
                }
            }
            Err(e) => eprintln!("Failed to read history file: {}", e),
        }
    }

    fn save_history(&self) {
        match serde_json::to_string_pretty(&self.history) {
            Ok(content) => {
                if let Some(parent) = self.persistence_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Err(e) = fs::write(&self.persistence_path, content) {
                    eprintln!("Failed to save history: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to serialize history: {}", e),
        }
    }

    // --- Monitoring / Reading ---

    pub fn get_current_text(&mut self) -> Result<String, arboard::Error> {
        // We unwrap internal map error because arboard::Error is the expected return type here
        // for the monitoring loop in main.rs
        Clipboard::new()?.get_text()
    }

    /// Try to get HTML content from clipboard. Returns None if not available.
    pub fn get_current_html(&self) -> Option<String> {
        let mut clipboard = get_system_clipboard().ok()?;
        clipboard.get().html().ok()
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

    /// Add text content to history, with optional HTML for rich text
    pub fn add_text(&mut self, text: String, html: Option<String>) -> Option<ClipboardItem> {
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

        // Create new item - use RichText if HTML is available, otherwise plain Text
        let item = match html {
            Some(html_content) if !html_content.trim().is_empty() => {
                ClipboardItem::new_rich_text(text, html_content)
            }
            _ => ClipboardItem::new_text(text),
        };
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
            match &item.content {
                ClipboardContent::Text(t) if t == text => return true,
                ClipboardContent::RichText { plain, .. } if plain == text => return true,
                _ => {}
            }
        }
        false
    }

    fn remove_duplicate_text_from_history(&mut self, text: &str) {
        if let Some(pos) = self.history.iter().position(|item| {
            if item.pinned {
                return false;
            }
            match &item.content {
                ClipboardContent::Text(t) => t == text,
                ClipboardContent::RichText { plain, .. } => plain == text,
                _ => false,
            }
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
        // If all items are pinned, insert at the end to preserve pinned ordering
        let insert_pos = self
            .history
            .iter()
            .position(|i| !i.pinned)
            .unwrap_or(self.history.len());
        self.history.insert(insert_pos, item);

        // Trim history
        self.enforce_history_limit();
        self.save_history();
    }

    /// Enforce the configured history size. Returns true if trimming occurred.
    fn enforce_history_limit(&mut self) -> bool {
        let before = self.history.len();
        while self.history.len() > self.max_history_size {
            // Remove from the end, skipping pinned items if possible
            if let Some(pos) = self.history.iter().rposition(|i| !i.pinned) {
                self.history.remove(pos);
            } else {
                // All items are pinned. We stopped removing to avoid deleting pins.
                break;
            }
        }
        self.history.len() < before
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
        self.save_history();
    }

    pub fn remove_item(&mut self, id: &str) {
        self.history.retain(|item| item.id != id);
        self.save_history();
    }

    pub fn toggle_pin(&mut self, id: &str) -> Option<ClipboardItem> {
        let item = self.history.iter_mut().find(|i| i.id == id)?;
        item.pinned = !item.pinned;
        let item_clone = item.clone();
        self.save_history();
        Some(item_clone)
    }

    // --- Paste Logic ---

    pub fn mark_as_pasted(&mut self, item: &ClipboardItem) {
        match &item.content {
            ClipboardContent::Text(text) => {
                self.last_pasted_text = Some(text.clone());
                self.last_pasted_image_hash = None;
            }
            ClipboardContent::RichText { plain, .. } => {
                self.last_pasted_text = Some(plain.clone());
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
            ClipboardContent::RichText { plain, html } => {
                // Set HTML with plain text as fallback - this preserves formatting
                clipboard
                    .set_html(html, Some(plain))
                    .map_err(|e| e.to_string())?;
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
