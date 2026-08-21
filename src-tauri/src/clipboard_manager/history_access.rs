//! History reads, paging, pinning, deletion, and retention.
//! خواندن تاریخچه، صفحه‌بندی، سنجاق، حذف و نگهداشت.

use super::*;

/// Upper bound for a single `get_history_page` request.
/// سقف بالای هر درخواست در `get_history_page`.
pub const MAX_PAGE_SIZE: usize = 200;

/// One bounded window of history plus paging metadata for the webview.
/// یک پنجرهٔ محدود از تاریخچه به‌همراه فرادادهٔ صفحه‌بندی برای webview.
///
/// Keeps large histories cheap over IPC: the UI can render thousands of
/// items by requesting successive windows instead of one full payload.
/// تاریخچه‌های بزرگ روی IPC ارزان می‌مانند: رابط کاربری می‌تواند
/// هزاران آیتم را با درخواست پنجره‌های متوالی رندر کند.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryPage {
    pub items: Vec<ClipboardItem>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

impl ClipboardManager {
    // -----------------------------------------------------------------
    // Dirty tracking
    // -----------------------------------------------------------------

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    // -----------------------------------------------------------------
    // Reads
    // -----------------------------------------------------------------

    /// Full history (owned clones) — used by tests and internal callers.
    /// تاریخچهٔ کامل (کپی دارای مالکیت) — برای تست‌ها و فراخوان‌های داخلی.
    pub fn get_history(&self) -> Vec<ClipboardItem> {
        self.history.clone()
    }

    /// History payload for the webview: preview + truncated plain text, no HTML.
    /// بار تاریخچه برای webview: پیش‌نمایش + متن محدودشده، بدون HTML.
    pub fn get_history_for_ui(&self) -> Vec<ClipboardItem> {
        self.history.iter().map(ClipboardItem::for_ipc).collect()
    }

    /// Bounded, IPC-safe window of the history.
    /// پنجرهٔ محدود و امن برای IPC از تاریخچه.
    ///
    /// `limit` is clamped to `1..=MAX_PAGE_SIZE` and `offset` to the
    /// collection length, so no input can request unbounded payloads.
    /// مقدار `limit` به بازهٔ `1..=MAX_PAGE_SIZE` و `offset` به طول
    /// مجموعه محدود می‌شود؛ هیچ ورودی نمی‌تواند بار نامحدود بخواهد.
    pub fn get_history_page(&self, limit: usize, offset: usize) -> HistoryPage {
        let total = self.history.len();
        let clamped_limit = limit.clamp(1, MAX_PAGE_SIZE);
        let clamped_offset = offset.min(total);
        let end = clamped_offset.saturating_add(clamped_limit).min(total);
        HistoryPage {
            items: self.history[clamped_offset..end]
                .iter()
                .map(ClipboardItem::for_ipc)
                .collect(),
            total,
            limit: clamped_limit,
            offset: clamped_offset,
        }
    }

    pub fn get_item(&self, id: &str) -> Option<&ClipboardItem> {
        self.history.iter().find(|item| item.id == id)
    }

    // -----------------------------------------------------------------
    // Mutations
    // -----------------------------------------------------------------

    /// Remove every unpinned item (pinned items survive).
    /// حذف همهٔ آیتم‌های unpinned (آیتم‌های سنجاق‌شده می‌مانند).
    pub fn clear(&mut self) {
        let removed: Vec<String> = self
            .history
            .iter()
            .filter(|i| !i.pinned)
            .map(|i| i.id.clone())
            .collect();
        self.history.retain(|item| item.pinned);
        for id in &removed {
            self.remove_image_file(id);
        }
        self.dirty = true;
        self.rebuild_hash_index();
        if let Err(e) = self.persist_delete_ids(&removed) {
            tracing::warn!("[ClipboardManager] Clear delete failed ({e}); rewriting");
            self.save_history();
            return;
        }
        self.persist_mutation();
    }

    pub fn remove_item(&mut self, id: &str) {
        self.history.retain(|item| item.id != id);
        self.remove_image_file(id);
        self.dirty = true;
        self.rebuild_hash_index();
        if let Err(e) = self.persist_delete_ids(&[id.to_string()]) {
            tracing::warn!("[ClipboardManager] Delete failed ({e}); rewriting");
            self.save_history();
            return;
        }
        self.persist_mutation();
    }

    pub fn toggle_pin(&mut self, id: &str) -> Option<ClipboardItem> {
        let pos = self.history.iter().position(|i| i.id == id)?;
        self.history[pos].pinned = !self.history[pos].pinned;
        let item = self.history.remove(pos);
        let insert_pos = self
            .history
            .iter()
            .position(|i| !i.pinned)
            .unwrap_or(self.history.len());
        self.history.insert(insert_pos, item);
        let item_clone = self.history[insert_pos].clone();
        self.dirty = true;
        self.persist_mutation();
        Some(item_clone)
    }

    pub fn move_item_to_top(&mut self, id: &str) -> bool {
        let current_pos = match self.history.iter().position(|i| i.id == id) {
            Some(pos) => pos,
            None => return false,
        };
        let item_pinned = self.history[current_pos].pinned;
        let insert_pos = if item_pinned {
            0
        } else {
            self.history
                .iter()
                .position(|i| !i.pinned)
                .unwrap_or(self.history.len())
        };
        if insert_pos == current_pos {
            return true;
        }
        let item = self.history.remove(current_pos);
        self.history.insert(insert_pos, item);
        self.dirty = true;
        self.persist_mutation();
        true
    }

    // -----------------------------------------------------------------
    // Retention
    // -----------------------------------------------------------------

    /// Drop unpinned items older than `interval_minutes`.
    /// حذف آیتم‌های unpinned قدیمی‌تر از `interval_minutes`.
    pub fn cleanup_old_items(&mut self, interval_minutes: u64) -> bool {
        if interval_minutes == 0 {
            return false;
        }
        let now = Utc::now();
        let interval_seconds = (interval_minutes * 60) as i64;
        let mut removed_ids = Vec::new();
        self.history.retain(|item| {
            if item.pinned {
                return true;
            }
            let age_seconds = now.signed_duration_since(item.timestamp).num_seconds();
            let keep = age_seconds < interval_seconds;
            if !keep {
                removed_ids.push(item.id.clone());
            }
            keep
        });
        for id in &removed_ids {
            self.remove_image_file(id);
        }
        if !removed_ids.is_empty() {
            self.dirty = true;
            self.rebuild_hash_index();
            self.save_history();
            return true;
        }
        false
    }

    // -----------------------------------------------------------------
    // Paste bookkeeping
    // -----------------------------------------------------------------

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

    pub fn mark_text_as_pasted(&mut self, text: &str) {
        self.last_pasted_text = Some(text.to_string());
        self.last_added_text_hash = Some(calculate_hash(&text));
    }
}
