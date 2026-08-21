//! gsettings-based shortcut backends: GNOME, Cinnamon and MATE.
//!
//! These desktop environments expose keybindings through `gsettings` schemas,
//! so they share the [`GSettings`] adapter and only differ in schema names.

use super::shortcut_config::ShortcutConfig;
use super::shortcut_error::{Result, ShortcutError};
use super::shortcut_utils::Utils;
use super::shortcut_handler::ShortcutHandler;
use std::io;

/// Thin wrapper over `gsettings` for a schema + custom-keybinding list.
struct GSettings {
    schema: &'static str,
    list_key: &'static str,
    path_prefix: &'static str,
    binding_schema: &'static str,
}

impl GSettings {
    fn new_gnome() -> Self {
        Self {
            schema: "org.gnome.settings-daemon.plugins.media-keys",
            list_key: "custom-keybindings",
            path_prefix: "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings",
            binding_schema: "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding",
        }
    }

    fn new_cinnamon() -> Self {
        Self {
            schema: "org.cinnamon.desktop.keybindings",
            list_key: "custom-list",
            path_prefix: "/org/cinnamon/desktop/keybindings/custom-keybindings",
            binding_schema: "org.cinnamon.desktop.keybindings.custom-keybinding",
        }
    }

    fn get_list(&self) -> Result<Vec<String>> {
        let output = Utils::run("gsettings", &["get", self.schema, self.list_key])?;

        if output.contains("@as []") || output == "[]" || output.trim().is_empty() {
            return Ok(Vec::new());
        }

        let cleaned = output
            .trim_start_matches('[')
            .trim_end_matches(']')
            .replace(['\'', '"'], ""); // Remove both single and double quotes for parsing

        Ok(cleaned
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    fn set_list(&self, items: &[String]) -> Result<()> {
        let formatted_list = if items.is_empty() {
            "[]".to_string()
        } else {
            // Reconstruct safely
            let inner = items
                .iter()
                .map(|s| format!("'{s}'"))
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{inner}]")
        };
        Utils::run(
            "gsettings",
            &["set", self.schema, self.list_key, &formatted_list],
        )
        .map(|_| ())
    }

    fn register(&self, shortcut: &ShortcutConfig, use_array_for_binding: bool) -> Result<()> {
        if !Utils::command_exists("gsettings") {
            return Err(ShortcutError::DependencyMissing("gsettings".into()));
        }

        let path = format!("{}/{}/", self.path_prefix, shortcut.id);
        let schema_path = format!("{}:{}", self.binding_schema, path);
        let full_cmd = shortcut.full_command();

        // Idempotent setting
        Utils::run("gsettings", &["set", &schema_path, "name", shortcut.name])?;
        Utils::run("gsettings", &["set", &schema_path, "command", &full_cmd])?;

        let binding_val = if use_array_for_binding {
            format!("['{}']", shortcut.gnome_binding)
        } else {
            format!("'{}'", shortcut.gnome_binding)
        };
        Utils::run("gsettings", &["set", &schema_path, "binding", &binding_val])?;

        let mut list = self.get_list()?;
        let entry_check = if self.path_prefix.contains("cinnamon") {
            shortcut.id
        } else {
            &path
        };

        if !list.iter().any(|x| x.contains(entry_check)) {
            list.push(entry_check.to_string());
            self.set_list(&list)?;
        }
        Ok(())
    }

    fn unregister(&self, shortcut: &ShortcutConfig) -> Result<()> {
        if !Utils::command_exists("gsettings") {
            return Ok(());
        }

        let path = format!("{}/{}/", self.path_prefix, shortcut.id);
        let schema_path = format!("{}:{}", self.binding_schema, path);

        let _ = Utils::run("gsettings", &["reset", &schema_path, "name"]);
        let _ = Utils::run("gsettings", &["reset", &schema_path, "command"]);
        let _ = Utils::run("gsettings", &["reset", &schema_path, "binding"]);

        let mut list = self.get_list()?;
        let initial_len = list.len();
        let entry_check = if self.path_prefix.contains("cinnamon") {
            shortcut.id
        } else {
            &path
        };

        list.retain(|x| !x.contains(entry_check));

        if list.len() != initial_len {
            self.set_list(&list)?;
        }
        Ok(())
    }
}

// Wrappers
pub struct GnomeHandler;
impl ShortcutHandler for GnomeHandler {
    fn name(&self) -> &str {
        "GNOME/Unity"
    }
    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        GSettings::new_gnome().register(s, false)
    }
    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        GSettings::new_gnome().unregister(s)
    }
}

pub struct CinnamonHandler;
impl ShortcutHandler for CinnamonHandler {
    fn name(&self) -> &str {
        "Cinnamon"
    }
    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        GSettings::new_cinnamon().register(s, true)
    }
    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        GSettings::new_cinnamon().unregister(s)
    }
}

// --- MATE ---

pub struct MateHandler;
impl ShortcutHandler for MateHandler {
    fn name(&self) -> &str {
        "MATE"
    }
    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        if !Utils::command_exists("gsettings") {
            return Err(ShortcutError::DependencyMissing("gsettings".into()));
        }

        let full_cmd = s.full_command();

        // Scan the 12 MATE command slots; fill the first free one.
        for i in 1..=12 {
            let cmd_key = format!("command-{i}");
            let current = Utils::run(
                "gsettings",
                &["get", "org.mate.Marco.keybinding-commands", &cmd_key],
            )?;
            let current = current.trim_matches('\'');

            if current == full_cmd {
                return Ok(()); // Already done
            }

            if current.is_empty() {
                let binding_key = format!("run-command-{i}");
                Utils::run(
                    "gsettings",
                    &[
                        "set",
                        "org.mate.Marco.keybinding-commands",
                        &cmd_key,
                        &full_cmd,
                    ],
                )?;
                Utils::run(
                    "gsettings",
                    &[
                        "set",
                        "org.mate.Marco.global-keybindings",
                        &binding_key,
                        s.gnome_binding,
                    ],
                )?;
                return Ok(());
            }
        }
        Err(ShortcutError::Io(io::Error::other(
            "MATE keybinding slots full",
        )))
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        if !Utils::command_exists("gsettings") {
            return Ok(());
        }
        let full_cmd = s.full_command();
        for i in 1..=12 {
            let cmd_key = format!("command-{i}");
            let current = Utils::run(
                "gsettings",
                &["get", "org.mate.Marco.keybinding-commands", &cmd_key],
            )?;

            if current.contains(&full_cmd) {
                Utils::run(
                    "gsettings",
                    &["reset", "org.mate.Marco.keybinding-commands", &cmd_key],
                )?;
                Utils::run(
                    "gsettings",
                    &[
                        "reset",
                        "org.mate.Marco.global-keybindings",
                        &format!("run-command-{i}"),
                    ],
                )?;
            }
        }
        Ok(())
    }
}
