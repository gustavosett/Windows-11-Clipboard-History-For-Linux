// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Monitor, PhysicalPosition, PhysicalSize, State, WebviewWindow,
    WindowEvent,
};
use win11_clipboard_history_lib::autostart_manager;
use win11_clipboard_history_lib::clipboard_manager::{ClipboardItem, ClipboardManager};
use win11_clipboard_history_lib::config_manager::{resolve_window_position, ConfigManager};
use win11_clipboard_history_lib::emoji_manager::{EmojiManager, EmojiUsage};
use win11_clipboard_history_lib::focus_manager::{restore_focused_window, save_focused_window};
use win11_clipboard_history_lib::input_simulator::simulate_paste_keystroke;
use win11_clipboard_history_lib::permission_checker;
use win11_clipboard_history_lib::rendering_env;
use win11_clipboard_history_lib::session::is_wayland;
use win11_clipboard_history_lib::shortcut_setup;
use win11_clipboard_history_lib::theme_manager::{self, ThemeInfo};
use win11_clipboard_history_lib::user_settings::{UserSettings, UserSettingsManager};

mod popup_activation;
mod window_interaction;
use popup_activation::PopupActivation;
use window_interaction::NativeWindowInteraction;

/// Global flag to track if we started in background mode
/// This is used to block the initial window show
static STARTED_IN_BACKGROUND: AtomicBool = AtomicBool::new(false);

/// Global flag indicating whether the initial show is allowed
/// While false, background mode will still hide the window on focus
/// After the first user toggle, this is set to true to allow normal show/hide behavior
static INITIAL_SHOW_ALLOWED: AtomicBool = AtomicBool::new(false);

/// Application state shared across all handlers
pub struct AppState {
    clipboard_manager: Arc<Mutex<ClipboardManager>>,
    emoji_manager: Arc<Mutex<EmojiManager>>,
    config_manager: Arc<Mutex<ConfigManager>>,
    /// Serializes the complete clipboard/focus/input transaction. Locking only
    /// the virtual device still allows two commands to swap clipboard content
    /// before either chord is emitted.
    paste_gate: tokio::sync::Mutex<()>,
}

// --- Commands ---

#[tauri::command]
fn get_history(state: State<AppState>) -> Vec<ClipboardItem> {
    state.clipboard_manager.lock().get_history()
}

#[tauri::command]
fn clear_history(state: State<AppState>) {
    state.clipboard_manager.lock().clear();
}

#[tauri::command]
fn delete_item(state: State<AppState>, id: String) {
    state.clipboard_manager.lock().remove_item(&id);
}

#[tauri::command]
fn toggle_pin(state: State<AppState>, id: String) -> Option<ClipboardItem> {
    let result = state.clipboard_manager.lock().toggle_pin(&id);
    if result.is_none() {
        eprintln!("[toggle_pin] Item with id '{}' not found in history.", id);
    }
    result
}

#[tauri::command]
fn get_recent_emojis(state: State<AppState>) -> Vec<EmojiUsage> {
    state.emoji_manager.lock().get_recent()
}

// --- User Settings Commands ---

#[tauri::command]
fn get_user_settings() -> Result<UserSettings, String> {
    let manager = UserSettingsManager::new();
    Ok(manager.load())
}

#[tauri::command]
fn set_user_settings(
    app: AppHandle,
    state: State<AppState>,
    new_settings: UserSettings,
) -> Result<(), String> {
    let manager = UserSettingsManager::new();
    manager.save(&new_settings)?;

    // Update clipboard manager's max history size if it changed
    {
        let mut clipboard_manager = state.clipboard_manager.lock();
        if clipboard_manager.get_max_history_size() != new_settings.max_history_size {
            clipboard_manager.set_max_history_size(new_settings.max_history_size);
        }
    }

    // Emit event to notify all windows that settings have changed
    app.emit("app-settings-changed", &new_settings)
        .map_err(|e| format!("Failed to emit settings changed event: {}", e))?;

    // Refresh tray icon immediately to reflect possible dynamic setting change
    theme_manager::update_dynamic_tray_flag(new_settings.enable_dynamic_tray_icon);

    let app_for_tray = app.clone();
    let settings_for_tray = new_settings.clone();
    tauri::async_runtime::spawn(async move {
        theme_manager::refresh_tray_icon(&app_for_tray, &settings_for_tray).await;
    });

    Ok(())
}

#[tauri::command]
fn is_settings_window_visible(app: AppHandle) -> bool {
    app.get_webview_window("settings")
        .map(|w| w.is_visible().unwrap_or(false))
        .unwrap_or(false)
}

#[tauri::command]
fn open_settings(app: AppHandle) {
    SettingsController::show(&app);
}

// --- Theme Detection Commands ---

/// Get system color scheme from XDG Desktop Portal (supports COSMIC and other modern DEs)
#[tauri::command]
async fn get_system_theme() -> ThemeInfo {
    theme_manager::get_system_color_scheme().await
}

/// Clear the cached theme value (useful when system theme might have changed)
#[tauri::command]
async fn refresh_system_theme() -> ThemeInfo {
    theme_manager::clear_theme_cache().await;
    theme_manager::get_system_color_scheme().await
}

/// Check if the D-Bus event listener is running for theme changes
#[tauri::command]
fn is_theme_listener_active() -> bool {
    theme_manager::is_event_listener_running()
}

#[tauri::command]
async fn paste_item(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    keep_open: Option<bool>,
) -> Result<(), String> {
    let _paste_guard = state.paste_gate.lock().await;

    // 1. Get Item (Scope lock tightly)
    let item = {
        let manager = state.clipboard_manager.lock();
        manager.get_item(&id).cloned()
    };

    match item {
        Some(item) => {
            // 2. Prepare Environment (Hide Window -> Restore Focus)
            WindowController::hide(&app);
            PasteHelper::prepare_target_window(&app).await?;

            // 3. Perform Paste
            let mut manager = state.clipboard_manager.lock();
            manager.paste_item(&item).map_err(|e| e.to_string())?;

            // 4. Notify frontend of history change (item moved to top)
            let history = manager.get_history();
            drop(manager); // Release lock before emitting
            let _ = app.emit("history-sync", &history);
            if keep_open.unwrap_or(false)
                && UserSettingsManager::new().load().reopen_after_paste
            {
                WindowController::show_after_selection(&app);
            }
        }
        None => {
            eprintln!(
                "[paste_item] Item with id '{}' not found in history. Syncing frontend...",
                id
            );
            // Emit event to trigger frontend refresh
            let history = state.clipboard_manager.lock().get_history();
            let _ = app.emit("history-sync", &history);
            return Err(format!("Item '{}' not found. History has been synced.", id));
        }
    }
    Ok(())
}

#[tauri::command]
async fn paste_text(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
    item_type: Option<String>,
    keep_open: Option<bool>,
) -> Result<(), String> {
    let _paste_guard = state.paste_gate.lock().await;

    // 0. Record usage if applicable
    if let Some(t) = item_type.as_deref() {
        if t == "emoji" {
            state.emoji_manager.lock().record_usage(&text);
        }
    }

    // 1. Prepare Environment
    WindowController::hide(&app);
    PasteHelper::prepare_target_window(&app).await?;

    // 2. Set Clipboard & Mark
    {
        let mut manager = state.clipboard_manager.lock();
        manager.mark_text_as_pasted(&text);
        manager.set_text_robust(&text)?;
    }

    // 3. Simulate Paste
    simulate_paste_keystroke().map_err(|e| e.to_string())?;

    if keep_open.unwrap_or(false) && UserSettingsManager::new().load().reopen_after_paste {
        WindowController::show_after_selection(&app);
    }

    Ok(())
}

#[tauri::command]
async fn paste_gif_from_url(
    app: AppHandle,
    state: State<'_, AppState>,
    url: String,
) -> Result<(), String> {
    let _paste_guard = state.paste_gate.lock().await;

    // 1. Download (Blocking) - Window stays open to show loading if UI supports it
    let url_clone = url.clone();
    let file_uri = tokio::task::spawn_blocking(move || {
        win11_clipboard_history_lib::gif_manager::paste_gif_to_clipboard_with_uri(&url_clone)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    // 2. Mark as pasted
    if let Some(uri) = file_uri {
        let mut manager = state.clipboard_manager.lock();
        manager.mark_text_as_pasted(&uri);
        if let Some(trimmed) = uri.strip_suffix('\n') {
            manager.mark_text_as_pasted(trimmed);
        }
    }

    // 3. Prepare Environment & Paste
    WindowController::hide(&app);
    PasteHelper::prepare_target_window(&app).await?;

    // The clipboard is already set by paste_gif_to_clipboard_with_uri, we just need to paste
    simulate_paste_keystroke().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn finish_paste(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let _paste_guard = state.paste_gate.lock().await;
    WindowController::hide(&app);
    PasteHelper::prepare_target_window(&app).await?;
    simulate_paste_keystroke().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn copy_text_to_clipboard(_state: State<'_, AppState>, text: String) -> Result<(), String> {
    // 1. Update Internal Manager (for history consistency, optional but good)
    // Only write to the system clipboard; the history manager is updated by the clipboard watcher if enabled.

    use arboard::Clipboard;
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn finish_setup(app: AppHandle) -> Result<(), String> {
    // 1. Mark first run as complete (redundant but safe)
    win11_clipboard_history_lib::permission_checker::mark_first_run_complete()
        .map_err(|e| e.to_string())?;

    // 2. Close setup window
    if let Some(setup_window) = app.get_webview_window("setup") {
        let _ = setup_window.close();
    }

    // 3. Show main window
    if let Some(main_window) = app.get_webview_window("main") {
        // Ensure it's ready to be shown
        WindowController::position_and_show(&main_window, &app);
    }

    // 4. Emit event to main window to update its state (stop waiting)
    // We emit to all just in case, or specifically to main
    let _ = app.emit("setup_complete", ());

    Ok(())
}

// --- Helper for Paste Logic ---

struct PasteHelper;

impl PasteHelper {
    const TARGET_READY_TIMEOUT: Duration = Duration::from_millis(750);
    const TARGET_READY_POLL_INTERVAL: Duration = Duration::from_millis(2);
    const TARGET_STABLE_SAMPLES: u8 = 2;

    /// Restores focus to the previous window and waits for it to settle.
    /// This ensures keystrokes are sent to the correct application.
    async fn prepare_target_window(app: &AppHandle) -> Result<(), String> {
        // `hide()` is dispatched to Tauri's event loop when commands run off
        // the main thread. Observe its completion before asking X11 to restore
        // focus, otherwise a pending UnmapNotify can steal focus back after an
        // apparently stable X11 sample.
        Self::wait_for_popup_to_release_focus(app).await?;

        if is_wayland() {
            return Ok(());
        }

        match restore_focused_window() {
            Ok(true) => {
                // Focus was *verified* on the target window by the poller;
                // no extra settle time is needed.
            }
            Ok(false) => {
                return Err("Target window did not acquire stable focus".to_string());
            }
            Err(e) => {
                return Err(format!("Focus restoration failed: {}", e));
            }
        }
        Ok(())
    }

    /// Waits for Tauri's asynchronous hide/focus events to settle. On Wayland
    /// this is the strongest target-readiness signal available; on X11 it is
    /// the prerequisite for explicitly restoring the previously focused app.
    async fn wait_for_popup_to_release_focus(app: &AppHandle) -> Result<(), String> {
        let window = app
            .get_webview_window("main")
            .ok_or("Main window is not available")?;
        let start = std::time::Instant::now();
        let mut stable_samples = 0;

        loop {
            let last_state = match (window.is_visible(), window.is_focused()) {
                (Ok(false), Ok(false)) => {
                    stable_samples += 1;
                    if stable_samples >= Self::TARGET_STABLE_SAMPLES {
                        return Ok(());
                    }
                    "visible=false, focused=false (settling)".to_string()
                }
                (visible, focused) => {
                    stable_samples = 0;
                    format!("visible={:?}, focused={:?}", visible, focused)
                }
            };

            if start.elapsed() >= Self::TARGET_READY_TIMEOUT {
                return Err(format!(
                    "Timed out waiting for the clipboard popup to release focus ({})",
                    last_state
                ));
            }
            tokio::time::sleep(Self::TARGET_READY_POLL_INTERVAL).await;
        }
    }
}

// --- Window Controller (Visibility & Positioning) ---

static POPUP_ACTIVATION: PopupActivation = PopupActivation::new();

struct WindowController;

impl WindowController {
    pub fn toggle(app: &AppHandle) {
        Self::toggle_with_tab(app, None);
    }

    /// Toggle window visibility with optional tab selection
    /// If tab is Some("emoji"), it will emit an event to switch to the emoji tab
    pub fn toggle_with_tab(app: &AppHandle, tab: Option<&str>) {
        // User-initiated toggle - mark that we're now allowing shows
        // This stops the background enforcer from hiding the window
        if STARTED_IN_BACKGROUND.load(Ordering::SeqCst) {
            INITIAL_SHOW_ALLOWED.store(true, Ordering::SeqCst);
        }

        if let Some(window) = app.get_webview_window("main") {
            if window.is_visible().unwrap_or(false) {
                // If window is visible, emit tab switch event if tab is specified
                // This allows Super+. to switch to emoji tab even when window is open
                if let Some(tab_name) = tab {
                    let _ = app.emit("switch-tab", tab_name);
                } else {
                    Self::hide(app);
                }
            } else {
                save_focused_window();
                // Emit tab switch event before showing window
                if let Some(tab_name) = tab {
                    let _ = app.emit("switch-tab", tab_name);
                }

                // Immediate cleanup of outdated items before showing
                if let Some(state) = app.try_state::<AppState>() {
                    let settings = UserSettingsManager::new().load();
                    let interval_in_minutes = settings.auto_delete_interval_in_minutes();

                    let mut manager = state.clipboard_manager.lock();
                    if manager.cleanup_old_items(interval_in_minutes) {
                        // Mirror background cleanup: persist history explicitly, then emit event
                        manager.save_history();
                        let _ = app.emit("history-cleared", ());
                    }
                }

                Self::position_and_show(&window, app);
            }
        }
    }

    pub fn hide(app: &AppHandle) {
        POPUP_ACTIVATION.invalidate();
        if let Some(window) = app.get_webview_window("main") {
            // FLUSH CONFIG TO DISK ON HIDE
            if let Some(state) = app.try_state::<AppState>() {
                if is_wayland() {
                    state.config_manager.lock().sync_to_disk();
                }
            }
            // Clear a temporary Wayland activation hint even when its delayed
            // cleanup was cancelled by this hide.
            if cfg!(target_os = "linux") && is_wayland() {
                let _ = window.set_always_on_top(false);
            }
            let _ = window.hide();
        }
    }

    /// Return the clipboard popup after pasting the selected item into the
    /// previously focused application.
    fn show_after_selection(app: &AppHandle) {
        if let Some(window) = app.get_webview_window("main") {
            Self::show_and_activate(&window, app, false);
        }
    }

    fn position_and_show(window: &WebviewWindow, app: &AppHandle) {
        Self::show_and_activate(window, app, true);
    }

    fn show_and_activate(window: &WebviewWindow, app: &AppHandle, position: bool) {
        let generation = POPUP_ACTIVATION.invalidate();
        let window_for_show = window.clone();
        let app = app.clone();
        // Check cancellation and apply window changes together on the UI thread.
        // A delayed worker must not queue focus changes for an older invocation.
        let _ = window.run_on_main_thread(move || {
            if !POPUP_ACTIVATION.is_current(generation) {
                return;
            }
            let window = window_for_show;
            let wayland = cfg!(target_os = "linux") && is_wayland();
            if position {
                if wayland {
                    Self::position_for_wayland(&window, &app.state::<AppState>());
                } else {
                    Self::position_for_non_wayland(&window);
                }
            }
            let _ = window.show();
            if wayland {
                let _ = window.set_always_on_top(true);
            }
            // Use the native Tauri activation API on Windows/macOS as well as
            // Linux; an unknown/non-Wayland session is not necessarily X11.
            let _ = window.set_focus();
            let _ = app.emit("window-shown", ());

            if wayland {
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(120)).await;
                    let window_for_check = window.clone();
                    let _ = window.run_on_main_thread(move || {
                        if !POPUP_ACTIVATION.is_current(generation)
                            || !window_for_check.is_visible().unwrap_or(false)
                        {
                            return;
                        }
                        // Only remove the temporary hint. Never refocus after
                        // a delay: the user may already be switching apps.
                        let _ = window_for_check.set_always_on_top(false);
                    });
                });
            }
        });
    }

    fn position_for_wayland(window: &WebviewWindow, state: &State<AppState>) {
        let config = state.config_manager.lock();

        if let Ok(monitors) = window.available_monitors() {
            if !monitors.is_empty() {
                let win_size = window.outer_size().unwrap_or(PhysicalSize::new(360, 480));

                let window_state = config.get_state();
                let pos = resolve_window_position(&window_state, &monitors, win_size);

                let _ = window.set_position(pos);
            }
        }
    }

    fn position_for_non_wayland(window: &WebviewWindow) {
        let (cursor_x, cursor_y) = match Self::get_cursor_position(window) {
            Some(pos) => pos,
            None => {
                // Fallback: center the window if we can't get cursor position
                let _ = window.center();
                return;
            }
        };

        let target_monitor = Self::find_monitor_containing(window, cursor_x, cursor_y)
            .or_else(|| window.current_monitor().ok().flatten())
            .or_else(|| window.primary_monitor().ok().flatten());

        if let Some(monitor) = target_monitor {
            let pos = Self::clamp_window_to_monitor(window, &monitor, cursor_x, cursor_y);
            let _ = window.set_position(pos);
        }
    }

    fn find_monitor_containing(window: &WebviewWindow, x: i32, y: i32) -> Option<Monitor> {
        window.available_monitors().ok()?.into_iter().find(|m| {
            let p = m.position();
            let s = m.size();
            x >= p.x && x < (p.x + s.width as i32) && y >= p.y && y < (p.y + s.height as i32)
        })
    }

    fn clamp_window_to_monitor(
        window: &WebviewWindow,
        monitor: &Monitor,
        x: i32,
        y: i32,
    ) -> PhysicalPosition<i32> {
        let win_size = window.outer_size().unwrap_or(PhysicalSize::new(360, 480));
        let m_pos = monitor.position();
        let m_size = monitor.size();

        let max_x = m_pos.x + m_size.width as i32 - win_size.width as i32;
        let max_y = m_pos.y + m_size.height as i32 - win_size.height as i32;

        // Clamp with 10px padding
        let safe_x = x.clamp(m_pos.x + 10, max_x - 10);
        let safe_y = y.clamp(m_pos.y + 10, max_y - 10);

        PhysicalPosition::new(safe_x, safe_y)
    }

    fn get_cursor_position(window: &WebviewWindow) -> Option<(i32, i32)> {
        if let Ok(pos) = window.cursor_position() {
            return Some((pos.x as i32, pos.y as i32));
        }

        #[cfg(target_os = "linux")]
        if win11_clipboard_history_lib::session::is_x11() {
            if let Some(p) = Self::get_cursor_xdotool() {
                return Some(p);
            }
            if let Some(p) = Self::get_cursor_x11() {
                return Some(p);
            }
        }

        None
    }

    #[cfg(target_os = "linux")]
    fn get_cursor_xdotool() -> Option<(i32, i32)> {
        let output = std::process::Command::new("xdotool")
            .args(["getmouselocation", "--shell"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let s = String::from_utf8_lossy(&output.stdout);
        let (mut x, mut y) = (None, None);
        for line in s.lines() {
            if let Some(v) = line.strip_prefix("X=") {
                x = v.parse().ok();
            }
            if let Some(v) = line.strip_prefix("Y=") {
                y = v.parse().ok();
            }
        }
        x.zip(y)
    }

    #[cfg(target_os = "linux")]
    fn get_cursor_x11() -> Option<(i32, i32)> {
        use x11rb::connection::Connection;
        use x11rb::protocol::xproto::ConnectionExt;
        let (conn, n) = x11rb::connect(None).ok()?;
        let root = conn.setup().roots.get(n)?.root;
        let r = conn.query_pointer(root).ok()?.reply().ok()?;
        Some((r.root_x as i32, r.root_y as i32))
    }
}

// --- Settings Window Controller ---

struct SettingsController;

impl SettingsController {
    /// Shows the settings window, recreating it if somehow destroyed
    pub fn show(app: &AppHandle) {
        use tauri::{WebviewUrl, WebviewWindowBuilder};

        match app.get_webview_window("settings") {
            Some(window) => {
                let _ = window.show();
                let _ = window.set_focus();
            }
            None => {
                // Fallback: recreate the window if it was somehow destroyed
                eprintln!(
                    "[SettingsController] Settings window missing, recreating as fallback..."
                );

                match WebviewWindowBuilder::new(
                    app,
                    "settings",
                    WebviewUrl::App("index.html".into()),
                )
                .title("Settings - Clipboard History")
                .inner_size(480.0, 520.0)
                .resizable(false)
                .decorations(true)
                .transparent(false)
                .visible(true)
                .skip_taskbar(false)
                .always_on_top(false)
                .center()
                .focused(true)
                .build()
                {
                    Ok(_) => {
                        println!("[SettingsController] Settings window recreated successfully")
                    }
                    Err(e) => eprintln!("[SettingsController] Failed to recreate window: {}", e),
                }
            }
        }
    }
}

// --- Window Event Helper ---

/// Native border resizing temporarily gives focus to the compositor. Register
/// on GTK itself: these presses do not reliably reach the React webview.
#[cfg(target_os = "linux")]
fn install_resize_focus_guard(
    window: &WebviewWindow,
    resize_focus_loss: Arc<NativeWindowInteraction>,
    focus_generation: Arc<AtomicU64>,
) {
    use gtk::prelude::*;

    if let Ok(native_window) = window.gtk_window() {
        let interaction_on_hide = resize_focus_loss.clone();
        native_window.connect_hide(move |_| {
            POPUP_ACTIVATION.invalidate();
            interaction_on_hide.reset();
        });
        // A normal button-press signal runs during bubbling and can be consumed
        // by GTK's resize handling before our callback receives it. Observe the
        // press during capture without claiming or blocking the gesture.
        let press = gtk::GestureMultiPress::new(&native_window);
        press.set_button(1);
        press.set_propagation_phase(gtk::PropagationPhase::Capture);
        let native_weak = native_window.downgrade();
        let interaction_on_release = resize_focus_loss.clone();
        press.connect_pressed(move |_, _, x, y| {
            let Some(native) = native_weak.upgrade() else {
                return;
            };
            // Gesture coordinates are local to the native window, avoiding
            // global screen-coordinate assumptions on Wayland.
            let (width, height) = native.size();
            let border = f64::from(native.scale_factor() * 5);
            let within_window = x >= -border
                && y >= -border
                && x <= f64::from(width) + border
                && y <= f64::from(height) + border;
            let on_border = x <= border
                || y <= border
                || x >= f64::from(width) - border
                || y >= f64::from(height) - border;
            // The top grip is draggable from the React header. GTK moves
            // the whole native window from this area, so it needs the same
            // protection as a border resize.
            let on_drag_handle = within_window && (0.0..=24.0).contains(&y);
            if within_window {
                resize_focus_loss.begin_press(
                    native.is_resizable(),
                    native.is_maximized(),
                    native.window().is_some_and(|window| {
                        window.state().contains(gtk::gdk::WindowState::FULLSCREEN)
                    }),
                    on_border,
                    on_drag_handle,
                );
            }
        });
        let native_weak = native_window.downgrade();
        let window_on_release = window.clone();
        press.connect_released(move |_, _, x, y| {
            interaction_on_release.release();
            if interaction_on_release.is_protected()
                || !window_on_release.is_visible().unwrap_or(false)
            {
                return;
            }
            let Some(native) = native_weak.upgrade() else {
                return;
            };
            let (width, height) = native.size();
            if x >= 0.0 && y >= 0.0 && x < f64::from(width) && y < f64::from(height) {
                // A held click inside must not be mistaken for an outside
                // click, even when GTK temporarily loses keyboard focus.
                if !native.is_active() {
                    let _ = window_on_release.set_focus();
                }
            } else {
                schedule_hide_after_focus_loss(
                    window_on_release.clone(),
                    focus_generation.clone(),
                    interaction_on_release.clone(),
                    focus_generation.load(Ordering::SeqCst),
                );
            }
        });
        // Retain the controller for the native window's lifetime.
        native_window.connect_destroy(move |_| press.reset());
    }
}

/// GTK focus-out events can occur during native resizing or internal focus
/// changes. Confirm that the top-level window really became inactive after
/// those events settle, rather than treating every event as an outside click.
fn schedule_hide_after_focus_loss(
    window: WebviewWindow,
    focus_generation: Arc<AtomicU64>,
    resize_focus_loss: Arc<NativeWindowInteraction>,
    generation: u64,
) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(150)).await;

        let window_for_check = window.clone();
        let _ = window.run_on_main_thread(move || {
            if focus_generation.load(Ordering::SeqCst) != generation
                || resize_focus_loss.focus_lost()
                || !window_for_check.is_visible().unwrap_or(false)
                // On Linux this queries GTK's top-level is_active property,
                // rather than relying on the earlier focus-out notification.
                || window_for_check.is_focused().unwrap_or(true)
            {
                return;
            }

            let app = window_for_check.app_handle();
            // Preserve the popup while Settings provides a live preview.
            if app
                .get_webview_window("settings")
                .is_some_and(|settings| settings.is_visible().unwrap_or(false))
            {
                return;
            }

            WindowController::hide(app);
        });
    });
}

fn handle_window_moved_for_wayland(
    window: &WebviewWindow,
    state: &State<AppState>,
    _pos: &PhysicalPosition<i32>,
) {
    if !is_wayland() || !window.is_visible().unwrap_or(false) {
        return;
    }

    let _monitor_name = window
        .current_monitor()
        .ok()
        .flatten()
        .and_then(|m| m.name().map(|n| n.to_string()));

    let _config = state.config_manager.lock();
    // UPDATE MEMORY ONLY (No Disk I/O here)
    // config.update_state(monitor_name, pos.x, pos.y);
}

// --- Background Listeners ---

fn start_clipboard_watcher(app: AppHandle, clipboard_manager: Arc<Mutex<ClipboardManager>>) {
    std::thread::spawn(move || {
        let mut last_text_hash: Option<u64> = None;
        let mut last_image_hash: Option<u64> = None;
        let mut cleanup_counter = 0;

        loop {
            std::thread::sleep(Duration::from_millis(500));
            cleanup_counter += 1;

            let mut manager = clipboard_manager.lock();

            // Background cleanup every ~30 seconds (60 * 500ms)
            if cleanup_counter >= 60 {
                cleanup_counter = 0;
                let settings = UserSettingsManager::new().load();
                let interval_in_minutes = settings.auto_delete_interval_in_minutes();

                if interval_in_minutes > 0 && manager.cleanup_old_items(interval_in_minutes) {
                    println!("[Watcher] Background cleanup triggered sync");
                    let _ = app.emit("history-cleared", ());
                }
            }

            // Text
            if let Ok(text) = manager.get_current_text() {
                if !text.is_empty() {
                    let text_hash =
                        win11_clipboard_history_lib::clipboard_manager::calculate_hash(&text);

                    if Some(text_hash) != last_text_hash {
                        last_text_hash = Some(text_hash);
                        last_image_hash = None;

                        // Try to get HTML content for rich text support
                        let html = manager.get_current_html();

                        if let Some(item) = manager.add_text(text, html) {
                            let _ = app.emit("clipboard-changed", &item);
                        }
                    }
                }
            }

            // Image
            if let Ok(Some((image_data, hash))) = manager.get_current_image() {
                if Some(hash) != last_image_hash {
                    last_image_hash = Some(hash);
                    last_text_hash = None;
                    if let Some(item) = manager.add_image(image_data, hash) {
                        let _ = app.emit("clipboard-changed", &item);
                    }
                }
            }
        }
    });
}

// --- Main ---

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Handle --version / -v
    if args.iter().any(|arg| arg == "--version" || arg == "-v") {
        println!("win11-clipboard-history {}", VERSION);
        return;
    }

    // Handle --help / -h
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("win11-clipboard-history {}", VERSION);
        println!();
        println!("USAGE:");
        println!("    win11-clipboard-history [OPTIONS]");
        println!();
        println!("OPTIONS:");
        println!("    -h, --help       Show this help message");
        println!("    -v, --version    Show version information");
        println!("        --background Start minimized to system tray (for autostart)");
        println!("        --settings   Open settings window on startup");
        println!("        --emoji      Open with emoji picker tab selected");
        println!();
        println!("SHORTCUTS:");
        println!("    Super+V          Open clipboard history");
        println!("    Super+.          Open emoji picker");
        println!("    Ctrl+Alt+V       Alternative shortcut");
        return;
    }

    // MUST run before Tauri / WebKit init – detects NVIDIA & AppImage and
    // sets WEBKIT_DISABLE_DMABUF_RENDERER=1 when needed.
    rendering_env::init();

    // Check if --background flag is present (start minimized to tray)
    let start_in_background = args.iter().any(|arg| arg == "--background");
    if start_in_background {
        println!("[Startup] Starting in background mode (system tray only)");
        STARTED_IN_BACKGROUND.store(true, Ordering::SeqCst);
    }

    // Check if --settings flag is present (for first instance startup)
    let open_settings_on_start = args.iter().any(|arg| arg == "--settings");

    // Check if --emoji flag is present (open with emoji tab)
    let open_emoji_on_start = args.iter().any(|arg| arg == "--emoji");

    // Clone for use in setup closure
    let start_in_background_clone = start_in_background;
    let open_emoji_on_start_clone = open_emoji_on_start;

    win11_clipboard_history_lib::session::init();

    let base_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("win11-clipboard-history");

    // Ensure base directory exists
    if let Err(e) = std::fs::create_dir_all(&base_dir) {
        eprintln!("Failed to create base directory: {}", e);
    }

    let history_path = base_dir.join("history.json");

    // Load user settings to get max_history_size
    let user_settings = UserSettingsManager::new().load();
    let clipboard_manager = Arc::new(Mutex::new(ClipboardManager::new(
        history_path,
        user_settings.max_history_size,
    )));

    let emoji_manager = Arc::new(Mutex::new(EmojiManager::new(base_dir.clone())));

    let config_manager = Arc::new(Mutex::new(ConfigManager::new(base_dir)));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        // Global shortcut plugin for cross-platform hotkeys
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        // Single Instance Plugin: When user triggers shortcut and app is already running,
        // the OS launches a new instance which signals the existing one to toggle
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            // Check if --settings flag is present
            if argv.iter().any(|arg| arg == "--settings") {
                println!(
                    "[SingleInstance] Secondary instance with --settings flag, opening settings..."
                );
                SettingsController::show(app);
            } else if argv.iter().any(|arg| arg == "--emoji") {
                println!(
                    "[SingleInstance] Secondary instance with --emoji flag, opening emoji picker..."
                );
                WindowController::toggle_with_tab(app, Some("emoji"));
            } else {
                println!("[SingleInstance] Secondary instance detected, toggling window...");
                WindowController::toggle(app);
            }
        }))
        .manage(AppState {
            clipboard_manager: clipboard_manager.clone(),
            emoji_manager: emoji_manager.clone(),
            config_manager: config_manager.clone(),
            paste_gate: tokio::sync::Mutex::new(()),
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                if window.label() == "setup" {
                    // Check if setup was effectively finished.
                    // If the user clicked "Start Using", `finish_setup` would have been called.
                    // `finish_setup` calls `mark_first_run_complete`.
                    if win11_clipboard_history_lib::permission_checker::is_first_run() {
                         println!("[Setup] Setup window closed without completion. Exiting app.");
                         window.app_handle().exit(0);
                    }
                }
            }
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // This runs only for the primary instance, after the
            // single-instance plugin has rejected shortcut helper processes.
            // Starting it earlier would hotplug a temporary keyboard every
            // time the desktop launches the shortcut command.
            win11_clipboard_history_lib::input_simulator::init();
            win11_clipboard_history_lib::paste_sync::init();

            // FIRST THING: If started in background mode, immediately hide the main window
            // This runs before anything else to prevent the window from appearing
            if start_in_background_clone {
                if let Some(main_window) = app.get_webview_window("main") {
                    WindowController::hide(main_window.app_handle());
                    println!("[Setup] Immediately hiding main window for background mode");
                }
            }

            // Auto-migrate old autostart entries to use the wrapper script
            // This fixes existing installations where autostart points to the binary directly
            match autostart_manager::autostart_migrate() {
                Ok(true) => println!("[Setup] Migrated autostart entry to use wrapper script"),
                Ok(false) => {} // No migration needed
                Err(e) => eprintln!("[Setup] Failed to migrate autostart: {}", e),
            }

            let show = MenuItem::with_id(app, "show", "Show Clipboard", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &settings, &quit])?;



            // Get temp directory for tray icon (avoids permission issues with XDG_RUNTIME_DIR)
            let temp_dir = std::env::temp_dir().join("win11-clipboard-history");
            std::fs::create_dir_all(&temp_dir).ok();

            // Initial Dynamic Icon Setup
            let settings_manager = UserSettingsManager::new();
            let settings = settings_manager.load();

            // Initialize atomic flag for the listener loop
            theme_manager::update_dynamic_tray_flag(settings.enable_dynamic_tray_icon);

            let (icon, use_template_icon) = theme_manager::initial_tray_icon(&settings);

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .icon_as_template(use_template_icon)
                .tooltip("Clipboard History")
                .temp_dir_path(temp_dir)
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => WindowController::toggle(app),
                    "settings" => SettingsController::show(app),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        WindowController::toggle(tray.app_handle());
                    }
                })
                .build(app)?;

            // Update icon asynchronously if dynamic is enabled (to fix the initial default icon)
            if settings.enable_dynamic_tray_icon {
                 let app_handle_bg = app.handle().clone();
                 let settings_bg = settings.clone();
                 tauri::async_runtime::spawn(async move {
                    theme_manager::refresh_tray_icon(&app_handle_bg, &settings_bg).await;
                 });
            }

            // Verify that settings window was created from config
            if app.get_webview_window("settings").is_none() {
                eprintln!("[Setup] FATAL: Settings window missing from config");
            } else {
                println!("[Setup] Settings window created successfully from config");
            }

            // Window Event Handlers (Focus & Move)
            let main_window = app.get_webview_window("main").unwrap();
            let w_clone = main_window.clone();
            let focus_generation = Arc::new(AtomicU64::new(0));
            let resize_focus_loss = Arc::new(NativeWindowInteraction::default());
            #[cfg(target_os = "linux")]
            install_resize_focus_guard(
                &main_window,
                resize_focus_loss.clone(),
                focus_generation.clone(),
            );

            main_window.on_window_event(move |event| match event {
                // Block any window show attempts when started in background mode
                // This catches cases where GTK/Tauri automatically shows the window
                WindowEvent::Focused(true) => {
                    resize_focus_loss.focus_gained();
                    // Cancel dismissal queued for an earlier focus loss.
                    focus_generation.fetch_add(1, Ordering::SeqCst);
                    // Load both flags atomically with SeqCst to avoid race conditions
                    let started_in_background = STARTED_IN_BACKGROUND.load(Ordering::SeqCst);
                    let initial_show_allowed = INITIAL_SHOW_ALLOWED.load(Ordering::SeqCst);

                    // If started in background and initial show hasn't been allowed yet,
                    // immediately hide the window
                    if started_in_background && !initial_show_allowed {
                        println!("[WindowController] Background mode: intercepted focus, hiding window");
                        WindowController::hide(w_clone.app_handle());
                    }
                }
                WindowEvent::Focused(false) => {
                    POPUP_ACTIVATION.invalidate();
                    if cfg!(target_os = "linux") && is_wayland() {
                        let _ = w_clone.set_always_on_top(false);
                    }
                    let generation = focus_generation.fetch_add(1, Ordering::SeqCst) + 1;
                    // Retain the interaction marker: GTK may report focus loss
                    // before the captured press reaches this event handler.
                    // Focused(true) clears it when the compositor returns focus.
                    if resize_focus_loss.focus_lost() {
                        println!("[WindowController] Ignoring focus loss during resize/move");
                        return;
                    }
                    schedule_hide_after_focus_loss(
                        w_clone.clone(),
                        focus_generation.clone(),
                        resize_focus_loss.clone(),
                        generation,
                    );
                }

                WindowEvent::Moved(pos) => {
                    let state = w_clone.state::<AppState>();
                    handle_window_moved_for_wayland(&w_clone, &state, pos);
                }
                _ => {}
            });

            start_clipboard_watcher(app_handle.clone(), clipboard_manager.clone());

            // Start theme change listener (D-Bus event-based, more efficient than polling)
            {
                let app_handle_for_theme = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) =
                        theme_manager::start_theme_listener(app_handle_for_theme).await
                    {
                        eprintln!("[ThemeManager] Failed to start theme listener: {}", e);
                    }
                });
            }

            // If --settings flag was passed on first startup, open the settings window
            if open_settings_on_start {
                SettingsController::show(&app_handle);
            }

            // If --emoji flag was passed on first startup, emit switch-tab event
            // This needs a small delay to ensure the frontend is ready
            if open_emoji_on_start_clone {
                let app_handle_for_emoji = app_handle.clone();
                std::thread::spawn(move || {
                    // Wait for frontend to be ready
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    let _ = app_handle_for_emoji.emit("switch-tab", "emoji");
                });
            }

            // If --background flag was passed, ensure the main window stays hidden
            // This is the primary mechanism for starting minimized to tray
            // Background mode: spawn enforcer thread as fallback
            // This catches cases where something shows the window after our initial hide
            if start_in_background_clone {
                if let Some(main_window) = app.get_webview_window("main") {
                    // Spawn a background task that keeps checking and hiding the window
                    // for the first few seconds, in case something shows it after we hide it
                    let window_clone = main_window.clone();
                    std::thread::spawn(move || {
                        for i in 0..10 {
                            std::thread::sleep(std::time::Duration::from_millis(200));

                            // User has already triggered a toggle, stop blocking
                            if INITIAL_SHOW_ALLOWED.load(Ordering::SeqCst) {
                                break;
                            }

                            // Check if window still exists and is visible, then hide it
                            // Use unwrap_or(false) to safely handle cases where window was destroyed
                            match window_clone.is_visible() {
                                Ok(true) => {
                                    println!("[Startup] Background enforcer #{}: window was visible, hiding again", i + 1);
                                    WindowController::hide(window_clone.app_handle());
                                }
                                Ok(false) => {} // Window exists but is hidden, nothing to do
                                Err(_) => break, // Window was destroyed, stop the enforcer
                            }
                        }
                        println!("[Startup] Background enforcer finished");
                    });
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_history,
            clear_history,
            delete_item,
            toggle_pin,
            paste_item,
            paste_text,
            get_recent_emojis,
            paste_gif_from_url,
            finish_paste,
            finish_setup, // Register the new command
            get_user_settings,
            set_user_settings,
            is_settings_window_visible,
            open_settings,
            copy_text_to_clipboard,
            get_system_theme,
            refresh_system_theme,
            is_theme_listener_active,
            permission_checker::check_permissions,
            permission_checker::fix_permissions_now,
            permission_checker::is_first_run,
            permission_checker::mark_first_run_complete,
            permission_checker::reset_first_run,
            shortcut_setup::get_desktop_environment,
            shortcut_setup::register_de_shortcut,
            shortcut_setup::check_shortcut_tools,
            shortcut_setup::detect_conflicts,
            shortcut_setup::resolve_conflicts,
            autostart_manager::autostart_enable,
            autostart_manager::autostart_disable,
            autostart_manager::autostart_is_enabled,
            autostart_manager::autostart_migrate,
            rendering_env::get_rendering_environment,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
