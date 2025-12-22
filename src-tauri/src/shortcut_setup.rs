//! Shortcut setup commands for the frontend
//! Provides Tauri commands to register/unregister shortcuts from the Setup Wizard

use std::env;

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
    } else {
        xdg_current.to_uppercase()
    }
}

/// Register the global shortcut with the desktop environment
/// This calls the existing linux_shortcut_manager
#[tauri::command]
pub fn register_de_shortcut() -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        // Run in a separate thread to avoid blocking
        std::thread::spawn(|| {
            crate::linux_shortcut_manager::register_global_shortcut();
        });
        Ok("Shortcut registration initiated. Check the app logs for details.".to_string())
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
            "GNOME" | "Cinnamon" | "MATE" | "Budgie" | "Deepin" => gsettings || dconf,
            "KDE Plasma" => kwriteconfig5 || kwriteconfig6,
            "XFCE" => xfconf_query,
            "LXQt" => true,   // Uses config files
            "LXDE" => true,   // Uses config files
            "COSMIC" => true, // Uses config files
            _ => gsettings,   // Fallback to gsettings
        };

        ShortcutToolsStatus {
            desktop_environment: de,
            gsettings_available: gsettings,
            kde_tools_available: kwriteconfig5 || kwriteconfig6,
            xfce_tools_available: xfconf_query,
            can_register_automatically: can_register,
            manual_instructions: get_manual_instructions(&get_desktop_environment()),
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
5. Shortcut: Press Super+V"#
            .to_string(),

        "KDE Plasma" => r#"**KDE System Settings:**
1. Open System Settings → Shortcuts → Custom Shortcuts
2. Click "Edit" → "New" → "Global Shortcut" → "Command/URL"
3. Name: "Clipboard History"
4. Trigger: Click and press Meta+V
5. Action: `win11-clipboard-history`"#
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
4. Binding: Super+V"#
            .to_string(),

        _ => r#"**Generic Instructions:**
1. Open your desktop environment's keyboard shortcuts settings
2. Add a new custom shortcut
3. Command: `win11-clipboard-history`
4. Shortcut: Super+V (or your preferred combination)"#
            .to_string(),
    }
}
