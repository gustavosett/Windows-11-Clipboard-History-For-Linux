//! Clipboard Manager — composition root for clipboard history.
//! مدیر کلیپ‌بورد — ریشهٔ ترکیب تاریخچهٔ کلیپ‌بورد.
//!
//! The original 1250-line module was split by concern (ADR: repository
//! hygiene). This module now only owns the shared state struct and its
//! construction; the behaviour lives in focused submodules:
//! ماژول اصلی پیشین (۱۲۵۰ خط) بر اساس نگرانی تفکیک شد. این ماژول فقط
//! ساختار وضعیت مشترک و ساخت آن را در اختیار دارد؛ رفتارها در
//! زیرماژول‌های متمرکز قرار دارند:
//!
//! - [`types`]           — domain model (`ClipboardItem`, `ClipboardContent`)
//! - [`persistence`]     — SQLite load/save (encrypted at rest)
//! - [`deduplication`]   — capture pipeline: privacy, dedup, ordering
//! - [`history_access`]  — reads, paging, pin, delete, retention
//! - [`clipboard_write`] — writing items back to the OS clipboard

use arboard::{Clipboard, ImageData};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use crate::history_crypto::{HistoryCrypto, KeyBackend};
use crate::history_store::{self, PersistRow};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::privacy::{self, PrivacyPolicy};

mod clipboard_write;
mod deduplication;
mod history_access;
mod persistence;
mod types;

pub use crate::content_hash::calculate_hash;
pub use history_access::{HistoryPage, MAX_PAGE_SIZE};
pub use types::{ClipboardContent, ClipboardItem};

pub const DEFAULT_MAX_HISTORY_SIZE: usize = 50;
pub const MAX_HISTORY_HARD_CAP: usize = 2_000;
const PREVIEW_TEXT_MAX_LEN: usize = 100;
const GIF_CACHE_MARKER: &str = "windows-11-style-clipboard-history-manager/gifs/";
const FILE_URI_PREFIX: &str = "file://";
const CLIPBOARD_HELPER_READY_TIMEOUT: Duration = Duration::from_secs(2);
const CLIPBOARD_HELPER_POLL_INTERVAL: Duration = Duration::from_millis(2);

/// Canonical application data directory (shared by `main` and commands).
/// دایرکتوری دادهٔ رسمی برنامه (مشترک بین `main` و commandها).
pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("windows-11-style-clipboard-history-manager")
}

fn truncate_chars(s: &str, max: usize) -> String {
    let mut chars = s.chars();
    let head: String = chars.by_ref().take(max).collect();
    if chars.next().is_none() {
        s.to_string()
    } else {
        head
    }
}

fn get_system_clipboard() -> Result<Clipboard, String> {
    Clipboard::new().map_err(|e| e.to_string())
}

/// Shared clipboard-history state guarded by the caller's mutex.
/// وضعیت مشترک تاریخچهٔ کلیپ‌بورد که با mutex فراخواننده محافظت می‌شود.
pub struct ClipboardManager {
    history: Vec<ClipboardItem>,
    text_hashes: HashSet<u64>,
    last_pasted_text: Option<String>,
    last_pasted_image_hash: Option<u64>,
    last_added_text_hash: Option<u64>,
    db_path: PathBuf,
    json_legacy_path: PathBuf,
    images_dir: PathBuf,
    conn: Connection,
    crypto: HistoryCrypto,
    /// When false, disk I/O is skipped so a missing/wrong key cannot
    /// re-encrypt history under a fresh key (fail-closed, ADR-0004/0006).
    /// وقتی نادرست باشد I/O دیسک انجام نمی‌شود تا کلید تازه تاریخچه را
    /// دوباره رمز نکند (fail-closed).
    persist_enabled: bool,
    image_paths: HashMap<String, PathBuf>,
    max_history_size: usize,
    dirty: bool,
    privacy: PrivacyPolicy,
    auto_delete_interval_minutes: u64,
}

impl ClipboardManager {
    fn clamp_max_history_size(size: usize) -> usize {
        match size {
            0 => DEFAULT_MAX_HISTORY_SIZE,
            1..=MAX_HISTORY_HARD_CAP => size,
            _ => MAX_HISTORY_HARD_CAP,
        }
    }

    /// Create with the classic file-backed encryption key.
    /// `data_dir` holds `history.db`, the images directory and the key files.
    /// ساخت با کلید رمزنگاری کلاسیک مبتنی بر فایل.
    /// `data_dir` جای `history.db`، پوشهٔ تصاویر و فایل‌های کلید است.
    pub fn new(data_dir: PathBuf, max_history_size: usize) -> Self {
        Self::new_with_key_backend(data_dir, max_history_size, KeyBackend::File)
    }

    /// Create with an explicit encryption-key backend (file or Secret Service).
    /// `data_dir` holds `history.db`, the images directory and the key files;
    /// a legacy pre-SQLite `history.json` inside it is migrated on load.
    /// ساخت با بک‌اند صریح کلید رمزنگاری (فایل یا Secret Service).
    /// `data_dir` جای `history.db`، پوشهٔ تصاویر و فایل‌های کلید است؛
    /// `history.json` قدیمیِ پیش از SQLite در آن هنگام بارگذاری مهاجرت می‌کند.
    ///
    /// If the requested backend is unavailable the loader falls back to the
    /// file key whenever it can prove — via the `history.key.check` marker —
    /// that it decrypts the existing data. See `history_crypto.rs`.
    /// اگر بک‌اند درخواستی در دسترس نباشد، بارگذار تا وقتی با نشانگر
    /// `history.key.check` ثابت کند دادهٔ موجود را رمزگشایی می‌کند، به
    /// کلید فایل بازمی‌گردد. برای جزئیات `history_crypto.rs` را ببینید.
    pub fn new_with_key_backend(
        data_dir: PathBuf,
        max_history_size: usize,
        key_backend: KeyBackend,
    ) -> Self {
        let max_size = Self::clamp_max_history_size(max_history_size);
        let base_dir = data_dir;
        let _ = fs::create_dir_all(&base_dir);
        crate::fs_atomic::restrict_permissions(&base_dir);

        let db_path = base_dir.join("history.db");
        // Pre-SQLite store: only read as a one-time migration source.
        // مخزن پیش از SQLite: فقط به‌عنوان منبع مهاجرت یک‌باره خوانده می‌شود.
        let json_legacy_path = base_dir.join("history.json");
        let images_dir = base_dir.join("images");
        let _ = fs::create_dir_all(&images_dir);
        crate::fs_atomic::restrict_permissions(&images_dir);

        let conn = history_store::open_database(&db_path).unwrap_or_else(|e| {
            error!("[ClipboardManager] Failed to open SQLite ({e}); using in-memory fallback");
            Connection::open_in_memory().expect("in-memory sqlite")
        });
        let (crypto, persist_enabled) =
            match HistoryCrypto::load_or_create_with_backend(&base_dir, key_backend) {
                Ok(c) => (c, true),
                Err(e) => {
                    error!(
                        "[ClipboardManager] No usable history key ({e}); refusing to adopt a \
                         fresh key that would corrupt existing history. Persistence is disabled \
                         for this session (fail-closed)."
                    );
                    // Process-local key that is NEVER written next to the real DB.
                    // کلید موقت جلسه که هرگز کنار دیتابیس واقعی نوشته نمی‌شود.
                    let ephemeral_dir = std::env::temp_dir()
                        .join(format!("windows-11-style-clipboard-history-unusable-{}", Uuid::new_v4()));
                    let _ = fs::create_dir_all(&ephemeral_dir);
                    (
                        HistoryCrypto::load_or_create(&ephemeral_dir)
                            .expect("session-local crypto"),
                        false,
                    )
                }
            };

        let mut manager = Self {
            history: Vec::with_capacity(max_size),
            text_hashes: HashSet::new(),
            last_pasted_text: None,
            last_pasted_image_hash: None,
            last_added_text_hash: None,
            db_path,
            json_legacy_path,
            images_dir,
            conn,
            crypto,
            persist_enabled,
            image_paths: HashMap::new(),
            max_history_size: max_size,
            dirty: false,
            privacy: PrivacyPolicy::default(),
            auto_delete_interval_minutes: 0,
        };
        if persist_enabled {
            manager.migrate_legacy_json();
            manager.load_from_db();
            manager.rebuild_hash_index();
        } else {
            warn!(
                "[ClipboardManager] Starting with an empty in-memory history (key unavailable)"
            );
        }
        manager
    }

    // -----------------------------------------------------------------
    // Runtime policy
    // سیاست‌های زمان اجرا
    // -----------------------------------------------------------------

    pub fn set_privacy_policy(&mut self, policy: PrivacyPolicy) {
        self.privacy = policy;
    }

    pub fn privacy_policy(&self) -> PrivacyPolicy {
        self.privacy.clone()
    }

    pub fn set_auto_delete_interval_minutes(&mut self, minutes: u64) {
        self.auto_delete_interval_minutes = minutes;
    }

    pub fn auto_delete_interval_minutes(&self) -> u64 {
        self.auto_delete_interval_minutes
    }

    pub fn set_max_history_size(&mut self, new_size: usize) {
        let mut clamped = Self::clamp_max_history_size(new_size);
        let pinned_count = self.history.iter().filter(|i| i.pinned).count();
        if clamped < pinned_count {
            clamped = pinned_count;
        }
        self.max_history_size = clamped;
        let trimmed = self.enforce_history_limit();
        if trimmed {
            self.save_history();
        }
    }

    pub fn get_max_history_size(&self) -> usize {
        self.max_history_size
    }

    /// Human-readable label of the *active* encryption-key backend.
    /// برچسب خوانای بک‌اند *فعال* کلید رمزنگاری.
    pub fn key_backend(&self) -> &'static str {
        self.crypto.backend_label()
    }
}

impl Drop for ClipboardManager {
    fn drop(&mut self) {
        if self.dirty {
            let _ = self.persist_sqlite();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    fn temp_manager(name: &str) -> ClipboardManager {
        let dir = temp_dir().join(format!("clip-hist-{name}-{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&dir);
        ClipboardManager::new(dir, 10)
    }

    #[test]
    fn persists_text_across_reload() {
        let dir = temp_dir().join(format!("clip-reload-{}", Uuid::new_v4()));
        {
            let mut mgr = ClipboardManager::new(dir.clone(), 10);
            assert!(mgr.add_text("hello persistence".into(), None).is_some());
        }
        let mgr2 = ClipboardManager::new(dir, 10);
        let hist = mgr2.get_history();
        assert_eq!(hist.len(), 1);
        match &hist[0].content {
            ClipboardContent::Text(t) => assert_eq!(t, "hello persistence"),
            _ => panic!("expected text"),
        }
    }

    #[test]
    fn secrets_are_not_stored() {
        let mut mgr = temp_manager("secrets");
        assert!(mgr
            .add_text("ghp_abcdefghijklmnopqrstuvwxyz0123456789".into(), None)
            .is_none());
        assert!(mgr.get_history().is_empty());
    }

    #[test]
    fn duplicate_text_is_not_reinserted() {
        let mut mgr = temp_manager("dup");
        assert!(mgr.add_text("same".into(), None).is_some());
        assert!(mgr.add_text("same".into(), None).is_none());
        assert_eq!(mgr.get_history().len(), 1);
    }

    #[test]
    fn secrets_and_disk_are_encrypted() {
        let dir = temp_dir().join(format!("clip-enc-{}", Uuid::new_v4()));
        {
            let mut mgr = ClipboardManager::new(dir.clone(), 10);
            assert!(mgr.add_text("encrypt-me-please".into(), None).is_some());
        }
        let db = dir.join("history.db");
        let conn = rusqlite::Connection::open(&db).unwrap();
        let stored: String = conn
            .query_row("SELECT text FROM items LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_ne!(stored, "encrypt-me-please");
        let mgr2 = ClipboardManager::new(dir, 10);
        match &mgr2.get_history()[0].content {
            ClipboardContent::Text(t) => assert_eq!(t, "encrypt-me-please"),
            _ => panic!("expected text"),
        }
    }

    #[test]
    fn incremental_persist_keeps_order() {
        let dir = temp_dir().join(format!("clip-inc-{}", Uuid::new_v4()));
        {
            let mut mgr = ClipboardManager::new(dir.clone(), 10);
            assert!(mgr.add_text("one".into(), None).is_some());
            assert!(mgr.add_text("two".into(), None).is_some());
            mgr.remove_item(&mgr.get_history()[1].id.clone());
        }
        let mgr2 = ClipboardManager::new(dir, 10);
        let hist = mgr2.get_history();
        assert_eq!(hist.len(), 1);
        match &hist[0].content {
            ClipboardContent::Text(t) => assert_eq!(t, "two"),
            _ => panic!("expected remaining text"),
        }
    }

    #[test]
    fn history_page_returns_bounded_window() {
        // Paging contract: clamped limits, safe offsets, stable totals.
        // قرارداد صفحه‌بندی: سقف limit، offset امن و total پایدار.
        let mut mgr = temp_manager("paging");
        for i in 0..7 {
            assert!(mgr.add_text(format!("item-{i}"), None).is_some());
        }

        let page = mgr.get_history_page(3, 0);
        assert_eq!(page.items.len(), 3);
        assert_eq!(page.total, 7);
        assert_eq!(page.limit, 3);
        assert_eq!(page.offset, 0);

        let tail = mgr.get_history_page(3, 6);
        assert_eq!(tail.items.len(), 1);
        assert_eq!(tail.offset, 6);

        // Out-of-range and zero limits are clamped, never panic.
        let clamped = mgr.get_history_page(0, 999);
        assert_eq!(clamped.items.len(), 0);
        assert!(clamped.limit >= 1);
    }
}
