//! Windows 11 Clipboard History For Linux Library
//! This module re-exports the core functionality for use as a library

pub mod clipboard_manager;
pub mod config_manager;
pub mod emoji_manager;
pub mod focus_manager;
pub mod gif_manager;
pub mod input_simulator;
pub mod session;
pub mod user_settings;

#[cfg(target_os = "linux")]
pub mod linux_shortcut_manager;

pub use clipboard_manager::{ClipboardContent, ClipboardItem, ClipboardManager};
pub use config_manager::ConfigManager;
pub use emoji_manager::{EmojiManager, EmojiUsage};
pub use focus_manager::{restore_focused_window, save_focused_window};

#[cfg(target_os = "linux")]
pub use focus_manager::{x11_robust_activate, x11_activate_window_by_title};
pub use gif_manager::{paste_gif_to_clipboard, paste_gif_to_clipboard_with_uri};
pub use session::{get_session_type, is_wayland, is_x11, SessionType};
pub use user_settings::{UserSettings, UserSettingsManager};
