//! Tiling window manager shortcut backends: i3, Sway and Hyprland.
//!
//! These WMs are configured through plain-text config files. Edits are atomic
//! (temp file + rename) and — unless the user explicitly opts in — existing
//! `Super+V` bindings are never rewritten.

use super::shortcut_config::ShortcutConfig;
use super::shortcut_error::{Result, ShortcutError};
use super::shortcut_utils::Utils;
use super::allow_wm_config_rewrite;
use super::shortcut_handler::ShortcutHandler;
use std::env;
use std::path::PathBuf;
use tracing::info;

/// Check if a line contains a `$mod+v` or `mod4+v` binding with proper word
/// boundaries. This ensures we match "bindsym $mod+v" even at end of line or
/// followed by comments.
fn has_mod_v_binding(trimmed_line: &str) -> bool {
    for pattern in &["$mod+v", "mod4+v"] {
        if let Some(idx) = trimmed_line.find(pattern) {
            // Check what follows the pattern
            let after = trimmed_line[idx + pattern.len()..].chars().next();
            // Valid word boundaries: end of string, space, tab, comment, semicolon
            if matches!(after, None | Some(' ') | Some('\t') | Some('#') | Some(';')) {
                return true;
            }
        }
    }
    false
}

/// Comment marker used when an existing binding is disabled (opt-in only).
const WM_COMMENT_MARKER: &str = "# Commented by windows-11-style-clipboard-history-manager";
const WM_ADDED_MARKER: &str = "# Clipboard History (added by windows-11-style-clipboard-history-manager)";

/// Appends `binding_line` to a WM config file.
/// Existing conflicting bindings are only commented out when
/// `allow_wm_config_rewrite()` is enabled (user opt-in).
fn append_wm_binding(
    path: &PathBuf,
    binding_line: &str,
    full_cmd: &str,
    comment_predicate: impl Fn(&str) -> bool,
) -> Result<bool> {
    Utils::modify_file_atomic(path, |content| {
        if content.contains(full_cmd) {
            return Ok(None); // Already registered
        }

        let mut lines: Vec<String> = content.lines().map(String::from).collect();
        let mut had_existing = false;

        if allow_wm_config_rewrite() {
            for line in lines.iter_mut() {
                let trimmed = line.trim().to_lowercase();
                // Skip if already a comment
                if trimmed.starts_with('#') {
                    continue;
                }
                if comment_predicate(&trimmed) {
                    *line = format!("# {line} {WM_COMMENT_MARKER}");
                    had_existing = true;
                }
            }
        }

        lines.push(format!("\n{WM_ADDED_MARKER}"));
        lines.push(binding_line.to_string());

        if had_existing {
            info!("[TilingWM] Commented out existing binding(s)");
        }

        Ok(Some(lines.join("\n")))
    })
}

/// Removes our added line and restores any bindings we commented out.
fn restore_wm_binding(path: &PathBuf, full_cmd: &str) -> Result<bool> {
    Utils::modify_file_atomic(path, |content| {
        if !content.contains(full_cmd) {
            return Ok(None);
        }

        let lines: Vec<&str> = content.lines().collect();
        let mut new_lines: Vec<String> = Vec::new();
        let mut skip_comment = false;

        for line in lines {
            // Skip our comment line
            if line.contains(WM_ADDED_MARKER) {
                skip_comment = true;
                continue;
            }
            // Skip our binding line
            if skip_comment && line.contains(full_cmd) {
                skip_comment = false;
                continue;
            }
            skip_comment = false;

            // Restore commented out bindings
            if line.contains(WM_COMMENT_MARKER) {
                let restored = line
                    .replace("# ", "")
                    .replace(&format!(" {WM_COMMENT_MARKER}"), "");
                new_lines.push(restored);
            } else {
                new_lines.push(line.to_string());
            }
        }

        Ok(Some(new_lines.join("\n")))
    })
}

/// Resolve `$XDG_CONFIG_HOME`-aware home directory helper.
fn home_dir() -> Result<PathBuf> {
    env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))
}

// --- i3 Window Manager ---

pub struct I3Handler;
impl I3Handler {
    fn get_config_path() -> Result<PathBuf> {
        let home = home_dir()?;

        // Check common i3 config locations
        let paths = vec![
            PathBuf::from(&home).join(".config/i3/config"),
            PathBuf::from(&home).join(".i3/config"),
        ];

        for path in paths {
            if path.exists() {
                return Ok(path);
            }
        }

        // Default to the XDG config path
        Ok(PathBuf::from(&home).join(".config/i3/config"))
    }

    fn reload_i3() {
        // Send reload command to i3
        let _ = Utils::run("i3-msg", &["reload"]);
    }
}

impl ShortcutHandler for I3Handler {
    fn name(&self) -> &str {
        "i3"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;
        let full_cmd = s.full_command();
        let binding_line = format!("bindsym {} exec {full_cmd}", s.i3_binding);

        let modified = append_wm_binding(&path, &binding_line, &full_cmd, |trimmed| {
            trimmed.starts_with("bindsym") && has_mod_v_binding(trimmed)
        })?;

        // Reload i3 only after file was successfully written
        if modified {
            Self::reload_i3();
        }
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;

        if !path.exists() {
            return Ok(());
        }

        let full_cmd = s.full_command();
        let modified = restore_wm_binding(&path, &full_cmd)?;

        // Reload i3 only after file was successfully written
        if modified {
            Self::reload_i3();
        }
        Ok(())
    }
}

// --- Sway ---

pub struct SwayHandler;
impl SwayHandler {
    fn get_config_path() -> Result<PathBuf> {
        let home = home_dir()?;

        let paths = vec![
            PathBuf::from(&home).join(".config/sway/config"),
            PathBuf::from(&home).join(".sway/config"),
        ];

        for path in paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Ok(PathBuf::from(&home).join(".config/sway/config"))
    }

    fn reload_sway() {
        let _ = Utils::run("swaymsg", &["reload"]);
    }
}

impl ShortcutHandler for SwayHandler {
    fn name(&self) -> &str {
        "Sway"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;
        let full_cmd = s.full_command();
        let binding_line = format!("bindsym {} exec {full_cmd}", s.sway_binding);

        let modified = append_wm_binding(&path, &binding_line, &full_cmd, |trimmed| {
            trimmed.starts_with("bindsym") && has_mod_v_binding(trimmed)
        })?;

        // Reload Sway only after file was successfully written
        if modified {
            Self::reload_sway();
        }
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;

        if !path.exists() {
            return Ok(());
        }

        let full_cmd = s.full_command();
        let modified = restore_wm_binding(&path, &full_cmd)?;

        // Reload Sway only after file was successfully written
        if modified {
            Self::reload_sway();
        }
        Ok(())
    }
}

// --- Hyprland ---

pub struct HyprlandHandler;
impl HyprlandHandler {
    fn get_config_path() -> Result<PathBuf> {
        let home = home_dir()?;
        let xdg_config =
            env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home.display()));

        let path = PathBuf::from(&xdg_config).join("hypr/hyprland.conf");
        Ok(path)
    }
}

impl ShortcutHandler for HyprlandHandler {
    fn name(&self) -> &str {
        "Hyprland"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;
        let full_cmd = s.full_command();

        // Hyprland format: bind = SUPER, V, exec, command
        let binding_line = format!("bind = {}, exec, {full_cmd}", s.hyprland_binding);

        let _modified = append_wm_binding(&path, &binding_line, &full_cmd, |trimmed| {
            trimmed.starts_with("bind")
                && trimmed.contains("super")
                && (trimmed.contains(", v,") || trimmed.contains(",v,"))
        })?;

        // Hyprland auto-reloads config, no explicit reload needed
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;

        if !path.exists() {
            return Ok(());
        }

        let full_cmd = s.full_command();
        let _modified = restore_wm_binding(&path, &full_cmd)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_v_binding_boundaries() {
        assert!(has_mod_v_binding("bindsym $mod+v exec foo"));
        assert!(has_mod_v_binding("bindsym $mod+v"));
        assert!(has_mod_v_binding("bindsym mod4+v exec foo"));
        assert!(!has_mod_v_binding("bindsym $mod+vim exec foo"));
        assert!(!has_mod_v_binding("bindsym $mod+shift+v exec foo"));
    }

    #[test]
    fn comment_markers_are_consistent() {
        assert!(WM_COMMENT_MARKER.starts_with("# "));
        assert!(WM_ADDED_MARKER.starts_with("# "));
    }
}
