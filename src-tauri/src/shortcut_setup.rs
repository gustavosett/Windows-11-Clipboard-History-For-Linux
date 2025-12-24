//! Shortcut setup commands for the frontend
//! Provides Tauri commands to register/unregister shortcuts from the Setup Wizard

use std::env;

use crate::shortcut_conflict_detector::{
    auto_resolve_conflicts, detect_shortcut_conflicts, ConflictDetectionResult,
};

/// Get the current desktop environment name
#[tauri::command]
pub fn get_desktop_environment() -> String {
    let xdg_current = env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();
    let xdg_session = env::var("XDG_SESSION_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();
    let combined = format!("{} {}", xdg_current, xdg_session);

    // Check for Pop!_OS specifically (uses pop:GNOME)
    if combined.contains("pop") {
        return "Pop!_OS".to_string();
    }
    if combined.contains("gnome") || combined.contains("unity") || combined.contains("pantheon") {
        "GNOME".to_string()
    } else if combined.contains("cinnamon") {
        "Cinnamon".to_string()
    } else if combined.contains("kde") || combined.contains("plasma") {
        "KDE Plasma".to_string()
    } else if combined.contains("xfce") {
        "XFCE".to_string()
    } else if combined.contains("mate") {
        "MATE".to_string()
    } else if combined.contains("lxde") {
        "LXDE".to_string()
    } else if combined.contains("lxqt") {
        "LXQt".to_string()
    } else if combined.contains("cosmic") {
        "COSMIC".to_string()
    } else if combined.contains("budgie") {
        "Budgie".to_string()
    } else if combined.contains("deepin") {
        "Deepin".to_string()
    } else if combined.contains("i3") {
        "i3".to_string()
    } else if combined.contains("sway") {
        "Sway".to_string()
    } else if combined.contains("hyprland") {
        "Hyprland".to_string()
    } else {
        // Check for running tiling WMs
        if is_process_running("i3") {
            "i3".to_string()
        } else if is_process_running("sway") {
            "Sway".to_string()
        } else if is_process_running("hyprland") || is_process_running("Hyprland") {
            "Hyprland".to_string()
        } else {
            xdg_current.to_uppercase()
        }
    }
}

#[cfg(target_os = "linux")]
fn is_process_running(name: &str) -> bool {
    std::process::Command::new("pgrep")
        .arg("-x")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "linux"))]
fn is_process_running(_name: &str) -> bool {
    false
}

/// Detect shortcut conflicts for Super+V
#[tauri::command]
pub fn detect_conflicts() -> ConflictDetectionResult {
    detect_shortcut_conflicts()
}

/// Automatically resolve detected conflicts
#[tauri::command]
pub fn resolve_conflicts() -> Result<Vec<String>, String> {
    auto_resolve_conflicts()
}

/// Register the global shortcut with the desktop environment
/// This calls the existing linux_shortcut_manager
#[tauri::command]
pub fn register_de_shortcut() -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        // Run in a separate thread but wait for completion to avoid race conditions
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            crate::linux_shortcut_manager::register_global_shortcut();
            let _ = tx.send(());
        });

        match rx.recv() {
            Ok(()) => {
                Ok("Shortcut registration completed. Check the app logs for details.".to_string())
            }
            Err(_) => Err("Shortcut registration thread failed unexpectedly.".to_string()),
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err("Shortcut registration is only supported on Linux.".to_string())
    }
}

/// Check if the DE shortcut manager has the tools needed
#[tauri::command]
pub fn check_shortcut_tools() -> ShortcutToolsStatus {
    #[cfg(target_os = "linux")]
    {
        let gsettings = command_exists("gsettings");
        let kwriteconfig5 = command_exists("kwriteconfig5");
        let kwriteconfig6 = command_exists("kwriteconfig6");
        let xfconf_query = command_exists("xfconf-query");
        let dconf = command_exists("dconf");

        let de = get_desktop_environment();

        let can_register = match de.as_str() {
            "GNOME" | "Pop!_OS" | "Cinnamon" | "MATE" | "Budgie" | "Deepin" => gsettings || dconf,
            "KDE Plasma" => kwriteconfig5 || kwriteconfig6,
            "XFCE" => xfconf_query,
            "LXQt" => true,     // Uses config files
            "LXDE" => true,     // Uses config files
            "COSMIC" => true,   // Uses config files
            "i3" => true,       // Uses config files
            "Sway" => true,     // Uses config files
            "Hyprland" => true, // Uses config files
            _ => gsettings,     // Fallback to gsettings
        };

        // Check for conflicts
        let conflicts = detect_shortcut_conflicts();

        ShortcutToolsStatus {
            desktop_environment: de.clone(),
            gsettings_available: gsettings,
            kde_tools_available: kwriteconfig5 || kwriteconfig6,
            xfce_tools_available: xfconf_query,
            can_register_automatically: can_register,
            manual_instructions: get_manual_instructions(&de),
            has_conflicts: !conflicts.conflicts.is_empty(),
            conflict_count: conflicts.conflicts.len(),
            can_auto_resolve_conflicts: conflicts.can_auto_resolve,
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        ShortcutToolsStatus {
            desktop_environment: "Unknown".to_string(),
            gsettings_available: false,
            kde_tools_available: false,
            xfce_tools_available: false,
            can_register_automatically: false,
            manual_instructions: "This feature is only available on Linux.".to_string(),
            has_conflicts: false,
            conflict_count: 0,
            can_auto_resolve_conflicts: false,
        }
    }
}

#[derive(serde::Serialize)]
pub struct ShortcutToolsStatus {
    pub desktop_environment: String,
    pub gsettings_available: bool,
    pub kde_tools_available: bool,
    pub xfce_tools_available: bool,
    pub can_register_automatically: bool,
    pub manual_instructions: String,
    pub has_conflicts: bool,
    pub conflict_count: usize,
    pub can_auto_resolve_conflicts: bool,
}

#[cfg(target_os = "linux")]
fn command_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn get_manual_instructions(de: &str) -> String {
    match de {
        "GNOME" => r#"**GNOME Settings:**
1. Open Settings → Keyboard → Keyboard Shortcuts → Custom Shortcuts
2. Click "+" to add a new shortcut
3. Name: "Clipboard History"
4. Command: `win11-clipboard-history`
5. Shortcut: Press Super+V

**⚠️ Note:** GNOME uses Super+V for the Notification Center by default.
To free up Super+V, run:
```
gsettings set org.gnome.shell.keybindings toggle-message-tray "['<Super><Shift>v']"
```"#
            .to_string(),

        "Pop!_OS" => r#"**Pop!_OS / Pop Shell:**
1. Open Settings → Keyboard → Keyboard Shortcuts → Custom Shortcuts
2. Add a new shortcut with command: `win11-clipboard-history`
3. Set the shortcut to Super+V

**⚠️ Note:** Pop!_OS inherits GNOME's Super+V for Notification Center.
To free up Super+V, run:
```
gsettings set org.gnome.shell.keybindings toggle-message-tray "['<Super><Shift>v']"
```

If Pop Shell uses Super+V for tiling, change it in:
Settings → Keyboard → Customize Shortcuts → Pop Shell"#
            .to_string(),

        "KDE Plasma" => r#"**KDE System Settings:**
1. Open System Settings → Shortcuts → Custom Shortcuts
2. Click "Edit" → "New" → "Global Shortcut" → "Command/URL"
3. Name: "Clipboard History"
4. Trigger: Click and press Meta+V
5. Action: `win11-clipboard-history`

**⚠️ Note:** If Klipper (KDE's clipboard) uses Meta+V:
1. Right-click Klipper in system tray → Configure
2. Change or disable its shortcut"#
            .to_string(),

        "Cinnamon" => r#"**Cinnamon Settings:**
1. Open System Settings → Keyboard → Shortcuts → Custom Shortcuts
2. Click "Add custom shortcut"
3. Name: "Clipboard History"
4. Command: `win11-clipboard-history`
5. Click on the shortcut area and press Super+V"#
            .to_string(),

        "XFCE" => r#"**XFCE Settings:**
1. Open Settings → Keyboard → Application Shortcuts
2. Click "Add"
3. Command: `win11-clipboard-history`
4. Press Super+V when prompted"#
            .to_string(),

        "MATE" => r#"**MATE Control Center:**
1. Open Control Center → Keyboard Shortcuts
2. Click "Add"
3. Name: "Clipboard History"
4. Command: `win11-clipboard-history`
5. Click on the shortcut and press Super+V"#
            .to_string(),

        "LXQt" => r#"**LXQt Configuration:**
1. Open LXQt Configuration → Shortcut Keys
2. Click "Add"
3. Description: "Clipboard History"
4. Command: `win11-clipboard-history`
5. Set shortcut to Meta+V"#
            .to_string(),

        "LXDE" => r#"**LXDE/Openbox:**
1. Edit ~/.config/openbox/lxde-rc.xml
2. Add in <keyboard> section:

<keybind key="Super_L+v">
  <action name="Execute">
    <command>win11-clipboard-history</command>
  </action>
</keybind>

3. Run: openbox --reconfigure"#
            .to_string(),

        "COSMIC" => r#"**COSMIC Settings:**
1. Open Settings → Keyboard → Custom Shortcuts
2. Add new shortcut
3. Command: `win11-clipboard-history`
4. Binding: Super+V

**Note:** If there's a conflict, check System shortcuts for Super+V bindings."#
            .to_string(),

        "i3" => r#"**i3 Configuration:**
1. Edit your i3 config: `~/.config/i3/config`
2. Comment out or remove any existing `bindsym $mod+v` line
3. Add this line:
```
bindsym $mod+v exec win11-clipboard-history
```
4. Reload i3: Press $mod+Shift+r

**Alternative shortcut:**
```
bindsym Ctrl+Mod1+v exec win11-clipboard-history
```"#
            .to_string(),

        "Sway" => r#"**Sway Configuration:**
1. Edit your Sway config: `~/.config/sway/config`
2. Comment out or remove any existing `bindsym $mod+v` line
3. Add this line:
```
bindsym $mod+v exec win11-clipboard-history
```
4. Reload Sway: Press $mod+Shift+c

**Alternative shortcut:**
```
bindsym Ctrl+Mod1+v exec win11-clipboard-history
```"#
            .to_string(),

        "Hyprland" => r#"**Hyprland Configuration:**
1. Edit your Hyprland config: `~/.config/hypr/hyprland.conf`
2. Comment out or remove any existing `bind = SUPER, V, ...` line
3. Add this line:
```
bind = SUPER, V, exec, win11-clipboard-history
```
4. Config auto-reloads (or press Super+M to reload manually)

**Alternative shortcut:**
```
bind = CTRL ALT, V, exec, win11-clipboard-history
```"#
            .to_string(),

        _ => r#"**Generic Instructions:**
1. Open your desktop environment's keyboard shortcuts settings
2. Add a new custom shortcut
3. Command: `win11-clipboard-history`
4. Shortcut: Super+V (or your preferred combination)

**Alternative:** Use Ctrl+Alt+V if Super+V conflicts with your DE."#
            .to_string(),
    }
}
