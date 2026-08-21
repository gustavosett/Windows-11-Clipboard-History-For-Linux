//! i3 / Sway / Hyprland / COSMIC conflict detectors.
//! / تشخیص تداخل مدیرهای پنجرهٔ کاشی‌وار و COSMIC.

use super::ShortcutConflict;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn detect_cosmic_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

    let home = match env::var("HOME") {
        Ok(h) => h,
        Err(_) => return conflicts,
    };

    let shortcuts_path =
        PathBuf::from(&home).join(".config/cosmic/com.system76.CosmicSettings.Shortcuts/v1/custom");

    if let Ok(content) = fs::read_to_string(&shortcuts_path) {
        if content.to_lowercase().contains("super")
            && content.to_lowercase().contains("\"v\"")
            && !content.contains("windows-11-style-clipboard-history-manager")
            && !content.contains("modern-clipboard-history-for-linux")
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

pub fn detect_i3_conflicts() -> Vec<ShortcutConflict> {
    detect_bindsym_conflicts(
        &get_i3_config_paths(),
        "i3 config",
        r#"**i3 Config Conflict:**
Found in: {path}

**To resolve:**
1. Edit your i3 config: `{path}`
2. Find the line with `bindsym $mod+v` or `bindsym Mod4+v`
3. Change it to a different binding or comment it out

**Then add:**
```
bindsym $mod+v exec windows-11-style-clipboard-history-manager
```

4. Reload i3: Press $mod+Shift+r"#,
    )
}

pub fn detect_sway_conflicts() -> Vec<ShortcutConflict> {
    detect_bindsym_conflicts(
        &get_sway_config_paths(),
        "Sway config",
        r#"**Sway Config Conflict:**
Found in: {path}

**To resolve:**
1. Edit your Sway config: `{path}`
2. Find the line with `bindsym $mod+v`
3. Change it to a different binding or comment it out

**Then add:**
```
bindsym $mod+v exec windows-11-style-clipboard-history-manager
```

4. Reload Sway: Press $mod+Shift+c"#,
    )
}

fn detect_bindsym_conflicts(paths: &[PathBuf], owner: &str, steps: &str) -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

    for path in paths {
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let line_lower = line.to_lowercase().trim().to_string();

                if line_lower.starts_with('#') {
                    continue;
                }

                if (line_lower.contains("bindsym") || line_lower.contains("bindcode"))
                    && (line_lower.contains("mod4+v") || line_lower.contains("$mod+v"))
                    && !line_lower.contains("clipboard-history")
                    && !line_lower.contains("windows-11-style-clipboard-history")
                    && !line_lower.contains("modern-clipboard-history")
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
                        owner: owner.to_string(),
                        resolution_command: None,
                        resolution_steps: steps.replace("{path}", &path.display().to_string()),
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

fn get_sway_config_paths() -> Vec<PathBuf> {
    let home = env::var("HOME").unwrap_or_default();
    vec![
        PathBuf::from(&home).join(".config/sway/config"),
        PathBuf::from(&home).join(".sway/config"),
        PathBuf::from("/etc/sway/config"),
    ]
}

pub fn detect_hyprland_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

    for path in get_hyprland_config_paths() {
        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                let line_lower = line.to_lowercase().trim().to_string();

                if line_lower.starts_with('#') {
                    continue;
                }

                if line_lower.starts_with("bind")
                    && line_lower.contains("super")
                    && (line_lower.contains(", v,") || line_lower.contains(",v,"))
                    && !line_lower.contains("clipboard-history")
                    && !line_lower.contains("windows-11-style-clipboard-history")
                    && !line_lower.contains("modern-clipboard-history")
                {
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
bind = SUPER, V, exec, windows-11-style-clipboard-history-manager
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
