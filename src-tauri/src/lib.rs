//! Windows 11 Clipboard History For Linux Library
//! This module re-exports the core functionality for use as a library

pub mod clipboard_manager;
pub mod config_manager;
pub mod emoji_manager;
pub mod focus_manager;
pub mod gif_manager;
pub mod input_simulator;
pub mod permission_checker;
pub mod session;
pub mod shortcut_conflict_detector;
pub mod shortcut_setup;
pub mod user_settings;

#[cfg(target_os = "linux")]
pub mod linux_shortcut_manager;

pub use clipboard_manager::{ClipboardContent, ClipboardItem, ClipboardManager};
pub use config_manager::ConfigManager;
pub use emoji_manager::{EmojiManager, EmojiUsage};
pub use focus_manager::{restore_focused_window, save_focused_window};

#[cfg(target_os = "linux")]
pub use focus_manager::{x11_activate_window_by_title, x11_robust_activate};
pub use gif_manager::{paste_gif_to_clipboard, paste_gif_to_clipboard_with_uri};
pub use permission_checker::{
    check_permissions, fix_permissions_now, is_first_run, mark_first_run_complete, reset_first_run,
    PermissionStatus,
};
pub use session::{get_session_type, is_wayland, is_x11, SessionType};
pub use shortcut_conflict_detector::{
    auto_resolve_conflicts, detect_shortcut_conflicts, ConflictDetectionResult, ShortcutConflict,
};
pub use shortcut_setup::{
    check_shortcut_tools, detect_conflicts, get_desktop_environment, register_de_shortcut,
    resolve_conflicts, ShortcutToolsStatus,
};
pub use user_settings::{UserSettings, UserSettingsManager};
