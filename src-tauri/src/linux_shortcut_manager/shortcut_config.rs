//! Shortcut configuration: the shared config table and per-DE bindings.
//!
//! This module is the single source of truth for *which* shortcuts exist and
//! what each desktop environment should bind them to. Handlers (per-DE
//! implementations) live in sibling modules and only consume [`ShortcutConfig`].

use crate::linux_shortcut_manager::shortcut_utils::Utils;
use percent_encoding::AsciiSet;
use percent_encoding::CONTROLS;
use std::env;

/// Characters that need encoding in INI section names: / \ [ ] = ; # and control chars
pub const INI_SECTION_ENCODE: &AsciiSet = &CONTROLS
    .add(b'/')
    .add(b'\\')
    .add(b'[')
    .add(b']')
    .add(b'=')
    .add(b';')
    .add(b'#')
    .add(b' ');

/// Escape special XML characters to prevent XML injection
pub fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// =============================================================================
// Configuration
// =============================================================================

#[derive(Debug, Clone)]
pub struct ShortcutConfig {
    pub id: &'static str,
    pub name: &'static str,
    pub command: &'static str,
    pub args: &'static str, // Command line arguments (e.g., "--emoji")
    pub gnome_binding: &'static str,
    pub kde_binding: &'static str,
    pub xfce_binding: &'static str,
    pub cosmic_mods: &'static str,
    pub cosmic_key: &'static str,
    // Tiling WM bindings
    pub i3_binding: &'static str,
    pub sway_binding: &'static str,
    pub hyprland_binding: &'static str,
    pub lxde_binding: &'static str,
}

impl ShortcutConfig {
    /// Returns the full command string including any arguments
    pub fn full_command(&self) -> String {
        if self.args.is_empty() {
            self.command.to_string()
        } else {
            format!("{} {}", self.command, self.args)
        }
    }
}

/// Resolve the executable path used inside registered shortcuts.
///
/// Prefers the wrapper script found in `PATH` (production installs); falls
/// back to the current executable (development builds). Called once at
/// startup, so the `Box::leak` is intentional and bounded.
pub fn get_command_path() -> &'static str {
    // First, check if binary is in PATH (production install)
    if Utils::command_exists("windows-11-style-clipboard-history-manager") {
        return "windows-11-style-clipboard-history-manager";
    }

    // Try to find the current executable path (for development)
    if let Ok(exe_path) = env::current_exe() {
        let path_str = exe_path.to_string_lossy().to_string();
        return Box::leak(path_str.into_boxed_str());
    }

    // Fallback to just the name
    "windows-11-style-clipboard-history-manager"
}

/// All shortcuts managed by the application. `command` is replaced at runtime
/// with the resolved executable path (see [`get_command_path`]).
pub const SHORTCUTS: &[ShortcutConfig] = &[
    ShortcutConfig {
        id: "windows-11-style-clipboard-history-manager",
        name: "Clipboard History",
        command: "windows-11-style-clipboard-history-manager", // Will be replaced at runtime
        args: "",
        gnome_binding: "<Super>v",
        kde_binding: "Meta+V",
        xfce_binding: "<Super>v",
        cosmic_mods: "Super",
        cosmic_key: "v",
        i3_binding: "$mod+v",
        sway_binding: "$mod+v",
        hyprland_binding: "SUPER, V",
        lxde_binding: "W-v",
    },
    ShortcutConfig {
        id: "windows-11-style-clipboard-history-manager-alt",
        name: "Clipboard History (Alt)",
        command: "windows-11-style-clipboard-history-manager", // Will be replaced at runtime
        args: "",
        gnome_binding: "<Ctrl><Alt>v",
        kde_binding: "Ctrl+Alt+V",
        xfce_binding: "<Primary><Alt>v",
        cosmic_mods: "Ctrl, Alt",
        cosmic_key: "v",
        i3_binding: "Ctrl+Mod1+v",
        sway_binding: "Ctrl+Mod1+v",
        hyprland_binding: "CTRL ALT, V",
        lxde_binding: "C-A-v",
    },
    ShortcutConfig {
        id: "windows-11-style-clipboard-history-manager-emoji",
        name: "Emoji Picker",
        command: "windows-11-style-clipboard-history-manager", // Will be replaced at runtime
        args: "--emoji",
        gnome_binding: "<Super>period",
        kde_binding: "Meta+.",
        xfce_binding: "<Super>period",
        cosmic_mods: "Super",
        cosmic_key: "period",
        i3_binding: "$mod+period",
        sway_binding: "$mod+period",
        hyprland_binding: "SUPER, period",
        lxde_binding: "W-period",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_escape_prevents_injection() {
        assert_eq!(escape_xml("<cmd>"), "&lt;cmd&gt;");
        assert_eq!(escape_xml("a&b"), "a&amp;b");
    }

    #[test]
    fn full_command_appends_args() {
        let config = &SHORTCUTS[2];
        assert_eq!(config.full_command(), "windows-11-style-clipboard-history-manager --emoji");
    }

    #[test]
    fn every_shortcut_has_unique_id_and_bindings() {
        let mut ids = std::collections::HashSet::new();
        for s in SHORTCUTS {
            assert!(ids.insert(s.id), "duplicate shortcut id: {}", s.id);
            assert!(!s.gnome_binding.is_empty());
            assert!(!s.kde_binding.is_empty());
            assert!(!s.i3_binding.is_empty());
        }
    }
}
