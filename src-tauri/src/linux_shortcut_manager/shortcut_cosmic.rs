//! COSMIC Epoch shortcut backend (RON custom map).
//! / بک‌اند میانبر COSMIC Epoch.

use super::shortcut_config::ShortcutConfig;
use super::shortcut_error::{Result, ShortcutError};
use super::shortcut_handler::ShortcutHandler;
use super::shortcut_utils::Utils;
use std::env;
use std::path::PathBuf;

const COSMIC_ENTRY_INDENT: &str = "    ";
const COSMIC_FIELD_INDENT: &str = "        ";
const COSMIC_MODIFIER_INDENT: &str = "            ";

pub(super) struct CosmicHandler;

impl CosmicHandler {
    fn escape_ron_string(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    fn format_modifiers(mods: &str) -> String {
        let formatted: Vec<String> = mods
            .split(',')
            .map(|m| m.trim())
            .filter(|m| !m.is_empty())
            .map(|m| {
                let normalized: String = match m.to_lowercase().as_str() {
                    "ctrl" | "control" => "Ctrl".to_string(),
                    "alt" => "Alt".to_string(),
                    "super" | "meta" => "Super".to_string(),
                    "shift" => "Shift".to_string(),
                    _ => {
                        let mut chars = m.chars();
                        match chars.next() {
                            Some(first) => {
                                let mut result = first.to_uppercase().to_string();
                                result.push_str(&chars.as_str().to_lowercase());
                                result
                            }
                            None => String::new(),
                        }
                    }
                };
                format!("{COSMIC_MODIFIER_INDENT}{normalized},")
            })
            .collect();
        formatted.join("\n")
    }

    fn build_entry(s: &ShortcutConfig) -> String {
        let mods_formatted = Self::format_modifiers(s.cosmic_mods);
        let full_cmd = Self::escape_ron_string(&s.full_command());
        let name = Self::escape_ron_string(s.name);
        let key = Self::escape_ron_string(s.cosmic_key);

        format!(
            r#"{}(\n{}modifiers: [\n{}\n{}],\n{}key: "{}",\n{}description: Some("{}"),\n{}): Spawn("{}"),"#,
            COSMIC_ENTRY_INDENT,
            COSMIC_FIELD_INDENT,
            mods_formatted,
            COSMIC_FIELD_INDENT,
            COSMIC_FIELD_INDENT,
            key,
            COSMIC_FIELD_INDENT,
            name,
            COSMIC_ENTRY_INDENT,
            full_cmd
        )
    }
}

impl ShortcutHandler for CosmicHandler {
    fn name(&self) -> &str {
        "COSMIC (Epoch)"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;
        let path = PathBuf::from(home)
            .join(".config/cosmic/com.system76.CosmicSettings.Shortcuts/v1/custom");

        let full_cmd = s.full_command();
        let entry = Self::build_entry(s);

        Utils::modify_file_atomic(&path, |content| {
            if content.contains(&format!("Spawn(\"{full_cmd}\")")) {
                return Ok(None);
            }

            let trimmed = content.trim();

            if trimmed.is_empty() {
                return Ok(Some(format!("{{\n{entry}\n}}")));
            }

            if !trimmed.starts_with('{') {
                return Err(ShortcutError::ParseError(
                    "Invalid COSMIC config format - expected RON map starting with '{'".into(),
                ));
            }

            if let Some(pos) = content.rfind('}') {
                let mut new_content = content.to_string();
                new_content.insert_str(pos, &format!("{entry}\n"));
                return Ok(Some(new_content));
            }

            Err(ShortcutError::ParseError(
                "Invalid COSMIC config format - missing closing brace".into(),
            ))
        })?;
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME").unwrap_or_default();
        let path = PathBuf::from(home)
            .join(".config/cosmic/com.system76.CosmicSettings.Shortcuts/v1/custom");

        if !path.exists() {
            return Ok(());
        }

        let full_cmd = s.full_command();
        let spawn_pattern = format!("Spawn(\"{full_cmd}\")");

        Utils::modify_file_atomic(&path, |content| {
            if !content.contains(&spawn_pattern) {
                return Ok(None);
            }

            let mut result = String::new();
            let mut depth = 0;
            let mut in_entry = false;
            let mut entry_start = 0;
            let mut prev_depth: i32;

            for c in content.chars() {
                prev_depth = depth;

                if c == '{' || c == '(' {
                    depth += 1;
                } else if c == '}' || c == ')' {
                    depth -= 1;
                }

                if c == '(' && prev_depth == 1 && depth == 2 {
                    entry_start = result.len();
                    in_entry = true;
                }

                result.push(c);

                if in_entry && depth == 1 && c == ',' {
                    let entry_content = &result[entry_start..];
                    if entry_content.contains(&spawn_pattern) {
                        let trim_start = result[..entry_start].trim_end().len();
                        result.truncate(trim_start);
                        result.push('\n');
                    }
                    in_entry = false;
                }
            }

            let mut cleaned = String::with_capacity(result.len());
            let mut newline_count = 0;
            for ch in result.chars() {
                if ch == '\n' {
                    if newline_count < 2 {
                        cleaned.push('\n');
                    }
                    newline_count += 1;
                } else {
                    newline_count = 0;
                    cleaned.push(ch);
                }
            }

            Ok(Some(cleaned))
        })?;
        Ok(())
    }
}
