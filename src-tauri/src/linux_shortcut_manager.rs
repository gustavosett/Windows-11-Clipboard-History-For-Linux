//! Linux Desktop Environment Shortcut Manager.
//! / مدیر میانبر محیط رومیزی لینوکس.
//!
//! Detects the active desktop environment and registers/unregisters the
//! application shortcuts through the appropriate backend.
//! محیط رومیزی فعال را تشخیص می‌دهد و میانبرها را از بک‌اند مناسب ثبت می‌کند.
//!
//! - `shortcut_config`    — shared shortcut table
//! - `shortcut_error`     — shared error type
//! - `shortcut_utils`     — process/file helpers (no shell, atomic writes)
//! - `shortcut_gsettings` — GNOME / Cinnamon / MATE
//! - `shortcut_tiling`    — i3 / Sway / Hyprland
//! - `shortcut_kde`       — KDE Plasma
//! - `shortcut_xfce`      — XFCE
//! - `shortcut_cosmic`    — COSMIC Epoch
//! - `shortcut_lxqt`      — LXQt
//! - `shortcut_lxde`      — LXDE / Openbox

mod shortcut_config;
mod shortcut_cosmic;
mod shortcut_error;
mod shortcut_gsettings;
mod shortcut_handler;
mod shortcut_kde;
mod shortcut_lxde;
mod shortcut_lxqt;
mod shortcut_tiling;
mod shortcut_utils;
mod shortcut_xfce;

use self::shortcut_config::{get_command_path, SHORTCUTS};
use self::shortcut_cosmic::CosmicHandler;
use self::shortcut_gsettings::{CinnamonHandler, GnomeHandler, MateHandler};
use self::shortcut_handler::ShortcutHandler;
use self::shortcut_kde::KdeHandler;
use self::shortcut_lxde::LxdeHandler;
use self::shortcut_lxqt::LxqtHandler;
use self::shortcut_tiling::{HyprlandHandler, I3Handler, SwayHandler};
use self::shortcut_utils::Utils;
use self::shortcut_xfce::XfceHandler;
use std::env;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

/// When false (default), tiling WM handlers never comment out the user's existing bindings.
static ALLOW_WM_CONFIG_REWRITE: AtomicBool = AtomicBool::new(false);

/// Opt-in: allow commenting out existing Super+V bindings in i3/Sway/Hyprland configs.
/// / اختیاری: اجازهٔ کامنت‌کردن میانبرهای Super+V موجود در کانفیگ i3/Sway/Hyprland.
pub fn set_allow_wm_config_rewrite(allow: bool) {
    ALLOW_WM_CONFIG_REWRITE.store(allow, Ordering::Relaxed);
}

fn allow_wm_config_rewrite() -> bool {
    ALLOW_WM_CONFIG_REWRITE.load(Ordering::Relaxed)
}

pub use self::shortcut_error::{Result as ShortcutResult, ShortcutError};

pub fn register_global_shortcut() {
    let handler = detect_handler();
    tracing::info!("[ShortcutManager] Detected Environment: {}", handler.name());

    let command_path = get_command_path();
    tracing::info!("[ShortcutManager] Using command path: {command_path}");

    for shortcut in SHORTCUTS {
        let mut config = shortcut.clone();
        config.command = command_path;

        match handler.register(&config) {
            Ok(_) => tracing::info!("[ShortcutManager] ✓ Registered '{}'", config.name),
            Err(e) => tracing::warn!("[ShortcutManager] ✗ Failed '{}': {e}", config.name),
        }
    }
}

pub fn unregister_global_shortcut() {
    let handler = detect_handler();
    tracing::info!("[ShortcutManager] Environment: {}", handler.name());

    let command_path = get_command_path();

    for shortcut in SHORTCUTS {
        let mut config = shortcut.clone();
        config.command = command_path;

        match handler.unregister(&config) {
            Ok(_) => tracing::info!("[ShortcutManager] ✓ Unregistered '{}'", config.name),
            Err(e) => tracing::warn!("[ShortcutManager] ✗ Failed '{}': {e}", config.name),
        }
    }
}

fn detect_handler() -> Box<dyn ShortcutHandler> {
    let xdg_current = env_var("XDG_CURRENT_DESKTOP").to_lowercase();
    let xdg_session = env_var("XDG_SESSION_DESKTOP").to_lowercase();
    let combined = format!("{xdg_current} {xdg_session}");

    if combined.contains("gnome") || combined.contains("unity") || combined.contains("pantheon") {
        return Box::new(GnomeHandler);
    }
    if combined.contains("cinnamon") {
        return Box::new(CinnamonHandler);
    }
    if combined.contains("kde") || combined.contains("plasma") {
        return Box::new(KdeHandler);
    }
    if combined.contains("xfce") {
        return Box::new(XfceHandler);
    }
    if combined.contains("mate") {
        return Box::new(MateHandler);
    }
    if combined.contains("cosmic") {
        return Box::new(CosmicHandler);
    }
    if combined.contains("lxqt") {
        return Box::new(LxqtHandler);
    }
    if combined.contains("lxde") {
        return Box::new(LxdeHandler);
    }
    if combined.contains("budgie") {
        return Box::new(GnomeHandler);
    }
    if combined.contains("deepin") {
        return Box::new(GnomeHandler);
    }
    if combined.contains("i3") {
        return Box::new(I3Handler);
    }
    if combined.contains("sway") {
        return Box::new(SwayHandler);
    }
    if combined.contains("hyprland") {
        return Box::new(HyprlandHandler);
    }

    if is_process_running("i3") {
        return Box::new(I3Handler);
    }
    if is_process_running("sway") {
        return Box::new(SwayHandler);
    }
    if is_process_running("hyprland") || is_process_running("Hyprland") {
        return Box::new(HyprlandHandler);
    }

    if Utils::command_exists("kwriteconfig5") || Utils::command_exists("kwriteconfig6") {
        return Box::new(KdeHandler);
    }
    if Utils::command_exists("xfconf-query") {
        return Box::new(XfceHandler);
    }

    Box::new(GnomeHandler)
}

fn is_process_running(name: &str) -> bool {
    Command::new("pgrep")
        .arg("-x")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn env_var(key: &str) -> String {
    env::var(key).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wm_rewrite_defaults_off() {
        assert!(!allow_wm_config_rewrite());
        set_allow_wm_config_rewrite(true);
        assert!(allow_wm_config_rewrite());
        set_allow_wm_config_rewrite(false);
        assert!(!allow_wm_config_rewrite());
    }
}
