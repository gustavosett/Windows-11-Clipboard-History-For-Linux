//! Tauri Command Handlers
//!
//! All `#[tauri::command]` functions live here. They are thin wrappers that
//! delegate to the domain modules (`ClipboardManager`, `EmojiManager`, etc.)
//! and return [`AppError`] for structured, serialisable error reporting.

use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

use crate::clipboard_manager::ClipboardItem;
use crate::emoji_manager::EmojiUsage;
use crate::error::AppError;
use crate::history_crypto::HistoryCrypto;
use crate::input_simulator::simulate_paste_keystroke;
use crate::theme_manager::{self, ThemeInfo};
use crate::user_settings::{UserSettings, UserSettingsManager};
use crate::AppState;

// ---------------------------------------------------------------------------
// History commands
// ---------------------------------------------------------------------------

/// Bounded history window for large histories (see ADR-0007).
/// پنجرهٔ محدود تاریخچه برای تاریخچه‌های بزرگ (ADR-0007).
///
/// `limit` is clamped server-side to 1..=200, so the webview can never
/// request an unbounded payload. Missing arguments default to a full read.
/// مقدار `limit` سمت سرور به 1..=200 محدود می‌شود؛ وب‌ویو هرگز نمی‌تواند
/// بار نامحدود بخواهد. نبود آرگومان‌ها به خواندن کامل پیش‌فرض است.
#[tauri::command]
pub fn get_history_page(
    window: WebviewWindow,
    state: State<AppState>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<crate::clipboard_manager::HistoryPage, AppError> {
    require_main_window(&window)?;
    Ok(state.clipboard_manager.lock().get_history_page(
        limit.unwrap_or(crate::clipboard_manager::MAX_PAGE_SIZE),
        offset.unwrap_or(0),
    ))
}


#[tauri::command]
pub fn get_item(window: WebviewWindow, state: State<AppState>, id: String) -> Result<ClipboardItem, AppError> {
    require_main_window(&window)?;
    state
        .clipboard_manager
        .lock()
        .get_item(&id)
        .map(ClipboardItem::for_ipc)
        .ok_or(AppError::NotFound { id })
}

#[tauri::command]
pub fn clear_history(window: WebviewWindow, state: State<AppState>) -> Result<(), AppError> {
    require_main_window(&window)?;
    state.clipboard_manager.lock().clear();
    Ok(())
}

#[tauri::command]
pub fn delete_item(window: WebviewWindow, state: State<AppState>, id: String) -> Result<(), AppError> {
    require_main_window(&window)?;
    state.clipboard_manager.lock().remove_item(&id);
    Ok(())
}

#[tauri::command]
pub fn toggle_pin(window: WebviewWindow, state: State<AppState>, id: String) -> Result<Option<ClipboardItem>, AppError> {
    require_main_window(&window)?;
    let result = state.clipboard_manager.lock().toggle_pin(&id);
    if result.is_none() {
        tracing::warn!("[toggle_pin] Item with id '{}' not found in history.", id);
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Emoji commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_recent_emojis(window: WebviewWindow, state: State<AppState>) -> Result<Vec<EmojiUsage>, AppError> {
    require_main_window(&window)?;
    Ok(state.emoji_manager.lock().get_recent())
}

// ---------------------------------------------------------------------------
// Mouse state
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn set_mouse_state(window: WebviewWindow, state: State<AppState>, inside: bool) -> Result<(), AppError> {
    require_main_window(&window)?;
    state.is_mouse_inside.store(inside, Ordering::Relaxed);
    Ok(())
}

// ---------------------------------------------------------------------------
// User Settings commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_user_settings(window: WebviewWindow) -> Result<UserSettings, AppError> {
    crate::window_policy::require(
        &window,
        &[
            crate::window_policy::WindowRole::Main,
            crate::window_policy::WindowRole::Settings,
            crate::window_policy::WindowRole::Setup,
        ],
    )?;
    let manager = UserSettingsManager::new();
    Ok(manager.load())
}

#[tauri::command]
pub fn set_user_settings(
    window: WebviewWindow,
    app: AppHandle,
    state: State<AppState>,
    new_settings: UserSettings,
) -> Result<(), AppError> {
    // Mutating settings (privacy policy, history size, key backend) must
    // come from the settings window — never from web content in `main`.
    // تغییر تنظیمات (سیاست حریم خصوصی، حجم تاریخچه، بک‌اند کلید) فقط از
    // پنجرهٔ تنظیمات مجاز است — هرگز از محتوای وب در پنجرهٔ `main`.
    require_settings_window(&window)?;
    let manager = UserSettingsManager::new();
    manager.save(&new_settings)?;

    {
        let mut clipboard_manager = state.clipboard_manager.lock();
        if clipboard_manager.get_max_history_size() != new_settings.max_history_size {
            clipboard_manager.set_max_history_size(new_settings.max_history_size);
        }
        clipboard_manager.set_privacy_policy(new_settings.privacy_policy());
        clipboard_manager
            .set_auto_delete_interval_minutes(new_settings.auto_delete_interval_in_minutes());
    }
    crate::linux_shortcut_manager::set_allow_wm_config_rewrite(
        new_settings.allow_wm_config_rewrite,
    );

    app.emit("app-settings-changed", &new_settings)
        .map_err(|e| AppError::Other(format!("Failed to emit settings changed event: {e}")))?;

    theme_manager::update_dynamic_tray_flag(new_settings.enable_dynamic_tray_icon);

    let app_for_tray = app.clone();
    let settings_for_tray = new_settings.clone();
    tauri::async_runtime::spawn(async move {
        theme_manager::refresh_tray_icon(&app_for_tray, &settings_for_tray).await;
    });

    Ok(())
}

#[tauri::command]
pub fn is_settings_window_visible(window: WebviewWindow, app: AppHandle) -> Result<bool, AppError> {
    require_main_window(&window)?;
    Ok(app.get_webview_window("settings")
        .map(|w| w.is_visible().unwrap_or(false))
        .unwrap_or(false))
}

#[tauri::command]
pub fn get_default_settings(window: WebviewWindow) -> Result<UserSettings, AppError> {
    crate::window_policy::require_configuration(&window)?;
    Ok(UserSettings::default())
}

#[tauri::command]
pub async fn set_app_language(
    window: WebviewWindow,
    app: AppHandle,
    lang: String,
    _state: State<'_, AppState>,
) -> Result<(), AppError> {
    // Language changes are limited to configuration surfaces (settings/setup).
    // تغییر زبان فقط در سطوح پیکربندی (تنظیمات/راه‌اندازی) مجاز است.
    crate::window_policy::require_configuration(&window)?;
    if lang != "en" && lang != "fa" {
        return Err(AppError::Other(
            "Invalid language. Supported: en, fa".into(),
        ));
    }

    let manager = UserSettingsManager::new();
    let mut settings = manager.load();
    settings.set_language(&lang);
    manager.save(&settings)?;

    app.emit("app-language-changed", &lang)
        .map_err(|e| AppError::Other(format!("Failed to emit language change event: {e}")))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Theme detection commands
// ---------------------------------------------------------------------------

/// Get system color scheme from XDG Desktop Portal
#[tauri::command]
pub async fn get_system_theme() -> ThemeInfo {
    theme_manager::get_system_color_scheme().await
}

/// Clear the cached theme value
#[tauri::command]
pub async fn refresh_system_theme() -> ThemeInfo {
    theme_manager::clear_theme_cache().await;
    theme_manager::get_system_color_scheme().await
}

/// Check if the D-Bus event listener is running
#[tauri::command]
pub fn is_theme_listener_active() -> bool {
    theme_manager::is_event_listener_running()
}

// ---------------------------------------------------------------------------
// Paste commands
// ---------------------------------------------------------------------------

const MAX_PASTE_TEXT_BYTES: usize = 1024 * 1024; // 1 MiB

/// Keystroke injection is only accepted from the main clipboard window.
/// تزریق کلیدstroke فقط از پنجرهٔ اصلی کلیپ‌بورد پذیرفته می‌شود.
fn require_main_window(window: &WebviewWindow) -> Result<(), AppError> {
    crate::window_policy::require_main(window)
}

/// Settings-only commands (key migration) refuse other windows.
/// فرمان‌های مخصوص تنظیمات (مهاجرت کلید) پنجره‌های دیگر را رد می‌کنند.
fn require_settings_window(window: &WebviewWindow) -> Result<(), AppError> {
    crate::window_policy::require_settings(window)
}

/// Hide popup, restore target focus, and inject Ctrl+V only after a real
/// clipboard write in the last 5 seconds. Does **not** mint a paste ticket —
/// tickets stay reserved for the GIF path (`finish_paste`).
/// پنجره را مخفی می‌کند، فوکوس مقصد را برمی‌گرداند و فقط پس از نوشتن واقعی
/// کلیپ‌بورد در ۵ ثانیهٔ اخیر Ctrl+V تزریق می‌کند. بلیت صادر نمی‌شود.
async fn inject_authorized_paste(
    app: &AppHandle,
    window: &WebviewWindow,
) -> Result<(), AppError> {
    require_main_window(window)?;
    if !crate::clipboard_io::wrote_recently(Duration::from_secs(5)) {
        return Err(AppError::Other(
            "Refusing to inject Ctrl+V: no clipboard write was recorded in the last 5 seconds"
                .into(),
        ));
    }
    crate::window_controller::WindowController::hide(app);
    crate::window_controller::PasteHelper::prepare_target_window(app).await?;
    simulate_paste_keystroke()?;
    Ok(())
}

#[tauri::command]
pub async fn paste_item(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    let _paste_guard = state.paste_gate.lock().await;

    let item = {
        let manager = state.clipboard_manager.lock();
        manager.get_item(&id).cloned()
    };

    match item {
        Some(item) => {
            {
                let mut manager = state.clipboard_manager.lock();
                manager.write_item_to_clipboard(&item)?;
                let page = manager.get_history_page(crate::clipboard_manager::MAX_PAGE_SIZE, 0);
                drop(manager);
                let _ = app.emit("history-sync", &page);
            }
            inject_authorized_paste(&app, &window).await?;
        }
        None => {
            tracing::warn!(
                "[paste_item] Item with id '{}' not found in history. Syncing frontend...",
                id
            );
            let page = state
                .clipboard_manager
                .lock()
                .get_history_page(crate::clipboard_manager::MAX_PAGE_SIZE, 0);
            let _ = app.emit("history-sync", &page);
            return Err(AppError::NotFound { id });
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn paste_text(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    text: String,
    item_type: Option<String>,
) -> Result<(), AppError> {
    let _paste_guard = state.paste_gate.lock().await;

    if text.len() > MAX_PASTE_TEXT_BYTES {
        return Err(AppError::Other(
            "Paste text exceeds the 1 MiB safety limit".into(),
        ));
    }

    if let Some(t) = item_type.as_deref() {
        if t == "emoji" {
            state.emoji_manager.lock().record_usage(&text);
        }
    }

    {
        let mut manager = state.clipboard_manager.lock();
        manager.mark_text_as_pasted(&text);
        manager.set_text_robust(&text)?;
    }

    inject_authorized_paste(&app, &window).await?;
    Ok(())
}

#[tauri::command]
pub async fn paste_gif_from_url(
    window: WebviewWindow,
    state: State<'_, AppState>,
    url: String,
) -> Result<String, AppError> {
    require_main_window(&window)?;
    #[cfg(not(feature = "gif-search"))]
    {
        let _ = (state, url);
        return Err(AppError::Other(
            "GIF search is disabled in this build (compile with --features gif-search)".into(),
        ));
    }
    #[cfg(feature = "gif-search")]
    {
        let _paste_guard = state.paste_gate.lock().await;

        let url_clone = url.clone();
        let file_uri = tokio::task::spawn_blocking(move || {
            crate::gif_manager::paste_gif_to_clipboard_with_uri(&url_clone)
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
        .map_err(|e| AppError::Other(e.to_string()))?;

        if let Some(uri) = file_uri {
            let mut manager = state.clipboard_manager.lock();
            manager.mark_text_as_pasted(&uri);
            if let Some(trimmed) = uri.strip_suffix('\n') {
                manager.mark_text_as_pasted(trimmed);
            }
        }

        Ok(state.issue_paste_ticket())
    }
}

/// Tenor search proxy. Stubbed out when GIF search is not compiled in.
/// پروکسی جستجوی Tenor. بدون feature به خطا برمی‌گردد.
#[cfg(feature = "gif-search")]
#[tauri::command]
pub async fn search_tenor(
    window: WebviewWindow,
    query: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<crate::tenor_api::GifResult>, AppError> {
    require_main_window(&window)?;
    crate::tenor_api::search_tenor(query, limit)
        .await
        .map_err(AppError::Network)
}

#[cfg(not(feature = "gif-search"))]
#[tauri::command]
pub async fn search_tenor(
    window: WebviewWindow,
    _query: Option<String>,
    _limit: Option<u32>,
) -> Result<Vec<serde_json::Value>, AppError> {
    require_main_window(&window)?;
    Err(AppError::Other(
        "GIF search is disabled in this build (compile with --features gif-search)".into(),
    ))
}

#[tauri::command]
pub async fn finish_paste(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    ticket: String,
) -> Result<(), AppError> {
    require_main_window(&window)?;
    let _paste_guard = state.paste_gate.lock().await;
    if !state.consume_paste_ticket(&ticket) {
        return Err(AppError::PermissionDenied(
            "Invalid or expired paste ticket".into(),
        ));
    }
    if !crate::clipboard_io::wrote_recently(Duration::from_secs(5)) {
        return Err(AppError::Other(
            "Refusing to inject Ctrl+V: no clipboard write was recorded in the last 5 seconds"
                .into(),
        ));
    }
    crate::window_controller::WindowController::hide(&app);
    crate::window_controller::PasteHelper::prepare_target_window(&app).await?;
    simulate_paste_keystroke()?;
    Ok(())
}

#[tauri::command]
pub fn open_safe_url(window: WebviewWindow, url: String) -> Result<(), AppError> {
    require_main_window(&window)?;
    crate::open_url::open_safe_url(&url).map(|_| ())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Encryption key backend commands (see ADR-0006)
// فرمان‌های بک‌اند کلید رمزنگاری (ADR-0006)
// ---------------------------------------------------------------------------

/// Snapshot of the key-backend state for the Settings UI.
/// وضعیت لحظه‌ای بک‌اند کلید برای رابط تنظیمات.
#[derive(Debug, Clone, serde::Serialize)]
pub struct KeyBackendStatus {
    /// Backend requested by the persisted user setting.
    /// بک‌اند درخواستی طبق تنظیم ذخیره‌شده.
    pub setting: String,
    /// Backend actually in use by this process.
    /// بک‌اند واقعاً استفاده‌شده در این فرآیند.
    pub active: String,
    /// True when `secret-tool` (Secret Service) is usable on this machine.
    /// وقتی `secret-tool` (Secret Service) روی این ماشین قابل استفاده است.
    pub secret_service_available: bool,
    /// True when the active backend differs from the setting (restart needed).
    /// وقتی بک‌اند فعال با تنظیم تفاوت دارد (نیاز به راه‌اندازی مجدد).
    pub restart_required: bool,
}

#[tauri::command]
pub fn get_history_key_backend_status(
    window: WebviewWindow,
    state: State<AppState>,
) -> Result<KeyBackendStatus, AppError> {
    require_settings_window(&window)?;
    let setting = UserSettingsManager::new().load().history_key_backend;
    let active = state.clipboard_manager.lock().key_backend().to_string();
    Ok(KeyBackendStatus {
        restart_required: setting != active,
        secret_service_available: HistoryCrypto::secret_service_available(),
        setting,
        active,
    })
}

/// Move the encryption key into the freedesktop Secret Service.
/// The key material is verified by read-back before the file key is renamed;
/// the new backend takes effect on the next launch.
/// انتقال کلید رمزنگاری به Secret Service. پیش از تغییر نام کلید فایل،
/// کلید با read-back راستی‌آزمایی می‌شود؛ بک‌اند جدید از اجرای بعدی فعال است.
#[tauri::command]
pub fn migrate_history_key_to_secret_service(
    window: WebviewWindow,
) -> Result<KeyBackendStatus, AppError> {
    require_settings_window(&window)?;
    let data_dir = crate::clipboard_manager::data_dir();
    HistoryCrypto::migrate_to_secret_service(&data_dir)?;

    let manager = UserSettingsManager::new();
    let mut settings = manager.load();
    settings.history_key_backend = "secret-service".to_string();
    manager.save(&settings)?;

    Ok(KeyBackendStatus {
        setting: "secret-service".to_string(),
        active: "file".to_string(),
        secret_service_available: true,
        restart_required: true,
    })
}

/// Move the encryption key back to the file backend (undo migration).
/// بازگرداندن کلید رمزنگاری به بک‌اند فایل (واگرد مهاجرت).
#[tauri::command]
pub fn migrate_history_key_to_file(window: WebviewWindow) -> Result<KeyBackendStatus, AppError> {
    require_settings_window(&window)?;
    let data_dir = crate::clipboard_manager::data_dir();
    HistoryCrypto::migrate_to_file(&data_dir)?;

    let manager = UserSettingsManager::new();
    let mut settings = manager.load();
    settings.history_key_backend = "file".to_string();
    manager.save(&settings)?;

    Ok(KeyBackendStatus {
        setting: "file".to_string(),
        active: "file".to_string(),
        secret_service_available: HistoryCrypto::secret_service_available(),
        restart_required: false,
    })
}

#[tauri::command]
pub async fn copy_text_to_clipboard(window: WebviewWindow, text: String) -> Result<(), AppError> {
    require_main_window(&window)?;
    crate::clipboard_io::write(&crate::clipboard_io::Payload::Text(&text))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Setup commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn finish_setup(window: WebviewWindow, app: AppHandle) -> Result<(), AppError> {
    crate::window_policy::require_setup(&window)?;
    crate::permission_checker::complete_first_run()?;

    if let Some(setup_window) = app.get_webview_window("setup") {
        let _ = setup_window.close();
    }

    if let Some(main_window) = app.get_webview_window("main") {
        crate::window_controller::WindowController::position_and_show(&main_window, &app);
    }

    let _ = app.emit("setup_complete", ());
    Ok(())
}
