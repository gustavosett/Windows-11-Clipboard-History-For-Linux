//! SQLite persistence for clipboard history.
//! ماندگاری SQLite برای تاریخچهٔ کلیپ‌بورد.
//!
//! All disk I/O for history lives here: legacy JSON migration, loading,
//! full rewrites, incremental upserts/deletes, and ordering updates.
//! Sensitive columns are encrypted by `HistoryCrypto` before they reach
//! SQL (fail-closed).
//! تمام I/O دیسک تاریخچه اینجاست: مهاجرت JSON قدیمی، بارگذاری، بازنویسی
//! کامل، upsert/delete افزایشی و به‌روزرسانی ترتیب. ستون‌های حساس پیش از
//! رسیدن به SQL توسط `HistoryCrypto` رمز می‌شوند (fail-closed).

use super::*;

impl ClipboardManager {
    /// One-time migration from the pre-SQLite `history.json`.
    /// مهاجرت یک‌باره از `history.json` پیش از SQLite.
    pub(super) fn migrate_legacy_json(&mut self) {
        if !self.json_legacy_path.exists() {
            return;
        }
        // Only migrate when the database is empty.
        // فقط وقتی دیتابیس خالی است مهاجرت انجام شود.
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM items", [], |r| r.get(0))
            .unwrap_or(0);
        if count > 0 {
            return;
        }

        let Ok(content) = fs::read_to_string(&self.json_legacy_path) else {
            return;
        };
        let Ok(items) = serde_json::from_str::<Vec<ClipboardItem>>(&content) else {
            warn!("[ClipboardManager] Legacy history.json is unreadable; leaving in place");
            return;
        };

        info!(
            "[ClipboardManager] Migrating {} items from history.json → SQLite",
            items.len()
        );
        for mut item in items {
            if let ClipboardContent::Image { base64, width, height } = &item.content {
                if let Ok(png) = BASE64.decode(base64) {
                    if let Ok(stored) =
                        crate::image_store::store_png_bytes(
                            &self.images_dir,
                            &item.id,
                            &png,
                            &self.crypto,
                        )
                    {
                        self.image_paths
                            .insert(item.id.clone(), stored.full_path.clone());
                        item.content = ClipboardContent::Image {
                            base64: stored.thumb_base64,
                            width: *width,
                            height: *height,
                        };
                    }
                }
            }
            self.history.push(item);
        }
        let _ = self.enforce_history_limit();
        self.save_history();

        let bak = self.json_legacy_path.with_extension("json.bak");
        if fs::rename(&self.json_legacy_path, &bak).is_err() {
            let _ = fs::remove_file(&self.json_legacy_path);
        }
        info!("[ClipboardManager] Legacy JSON migrated");
    }

    /// Load the full history from SQLite into memory (decrypting fields).
    /// بارگذاری کامل تاریخچه از SQLite در حافظه (با رمزگشایی فیلدها).
    pub(super) fn load_from_db(&mut self) {
        if !self.history.is_empty() {
            return;
        }
        let rows = match history_store::load_rows(&self.conn) {
            Ok(r) => r,
            Err(e) => {
                warn!("[ClipboardManager] Failed to load: {e}");
                return;
            }
        };

        // Rows whose encrypted fields fail to decrypt are quarantined instead
        // of being silently dropped, so no history is lost without a trace.
        // ردیف‌هایی که ستون‌های رمزشان رمزگشایی نشود به قرنطینه می‌روند تا
        // هیچ تاریخی بدون ردپا گم نشود.
        let mut quarantined: Vec<(String, String)> = Vec::new();

        for row in rows {
            let mut row_quarantined = false;

            let text = match self.crypto.decrypt_optional(row.text) {
                Ok(v) => v,
                Err(e) => {
                    quarantined.push((row.id.clone(), format!("text decrypt: {e}")));
                    row_quarantined = true;
                    None
                }
            };
            let html = match self.crypto.decrypt_optional(row.html) {
                Ok(v) => v,
                Err(e) => {
                    quarantined.push((row.id.clone(), format!("html decrypt: {e}")));
                    row_quarantined = true;
                    None
                }
            };
            let mut preview = match self.crypto.decrypt_str(&row.preview) {
                Ok(v) => v,
                Err(e) => {
                    quarantined.push((row.id.clone(), format!("preview decrypt: {e}")));
                    row_quarantined = true;
                    String::new()
                }
            };
            let thumb_base64 = match self.crypto.decrypt_optional(row.thumb_base64) {
                Ok(v) => v,
                Err(e) => {
                    quarantined.push((row.id.clone(), format!("thumb decrypt: {e}")));
                    row_quarantined = true;
                    None
                }
            };

            if row_quarantined {
                // Never surface a partially-decrypted item to the user.
                // هرگز آیتم ناقص به کاربر نمایش داده نمی‌شود.
                continue;
            }
            let content = match row.kind.as_str() {
                "richtext" => ClipboardContent::RichText {
                    plain: text.unwrap_or_default(),
                    html: html.unwrap_or_default(),
                },
                "image" => {
                    if let Some(path) = row.image_path.clone() {
                        self.image_paths
                            .insert(row.id.clone(), PathBuf::from(&path));
                    }
                    ClipboardContent::Image {
                        base64: thumb_base64.unwrap_or_default(),
                        width: row.width.unwrap_or(0) as u32,
                        height: row.height.unwrap_or(0) as u32,
                    }
                }
                _ => ClipboardContent::Text(text.unwrap_or_default()),
            };

            let timestamp = DateTime::<Utc>::from_timestamp_millis(row.created_at)
                .unwrap_or_else(Utc::now);

            if matches!(content, ClipboardContent::Image { .. }) {
                if let Some(hash) = row.image_hash {
                    if !preview.contains('#') {
                        preview = format!("{preview} #{hash}");
                    }
                }
            }

            self.history.push(ClipboardItem {
                id: row.id,
                content,
                timestamp,
                pinned: row.pinned,
                preview,
            });
        }

        if !quarantined.is_empty() {
            self.record_quarantine(&quarantined);
        }

        let _ = self.enforce_history_limit();
        if let Some(first) = self.history.first() {
            match &first.content {
                ClipboardContent::Text(text) => {
                    self.last_added_text_hash = Some(calculate_hash(text));
                }
                ClipboardContent::RichText { plain, .. } => {
                    self.last_added_text_hash = Some(calculate_hash(plain));
                }
                ClipboardContent::Image { .. } => {
                    self.last_added_text_hash = None;
                }
            }
        }
        debug!(
            "[ClipboardManager] Loaded {} items from SQLite",
            self.history.len()
        );
    }

    /// Append undecryptable rows to a quarantine log so nothing is lost without
    /// a trace. The log is plaintext (ids + reasons) and lives next to the DB.
    /// ردیف‌های غیرقابل‌رمزگشایی را به یک لاگ قرنطینه اضافه می‌کند تا چیزی
    /// بدون ردپا گم نشود. لاگ متن‌ساده (شناسه‌ها + دلایل) است و کنار DB می‌ماند.
    pub(super) fn record_quarantine(&self, entries: &[(String, String)]) {
        let Some(base_dir) = self.db_path.parent() else {
            return;
        };
        let path = base_dir.join("quarantine.log");
        let mut content = String::new();
        content.push_str(&format!(
            "--- {} ({} quarantined) ---\n",
            chrono::Utc::now().to_rfc3339(),
            entries.len()
        ));
        for (id, reason) in entries {
            content.push_str(&format!("{id}\t{reason}\n"));
        }
        // Append without leaking the raw clipboard text (reasons are generic).
        // الحاق بدون درز محتوای کلیپ‌بورد (دلایل عمومی‌اند).
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            use std::io::Write;
            let _ = file.write_all(content.as_bytes());
        }
    }

    /// Persist everything (full rewrite). Used for migrations and fallbacks.
    /// ذخیرهٔ همه‌چیز (بازنویسی کامل). برای مهاجرت و مسیرهای جایگزین.
    pub fn save_history(&mut self) {
        if let Err(e) = self.persist_sqlite() {
            error!("[ClipboardManager] Failed to save history: {e}");
        } else {
            self.dirty = false;
        }
    }

    pub(super) fn persist_sqlite(&mut self) -> Result<(), String> {
        if !self.persist_enabled {
            return Err(
                "history persistence disabled: encryption key unavailable (fail-closed)".into(),
            );
        }
        let rows = self.collect_persist_rows()?;
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM items", []).map_err(|e| e.to_string())?;
        {
            let mut stmt = tx
                .prepare(history_store::INSERT_ITEM_SQL)
                .map_err(|e| e.to_string())?;
            for row in &rows {
                stmt.execute(params![
                    row.id,
                    row.kind,
                    row.text,
                    row.html,
                    row.image_path,
                    row.image_hash,
                    row.width,
                    row.height,
                    row.preview,
                    row.pinned,
                    row.created_at,
                    row.thumb,
                    row.sort_index,
                ])
                .map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        crate::fs_atomic::restrict_sqlite_files(&self.db_path);
        Ok(())
    }

    /// Incrementally upsert a single item (no full rewrite).
    /// درج/به‌روزرسانی افزایشی یک آیتم (بدون بازنویسی کامل).
    pub(super) fn persist_upsert_item(&mut self, item: &ClipboardItem, sort_index: i64) -> Result<(), String> {
        if !self.persist_enabled {
            return Err(
                "history persistence disabled: encryption key unavailable (fail-closed)".into(),
            );
        }
        let row = persist_row_from_item(
            sort_index as usize,
            item,
            self.image_paths
                .get(&item.id)
                .map(|p| p.to_string_lossy().into_owned()),
            &self.crypto,
        )?;
        history_store::execute_insert(&self.conn, &row)?;
        crate::fs_atomic::restrict_sqlite_files(&self.db_path);
        Ok(())
    }

    pub(super) fn persist_delete_ids(&mut self, ids: &[String]) -> Result<(), String> {
        if !self.persist_enabled {
            return Err(
                "history persistence disabled: encryption key unavailable (fail-closed)".into(),
            );
        }
        if ids.is_empty() {
            return Ok(());
        }
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        for id in ids {
            tx.execute("DELETE FROM items WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn persist_meta(&mut self) -> Result<(), String> {
        if !self.persist_enabled {
            return Err(
                "history persistence disabled: encryption key unavailable (fail-closed)".into(),
            );
        }
        let meta: Vec<(String, i64, i64)> = self
            .history
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                (
                    item.id.clone(),
                    idx as i64,
                    if item.pinned { 1 } else { 0 },
                )
            })
            .collect();
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        {
            let mut stmt = tx
                .prepare("UPDATE items SET sort_index = ?1, pinned = ?2 WHERE id = ?3")
                .map_err(|e| e.to_string())?;
            for (id, idx, pinned) in &meta {
                stmt.execute(params![idx, pinned, id])
                    .map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Persist an in-place mutation (order/pin) incrementally, falling back
    /// to a full rewrite when the incremental path fails.
    /// ذخیرهٔ تغییر درجا (ترتیب/pin) به‌صورت افزایشی؛ در خطا، بازنویسی کامل.
    pub(super) fn persist_mutation(&mut self) {
        if let Err(e) = self.persist_meta() {
            tracing::warn!("[ClipboardManager] Incremental persist failed ({e}); rewriting");
            if let Err(e) = self.persist_sqlite() {
                error!("[ClipboardManager] Failed to save history: {e}");
                return;
            }
        }
        self.dirty = false;
    }

    fn collect_persist_rows(&self) -> Result<Vec<PersistRow>, String> {
        self.history
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                persist_row_from_item(
                    idx,
                    item,
                    self.image_paths
                        .get(&item.id)
                        .map(|p| p.to_string_lossy().into_owned()),
                    &self.crypto,
                )
            })
            .collect()
    }
}

/// Build an encrypted persist row from an in-memory item.
/// ساخت ردیف رمزنگاری‌شده برای ذخیره از آیتم حافظه.
fn persist_row_from_item(
    idx: usize,
    item: &ClipboardItem,
    image_path: Option<String>,
    crypto: &HistoryCrypto,
) -> Result<PersistRow, String> {
    let (kind, text, html, image_hash, width, height, thumb) = match &item.content {
        ClipboardContent::Text(t) => ("text", Some(t.clone()), None, None, None, None, None),
        ClipboardContent::RichText { plain, html } => (
            "richtext",
            Some(plain.clone()),
            Some(html.clone()),
            None,
            None,
            None,
            None,
        ),
        ClipboardContent::Image {
            base64,
            width,
            height,
        } => (
            "image",
            None,
            None,
            item.extract_image_hash().map(|h| h as i64),
            Some(*width as i64),
            Some(*height as i64),
            Some(base64.clone()),
        ),
    };
    Ok(PersistRow {
        id: item.id.clone(),
        kind,
        text: crypto.encrypt_optional(text.as_deref())?,
        html: crypto.encrypt_optional(html.as_deref())?,
        image_path,
        image_hash,
        width,
        height,
        preview: crypto.encrypt_str(&item.preview)?,
        pinned: if item.pinned { 1 } else { 0 },
        created_at: item.timestamp.timestamp_millis(),
        thumb: crypto.encrypt_optional(thumb.as_deref())?,
        sort_index: idx as i64,
    })
}
