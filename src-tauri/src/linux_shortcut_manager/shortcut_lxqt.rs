//! LXQt globalkeyshortcuts.ini backend.
//! / بک‌اند میانبر LXQt.

use super::shortcut_config::{ShortcutConfig, INI_SECTION_ENCODE};
use super::shortcut_error::{Result, ShortcutError};
use super::shortcut_handler::ShortcutHandler;
use super::shortcut_utils::Utils;
use percent_encoding::utf8_percent_encode;
use std::env;
use std::path::PathBuf;

pub(super) struct LxqtHandler;

impl ShortcutHandler for LxqtHandler {
    fn name(&self) -> &str {
        "LXQt"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;
        let path = PathBuf::from(home).join(".config/lxqt/globalkeyshortcuts.conf");

        let full_cmd = s.full_command();
        let encoded_binding = utf8_percent_encode(s.kde_binding, INI_SECTION_ENCODE).to_string();
        let section = format!("{encoded_binding}/{}", s.id);
        let entry = format!("\n[{section}]\nComment={}\nEnabled=true\nExec={full_cmd}", s.name);

        Utils::modify_file_atomic(&path, |content| {
            if content.contains(&format!("[{section}]")) {
                return Ok(None);
            }

            let mut new_content = content.clone();
            new_content.push_str(&entry);
            Ok(Some(new_content))
        })?;
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;
        let path = PathBuf::from(home).join(".config/lxqt/globalkeyshortcuts.conf");

        if !path.exists() {
            return Ok(());
        }

        let encoded_binding = utf8_percent_encode(s.kde_binding, INI_SECTION_ENCODE).to_string();
        let section = format!("{encoded_binding}/{}", s.id);

        Utils::modify_file_atomic(&path, |content| {
            if !content.contains(&format!("[{section}]")) {
                return Ok(None);
            }

            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines = Vec::new();
            let mut skip_block = false;

            for line in lines {
                if line.trim() == format!("[{section}]") {
                    skip_block = true;
                    continue;
                }
                if line.starts_with('[') && skip_block {
                    skip_block = false;
                }
                if !skip_block {
                    new_lines.push(line.to_string());
                }
            }
            Ok(Some(new_lines.join("\n")))
        })?;
        Ok(())
    }
}
