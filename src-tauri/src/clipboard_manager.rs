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
use std::process::Child;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

// --- Constants ---

pub const DEFAULT_MAX_HISTORY_SIZE: usize = 50;
const PREVIEW_TEXT_MAX_LEN: usize = 100;
const GIF_CACHE_MARKER: &str = "win11-clipboard-history/gifs/";
const FILE_URI_PREFIX: &str = "file://";
const WL_COPY_SETTLE_TIME: u64 = 150;

// arboard retry & log-rate-limit intervals (tunable without touching logic).
const ARBOARD_RETRY_INTERVAL: Duration = Duration::from_secs(120);
const LOG_INTERVAL: Duration = Duration::from_secs(30);

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

/// Initialise and cache a long-lived arboard Clipboard connection.
///
/// arboard docs: "If your application is longer-running (ie a GUI, TUI, etc),
/// it is highly recommended that you either store the Clipboard object in some
/// long-lived data structure (like app context, etc)"
///
/// arboard docs on Wayland:
/// "Wayland support relies on the wlr-data-control protocol extension(s),
/// which are not supported by all Wayland compositors."
///
/// When initialisation fails on Wayland we skip arboard for the entire
/// session and fall back to wl-copy/wl-paste, avoiding repeated connection
/// attempts every poll cycle.
fn init_clipboard() -> Option<Clipboard> {
    match Clipboard::new() {
        Ok(cb) => {
            println!("[ClipboardManager] Initialised long-lived clipboard connection");
            Some(cb)
        }
        Err(e) => {
            eprintln!(
                "[ClipboardManager] Failed to create arboard clipboard: {}\n\
                 This is expected on Wayland compositors that do not support\n\
                 the wlr-data-control protocol. Falling back to external\n\
                 clipboard tools (wl-copy/wl-paste/xclip).",
                e
            );
            None
        }
    }
}

// arboard docs on paste semantics:
// "Nothing you copy to the clipboard is sent anywhere to start.
//  It stays inside arboard until something else requests it."
//
// This means no data leaves our process during set_text/set_image/etc
// until another application actively reads from the clipboard.

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
    clipboard_server: Mutex<Option<Child>>,
    /// When we last printed an error message (for log rate-limiting).
    last_error_log: Option<Instant>,
    /// When we last retried arboard initialisation (for transient failures).
    last_clipboard_retry: Option<Instant>,
    /// Long-lived arboard Clipboard connection (per arboard docs).
    ///
    /// `None` means arboard could not be initialised (e.g. Wayland compositor
    /// lacks wlr-data-control). We skip arboard entirely and use external
    /// tools (wl-copy/wl-paste/xclip) instead.
    clipboard: Option<Clipboard>,
}

impl ClipboardManager {
    /// Kill the tracked clipboard child process (if any) and reap it on a
    /// background thread so the caller never blocks waiting for the zombie.
    fn kill_and_reap_child(&self) {
        // Recover from a poisoned mutex so we still clean up the child;
        // a panic elsewhere shouldn't leave a zombie behind.
        let mut guard = match self.clipboard_server.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
    }
}

impl Drop for ClipboardManager {
    fn drop(&mut self) {
        self.kill_and_reap_child();
    }
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
            clipboard_server: Mutex::new(None),
            last_error_log: None,
            last_clipboard_retry: None,
            clipboard: init_clipboard(),
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

    pub fn save_history(&self) {
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
        // Retry arboard init periodically — the compositor may have gained
        // wlr-data-control support since startup (e.g. compositor switch).
        self.retry_clipboard_init();

        if let Some(ref mut clipboard) = self.clipboard {
            match clipboard.get_text() {
                Ok(text) => {
                    self.last_error_log = None;
                    return Ok(text);
                }
                Err(e) => {
                    self.log_error_rate_limited(&format!("arboard get_text failed: {}", e));
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            if crate::session::is_wayland() {
                // Always try wl-paste — transient failures should not
                // suppress subsequent attempts.
                if let Some(text) = self.get_text_via_wl_paste() {
                    self.last_error_log = None;
                    return Ok(text);
                }

                self.log_error_rate_limited(
                    "wl-paste could not read clipboard (clipboard may be empty)",
                );
            }
        }

        Err(arboard::Error::ContentNotAvailable)
    }

    /// Periodically retry arboard initialisation when it was `None` at
    /// startup (e.g. Wayland compositor without wlr-data-control).
    /// Only logs on state change (unavailable → available), not on repeated
    /// failures — the fallback (wl-copy/wl-paste) works fine in the meantime.
    fn retry_clipboard_init(&mut self) {
        if self.clipboard.is_some() {
            return;
        }
        if self
            .last_clipboard_retry
            .is_some_and(|t| t.elapsed() < ARBOARD_RETRY_INTERVAL)
        {
            return;
        }
        self.last_clipboard_retry = Some(Instant::now());

        if let Ok(cb) = Clipboard::new() {
            println!("[ClipboardManager] Acquired clipboard connection (was unavailable)");
            self.clipboard = Some(cb);
        }
    }

    /// Print an error message at most once per `LOG_INTERVAL` to avoid
    /// log spam from the polling watcher when clipboard reads fail.
    fn log_error_rate_limited(&mut self, msg: &str) {
        let now = Instant::now();
        let should_log = self
            .last_error_log
            .map_or(true, |t| now - t >= LOG_INTERVAL);
        if should_log {
            self.last_error_log = Some(now);
            eprintln!("[ClipboardManager] {}", msg);
        }
    }

    #[cfg(target_os = "linux")]
    fn get_text_via_wl_paste(&self) -> Option<String> {
        use std::process::{Command, Stdio};
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;

        // Circuit breaker: only one wl-paste background invocation at a time,
        // preventing unbounded thread accumulation if wl-paste hangs repeatedly.
        static WL_PASTE_RUNNING: AtomicBool = AtomicBool::new(false);
        if WL_PASTE_RUNNING
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return None;
        }

        // Spawn wl-paste with piped stdio so we hold a Child handle and can
        // kill the process on timeout (unlike .output() which blocks forever).
        let child = match Command::new("wl-paste")
            .args(["--no-newline"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => {
                // Release the circuit breaker since we won't spawn a thread.
                WL_PASTE_RUNNING.store(false, Ordering::Release);
                return None;
            }
        };

        let child_id = child.id();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let result = child.wait_with_output();
            // Reset the circuit breaker so the next call can proceed.
            WL_PASTE_RUNNING.store(false, Ordering::Release);
            // Ignore send errors (e.g. if receiver was dropped due to timeout).
            let _ = tx.send(result);
        });

        // Reasonable upper bound to avoid hanging the caller; adjust if needed.
        let timeout = Duration::from_millis(500);

        let output = match rx.recv_timeout(timeout) {
            Ok(Ok(output)) => output,
            Ok(Err(_)) | Err(mpsc::RecvTimeoutError::Disconnected) => return None,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Kill the hung wl-paste process so it does not linger. After
                // this the spawned thread's wait_with_output() will return and
                // automatically reap the child, releasing the circuit breaker.
                let _ = Command::new("kill")
                    .arg("-9")
                    .arg(child_id.to_string())
                    .output();
                return None;
            }
        };

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }

        None
    }

    /// Try to get HTML content from clipboard. Returns None if not available.
    ///
    /// Calls `retry_clipboard_init` so HTML reads benefit from the same
    /// periodic arboard retry logic as `get_current_text`.
    pub fn get_current_html(&mut self) -> Option<String> {
        self.retry_clipboard_init();
        self.clipboard.as_mut()?.get().html().ok()
    }

    pub fn get_current_image(
        &mut self,
    ) -> Result<Option<(ImageData<'static>, u64)>, arboard::Error> {
        let clipboard = match self.clipboard.as_mut() {
            Some(cb) => cb,
            None => return Ok(None),
        };

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
            if pasted == text {
                self.last_pasted_text = None;
                return true;
            }
            // Clipboard has changed to something else; the paste echo window has passed.
            self.last_pasted_text = None;
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
        self.history.len() != before
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
        // Find the item and toggle its pin status
        let pos = self.history.iter().position(|i| i.id == id)?;
        self.history[pos].pinned = !self.history[pos].pinned;

        // Reposition the item so the invariant
        let item = self.history.remove(pos);
        let insert_pos = self
            .history
            .iter()
            .position(|i| !i.pinned)
            .unwrap_or(self.history.len());
        self.history.insert(insert_pos, item);

        let item_clone = self.history[insert_pos].clone();
        self.save_history();
        Some(item_clone)
    }

    /// Move an item to the top of the history (respecting pinned items)
    /// If the item is pinned, it moves to the top of pinned items
    /// If not pinned, it moves to the first non-pinned position
    pub fn move_item_to_top(&mut self, id: &str) -> bool {
        // Find the item's current position
        let current_pos = match self.history.iter().position(|i| i.id == id) {
            Some(pos) => pos,
            None => return false, // Item not found
        };
        // Determine where we *would* insert based on pinned status, without mutating yet
        let item_pinned = self.history[current_pos].pinned;
        let insert_pos = if item_pinned {
            // Move to top of pinned items (position 0)
            0
        } else {
            // Move to first non-pinned position (right after all pinned items)
            self.history
                .iter()
                .position(|i| !i.pinned)
                .unwrap_or(self.history.len())
        };
        // If the item is already at the correct position, avoid unnecessary mutation and I/O
        if insert_pos == current_pos {
            return true;
        }
        // Now actually move the item
        let item = self.history.remove(current_pos);
        self.history.insert(insert_pos, item);
        self.save_history();
        true
    }

    pub fn cleanup_old_items(&mut self, interval_minutes: u64) -> bool {
        if interval_minutes == 0 {
            return false;
        }

        let now = Utc::now();
        let mut changed = false;

        // Use a more robust time comparison
        self.history.retain(|item| {
            if item.pinned {
                return true;
            }

            let age_seconds = now.signed_duration_since(item.timestamp).num_seconds();
            let interval_seconds = (interval_minutes * 60) as i64;
            let keep = age_seconds < interval_seconds;

            if !keep {
                changed = true;
                println!(
                    "[ClipboardManager] Auto-deleting old item: {} (age: {}s, limit: {}s)",
                    item.id, age_seconds, interval_seconds
                );
            }
            keep
        });

        if changed {
            self.save_history();
        }

        changed
    }

    // --- Paste Logic ---

    pub fn mark_as_pasted(&mut self, item: &ClipboardItem) {
        match &item.content {
            ClipboardContent::Text(text) => {
                self.last_pasted_text = Some(text.clone());
                self.last_pasted_image_hash = None;
            }
            ClipboardContent::RichText { plain, html: _ } => {
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
        match &item.content {
            ClipboardContent::Text(text) => {
                self.set_text_robust(text)?;
            }
            ClipboardContent::RichText { plain, html } => {
                // Set HTML with plain text as fallback - this preserves formatting
                self.set_html_robust(html, plain)?;
            }
            ClipboardContent::Image { base64, .. } => {
                self.set_image_robust(base64)?;
            }
        }

        // 3. Simulate User Input
        self.simulate_paste_action()?;

        // 4. Move item to top of history so it's easily accessible for repeated use
        self.move_item_to_top(&item.id);

        Ok(())
    }

    fn simulate_paste_action(&self) -> Result<(), String> {
        // Wait for clipboard write to settle
        thread::sleep(Duration::from_millis(60));

        // Trigger keystroke
        crate::input_simulator::simulate_paste_keystroke()?;

        // before the clipboard ownership changes or the app reads it.
        thread::sleep(Duration::from_millis(250));

        Ok(())
    }

    /// Robustly set text to clipboard using xclip/wl-copy on Linux if available,
    /// falling back to arboard. This fixes issues on distros like Kali Linux.
    pub fn set_text_robust(&mut self, text: &str) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            if crate::session::is_wayland() {
                if let Ok(()) = self.set_clipboard_external(
                    "wl-copy",
                    &["--type", "text/plain;charset=utf-8"],
                    text,
                ) {
                    return Ok(());
                }
            } else if let Ok(()) = self.set_clipboard_external(
                "xclip",
                &[
                    "-selection",
                    "clipboard",
                    "-t",
                    "UTF8_STRING",
                    "-loops",
                    "0",
                ],
                text,
            ) {
                return Ok(());
            }
        }

        // Fallback to arboard
        if let Some(ref mut clipboard) = self.clipboard {
            clipboard.set_text(text).map_err(|e| e.to_string())
        } else {
            Err("Clipboard not available (no arboard connection; external tools may have also failed)".to_string())
        }
    }

    /// Robustly set HTML to clipboard using xclip/wl-copy on Linux if available,
    /// falling back to arboard.
    pub fn set_html_robust(&mut self, html: &str, plain: &str) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            if crate::session::is_wayland() {
                if let Ok(()) =
                    self.set_clipboard_external("wl-copy", &["--type", "text/html"], html)
                {
                    let _ = self.set_clipboard_external_no_kill(
                        "wl-copy",
                        &["--type", "text/plain;charset=utf-8"],
                        plain,
                    );
                    return Ok(());
                }
            } else if let Ok(()) = self.set_clipboard_external(
                "xclip",
                &["-selection", "clipboard", "-t", "text/html", "-loops", "0"],
                html,
            ) {
                let _ = self.set_clipboard_external_no_kill(
                    "xclip",
                    &["-selection", "clipboard", "-t", "UTF8_STRING"],
                    plain,
                );
                return Ok(());
            }
        }

        // Fallback to arboard (which handles multiple MIME types correctly)
        if let Some(ref mut clipboard) = self.clipboard {
            clipboard
                .set_html(html, Some(plain))
                .map_err(|e| e.to_string())
        } else {
            Err("Clipboard not available (no arboard connection; external tools may have also failed)".to_string())
        }
    }

    fn set_clipboard_external(&self, cmd: &str, args: &[&str], data: &str) -> Result<(), String> {
        self.set_clipboard_external_impl(cmd, args, data.as_bytes(), true)
    }

    fn set_clipboard_external_no_kill(
        &self,
        cmd: &str,
        args: &[&str],
        data: &str,
    ) -> Result<(), String> {
        self.set_clipboard_external_impl(cmd, args, data.as_bytes(), false)
    }

    fn set_clipboard_external_bytes(
        &self,
        cmd: &str,
        args: &[&str],
        data: &[u8],
    ) -> Result<(), String> {
        self.set_clipboard_external_impl(cmd, args, data, true)
    }

    #[allow(dead_code)]
    fn set_clipboard_external_no_kill_bytes(
        &self,
        cmd: &str,
        args: &[&str],
        data: &[u8],
    ) -> Result<(), String> {
        self.set_clipboard_external_impl(cmd, args, data, false)
    }

    fn set_clipboard_external_impl(
        &self,
        cmd: &str,
        args: &[&str],
        data: &[u8],
        kill_previous: bool,
    ) -> Result<(), String> {
        use std::io::{Read, Write};
        use std::process::{Command, Stdio};

        if kill_previous {
            self.kill_and_reap_child();
        }

        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", cmd, e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(data)
                .map_err(|e| format!("Pipe write error: {}", e))?;
        }

        thread::sleep(Duration::from_millis(WL_COPY_SETTLE_TIME));

        match child.try_wait() {
            Ok(Some(status)) if !status.success() => {
                let mut stderr = String::new();
                if let Some(mut stderr_pipe) = child.stderr.take() {
                    let _ = stderr_pipe.read_to_string(&mut stderr);
                }
                Err(format!(
                    "{} exited with status {}. Stderr: {}",
                    cmd,
                    status,
                    stderr.trim()
                ))
            }
            Ok(_) => {
                // Only track this child when replacing the previous server.
                // The no-kill path (plain-text fallback in set_html_robust) leaves
                // the HTML server tracked so it can be cleaned up later.
                if kill_previous {
                    let mut guard = self
                        .clipboard_server
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    *guard = Some(child);
                } else {
                    // Detach: reap on background thread so the no-kill child
                    // doesn't become a zombie when it eventually exits.
                    std::thread::spawn(move || {
                        let _ = child.wait();
                    });
                }
                Ok(())
            }
            Err(e) => Err(format!("Process status check failed: {}", e)),
        }
    }

    /// Robustly set image to clipboard using xclip/wl-copy on Linux if available,
    /// falling back to arboard. Fixes X11 clipboard ownership loss (issue #259).
    fn set_image_robust(&mut self, base64_str: &str) -> Result<(), String> {
        let bytes = BASE64
            .decode(base64_str)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;

        #[cfg(target_os = "linux")]
        {
            if crate::session::is_wayland() {
                if let Ok(()) =
                    self.set_clipboard_external_bytes("wl-copy", &["--type", "image/png"], &bytes)
                {
                    return Ok(());
                }
            } else if let Ok(()) = self.set_clipboard_external_bytes(
                "xclip",
                &["-selection", "clipboard", "-t", "image/png", "-loops", "0"],
                &bytes,
            ) {
                return Ok(());
            }
        }

        // Fallback to arboard
        if let Some(ref mut clipboard) = self.clipboard {
            let img =
                image::load_from_memory(&bytes).map_err(|e| format!("Image load failed: {}", e))?;
            let rgba = img.to_rgba8();
            let image_data = ImageData {
                width: rgba.width() as usize,
                height: rgba.height() as usize,
                bytes: rgba.into_raw().into(),
            };
            clipboard.set_image(image_data).map_err(|e| e.to_string())
        } else {
            Err("Clipboard not available (no arboard connection; external tools may have also failed)".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;
    use std::time::Instant;

    static INIT: Once = Once::new();

    fn init_test_env() {
        INIT.call_once(|| {
            crate::session::init();
        });
    }

    fn try_get_clipboard() -> Option<Clipboard> {
        Clipboard::new().ok()
    }

    /// Poll the clipboard text with a timeout, sleeping `interval` between
    /// attempts.  Returns the first successful read or an error if the deadline
    /// expires.  Extracted as a helper to avoid duplicating the polling pattern
    /// across multiple integration tests.
    fn poll_clipboard_text(
        clipboard: &mut Clipboard,
        timeout: Duration,
        interval: Duration,
    ) -> Result<String, arboard::Error> {
        let deadline = Instant::now() + timeout;
        loop {
            match clipboard.get_text() {
                Ok(text) => return Ok(text),
                Err(e) if Instant::now() >= deadline => return Err(e),
                _ => std::thread::sleep(interval),
            }
        }
    }

    fn make_manager(path: &str, max_size: usize) -> ClipboardManager {
        let unique_name = format!("clip-hist-test-{}_{}", path, Uuid::new_v4());
        let p = std::env::temp_dir().join(unique_name);
        let _ = std::fs::remove_file(&p);
        ClipboardManager::new(p, max_size)
    }

    fn make_text_item(text: &str) -> ClipboardItem {
        ClipboardItem::new_text(text.to_string())
    }

    // ── Pure logic tests (no display server, no sleeps, independent temp dirs) ──

    #[test]
    fn should_skip_empty_text() {
        let mut m = make_manager("skip_empty", 10);
        assert!(m.should_skip_text(""));
        assert!(m.should_skip_text("   "));
        assert!(!m.should_skip_text("hi"));
    }

    #[test]
    fn should_skip_self_pasted() {
        let mut m = make_manager("skip_self", 10);
        m.mark_text_as_pasted("pasted");
        assert!(m.should_skip_text("pasted"));
        assert!(!m.should_skip_text("pasted"));
    }

    #[test]
    fn is_duplicate_detects_top() {
        let mut m = make_manager("dup_top", 10);
        m.insert_item(make_text_item("hello"));
        assert!(m.is_duplicate_text("hello"));
        assert!(!m.is_duplicate_text("world"));
    }

    #[test]
    fn is_duplicate_ignores_pinned() {
        let mut m = make_manager("dup_pin", 10);
        m.insert_item(make_text_item("keep"));
        let id = m.history[0].id.clone();
        m.toggle_pin(&id);
        m.insert_item(make_text_item("hello"));
        assert!(!m.is_duplicate_text("keep"));
        assert!(m.is_duplicate_text("hello"));
    }

    #[test]
    fn remove_duplicate_text_works() {
        let mut m = make_manager("rm_dup", 10);
        m.insert_item(make_text_item("a"));
        m.insert_item(make_text_item("b"));
        m.insert_item(make_text_item("c"));
        m.remove_duplicate_text_from_history("b");
        assert_eq!(m.history.len(), 2);
    }

    #[test]
    fn enforce_limit_trims() {
        let mut m = make_manager("limit", 3);
        for i in 0..5 {
            m.insert_item(make_text_item(&format!("item{}", i)));
        }
        assert_eq!(m.history.len(), 3);
    }

    #[test]
    fn enforce_limit_preserves_pinned() {
        let mut m = make_manager("limit_pin", 10);
        m.insert_item(make_text_item("a"));
        m.insert_item(make_text_item("b"));
        m.insert_item(make_text_item("c"));
        let b = m.history[1].id.clone();
        let a = m.history[2].id.clone();
        m.toggle_pin(&b);
        m.toggle_pin(&a);
        m.set_max_history_size(2);
        assert_eq!(m.history.len(), 2);
        assert!(m.history.iter().all(|i| i.pinned));
    }

    #[test]
    fn add_text_inserts() {
        let mut m = make_manager("add", 10);
        assert!(m.add_text("new".into(), None).is_some());
        assert_eq!(m.history.len(), 1);
    }

    #[test]
    fn add_text_skips_top_duplicate() {
        let mut m = make_manager("add_dup", 10);
        assert!(m.add_text("same".into(), None).is_some());
        assert!(m.add_text("same".into(), None).is_none());
    }

    #[test]
    fn add_text_moves_duplicate_to_top() {
        let mut m = make_manager("add_move", 10);
        m.add_text("first".into(), None);
        m.add_text("second".into(), None);
        m.add_text("first".into(), None);
        assert_eq!(m.history.len(), 2);
    }

    #[test]
    fn add_text_with_html() {
        let mut m = make_manager("add_html", 10);
        m.add_text("plain".into(), Some("<b>bold</b>".into()));
        match &m.history[0].content {
            ClipboardContent::RichText { plain, html } => {
                assert_eq!(plain, "plain");
                assert_eq!(html, "<b>bold</b>");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn mark_as_pasted_skips() {
        let mut m = make_manager("mark", 10);
        m.mark_text_as_pasted("emoji");
        assert!(m.should_skip_text("emoji"));
    }

    #[test]
    fn clear_keeps_pinned() {
        let mut m = make_manager("clear", 10);
        m.insert_item(make_text_item("a"));
        m.insert_item(make_text_item("b"));
        m.insert_item(make_text_item("c"));
        let b = m.history[1].id.clone();
        m.toggle_pin(&b);
        m.clear();
        assert_eq!(m.history.len(), 1);
        assert!(m.history[0].pinned);
    }

    #[test]
    fn toggle_pin_flips() {
        let mut m = make_manager("pin", 10);
        m.insert_item(make_text_item("pin"));
        let id = m.history[0].id.clone();
        assert!(!m.history[0].pinned);
        assert!(m.toggle_pin(&id).unwrap().pinned);
        assert!(!m.toggle_pin(&id).unwrap().pinned);
    }

    // ── Integration test: clipboard persistence (display server required) ──

    /// Verifies clipboard data set via set_text_robust can be read back,
    /// using polling with timeout instead of fixed sleep to avoid flakiness.
    #[test]
    fn set_text_robust_persists_data() {
        init_test_env();

        let mut clipboard = match try_get_clipboard() {
            Some(c) => c,
            None => {
                eprintln!("skipping — no display server");
                return;
            }
        };

        let mut m = make_manager("persist", 10);
        let test_text = "persistence-test-42";

        // Set text via robust path (which uses external tools on Linux)
        m.set_text_robust(test_text)
            .expect("set_text_robust should succeed");

        // Poll with timeout instead of fixed sleep
        let read_back = poll_clipboard_text(
            &mut clipboard,
            Duration::from_millis(2000),
            Duration::from_millis(50),
        )
        .expect("clipboard should be readable within timeout");

        assert_eq!(
            read_back, test_text,
            "Clipboard data should persist after set_text_robust"
        );
    }

    /// Test that set_html_robust persists HTML content.
    #[test]
    fn test_html_robust_persists_data() {
        init_test_env();

        let mut clipboard = match try_get_clipboard() {
            Some(c) => c,
            None => {
                eprintln!("test_html_robust_persists_data: skipping — no display server available");
                return;
            }
        };

        let mut m = make_manager("html_persist", 10);
        let html = "<b>bold text</b>";
        let plain = "bold text";

        // Set HTML via robust path
        m.set_html_robust(html, plain)
            .expect("set_html_robust should succeed");

        // Poll with timeout instead of fixed sleep
        let read_back = poll_clipboard_text(
            &mut clipboard,
            Duration::from_millis(2000),
            Duration::from_millis(50),
        )
        .unwrap_or_default();

        assert!(
            read_back.contains("bold"),
            "Plain text fallback should persist after set_html_robust. Got: '{}'",
            read_back
        );
    }

    /// Test that repeated set_text_robust calls work correctly
    /// (verifies no regression from adding -loops 0 to xclip)
    #[test]
    fn test_repeated_set_text_robust() {
        init_test_env();

        let mut clipboard = match try_get_clipboard() {
            Some(c) => c,
            None => {
                eprintln!("test_repeated_set_text_robust: skipping — no display server available");
                return;
            }
        };

        let mut m = make_manager("repeat", 10);

        for i in 0..5 {
            let text = format!("repeat-test-{}", i);
            m.set_text_robust(&text).unwrap_or_else(|e| {
                panic!("set_text_robust iteration {} should succeed: {}", i, e)
            });

            // Poll with timeout instead of fixed sleep
            let read_back = poll_clipboard_text(
                &mut clipboard,
                Duration::from_millis(2000),
                Duration::from_millis(50),
            )
            .expect("clipboard should be readable within timeout");

            assert_eq!(
                read_back, text,
                "Iteration {}: clipboard should contain '{}'",
                i, text
            );
        }
    }

    // ── Integration tests (display server required) ──

    #[test]
    fn process_not_accumulated() {
        init_test_env();
        if try_get_clipboard().is_none() {
            eprintln!("skipping — no display");
            return;
        }
        // Bail early with a clear message if pgrep isn't on PATH.
        if std::process::Command::new("pgrep")
            .arg("--version")
            .output()
            .is_err()
        {
            eprintln!("skipping — pgrep not available");
            return;
        }
        let mut m = make_manager("proc", 10);
        for i in 0..5 {
            m.set_text_robust(&format!("p{}", i)).unwrap();
            std::thread::sleep(Duration::from_millis(100));
        }
        let count = || -> usize {
            let run = |name: &str| -> usize {
                std::process::Command::new("pgrep")
                    .args(["-c", "-x", name])
                    .output()
                    .map(|o| {
                        String::from_utf8_lossy(&o.stdout)
                            .trim()
                            .parse()
                            .unwrap_or(0)
                    })
                    .unwrap_or(0)
            };
            run("xclip") + run("wl-copy")
        };
        assert!(count() <= 1, "orphaned clipboard servers: {}", count());
    }

    #[test]
    fn construct_and_drop_ok() {
        let m = make_manager("drop", 10);
        assert_eq!(m.get_max_history_size(), 10);
        drop(m);
    }
}
