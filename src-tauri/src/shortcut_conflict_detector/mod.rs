//! Shortcut conflict detection across desktop environments.
//! / تشخیص تداخل میانبر در محیط‌های دسکتاپ مختلف.
//!
//! Per-DE detectors live in sibling modules so each backend stays testable.
//! تشخیص‌گر هر محیط در ماژول جداست تا تست‌پذیر بماند.

mod gnome;
mod kde;
mod tiling;
mod xfce;

use std::env;
use std::process::Command;

/// A detected shortcut conflict. / یک تداخل میانبر شناسایی‌شده.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ShortcutConflict {
    pub binding: String,
    pub current_action: String,
    pub owner: String,
    pub resolution_command: Option<String>,
    pub resolution_steps: String,
}

/// Result of conflict detection for all shortcuts. / نتیجهٔ تشخیص برای همهٔ میانبرها.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConflictDetectionResult {
    pub desktop_environment: String,
    pub conflicts: Vec<ShortcutConflict>,
    pub can_auto_resolve: bool,
    pub message: String,
}

/// Main entry point for conflict detection. / نقطهٔ ورود تشخیص تداخل.
pub fn detect_shortcut_conflicts() -> ConflictDetectionResult {
    let de = get_desktop_environment();
    let conflicts = match de.as_str() {
        "GNOME" => gnome::detect_gnome_conflicts(),
        "Pop" | "Pop!_OS" => gnome::detect_pop_shell_conflicts(),
        "COSMIC" => tiling::detect_cosmic_conflicts(),
        "KDE Plasma" => kde::detect_kde_conflicts(),
        "i3" | "i3wm" => tiling::detect_i3_conflicts(),
        "Sway" => tiling::detect_sway_conflicts(),
        "Hyprland" => tiling::detect_hyprland_conflicts(),
        "Cinnamon" => gnome::detect_cinnamon_conflicts(),
        "XFCE" => xfce::detect_xfce_conflicts(),
        _ => Vec::new(),
    };

    let can_auto_resolve =
        !conflicts.is_empty() && conflicts.iter().all(|c| c.resolution_command.is_some());
    let message = if conflicts.is_empty() {
        "No shortcut conflicts detected.".to_string()
    } else {
        format!(
            "{} shortcut conflict(s) detected that may prevent Super+V from working.",
            conflicts.len()
        )
    };

    ConflictDetectionResult {
        desktop_environment: de,
        conflicts,
        can_auto_resolve,
        message,
    }
}

/// Resolve all detected conflicts automatically where possible.
/// / حل خودکار تداخل‌های قابل‌حل.
pub fn auto_resolve_conflicts() -> Result<Vec<String>, String> {
    let result = detect_shortcut_conflicts();
    let mut resolved = Vec::new();

    for conflict in result.conflicts {
        if let Some(cmd) = conflict.resolution_command {
            match run_resolution_command(&cmd) {
                Ok(_) => resolved.push(format!(
                    "Resolved: {} ({})",
                    conflict.owner, conflict.binding
                )),
                Err(e) => return Err(format!("Failed to resolve {}: {}", conflict.owner, e)),
            }
        }
    }

    Ok(resolved)
}

pub(super) fn get_desktop_environment() -> String {
    let xdg_current = env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();
    let xdg_session = env::var("XDG_SESSION_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();
    let combined = format!("{} {}", xdg_current, xdg_session);

    if combined.contains("pop") {
        return "Pop".to_string();
    }
    if combined.contains("gnome") || combined.contains("unity") || combined.contains("pantheon") {
        return "GNOME".to_string();
    }
    if combined.contains("cosmic") {
        return "COSMIC".to_string();
    }
    if combined.contains("kde") || combined.contains("plasma") {
        return "KDE Plasma".to_string();
    }
    if combined.contains("cinnamon") {
        return "Cinnamon".to_string();
    }
    if combined.contains("xfce") {
        return "XFCE".to_string();
    }
    if combined.contains("i3") {
        return "i3".to_string();
    }
    if combined.contains("sway") {
        return "Sway".to_string();
    }
    if combined.contains("hyprland") {
        return "Hyprland".to_string();
    }

    if is_process_running("i3") {
        return "i3".to_string();
    }
    if is_process_running("sway") {
        return "Sway".to_string();
    }
    if is_process_running("hyprland") || is_process_running("Hyprland") {
        return "Hyprland".to_string();
    }

    xdg_current.to_uppercase()
}

fn is_process_running(name: &str) -> bool {
    Command::new("pgrep")
        .arg("-x")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_resolution_command(cmd: &str) -> Result<(), String> {
    let args = parse_resolution_argv(cmd)?;
    if args.is_empty() {
        return Err("Empty resolution command".into());
    }
    let program = &args[0];
    if program != "gsettings" && program != "xfconf-query" {
        return Err(format!("Refusing to run untrusted resolver '{program}'"));
    }
    let output = Command::new(program)
        .args(&args[1..])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Split a trusted, quote-aware argv without invoking a shell.
/// / جداسازی argv بدون شل.
fn parse_resolution_argv(cmd: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = cmd.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    while let Some(c) = chars.next() {
        match c {
            '\\' if in_double => {
                if let Some(n) = chars.next() {
                    current.push(n);
                }
            }
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            c if c.is_whitespace() && !in_single && !in_double => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }
    if in_single || in_double {
        return Err("Unbalanced quotes in resolution command".into());
    }
    if !current.is_empty() {
        args.push(current);
    }
    Ok(args)
}

/// In-process `PATH` probe — no `which` subprocess (see `crate::exec_lookup`).
/// بررسی درون‌فرآیندی PATH — بدون subprocess ی `which` (`crate::exec_lookup`).
pub(super) fn command_exists(cmd: &str) -> bool {
    crate::exec_lookup::command_exists(cmd)
}

pub(super) fn gsettings_get(schema: &str, key: &str) -> Option<String> {
    if !command_exists("gsettings") {
        return None;
    }
    Command::new("gsettings")
        .args(["get", schema, key])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_conflicts_runs() {
        let _result = detect_shortcut_conflicts();
    }

    #[test]
    fn parse_gsettings_argv_keeps_quoted_value() {
        let args = parse_resolution_argv(
            r#"gsettings set org.gnome.shell.keybindings toggle-message-tray "['<Super><Shift>v']""#,
        )
        .unwrap();
        assert_eq!(args[0], "gsettings");
        assert_eq!(args[1], "set");
        assert_eq!(args.last().unwrap(), "['<Super><Shift>v']");
    }
}
