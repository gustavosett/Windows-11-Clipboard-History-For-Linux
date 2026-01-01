//! Theme Manager Module
//! Detects system color scheme preference via XDG Desktop Portal.
//! This is essential for DEs like COSMIC that use the portal standard
//! instead of GNOME settings.

use std::sync::OnceLock;
use tokio::sync::RwLock;

/// Cached system theme preference
static SYSTEM_THEME: OnceLock<RwLock<Option<ColorScheme>>> = OnceLock::new();

/// Color scheme values from the XDG Desktop Portal
/// See: https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorScheme {
    /// No preference (value 0)
    NoPreference,
    /// Prefer dark appearance (value 1)
    Dark,
    /// Prefer light appearance (value 2)
    Light,
}

impl ColorScheme {
    /// Convert portal value to ColorScheme
    fn from_portal_value(value: u32) -> Self {
        match value {
            1 => ColorScheme::Dark,
            2 => ColorScheme::Light,
            _ => ColorScheme::NoPreference,
        }
    }

    /// Whether this scheme represents dark mode
    pub fn is_dark(&self) -> bool {
        matches!(self, ColorScheme::Dark)
    }
}

/// Response from the theme detection
#[derive(Debug, Clone, serde::Serialize)]
pub struct ThemeInfo {
    /// The detected color scheme
    pub color_scheme: ColorScheme,
    /// Whether dark mode is preferred
    pub prefers_dark: bool,
    /// Source of the detection (for debugging)
    pub source: String,
}

/// Query the XDG Desktop Portal for the system color scheme.
/// This works with COSMIC, GNOME, KDE, and other portal-compliant DEs.
#[cfg(target_os = "linux")]
pub async fn get_system_color_scheme() -> ThemeInfo {
    // Try to get cached value first
    let cache = SYSTEM_THEME.get_or_init(|| RwLock::new(None));

    // Check cache
    if let Some(scheme) = *cache.read().await {
        return ThemeInfo {
            color_scheme: scheme,
            prefers_dark: scheme.is_dark(),
            source: "cache".to_string(),
        };
    }

    // Query the portal
    match query_portal_color_scheme().await {
        Ok(scheme) => {
            // Cache the result
            *cache.write().await = Some(scheme);
            ThemeInfo {
                color_scheme: scheme,
                prefers_dark: scheme.is_dark(),
                source: "xdg-portal".to_string(),
            }
        }
        Err(e) => {
            eprintln!(
                "[ThemeManager] Portal query failed: {}, trying fallbacks",
                e
            );
            // Try COSMIC config file fallback
            match read_cosmic_theme_file() {
                Ok(is_dark) => {
                    let scheme = if is_dark {
                        ColorScheme::Dark
                    } else {
                        ColorScheme::Light
                    };
                    ThemeInfo {
                        color_scheme: scheme,
                        prefers_dark: is_dark,
                        source: "cosmic-config".to_string(),
                    }
                }
                Err(_) => {
                    // Default to no preference (let frontend handle it)
                    ThemeInfo {
                        color_scheme: ColorScheme::NoPreference,
                        prefers_dark: false,
                        source: "default".to_string(),
                    }
                }
            }
        }
    }
}

/// Query the XDG Desktop Portal via D-Bus
#[cfg(target_os = "linux")]
async fn query_portal_color_scheme() -> Result<ColorScheme, Box<dyn std::error::Error + Send + Sync>>
{
    use zbus::zvariant::Value;
    use zbus::Connection;

    // Connect to the session bus
    let connection = Connection::session().await?;

    // Call the Settings.Read method
    // Interface: org.freedesktop.portal.Settings
    // Method: Read(namespace: string, key: string) -> variant
    let reply: zbus::zvariant::OwnedValue = connection
        .call_method(
            Some("org.freedesktop.portal.Desktop"),
            "/org/freedesktop/portal/desktop",
            Some("org.freedesktop.portal.Settings"),
            "Read",
            &("org.freedesktop.appearance", "color-scheme"),
        )
        .await?
        .body()
        .deserialize()?;

    // The return value is a variant containing the actual value
    // For color-scheme, it's a uint32 wrapped in a variant (sometimes double-wrapped)
    // Try to extract the u32 value, handling potential variant wrapping
    let value: u32 = match reply.downcast_ref::<u32>() {
        Ok(v) => v,
        Err(_) => {
            // The value might be wrapped in another variant
            if let Value::Value(inner) = &*reply {
                inner.downcast_ref::<u32>()?
            } else {
                return Err("Failed to parse color-scheme value".into());
            }
        }
    };

    Ok(ColorScheme::from_portal_value(value))
}

/// Fallback: Read COSMIC's theme config file directly
/// Path: ~/.config/cosmic/com.system76.CosmicTheme.Mode/v1/is_dark
#[cfg(target_os = "linux")]
fn read_cosmic_theme_file() -> Result<bool, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let config_path = home.join(".config/cosmic/com.system76.CosmicTheme.Mode/v1/is_dark");

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let is_dark = content.trim().eq_ignore_ascii_case("true");
        eprintln!(
            "[ThemeManager] Read COSMIC config file: is_dark={}",
            is_dark
        );
        return Ok(is_dark);
    }

    Err("COSMIC config file not found".into())
}

/// Clear the cached theme value (useful when system theme changes)
#[cfg(target_os = "linux")]
pub async fn clear_theme_cache() {
    if let Some(cache) = SYSTEM_THEME.get() {
        *cache.write().await = None;
    }
}

/// Non-Linux stub implementation
#[cfg(not(target_os = "linux"))]
pub async fn get_system_color_scheme() -> ThemeInfo {
    ThemeInfo {
        color_scheme: ColorScheme::NoPreference,
        prefers_dark: false,
        source: "unsupported-platform".to_string(),
    }
}

/// Non-Linux stub implementation
#[cfg(not(target_os = "linux"))]
pub async fn clear_theme_cache() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_scheme_from_portal_value() {
        assert_eq!(ColorScheme::from_portal_value(0), ColorScheme::NoPreference);
        assert_eq!(ColorScheme::from_portal_value(1), ColorScheme::Dark);
        assert_eq!(ColorScheme::from_portal_value(2), ColorScheme::Light);
        assert_eq!(
            ColorScheme::from_portal_value(99),
            ColorScheme::NoPreference
        );
    }

    #[test]
    fn test_is_dark() {
        assert!(ColorScheme::Dark.is_dark());
        assert!(!ColorScheme::Light.is_dark());
        assert!(!ColorScheme::NoPreference.is_dark());
    }
}
