//! GNOME / Pop Shell / Cinnamon conflict detectors.
//! / تشخیص تداخل GNOME، Pop Shell و Cinnamon.

use super::{gsettings_get, ShortcutConflict};

pub fn detect_gnome_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

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

pub fn detect_pop_shell_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = detect_gnome_conflicts();

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

pub fn detect_cinnamon_conflicts() -> Vec<ShortcutConflict> {
    let mut conflicts = Vec::new();

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
