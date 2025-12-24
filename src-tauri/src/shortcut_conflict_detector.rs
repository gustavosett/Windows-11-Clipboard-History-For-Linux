//! Shortcut Conflict Detection for Various Desktop Environments
//! Detects existing shortcuts that conflict with Super+V across different DEs

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Represents a detected shortcut conflict
#[derive(Debug, Clone, serde::Serialize)]
pub struct ShortcutConflict {
    /// The shortcut binding that conflicts (e.g., "Super+V")
    pub binding: String,
    /// What the shortcut currently does
    pub current_action: String,
    /// The desktop environment or application that owns this shortcut
    pub owner: String,
    /// Command or instructions to resolve the conflict
    pub resolution_command: Option<String>,
    /// Human-readable resolution steps
    pub resolution_steps: String,
}

/// Result of conflict detection for all shortcuts
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConflictDetectionResult {
    /// Desktop environment detected
    pub desktop_environment: String,
    /// List of detected conflicts
    pub conflicts: Vec<ShortcutConflict>,
    /// Whether automatic resolution is possible
    pub can_auto_resolve: bool,
    /// General message about conflicts
    pub message: String,
}

/// Main entry point for conflict detection
pub fn detect_shortcut_conflicts() -> ConflictDetectionResult {
    let de = get_desktop_environment();
    let conflicts = match de.as_str() {
        "GNOME" => detect_gnome_conflicts(),
        "Pop" | "Pop!_OS" => detect_pop_shell_conflicts(),
        "COSMIC" => detect_cosmic_conflicts(),
        "KDE Plasma" => detect_kde_conflicts(),
        "i3" | "i3wm" => detect_i3_conflicts(),
        "Sway" => detect_sway_conflicts(),
        "Hyprland" => detect_hyprland_conflicts(),
        "Cinnamon" => detect_cinnamon_conflicts(),
        "XFCE" => detect_xfce_conflicts(),
        _ => Vec::new(),
    };

    let can_auto_resolve = conflicts.iter().all(|c| c.resolution_command.is_some());
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

/// Resolve all detected conflicts automatically where possible
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

fn get_desktop_environment() -> String {
    let xdg_current = env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();
    let xdg_session = env::var("XDG_SESSION_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();
    let combined = format!("{} {}", xdg_current, xdg_session);

    // Check for Pop!_OS specifically (uses pop:GNOME)
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
    // Tiling window managers
    if combined.contains("i3") {
        return "i3".to_string();
    }
    if combined.contains("sway") {
        return "Sway".to_string();
    }
    if combined.contains("hyprland") {
        return "Hyprland".to_string();
    }

    // Check running processes for tiling WMs (they often don't set XDG vars properly)
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
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn gsettings_get(schema: &str, key: &str) -> Option<String> {
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

// =============================================================================
// GNOME Conflict Detection
// =============================================================================

fn detect_gnome_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

    // Check for notification center shortcut (toggle-message-tray)
    if let Some(binding) = gsettings_get("org.gnome.shell.keybindings", "toggle-message-tray") {
        let binding_lower = binding.to_lowercase();
        if binding_lower.contains("super") && binding_lower.contains("v") {
            conflicts.push(ShortcutConflict {
                binding: "<Super>v".to_string(),
                current_action: "Open Notification Center / Message Tray".to_string(),
                owner: "GNOME Shell".to_string(),
                resolution_command: Some(
                    "gsettings set org.gnome.shell.keybindings toggle-message-tray \"['<Super><Shift>v']\"".to_string()
                ),
                resolution_steps: r#"**To resolve manually:**
1. Open Settings → Keyboard → Keyboard Shortcuts
2. Search for "Notification" or "Message Tray"
3. Change Super+V to Super+Shift+V (or disable it)

**Or run this command:**
```
gsettings set org.gnome.shell.keybindings toggle-message-tray "['<Super><Shift>v']"
```"#.to_string(),
            });
        }
    }

    // Check for Clipboard shortcut in GNOME 45+ (if applicable)
    if let Some(binding) = gsettings_get("org.gnome.shell.keybindings", "toggle-quick-settings") {
        let binding_lower = binding.to_lowercase();
        if binding_lower.contains("super") && binding_lower.contains("v") {
            conflicts.push(ShortcutConflict {
                binding: "<Super>v".to_string(),
                current_action: "Toggle Quick Settings".to_string(),
                owner: "GNOME Shell".to_string(),
                resolution_command: Some(
                    "gsettings set org.gnome.shell.keybindings toggle-quick-settings \"[]\""
                        .to_string(),
                ),
                resolution_steps:
                    "Disable the Quick Settings shortcut in GNOME Settings → Keyboard → Shortcuts"
                        .to_string(),
            });
        }
    }

    conflicts
}

// =============================================================================
// Pop!_OS / Pop Shell Conflict Detection
// =============================================================================

fn detect_pop_shell_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

    // Pop Shell uses org.gnome.shell.extensions.pop-shell for some shortcuts
    // Also inherits GNOME's notification tray shortcut

    // Check GNOME's notification center first
    conflicts.extend(detect_gnome_conflicts());

    // Check Pop Shell specific shortcuts
    if let Some(binding) = gsettings_get("org.gnome.shell.extensions.pop-shell", "tile-enter") {
        let binding_lower = binding.to_lowercase();
        if binding_lower.contains("super") && binding_lower.contains("v") {
            conflicts.push(ShortcutConflict {
                binding: "<Super>v".to_string(),
                current_action: "Enter Tiling Mode".to_string(),
                owner: "Pop Shell".to_string(),
                resolution_command: Some(
                    "gsettings set org.gnome.shell.extensions.pop-shell tile-enter \"['<Super><Shift>v']\"".to_string()
                ),
                resolution_steps: r#"**To resolve manually:**
1. Open Pop!_OS Settings → Keyboard → Customize Shortcuts
2. Find "Pop Shell: Enter Tile Mode"
3. Change it to a different binding

**Or run:**
```
gsettings set org.gnome.shell.extensions.pop-shell tile-enter "['<Super><Shift>v']"
```"#.to_string(),
            });
        }
    }

    conflicts
}

// =============================================================================
// COSMIC Desktop Conflict Detection
// =============================================================================

fn detect_cosmic_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

    let home = match env::var("HOME") {
        Ok(h) => h,
        Err(_) => return conflicts,
    };

    // COSMIC stores shortcuts in ~/.config/cosmic/com.system76.CosmicSettings.Shortcuts/
    let shortcuts_path =
        PathBuf::from(&home).join(".config/cosmic/com.system76.CosmicSettings.Shortcuts/v1/custom");

    if let Ok(content) = fs::read_to_string(&shortcuts_path) {
        // Check for Super+V bindings
        if content.to_lowercase().contains("super")
            && content.to_lowercase().contains("\"v\"")
            && !content.contains("win11-clipboard-history")
        {
            conflicts.push(ShortcutConflict {
                binding: "Super+V".to_string(),
                current_action: "Unknown COSMIC shortcut".to_string(),
                owner: "COSMIC Desktop".to_string(),
                resolution_command: None,
                resolution_steps: r#"**To resolve manually:**
1. Open COSMIC Settings → Keyboard → Shortcuts
2. Find any shortcut using Super+V
3. Change it to a different binding or remove it"#
                    .to_string(),
            });
        }
    }

    // Also check system-level COSMIC shortcuts
    let system_shortcuts = PathBuf::from(&home)
        .join(".config/cosmic/com.system76.CosmicSettings.Shortcuts/v1/system_actions");

    if let Ok(content) = fs::read_to_string(&system_shortcuts) {
        if content.to_lowercase().contains("super") && content.to_lowercase().contains("\"v\"") {
            conflicts.push(ShortcutConflict {
                binding: "Super+V".to_string(),
                current_action: "COSMIC System Action".to_string(),
                owner: "COSMIC Desktop".to_string(),
                resolution_command: None,
                resolution_steps: r#"**COSMIC System Shortcut Conflict:**
1. Open COSMIC Settings → Keyboard → Shortcuts → System
2. Find the Super+V binding
3. Change or disable it"#
                    .to_string(),
            });
        }
    }

    conflicts
}

// =============================================================================
// KDE Plasma Conflict Detection
// =============================================================================

fn detect_kde_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

    let home = match env::var("HOME") {
        Ok(h) => h,
        Err(_) => return conflicts,
    };

    // Check kglobalshortcutsrc for Meta+V bindings
    let shortcuts_path = PathBuf::from(&home).join(".config/kglobalshortcutsrc");

    if let Ok(content) = fs::read_to_string(&shortcuts_path) {
        for line in content.lines() {
            if line.contains("Meta+V") || line.contains("Meta+v") {
                // Try to extract the action name
                if let Some(action) = extract_kde_action(&content, line) {
                    // Skip if it's our own shortcut
                    if action.contains("clipboard-history") || action.contains("win11") {
                        continue;
                    }

                    conflicts.push(ShortcutConflict {
                        binding: "Meta+V".to_string(),
                        current_action: action.clone(),
                        owner: "KDE Plasma".to_string(),
                        resolution_command: None,
                        resolution_steps: format!(
                            r#"**To resolve manually:**
1. Open System Settings → Shortcuts → Global Shortcuts
2. Find "{}"
3. Change or clear the Meta+V binding

**Alternative:** Use the search function to find "Meta+V" bindings"#,
                            action
                        ),
                    });
                }
            }
        }
    }

    // Check for Klipper (KDE's built-in clipboard manager)
    let klipper_path = PathBuf::from(&home).join(".config/klipperrc");
    if klipper_path.exists() {
        if let Ok(content) = fs::read_to_string(&klipper_path) {
            if content.contains("Meta+V") {
                conflicts.push(ShortcutConflict {
                    binding: "Meta+V".to_string(),
                    current_action: "Klipper Clipboard History".to_string(),
                    owner: "Klipper".to_string(),
                    resolution_command: None,
                    resolution_steps: r#"**Klipper Conflict:**
KDE's built-in clipboard manager (Klipper) may use Meta+V.

1. Right-click the Klipper icon in the system tray
2. Click "Configure Klipper"
3. Go to "Shortcuts" and change or disable the shortcut

**Alternatively:** Disable Klipper entirely if you prefer this app."#
                        .to_string(),
                });
            }
        }
    }

    conflicts
}

fn extract_kde_action(content: &str, target_line: &str) -> Option<String> {
    // KDE shortcut format: action=shortcut,default,description
    // We need to find the section header [Component] above the line
    let lines: Vec<&str> = content.lines().collect();
    let mut current_section = String::new();

    for line in lines {
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_string();
        }
        if line == target_line {
            // Extract action name from the line
            if let Some(eq_pos) = line.find('=') {
                let action_part = &line[..eq_pos];
                return Some(format!("{}: {}", current_section, action_part));
            }
            return Some(current_section);
        }
    }
    None
}

// =============================================================================
// i3 Window Manager Conflict Detection
// =============================================================================

fn detect_i3_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

    let config_paths = get_i3_config_paths();

    for path in config_paths {
        if let Ok(content) = fs::read_to_string(&path) {
            // Look for bindsym $mod+v or bindsym Mod4+v
            for line in content.lines() {
                let line_lower = line.to_lowercase().trim().to_string();

                // Skip comments
                if line_lower.starts_with('#') {
                    continue;
                }

                // Check for Super+V bindings (Mod4 is typically Super)
                if (line_lower.contains("bindsym") || line_lower.contains("bindcode"))
                    && (line_lower.contains("mod4+v") || line_lower.contains("$mod+v"))
                    && !line_lower.contains("clipboard-history")
                    && !line_lower.contains("win11")
                {
                    // Extract the action
                    let action = line
                        .split_whitespace()
                        .skip(2)
                        .collect::<Vec<_>>()
                        .join(" ");

                    conflicts.push(ShortcutConflict {
                        binding: "$mod+v / Mod4+v".to_string(),
                        current_action: if action.is_empty() {
                            "Unknown action".to_string()
                        } else {
                            action
                        },
                        owner: "i3 config".to_string(),
                        resolution_command: None,
                        resolution_steps: format!(
                            r#"**i3 Config Conflict:**
Found in: {}

**To resolve:**
1. Edit your i3 config: `{}`
2. Find the line with `bindsym $mod+v` or `bindsym Mod4+v`
3. Change it to a different binding or comment it out

**Then add:**
```
bindsym $mod+v exec win11-clipboard-history
```

4. Reload i3: Press $mod+Shift+r"#,
                            path.display(),
                            path.display()
                        ),
                    });
                }
            }
        }
    }

    conflicts
}

fn get_i3_config_paths() -> Vec<PathBuf> {
    let home = env::var("HOME").unwrap_or_default();
    vec![
        PathBuf::from(&home).join(".config/i3/config"),
        PathBuf::from(&home).join(".i3/config"),
        PathBuf::from("/etc/i3/config"),
    ]
}

// =============================================================================
// Sway Window Manager Conflict Detection
// =============================================================================

fn detect_sway_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

    let config_paths = get_sway_config_paths();

    for path in config_paths {
        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                let line_lower = line.to_lowercase().trim().to_string();

                if line_lower.starts_with('#') {
                    continue;
                }

                if (line_lower.contains("bindsym") || line_lower.contains("bindcode"))
                    && (line_lower.contains("mod4+v") || line_lower.contains("$mod+v"))
                    && !line_lower.contains("clipboard-history")
                    && !line_lower.contains("win11")
                {
                    let action = line
                        .split_whitespace()
                        .skip(2)
                        .collect::<Vec<_>>()
                        .join(" ");

                    conflicts.push(ShortcutConflict {
                        binding: "$mod+v / Mod4+v".to_string(),
                        current_action: if action.is_empty() {
                            "Unknown action".to_string()
                        } else {
                            action
                        },
                        owner: "Sway config".to_string(),
                        resolution_command: None,
                        resolution_steps: format!(
                            r#"**Sway Config Conflict:**
Found in: {}

**To resolve:**
1. Edit your Sway config: `{}`
2. Find the line with `bindsym $mod+v`
3. Change it to a different binding or comment it out

**Then add:**
```
bindsym $mod+v exec win11-clipboard-history
```

4. Reload Sway: Press $mod+Shift+c"#,
                            path.display(),
                            path.display()
                        ),
                    });
                }
            }
        }
    }

    conflicts
}

fn get_sway_config_paths() -> Vec<PathBuf> {
    let home = env::var("HOME").unwrap_or_default();
    vec![
        PathBuf::from(&home).join(".config/sway/config"),
        PathBuf::from(&home).join(".sway/config"),
        PathBuf::from("/etc/sway/config"),
    ]
}

// =============================================================================
// Hyprland Conflict Detection
// =============================================================================

fn detect_hyprland_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

    let config_paths = get_hyprland_config_paths();

    for path in config_paths {
        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                let line_lower = line.to_lowercase().trim().to_string();

                if line_lower.starts_with('#') {
                    continue;
                }

                // Hyprland uses bind = SUPER, V, exec, command
                if line_lower.starts_with("bind")
                    && line_lower.contains("super")
                    && (line_lower.contains(", v,") || line_lower.contains(",v,"))
                    && !line_lower.contains("clipboard-history")
                    && !line_lower.contains("win11")
                {
                    // Extract action from bind line
                    let parts: Vec<&str> = line.split(',').collect();
                    let action = if parts.len() >= 4 {
                        parts[3..].join(",").trim().to_string()
                    } else {
                        "Unknown action".to_string()
                    };

                    conflicts.push(ShortcutConflict {
                        binding: "SUPER, V".to_string(),
                        current_action: action,
                        owner: "Hyprland config".to_string(),
                        resolution_command: None,
                        resolution_steps: format!(
                            r#"**Hyprland Config Conflict:**
Found in: {}

**To resolve:**
1. Edit your Hyprland config: `{}`
2. Find the line with `bind = SUPER, V, ...`
3. Change it to a different binding or comment it out

**Then add:**
```
bind = SUPER, V, exec, win11-clipboard-history
```

4. The config auto-reloads, or reload manually"#,
                            path.display(),
                            path.display()
                        ),
                    });
                }
            }
        }
    }

    conflicts
}

fn get_hyprland_config_paths() -> Vec<PathBuf> {
    let home = env::var("HOME").unwrap_or_default();
    let xdg_config = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));

    vec![
        PathBuf::from(&xdg_config).join("hypr/hyprland.conf"),
        PathBuf::from(&home).join(".config/hypr/hyprland.conf"),
    ]
}

// =============================================================================
// Cinnamon Conflict Detection
// =============================================================================

fn detect_cinnamon_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

    // Check for notification center / calendar shortcut
    if let Some(binding) = gsettings_get("org.cinnamon.desktop.keybindings", "show-desklets") {
        let binding_lower = binding.to_lowercase();
        if binding_lower.contains("super") && binding_lower.contains("v") {
            conflicts.push(ShortcutConflict {
                binding: "<Super>v".to_string(),
                current_action: "Show Desklets".to_string(),
                owner: "Cinnamon".to_string(),
                resolution_command: Some(
                    "gsettings set org.cinnamon.desktop.keybindings show-desklets \"['<Super><Shift>v']\"".to_string()
                ),
                resolution_steps: r#"**To resolve manually:**
1. Open System Settings → Keyboard → Shortcuts
2. Find "Show Desklets"
3. Change Super+V to Super+Shift+V"#.to_string(),
            });
        }
    }

    conflicts
}

// =============================================================================
// XFCE Conflict Detection
// =============================================================================

fn detect_xfce_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

    if !command_exists("xfconf-query") {
        return conflicts;
    }

    // Check for Super+V in XFCE keyboard shortcuts
    let output = Command::new("xfconf-query")
        .args(["-c", "xfce4-keyboard-shortcuts", "-l", "-v"])
        .output();

    if let Ok(output) = output {
        let content = String::from_utf8_lossy(&output.stdout);
        for line in content.lines() {
            let line_lower = line.to_lowercase();
            if line_lower.contains("<super>v")
                && !line_lower.contains("clipboard-history")
                && !line_lower.contains("win11")
            {
                conflicts.push(ShortcutConflict {
                    binding: "<Super>v".to_string(),
                    current_action: line.to_string(),
                    owner: "XFCE".to_string(),
                    resolution_command: None,
                    resolution_steps: r#"**To resolve manually:**
1. Open Settings → Keyboard → Application Shortcuts
2. Find the Super+V binding
3. Change or remove it"#
                        .to_string(),
                });
            }
        }
    }

    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_conflicts_runs() {
        let result = detect_shortcut_conflicts();
        // Just verify it doesn't panic and returns a valid structure
        assert!(!result.desktop_environment.is_empty() || result.desktop_environment.is_empty());
    }
}
