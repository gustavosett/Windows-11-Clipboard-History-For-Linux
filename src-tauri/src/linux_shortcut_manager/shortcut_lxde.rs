//! LXDE / Openbox XML shortcut backend.
//! / بک‌اند میانبر LXDE و Openbox.

use super::shortcut_config::{escape_xml, ShortcutConfig};
use super::shortcut_error::{Result, ShortcutError};
use super::shortcut_handler::ShortcutHandler;
use super::shortcut_utils::Utils;
use std::env;
use std::io;
use std::path::PathBuf;

pub(super) struct LxdeHandler;

impl ShortcutHandler for LxdeHandler {
    fn name(&self) -> &str {
        "LXDE/Openbox"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;

        let path = PathBuf::from(&home).join(".config/openbox/lxde-rc.xml");
        let path = if path.exists() {
            path
        } else {
            PathBuf::from(&home).join(".config/openbox/rc.xml")
        };

        if !path.exists() {
            return Err(ShortcutError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "Openbox config not found",
            )));
        }

        let full_cmd = s.full_command();
        let escaped_binding = escape_xml(s.lxde_binding);
        let escaped_cmd = escape_xml(&full_cmd);
        let keybind = format!(
            r#"    <keybind key="{escaped_binding}">
      <action name="Execute">
        <command>{escaped_cmd}</command>
      </action>
    </keybind>"#
        );

        Utils::modify_file_atomic(&path, |content| {
            if content.contains(&format!("<command>{escaped_cmd}</command>")) {
                return Ok(None);
            }

            if let Some(pos) = content.find("</keyboard>") {
                let mut new_content = content.clone();
                new_content.insert_str(pos, &format!("{keybind}\n  "));
                let _ = Utils::run("openbox", &["--reconfigure"]);
                return Ok(Some(new_content));
            }

            Err(ShortcutError::ParseError(
                "Could not find </keyboard> in Openbox config".into(),
            ))
        })?;
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;

        let path = PathBuf::from(&home).join(".config/openbox/lxde-rc.xml");
        let path = if path.exists() {
            path
        } else {
            PathBuf::from(&home).join(".config/openbox/rc.xml")
        };

        if !path.exists() {
            return Ok(());
        }

        let full_cmd = s.full_command();
        let escaped_binding = escape_xml(s.lxde_binding);
        let escaped_cmd = escape_xml(&full_cmd);

        Utils::modify_file_atomic(&path, |content| {
            if !content.contains(&format!("<command>{escaped_cmd}</command>")) {
                return Ok(None);
            }

            let pattern = format!(
                r#"    <keybind key="{escaped_binding}">
      <action name="Execute">
        <command>{escaped_cmd}</command>
      </action>
    </keybind>"#
            );

            let new_content = content.replace(&pattern, "");
            let _ = Utils::run("openbox", &["--reconfigure"]);
            Ok(Some(new_content))
        })?;
        Ok(())
    }
}
