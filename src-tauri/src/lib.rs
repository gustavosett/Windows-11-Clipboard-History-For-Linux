//! Windows 11 Clipboard History For Linux Library
//! / کتابخانهٔ مدیر تاریخچه کلیپ‌بورد ویندوز ۱۱ برای لینوکس
//!
//! This module re-exports the core functionality for use as a library.
//! Each public module handles a single concern (clipboard I/O, privacy,
//! input simulation, etc.) so that `main.rs` stays focused on application
//! bootstrap and Tauri plugin registration.
//! هر ماژول عمومی یک نگرانی واحد را پوشش می‌دهد تا `main.rs` فقط bootstrap باشد.

use parking_lot::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing_appender::non_blocking::WorkerGuard;
use std::sync::OnceLock;

static TRACING_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

const PASTE_TICKET_TTL: Duration = Duration::from_secs(5);

/// One-shot capability that authorizes `finish_paste` keystroke injection.
pub struct PasteTicket {
    nonce: String,
    expires_at: Instant,
}

/// Application state shared across all Tauri command handlers.
pub struct AppState {
    pub clipboard_manager: Arc<Mutex<clipboard_manager::ClipboardManager>>,
    pub emoji_manager: Arc<Mutex<emoji_manager::EmojiManager>>,
    pub config_manager: Arc<Mutex<config_manager::ConfigManager>>,
    pub is_mouse_inside: Arc<AtomicBool>,
    /// Serializes the complete clipboard/focus/input transaction.
    pub paste_gate: tokio::sync::Mutex<()>,
    /// Required by paste commands so the webview cannot inject Ctrl+V at will.
    /// / بلیت یک‌بارمصرف برای جلوگیری از تزریق آزاد Ctrl+V از webview.
    pub paste_ticket: Mutex<Option<PasteTicket>>,
}

impl AppState {
    /// Record that a clipboard write just happened and return a nonce the UI
    /// must present to `finish_paste` within 5 seconds.
    pub fn issue_paste_ticket(&self) -> String {
        let nonce = uuid::Uuid::new_v4().to_string();
        *self.paste_ticket.lock() = Some(PasteTicket {
            nonce: nonce.clone(),
            expires_at: Instant::now() + PASTE_TICKET_TTL,
        });
        nonce
    }

    /// Consume a ticket. Mismatched or expired tickets fail closed.
    pub fn consume_paste_ticket(&self, nonce: &str) -> bool {
        let mut slot = self.paste_ticket.lock();
        match slot.take() {
            Some(ticket)
                if ticket.nonce == nonce && Instant::now() <= ticket.expires_at && !nonce.is_empty() =>
            {
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod paste_ticket_tests {
    use super::*;
    use parking_lot::Mutex as PMutex;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    fn empty_state() -> AppState {
        let dir = std::env::temp_dir().join(format!("ticket-{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&dir);
        AppState {
            clipboard_manager: Arc::new(PMutex::new(
                clipboard_manager::ClipboardManager::new(dir.clone(), 8),
            )),
            emoji_manager: Arc::new(PMutex::new(emoji_manager::EmojiManager::new(dir.clone()))),
            config_manager: Arc::new(PMutex::new(config_manager::ConfigManager::new(dir.clone()))),
            is_mouse_inside: Arc::new(AtomicBool::new(false)),
            paste_gate: tokio::sync::Mutex::new(()),
            paste_ticket: PMutex::new(None),
        }
    }

    #[test]
    fn paste_ticket_is_one_shot_and_rejects_mismatch() {
        let state = empty_state();
        let nonce = state.issue_paste_ticket();
        assert!(!state.consume_paste_ticket("wrong"));
        assert!(state.consume_paste_ticket(&nonce));
        assert!(!state.consume_paste_ticket(&nonce));
        assert!(!state.consume_paste_ticket(""));
    }
}

/// Initialize tracing/logging. Called once at app startup.
/// The worker guard is kept alive for the process lifetime so logs flush.
pub fn init_tracing() {
    use tracing_subscriber::prelude::*;

    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("windows-11-style-clipboard-history-manager/logs");
    let _ = std::fs::create_dir_all(&log_dir);
    crate::fs_atomic::restrict_permissions(&log_dir);

    let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let _ = TRACING_GUARD.set(guard);

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::filter::EnvFilter::new("info,windows_11_style_clipboard_history_manager=debug")
            }),
        )
        .with_writer(non_blocking)
        .with_ansi(false)
        .try_init();
}

// ---------------------------------------------------------------------------
// Public modules
// ---------------------------------------------------------------------------

pub mod autostart_manager;
pub mod clipboard_io;
pub mod clipboard_manager;
pub mod clipboard_events;
pub mod content_hash;
pub mod clipboard_watcher;
pub mod commands;
pub mod config_manager;
pub mod emoji_manager;
pub mod error;
pub mod exec_lookup;
pub mod focus_manager;
pub mod fs_atomic;
#[cfg(feature = "gif-search")]
pub mod gif_manager;
pub mod history_crypto;
pub mod history_store;
pub mod image_store;
pub mod input_simulator;
pub mod linux_shortcut_manager;
pub mod net_policy;
pub mod open_url;
pub mod paste_sync;
pub mod permission_checker;
pub mod privacy;
pub mod rendering_env;
pub mod session;
pub mod shortcut_conflict_detector;
pub mod shortcut_setup;
pub mod ssrf;
#[cfg(feature = "gif-search")]
pub mod tenor_api;
pub mod theme_manager;
pub mod user_settings;
pub mod window_controller;
pub mod window_identity;
pub mod window_policy;
