//! KDE Plasma / Klipper conflict detectors.
//! / تشخیص تداخل KDE Plasma و Klipper.

use super::ShortcutConflict;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn detect_kde_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

    let home = match env::var("HOME") {
        Ok(h) => h,
        Err(_) => return conflicts,
    };

    let shortcuts_path = PathBuf::from(&home).join(".config/kglobalshortcutsrc");

    if let Ok(content) = fs::read_to_string(&shortcuts_path) {
        for line in content.lines() {
            if line.contains("Meta+V") || line.contains("Meta+v") {
                if let Some(action) = extract_kde_action(&content, line) {
                    if action.contains("clipboard-history") || action.contains("windows-11-style-clipboard-history") || action.contains("modern-clipboard-history") {
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
    let lines: Vec<&str> = content.lines().collect();
    let mut current_section = String::new();

    for line in lines {
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_string();
        }
        if line == target_line {
            if let Some(eq_pos) = line.find('=') {
                let action_part = &line[..eq_pos];
                return Some(format!("{}: {}", current_section, action_part));
            }
            return Some(current_section);
        }
    }
    None
}
