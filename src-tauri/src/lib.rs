//! Windows 11 Clipboard History For Linux Library
//! This module re-exports the core functionality for use as a library

pub mod clipboard_manager;
pub mod emoji_manager;
pub mod focus_manager;
pub mod gif_manager;
pub mod hotkey_manager;
pub mod input_simulator;

pub use clipboard_manager::{ClipboardContent, ClipboardItem, ClipboardManager};
pub use emoji_manager::{EmojiManager, EmojiUsage};
pub use focus_manager::{restore_focused_window, save_focused_window};
pub use gif_manager::{paste_gif_to_clipboard, paste_gif_to_clipboard_with_uri};
pub use hotkey_manager::{HotkeyAction, HotkeyManager};
