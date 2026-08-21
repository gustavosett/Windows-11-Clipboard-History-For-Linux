//! XFCE conflict detector. / تشخیص تداخل XFCE.

use super::{command_exists, ShortcutConflict};
use std::process::Command;

pub fn detect_xfce_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

    if !command_exists("xfconf-query") {
        return conflicts;
    }

    let output = Command::new("xfconf-query")
        .args(["-c", "xfce4-keyboard-shortcuts", "-l", "-v"])
        .output();

    if let Ok(output) = output {
        let content = String::from_utf8_lossy(&output.stdout);
        for line in content.lines() {
            let line_lower = line.to_lowercase();
            if line_lower.contains("<super>v")
                && !line_lower.contains("clipboard-history")
                && !line_lower.contains("windows-11-style-clipboard-history")
                && !line_lower.contains("modern-clipboard-history")
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
