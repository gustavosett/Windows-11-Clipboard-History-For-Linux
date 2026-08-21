//! Capture pipeline: privacy gating, deduplication, and ordering.
//! خط لولهٔ ثبت: گیت حریم خصوصی، تشخیص تکرار و ترتیب.

use super::*;

impl ClipboardManager {
    /// Ingest a text capture. Returns `None` when the item is a secret,
    /// an echo of our own paste, empty, or a duplicate.
    /// ثبت متن جدید. اگر آیتم «راز»، بازتاب paste خودمان، خالی یا تکراری
    /// باشد `None` برمی‌گرداند.
    pub fn add_text(&mut self, text: String, html: Option<String>) -> Option<ClipboardItem> {
        if self.privacy.filter_secrets && privacy::looks_like_secret(&text) {
            debug!("[ClipboardManager] Skipping secret-looking clipboard text");
            return None;
        }
        if self.should_skip_text(&text) {
            return None;
        }

        let text_hash = calculate_hash(&text);
        if Some(text_hash) == self.last_added_text_hash {
            return None;
        }
        if self.is_duplicate_text(&text) {
            self.last_added_text_hash = Some(text_hash);
            return None;
        }
        self.remove_duplicate_text_from_history(&text);

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

    /// Ingest an image capture. Returns `None` when image capture is
    /// disabled or the image duplicates the newest entry.
    /// ثبت تصویر جدید. اگر ذخیرهٔ تصویر خاموش باشد یا با جدیدترین
    /// آیتم تکراری باشد `None` برمی‌گرداند.
    pub fn add_image(&mut self, image_data: ImageData<'_>, hash: u64) -> Option<ClipboardItem> {
        if !self.privacy.save_images {
            debug!("[ClipboardManager] Image capture disabled by privacy settings");
            return None;
        }
        if self.should_skip_image(hash) {
            return None;
        }

        let id = Uuid::new_v4().to_string();
        let stored = match crate::image_store::store_rgba(
            &self.images_dir,
            &id,
            &image_data,
            &self.crypto,
        ) {
            Ok(s) => s,
            Err(e) => {
                warn!("[ClipboardManager] Failed to store image: {e}");
                return None;
            }
        };

        let mut item = ClipboardItem::new_image(
            stored.thumb_base64,
            stored.width,
            stored.height,
            hash,
        );
        item.id = id.clone();
        self.image_paths.insert(id, stored.full_path);
        self.insert_item(item.clone());
        Some(item)
    }

    /// True when a text capture must not enter history (empty, internal
    /// GIF URIs, or the echo of a paste we just performed).
    /// وقتی متن نباید وارد تاریخچه شود «درست» است (خالی، URI داخلی GIF،
    /// یا بازتاب pasteای که خودمان انجام دادیم).
    fn should_skip_text(&mut self, text: &str) -> bool {
        if text.trim().is_empty() {
            return true;
        }
        if text.contains(FILE_URI_PREFIX) && text.contains(GIF_CACHE_MARKER) {
            return true;
        }
        if let Some(ref pasted) = self.last_pasted_text {
            if pasted == text {
                self.last_pasted_text = None;
                return true;
            }
            self.last_pasted_text = None;
        }
        false
    }

    fn should_skip_image(&mut self, hash: u64) -> bool {
        if let Some(pasted_hash) = self.last_pasted_image_hash {
            if pasted_hash == hash {
                self.last_pasted_image_hash = None;
                return true;
            }
        }
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
        let hash = calculate_hash(text);
        if !self.text_hashes.contains(&hash) {
            return;
        }
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
            let removed = self.history.remove(pos);
            self.remove_image_file(&removed.id);
            self.rebuild_hash_index();
        }
    }

    /// Rebuild the text-hash index after any history mutation.
    /// بازسازی ایندکس هش متن پس از هر تغییر تاریخچه.
    pub(super) fn rebuild_hash_index(&mut self) {
        self.text_hashes.clear();
        for item in &self.history {
            if let Some(text) = match &item.content {
                ClipboardContent::Text(t) => Some(t),
                ClipboardContent::RichText { plain, .. } => Some(plain),
                _ => None,
            } {
                self.text_hashes.insert(calculate_hash(text));
            }
        }
    }

    /// Insert at the top of the unpinned section and enforce the size cap.
    /// درج در بالای بخش unpinned و اعمال سقف اندازه.
    fn insert_item(&mut self, item: ClipboardItem) {
        let insert_pos = self
            .history
            .iter()
            .position(|i| !i.pinned)
            .unwrap_or(self.history.len());
        let inserted_id = item.id.clone();
        self.history.insert(insert_pos, item);
        self.dirty = true;
        let overflow: Vec<String> = {
            let mut ids = Vec::new();
            while self.history.len() > self.max_history_size {
                if let Some(pos) = self.history.iter().rposition(|i| !i.pinned) {
                    let removed = self.history.remove(pos);
                    ids.push(removed.id.clone());
                    self.remove_image_file(&removed.id);
                } else {
                    break;
                }
            }
            ids
        };
        self.rebuild_hash_index();
        if let Some(item) = self.history.iter().find(|i| i.id == inserted_id).cloned() {
            if let Err(e) = self.persist_upsert_item(&item, insert_pos as i64) {
                tracing::warn!("[ClipboardManager] Upsert failed ({e}); rewriting");
                self.save_history();
                return;
            }
        }
        if let Err(e) = self.persist_delete_ids(&overflow) {
            tracing::warn!("[ClipboardManager] Overflow delete failed ({e})");
        }
        self.persist_mutation();
    }

    /// Drop oldest unpinned items above the configured cap.
    /// حذف قدیمی‌ترین آیتم‌های unpinned فراتر از سقف تنظیم‌شده.
    pub(super) fn enforce_history_limit(&mut self) -> bool {
        let before = self.history.len();
        while self.history.len() > self.max_history_size {
            if let Some(pos) = self.history.iter().rposition(|i| !i.pinned) {
                let removed = self.history.remove(pos);
                self.remove_image_file(&removed.id);
            } else {
                break;
            }
        }
        self.history.len() != before
    }

    pub(super) fn remove_image_file(&mut self, id: &str) {
        if let Some(path) = self.image_paths.remove(id) {
            crate::image_store::remove_image(&path);
        }
    }
}
