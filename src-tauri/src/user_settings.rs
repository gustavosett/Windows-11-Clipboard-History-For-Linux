//! User Settings Module
//! Handles persistence of user preferences in a separate JSON file.

use crate::privacy::PrivacyPolicy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const USER_SETTINGS_FILE: &str = "user_settings.json";

/// User-configurable settings for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub theme_mode: String,
    pub dark_background_opacity: f32,
    pub light_background_opacity: f32,

    #[serde(default = "default_language")]
    pub language: String,

    #[serde(default = "default_true")]
    pub enable_dynamic_tray_icon: bool,

    #[serde(default = "default_true")]
    pub enable_smart_actions: bool,

    #[serde(default = "default_true")]
    pub enable_ui_polish: bool,

    #[serde(default = "default_max_history_size")]
    pub max_history_size: usize,

    #[serde(default = "default_zero")]
    pub auto_delete_interval: u64,

    #[serde(default = "default_unit")]
    pub auto_delete_unit: String,

    #[serde(default)]
    pub custom_kaomojis: Vec<CustomKaomoji>,

    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,

    // --- Privacy ---
    /// Drop clipboard items that look like secrets (keys, tokens, passwords).
    #[serde(default = "default_true")]
    pub filter_secrets: bool,

    /// Persist captured images. When false, image copies are ignored.
    #[serde(default = "default_true")]
    pub save_images: bool,

    /// Skip capture from password managers and private-browsing windows (X11).
    #[serde(default = "default_true")]
    pub exclude_sensitive_apps: bool,

    /// Extra WM_CLASS / title fragments to treat as sensitive.
    #[serde(default)]
    pub extra_excluded_apps: Vec<String>,

    /// Allow the setup wizard to rewrite i3/Sway/Hyprland config files.
    /// Off by default — tiling WM configs are user-owned.
    /// اجازهٔ بازنویسی کانفیگ i3/Sway/Hyprland توسط جادوگر نصب.
    /// پیش‌فرض خاموش — کانفیگ مدیر پنجره متعلق به کاربر است.
    #[serde(default = "default_false")]
    pub allow_wm_config_rewrite: bool,

    /// Where the history encryption key is stored: "file" | "secret-service".
    /// Applied at startup; migrations happen via dedicated commands.
    /// محل ذخیرهٔ کلید رمزنگاری تاریخچه: "file" | "secret-service".
    /// در زمان راه‌اندازی اعمال می‌شود؛ مهاجرت با فرمان‌های اختصاصی است.
    #[serde(default = "default_key_backend")]
    pub history_key_backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomKaomoji {
    pub text: String,
    pub category: String,
    #[serde(default)]
    pub keywords: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_max_history_size() -> usize {
    crate::clipboard_manager::DEFAULT_MAX_HISTORY_SIZE
}

fn default_ui_scale() -> f32 {
    1.0
}

fn default_zero() -> u64 {
    0
}

fn default_unit() -> String {
    "hours".to_string()
}

fn default_language() -> String {
    "en".to_string()
}

fn default_key_backend() -> String {
    "file".to_string()
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            theme_mode: "system".to_string(),
            language: default_language(),
            dark_background_opacity: 0.70,
            light_background_opacity: 0.70,
            enable_dynamic_tray_icon: true,
            enable_smart_actions: true,
            enable_ui_polish: true,
            max_history_size: default_max_history_size(),
            auto_delete_interval: 0,
            auto_delete_unit: "hours".to_string(),
            custom_kaomojis: Vec::new(),
            ui_scale: default_ui_scale(),
            filter_secrets: true,
            save_images: true,
            exclude_sensitive_apps: true,
            extra_excluded_apps: Vec::new(),
            allow_wm_config_rewrite: false,
            history_key_backend: default_key_backend(),
        }
    }
}

impl UserSettings {
    pub fn set_language(&mut self, lang: &str) {
        if lang == "en" || lang == "fa" {
            self.language = lang.to_string();
        }
    }

    pub fn privacy_policy(&self) -> PrivacyPolicy {
        PrivacyPolicy {
            filter_secrets: self.filter_secrets,
            save_images: self.save_images,
            exclude_sensitive_apps: self.exclude_sensitive_apps,
            extra_excluded_apps: self.extra_excluded_apps.clone(),
        }
    }

    pub fn auto_delete_interval_in_minutes(&self) -> u64 {
        if self.auto_delete_interval == 0 {
            return 0;
        }

        let base = self.auto_delete_interval;

        match self.auto_delete_unit.as_str() {
            "minutes" => base,
            "hours" => base.saturating_mul(60),
            "days" => base.saturating_mul(60).saturating_mul(24),
            "weeks" => base.saturating_mul(60).saturating_mul(24).saturating_mul(7),
            _ => 0,
        }
    }

    pub fn validate(&mut self) {
        self.dark_background_opacity = self.dark_background_opacity.clamp(0.0, 1.0);
        self.light_background_opacity = self.light_background_opacity.clamp(0.0, 1.0);

        if !["system", "dark", "light"].contains(&self.theme_mode.as_str()) {
            self.theme_mode = "system".to_string();
        }

        self.max_history_size = self.max_history_size.clamp(
            1,
            crate::clipboard_manager::MAX_HISTORY_HARD_CAP,
        );
        self.ui_scale = self.ui_scale.clamp(0.5, 2.0);

        if !["minutes", "hours", "days", "weeks"].contains(&self.auto_delete_unit.as_str()) {
            self.auto_delete_unit = "hours".to_string();
        }

        if !["en", "fa"].contains(&self.language.as_str()) {
            self.language = "en".to_string();
        }

        if !["file", "secret-service"].contains(&self.history_key_backend.as_str()) {
            self.history_key_backend = "file".to_string();
        }

        self.extra_excluded_apps
            .retain(|s| !s.trim().is_empty() && s.len() < 128);
        self.extra_excluded_apps.truncate(32);
    }
}

pub struct UserSettingsManager {
    config_dir: PathBuf,
}

impl UserSettingsManager {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("windows-11-style-clipboard-history-manager");

        Self { config_dir }
    }

    fn settings_path(&self) -> PathBuf {
        self.config_dir.join(USER_SETTINGS_FILE)
    }

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
                    tracing::warn!(
                        "[UserSettings] Failed to parse settings file: {}. Using defaults.",
                        e
                    );
                    UserSettings::default()
                }
            },
            Err(e) => {
                tracing::warn!(
                    "[UserSettings] Failed to read settings file: {}. Using defaults.",
                    e
                );
                UserSettings::default()
            }
        }
    }

    pub fn save(&self, settings: &UserSettings) -> Result<(), String> {
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
            crate::fs_atomic::restrict_permissions(&self.config_dir);
        }

        let mut validated_settings = settings.clone();
        validated_settings.validate();

        crate::fs_atomic::write_json_atomic(&self.settings_path(), &validated_settings)
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
        assert!(settings.filter_secrets);
        assert!(settings.save_images);
        assert!(settings.exclude_sensitive_apps);
        assert!(!settings.allow_wm_config_rewrite);
        assert_eq!(settings.history_key_backend, "file");
    }

    #[test]
    fn key_backend_is_validated() {
        let mut settings = UserSettings {
            history_key_backend: "keyring-of-doom".to_string(),
            ..Default::default()
        };
        settings.validate();
        assert_eq!(settings.history_key_backend, "file");

        settings.history_key_backend = "secret-service".to_string();
        settings.validate();
        assert_eq!(settings.history_key_backend, "secret-service");
    }

    #[test]
    fn test_validate_clamps_values() {
        let mut settings = UserSettings {
            theme_mode: "invalid".to_string(),
            dark_background_opacity: 1.5,
            light_background_opacity: -0.5,
            ..Default::default()
        };
        settings.validate();

        assert_eq!(settings.theme_mode, "system");
        assert!((settings.dark_background_opacity - 1.0).abs() < f32::EPSILON);
        assert!(settings.light_background_opacity.abs() < f32::EPSILON);
    }

    #[test]
    fn privacy_policy_mirrors_flags() {
        let mut s = UserSettings::default();
        s.filter_secrets = false;
        s.save_images = false;
        let p = s.privacy_policy();
        assert!(!p.filter_secrets);
        assert!(!p.save_images);
    }
}
