//! Clipboard domain types.
//! انواع دامنهٔ کلیپ‌بورد.
//!
//! Split from the original `clipboard_manager.rs` so the data model stays
//! independent from storage, deduplication, and OS clipboard I/O.
//! این فایل از `clipboard_manager.rs` تفکیک شده تا مدل داده مستقل از
//! ذخیره‌سازی، تشخیص تکرار و I/O کلیپ‌بورد سیستم بماند.

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum ClipboardContent {
    Text(String),
    RichText { plain: String, html: String },
    Image {
        base64: String,
        width: u32,
        height: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub content: ClipboardContent,
    pub timestamp: DateTime<Utc>,
    pub pinned: bool,
    pub preview: String,
}

impl ClipboardItem {
    /// New plain-text item with a capped preview.
    /// آیتم متنی جدید با پیش‌نمایش محدودشده.
    pub fn new_text(text: String) -> Self {
        let preview = if text.chars().count() > PREVIEW_TEXT_MAX_LEN {
            format!(
                "{}...",
                text.chars().take(PREVIEW_TEXT_MAX_LEN).collect::<String>()
            )
        } else {
            text.clone()
        };
        Self::create(ClipboardContent::Text(text), preview)
    }

    /// New rich-text (HTML) item with a capped plain preview.
    /// آیتم متن غنی (HTML) جدید با پیش‌نمایش متنی محدودشده.
    pub fn new_rich_text(plain: String, html: String) -> Self {
        let preview = if plain.chars().count() > PREVIEW_TEXT_MAX_LEN {
            format!(
                "{}...",
                plain.chars().take(PREVIEW_TEXT_MAX_LEN).collect::<String>()
            )
        } else {
            plain.clone()
        };
        Self::create(ClipboardContent::RichText { plain, html }, preview)
    }

    /// New image item; the preview embeds the content hash for dedup.
    /// آیتم تصویری جدید؛ پیش‌نمایش، هش محتوا را برای تشخیص تکرار نگه می‌دارد.
    pub fn new_image(base64: String, width: u32, height: u32, hash: u64) -> Self {
        let preview = format!("Image ({}x{}) #{}", width, height, hash);
        Self::create(ClipboardContent::Image { base64, width, height }, preview)
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

    /// Strip HTML and cap text so IPC never carries the full clipboard payload.
    /// حذف HTML و محدودسازی متن تا IPC هرگز بار کامل کلیپ‌بورد را حمل نکند.
    pub fn for_ipc(&self) -> Self {
        const UI_TEXT_MAX: usize = 2048;
        let content = match &self.content {
            ClipboardContent::Text(text) => ClipboardContent::Text(truncate_chars(text, UI_TEXT_MAX)),
            ClipboardContent::RichText { plain, .. } => ClipboardContent::RichText {
                plain: truncate_chars(plain, UI_TEXT_MAX),
                html: String::new(),
            },
            other => other.clone(),
        };
        Self {
            id: self.id.clone(),
            content,
            timestamp: self.timestamp,
            pinned: self.pinned,
            preview: self.preview.clone(),
        }
    }

    /// Recover the image content hash encoded in the preview.
    /// بازیابی هش تصویر که در پیش‌نمایش نگهداری می‌شود.
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
