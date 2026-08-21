//! Authorized shortcut discovery and registration for Setup/Settings.
//! کشف و ثبت مجوزدار میانبر برای پنجره‌های راه‌اندازی/تنظیمات.

use std::env;

use tauri::WebviewWindow;

use crate::error::AppError;
use crate::shortcut_conflict_detector::{
    auto_resolve_conflicts, detect_shortcut_conflicts, ConflictDetectionResult,
};
use crate::window_policy;

/// Detect the current desktop environment for native callers.
/// تشخیص محیط دسکتاپ فعلی برای فراخوان‌های native.
fn desktop_environment() -> String {
    let current = env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();
    let session = env::var("XDG_SESSION_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();
    let combined = format!("{current} {session}");

    if combined.contains("pop") {
        return "Pop!_OS".to_string();
    }

    let known = [
        (("gnome", "unity", "pantheon"), "GNOME"),
        (("cinnamon", "", ""), "Cinnamon"),
        (("kde", "plasma", ""), "KDE Plasma"),
        (("xfce", "", ""), "XFCE"),
        (("mate", "", ""), "MATE"),
        (("lxde", "", ""), "LXDE"),
        (("lxqt", "", ""), "LXQt"),
        (("cosmic", "", ""), "COSMIC"),
        (("budgie", "", ""), "Budgie"),
        (("deepin", "", ""), "Deepin"),
        (("i3", "", ""), "i3"),
        (("sway", "", ""), "Sway"),
        (("hyprland", "", ""), "Hyprland"),
    ];
    for ((first, second, third), label) in known {
        if [first, second, third]
            .iter()
            .any(|needle| !needle.is_empty() && combined.contains(needle))
        {
            return label.to_string();
        }
    }

    for (process, label) in [("i3", "i3"), ("sway", "Sway"), ("Hyprland", "Hyprland")] {
        if is_process_running(process) {
            return label.to_string();
        }
    }

    current.to_uppercase()
}

/// Return the desktop environment to configuration windows only.
/// بازگرداندن محیط دسکتاپ فقط به پنجره‌های پیکربندی.
#[tauri::command]
pub fn get_desktop_environment(window: WebviewWindow) -> Result<String, AppError> {
    window_policy::require_configuration(&window)?;
    Ok(desktop_environment())
}

fn is_process_running(name: &str) -> bool {
    std::process::Command::new("pgrep")
        .arg("-x")
        .arg(name)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Detect `Super+V` conflicts without modifying desktop configuration.
/// تشخیص تداخل `Super+V` بدون تغییر پیکربندی دسکتاپ.
#[tauri::command]
pub fn detect_conflicts(window: WebviewWindow) -> Result<ConflictDetectionResult, AppError> {
    window_policy::require_configuration(&window)?;
    Ok(detect_shortcut_conflicts())
}

/// Resolve detected conflicts from first-run Setup only.
/// رفع تداخل‌های شناسایی‌شده فقط از راه‌اندازی نخست.
#[tauri::command]
pub fn resolve_conflicts(window: WebviewWindow) -> Result<Vec<String>, AppError> {
    window_policy::require_setup(&window)?;
    auto_resolve_conflicts().map_err(AppError::Other)
}

/// Register application shortcuts after an explicit configuration action.
/// ثبت میانبرهای برنامه پس از اقدام صریح در پنجرهٔ پیکربندی.
#[tauri::command]
pub fn register_de_shortcut(window: WebviewWindow) -> Result<String, AppError> {
    window_policy::require_configuration(&window)?;
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        crate::linux_shortcut_manager::register_global_shortcut();
        let _ = sender.send(());
    });

    receiver
        .recv()
        .map(|()| "shortcut_registration_completed".to_string())
        .map_err(|_| AppError::Other("shortcut_registration_thread_failed".into()))
}

/// Report tools available for desktop shortcut registration.
/// گزارش ابزارهای در دسترس برای ثبت میانبر دسکتاپ.
#[tauri::command]
pub fn check_shortcut_tools(window: WebviewWindow) -> Result<ShortcutToolsStatus, AppError> {
    window_policy::require_configuration(&window)?;
    let gsettings = command_exists("gsettings");
    let kwriteconfig5 = command_exists("kwriteconfig5");
    let kwriteconfig6 = command_exists("kwriteconfig6");
    let xfconf_query = command_exists("xfconf-query");
    let dconf = command_exists("dconf");
    let desktop = desktop_environment();

    let can_register = match desktop.as_str() {
        "GNOME" | "Pop!_OS" | "Cinnamon" | "MATE" | "Budgie" | "Deepin" => {
            gsettings || dconf
        }
        "KDE Plasma" => kwriteconfig5 || kwriteconfig6,
        "XFCE" => xfconf_query,
        "LXQt" | "LXDE" | "COSMIC" | "i3" | "Sway" | "Hyprland" => true,
        _ => gsettings,
    };
    let conflicts = detect_shortcut_conflicts();

    Ok(ShortcutToolsStatus {
        desktop_environment: desktop,
        gsettings_available: gsettings,
        kde_tools_available: kwriteconfig5 || kwriteconfig6,
        xfce_tools_available: xfconf_query,
        can_register_automatically: can_register,
        has_conflicts: !conflicts.conflicts.is_empty(),
        conflict_count: conflicts.conflicts.len(),
        can_auto_resolve_conflicts: conflicts.can_auto_resolve,
    })
}

#[derive(serde::Serialize)]
pub struct ShortcutToolsStatus {
    pub desktop_environment: String,
    pub gsettings_available: bool,
    pub kde_tools_available: bool,
    pub xfce_tools_available: bool,
    pub can_register_automatically: bool,
    pub has_conflicts: bool,
    pub conflict_count: usize,
    pub can_auto_resolve_conflicts: bool,
}

/// In-process `PATH` probe — no `which` subprocess (see `crate::exec_lookup`).
/// بررسی درون‌فرآیندی PATH — بدون subprocess ی `which` (`crate::exec_lookup`).
fn command_exists(command: &str) -> bool {
    crate::exec_lookup::command_exists(command)
}
