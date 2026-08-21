//! Window Controller Module
//!
//! Manages the visibility, positioning, and lifecycle of the main popup
//! window and the settings window. All window show/hide/toggle logic lives
//! here so that `main.rs` stays focused on application bootstrap.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{
    AppHandle, Emitter, Manager, Monitor, PhysicalPosition, PhysicalSize, State, WebviewWindow,
    WindowEvent,
};

use crate::config_manager::{resolve_window_position, ConfigManager};
use crate::focus_manager::{restore_focused_window, save_focused_window, x11_robust_activate};
use crate::session::is_wayland;
use crate::user_settings::UserSettingsManager;
use crate::AppState;

/// Global flag to track if we started in background mode
pub static STARTED_IN_BACKGROUND: AtomicBool = AtomicBool::new(false);

/// Global flag indicating whether the initial show is allowed
pub static INITIAL_SHOW_ALLOWED: AtomicBool = AtomicBool::new(false);

// ---------------------------------------------------------------------------
// PasteHelper — restores focus and waits for the target to settle
// ---------------------------------------------------------------------------

pub struct PasteHelper;

impl PasteHelper {
    const TARGET_READY_TIMEOUT: Duration = Duration::from_millis(750);
    const TARGET_READY_POLL_INTERVAL: Duration = Duration::from_millis(2);
    const TARGET_STABLE_SAMPLES: u8 = 2;

    /// Restores focus to the previous window and waits for it to settle.
    pub async fn prepare_target_window(app: &AppHandle) -> Result<(), String> {
        Self::wait_for_popup_to_release_focus(app).await?;

        if is_wayland() {
            return Ok(());
        }

        match restore_focused_window() {
            Ok(true) => { /* Focus verified */ }
            Ok(false) => {
                return Err("Target window did not acquire stable focus".to_string());
            }
            Err(e) => {
                return Err(format!("Focus restoration failed: {e}"));
            }
        }
        Ok(())
    }

    /// Waits for Tauri's asynchronous hide/focus events to settle.
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
                    format!("visible={visible:?}, focused={focused:?}")
                }
            };

            if start.elapsed() >= Self::TARGET_READY_TIMEOUT {
                return Err(format!(
                    "Timed out waiting for the clipboard popup to release focus ({last_state})"
                ));
            }
            tokio::time::sleep(Self::TARGET_READY_POLL_INTERVAL).await;
        }
    }
}

// ---------------------------------------------------------------------------
// WindowController — show / hide / toggle / position
// ---------------------------------------------------------------------------

pub struct WindowController;

impl WindowController {
    pub fn toggle(app: &AppHandle) {
        Self::toggle_with_tab(app, None);
    }

    /// Toggle window visibility with optional tab selection
    pub fn toggle_with_tab(app: &AppHandle, tab: Option<&str>) {
        if STARTED_IN_BACKGROUND.load(Ordering::SeqCst) {
            INITIAL_SHOW_ALLOWED.store(true, Ordering::SeqCst);
        }

        if let Some(window) = app.get_webview_window("main") {
            if window.is_visible().unwrap_or(false) {
                if let Some(tab_name) = tab {
                    let _ = app.emit("switch-tab", tab_name);
                } else {
                    let _ = window.hide();
                }
            } else {
                save_focused_window();
                if let Some(tab_name) = tab {
                    let _ = app.emit("switch-tab", tab_name);
                }

                // Immediate cleanup of outdated items before showing
                if let Some(state) = app.try_state::<AppState>() {
                    let settings = UserSettingsManager::new().load();
                    let interval_in_minutes = settings.auto_delete_interval_in_minutes();

                    let mut manager = state.clipboard_manager.lock();
                    if manager.cleanup_old_items(interval_in_minutes) {
                        manager.mark_dirty();
                        let _ = app.emit("history-cleared", ());
                    }
                }

                Self::position_and_show(&window, app);
            }
        }
    }

    pub fn hide(app: &AppHandle) {
        if let Some(window) = app.get_webview_window("main") {
            if let Some(state) = app.try_state::<AppState>() {
                if is_wayland() {
                    state.config_manager.lock().sync_to_disk();
                }
            }
            let _ = window.hide();
        }
    }

    pub fn position_and_show(window: &WebviewWindow, app: &AppHandle) {
        let state = app.state::<AppState>();

        if is_wayland() {
            Self::position_for_wayland(window, &state);
        } else {
            Self::position_for_non_wayland(window);
        }

        let is_wayland_session = is_wayland();

        if is_wayland_session {
            let _ = window.show();
            let _ = window.set_always_on_top(true);
            let _ = window.set_focus();
        } else {
            let _ = window.show();
        }

        let window_clone = window.clone();
        let app_clone = app.clone();

        std::thread::spawn(move || {
            if is_wayland_session {
                std::thread::sleep(Duration::from_millis(100));
                let _ = window_clone.set_always_on_top(false);
                let _ = window_clone.set_focus();
            } else {
                if let Err(e) = x11_robust_activate("Clipboard History") {
                    tracing::warn!("[WindowController] X11 activation failed: {e}");
                    let _ = Self::x11_activate_window_xdotool();
                }
            }

            let _ = app_clone.emit("window-shown", ());
        });
    }

    /// Activate window on X11 using xdotool (fallback method)
    fn x11_activate_window_xdotool() -> Result<(), String> {
        use std::process::Command;

        let output = Command::new("xdotool")
            .args(["search", "--name", "Clipboard History"])
            .output()
            .map_err(|e| format!("xdotool search failed: {e}"))?;

        let window_ids = String::from_utf8_lossy(&output.stdout);
        if let Some(window_id) = window_ids.lines().next() {
            Command::new("xdotool")
                .args(["windowactivate", "--sync", window_id])
                .output()
                .map_err(|e| format!("windowactivate failed: {e}"))?;
            Ok(())
        } else {
            Err("Window not found".to_string())
        }
    }

    fn position_for_wayland(window: &WebviewWindow, state: &State<AppState>) {
        let config = state.config_manager.lock();

        if let Ok(monitors) = window.available_monitors() {
            if !monitors.is_empty() {
                let win_size = window
                    .outer_size()
                    .unwrap_or(PhysicalSize::new(360, 480));

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

    fn find_monitor_containing(
        window: &WebviewWindow,
        x: i32,
        y: i32,
    ) -> Option<Monitor> {
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
        let win_size = window
            .outer_size()
            .unwrap_or(PhysicalSize::new(360, 480));
        let m_pos = monitor.position();
        let m_size = monitor.size();

        let max_x = m_pos.x + m_size.width as i32 - win_size.width as i32;
        let max_y = m_pos.y + m_size.height as i32 - win_size.height as i32;

        let safe_x = x.clamp(m_pos.x + 10, max_x - 10);
        let safe_y = y.clamp(m_pos.y + 10, max_y - 10);

        PhysicalPosition::new(safe_x, safe_y)
    }

    fn get_cursor_position(window: &WebviewWindow) -> Option<(i32, i32)> {
        if let Ok(pos) = window.cursor_position() {
            return Some((pos.x as i32, pos.y as i32));
        }

        if let Some(p) = Self::get_cursor_xdotool() {
            return Some(p);
        }
        if let Some(p) = Self::get_cursor_x11() {
            return Some(p);
        }

        None
    }

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

    fn get_cursor_x11() -> Option<(i32, i32)> {
        use x11rb::connection::Connection;
        use x11rb::protocol::xproto::ConnectionExt;
        let (conn, n) = x11rb::connect(None).ok()?;
        let root = conn.setup().roots.get(n)?.root;
        let r = conn.query_pointer(root).ok()?.reply().ok()?;
        Some((r.root_x as i32, r.root_y as i32))
    }
}

// ---------------------------------------------------------------------------
// SettingsController
// ---------------------------------------------------------------------------

pub struct SettingsController;

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
                tracing::warn!(
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
                        tracing::info!("[SettingsController] Settings window recreated successfully")
                    }
                    Err(e) => {
                        tracing::error!("[SettingsController] Failed to recreate window: {e}")
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Window event helpers
// ---------------------------------------------------------------------------

/// Handle window moved events for Wayland position tracking
pub fn handle_window_moved_for_wayland(
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

    let mut config = state.config_manager.lock();
    config.update_state(_monitor_name, _pos.x, _pos.y);
}

/// Register window event handlers for focus and move events
pub fn register_window_events(
    main_window: &WebviewWindow,
    app_handle: &AppHandle,
) {
    let w_clone = main_window.clone();
    let app_handle_for_event = app_handle.clone();

    main_window.on_window_event(move |event| match event {
        WindowEvent::Focused(true) => {
            let started_in_background = STARTED_IN_BACKGROUND.load(Ordering::SeqCst);
            let initial_show_allowed = INITIAL_SHOW_ALLOWED.load(Ordering::SeqCst);

            if started_in_background && !initial_show_allowed {
                tracing::debug!("[WindowController] Background mode: intercepted focus, hiding window");
                let _ = w_clone.hide();
            }
        }
        WindowEvent::Focused(false) => {
            let state = w_clone.state::<AppState>();
            if state.is_mouse_inside.load(Ordering::Relaxed) {
                return;
            }

            if let Some(settings_window) =
                app_handle_for_event.get_webview_window("settings")
            {
                if settings_window.is_visible().unwrap_or(false) {
                    return;
                }
            }

            if is_wayland() {
                state.config_manager.lock().sync_to_disk();
            }

            let _ = w_clone.hide();
        }

        WindowEvent::Moved(pos) => {
            let state = w_clone.state::<AppState>();
            handle_window_moved_for_wayland(&w_clone, &state, pos);
        }
        _ => {}
    });
}

/// Spawn a background enforcer thread that keeps the window hidden
/// during background-mode startup.
pub fn spawn_background_enforcer(main_window: &WebviewWindow) {
    let window_clone = main_window.clone();
    std::thread::spawn(move || {
        for i in 0..10 {
            std::thread::sleep(Duration::from_millis(200));

            if INITIAL_SHOW_ALLOWED.load(Ordering::SeqCst) {
                break;
            }

            match window_clone.is_visible() {
                Ok(true) => {
                    tracing::debug!(
                        "[Startup] Background enforcer #{}: window was visible, hiding again",
                        i + 1
                    );
                    let _ = window_clone.hide();
                }
                Ok(false) => {}
                Err(_) => break,
            }
        }
        tracing::debug!("[Startup] Background enforcer finished");
    });
}
