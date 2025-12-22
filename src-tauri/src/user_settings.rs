//! User Settings Module
//! Handles persistence of user preferences (theme mode, background opacity) in a separate JSON file.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const USER_SETTINGS_FILE: &str = "user_settings.json";

/// User-configurable settings for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    /// Theme mode: "system", "dark", or "light"
    pub theme_mode: String,
    /// Background opacity for dark mode (0.0 to 1.0)
    /// Default matches the original glass-effect alpha of 0.05
    pub dark_background_opacity: f32,
    /// Background opacity for light mode (0.0 to 1.0)
    /// Default matches the original glass-effect-light alpha of 0.85
    pub light_background_opacity: f32,
    
    // --- Feature Flags ---
    /// Enable Smart Actions (URL, Color, Email detection)
    #[serde(default = "default_true")]
    pub enable_smart_actions: bool,
    /// Enable Developer Tools (Transformers, Regex, JWT, Time)
    #[serde(default = "default_true")]
    pub enable_dev_tools: bool,
    /// Enable Favorites Tab
    #[serde(default = "default_true")]
    pub enable_favorites: bool,
    /// Enable UI Polish (Toast, Compact Mode capability)
    #[serde(default = "default_true")]
    pub enable_ui_polish: bool,
    
    // --- Custom Data ---
    /// User-defined Kaomojis
    #[serde(default = "Vec::new")]
    pub custom_kaomojis: Vec<CustomKaomoji>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomKaomoji {
    pub text: String,
    pub category: String, // Default "Custom"
    #[serde(default)]
    pub keywords: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            theme_mode: "system".to_string(),
            dark_background_opacity: 0.70,
            light_background_opacity: 0.70,
            enable_smart_actions: true,
            enable_dev_tools: true,
            enable_favorites: true,
            enable_ui_polish: true,
            custom_kaomojis: Vec::new(),
        }
    }
}

impl UserSettings {
    /// Validates and clamps opacity values to the valid range [0.0, 1.0]
    pub fn validate(&mut self) {
        self.dark_background_opacity = self.dark_background_opacity.clamp(0.0, 1.0);
        self.light_background_opacity = self.light_background_opacity.clamp(0.0, 1.0);

        // Validate theme_mode
        if !["system", "dark", "light"].contains(&self.theme_mode.as_str()) {
            self.theme_mode = "system".to_string();
        }
    }
}

/// Manages loading and saving of user settings
pub struct UserSettingsManager {
    config_dir: PathBuf,
}

impl UserSettingsManager {
    /// Creates a new UserSettingsManager
    /// Uses the OS-appropriate config directory (e.g., ~/.config/win11-clipboard-history/)
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("win11-clipboard-history");

        Self { config_dir }
    }

    /// Gets the path to the settings file
    fn settings_path(&self) -> PathBuf {
        self.config_dir.join(USER_SETTINGS_FILE)
    }

    /// Loads user settings from the config file
    /// Returns default settings if the file doesn't exist or is invalid
    pub fn load(&self) -> UserSettings {
        let path = self.settings_path();

        if !path.exists() {
            return UserSettings::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<UserSettings>(&content) {
                Ok(mut settings) => {
                    settings.validate();
                    settings
                }
                Err(e) => {
                    eprintln!(
                        "[UserSettings] Failed to parse settings file: {}. Using defaults.",
                        e
                    );
                    UserSettings::default()
                }
            },
            Err(e) => {
                eprintln!(
                    "[UserSettings] Failed to read settings file: {}. Using defaults.",
                    e
                );
                UserSettings::default()
            }
        }
    }

    /// Saves user settings to the config file
    pub fn save(&self, settings: &UserSettings) -> Result<(), String> {
        // Ensure the config directory exists
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        // Validate settings before saving
        let mut validated_settings = settings.clone();
        validated_settings.validate();

        let content = serde_json::to_string_pretty(&validated_settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(self.settings_path(), content)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        Ok(())
    }
}

impl Default for UserSettingsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = UserSettings::default();
        assert_eq!(settings.theme_mode, "system");
        assert!((settings.dark_background_opacity - 0.05).abs() < f32::EPSILON);
        assert!((settings.light_background_opacity - 0.85).abs() < f32::EPSILON);
    }

    #[test]
    fn test_validate_clamps_values() {
        let mut settings = UserSettings {
            theme_mode: "invalid".to_string(),
            dark_background_opacity: 1.5,
            light_background_opacity: -0.5,
        };
        settings.validate();

        assert_eq!(settings.theme_mode, "system");
        assert!((settings.dark_background_opacity - 1.0).abs() < f32::EPSILON);
        assert!(settings.light_background_opacity.abs() < f32::EPSILON);
    }
}
