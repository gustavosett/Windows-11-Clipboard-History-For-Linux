// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Monitor, PhysicalPosition, PhysicalSize, State, WebviewWindow,
    WindowEvent,
};
use win11_clipboard_history_lib::autostart_manager;
use win11_clipboard_history_lib::clipboard_manager::{ClipboardItem, ClipboardManager};
use win11_clipboard_history_lib::config_manager::{resolve_window_position, ConfigManager};
use win11_clipboard_history_lib::emoji_manager::{EmojiManager, EmojiUsage};
#[cfg(target_os = "linux")]
use win11_clipboard_history_lib::focus_manager::x11_robust_activate;
use win11_clipboard_history_lib::focus_manager::{restore_focused_window, save_focused_window};
use win11_clipboard_history_lib::input_simulator::simulate_paste_keystroke;
use win11_clipboard_history_lib::permission_checker;
use win11_clipboard_history_lib::session::is_wayland;
use win11_clipboard_history_lib::shortcut_setup;
use win11_clipboard_history_lib::user_settings::{UserSettings, UserSettingsManager};

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
    is_mouse_inside: Arc<AtomicBool>,
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

#[tauri::command]
fn set_mouse_state(state: State<AppState>, inside: bool) {
    state.is_mouse_inside.store(inside, Ordering::Relaxed);
}

// --- User Settings Commands ---

#[tauri::command]
fn get_user_settings() -> Result<UserSettings, String> {
    let manager = UserSettingsManager::new();
    Ok(manager.load())
}

#[tauri::command]
fn set_user_settings(app: AppHandle, new_settings: UserSettings) -> Result<(), String> {
    let manager = UserSettingsManager::new();
    manager.save(&new_settings)?;

    // Emit event to notify all windows that settings have changed
    app.emit("app-settings-changed", &new_settings)
        .map_err(|e| format!("Failed to emit settings changed event: {}", e))?;

    Ok(())
}

#[tauri::command]
fn is_settings_window_visible(app: AppHandle) -> bool {
    app.get_webview_window("settings")
        .map(|w| w.is_visible().unwrap_or(false))
        .unwrap_or(false)
}

#[tauri::command]
async fn paste_item(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    // 1. Get Item (Scope lock tightly)
    let item = {
        let manager = state.clipboard_manager.lock();
        manager.get_item(&id).cloned()
    };

    match item {
        Some(item) => {
            // 2. Prepare Environment (Hide Window -> Restore Focus)
            WindowController::hide(&app);
            PasteHelper::prepare_target_window().await?;

            // 3. Perform Paste
            let mut manager = state.clipboard_manager.lock();
            manager.paste_item(&item).map_err(|e| e.to_string())?;
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
) -> Result<(), String> {
    // 0. Record usage if applicable
    if let Some(t) = item_type.as_deref() {
        if t == "emoji" {
            state.emoji_manager.lock().record_usage(&text);
        }
    }

    // 1. Prepare Environment
    WindowController::hide(&app);
    PasteHelper::prepare_target_window().await?;

    // 2. Set Clipboard & Mark
    {
        let mut manager = state.clipboard_manager.lock();
        manager.mark_text_as_pasted(&text);

        use arboard::Clipboard;
        Clipboard::new()
            .map_err(|e| e.to_string())?
            .set_text(&text)
            .map_err(|e| e.to_string())?;
    }

    // 3. Simulate Paste
    simulate_paste_keystroke().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn paste_gif_from_url(
    app: AppHandle,
    state: State<'_, AppState>,
    url: String,
) -> Result<(), String> {
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
    PasteHelper::prepare_target_window().await?;

    // The clipboard is already set by paste_gif_to_clipboard_with_uri, we just need to paste
    simulate_paste_keystroke().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn finish_paste(app: AppHandle) -> Result<(), String> {
    WindowController::hide(&app);
    PasteHelper::prepare_target_window().await?;
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

// --- Helper for Paste Logic ---

struct PasteHelper;

impl PasteHelper {
    /// Restores focus to the previous window and waits for it to settle.
    /// This ensures keystrokes are sent to the correct application.
    async fn prepare_target_window() -> Result<(), String> {
        if let Err(e) = restore_focused_window() {
            eprintln!("[PasteHelper] Warning: Focus restoration failed: {}", e);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }
}

// --- Window Controller (Visibility & Positioning) ---

struct WindowController;

impl WindowController {
    pub fn toggle(app: &AppHandle) {
        // User-initiated toggle - mark that we're now allowing shows
        // This stops the background enforcer from hiding the window
        if STARTED_IN_BACKGROUND.load(Ordering::SeqCst) {
            INITIAL_SHOW_ALLOWED.store(true, Ordering::SeqCst);
        }

        if let Some(window) = app.get_webview_window("main") {
            if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
            } else {
                save_focused_window();
                Self::position_and_show(&window, app);
            }
        }
    }

    pub fn hide(app: &AppHandle) {
        if let Some(window) = app.get_webview_window("main") {
            // FLUSH CONFIG TO DISK ON HIDE
            if let Some(state) = app.try_state::<AppState>() {
                if is_wayland() {
                    state.config_manager.lock().sync_to_disk();
                }
            }
            let _ = window.hide();
        }
    }

    fn position_and_show(window: &WebviewWindow, app: &AppHandle) {
        let state = app.state::<AppState>();

        if is_wayland() {
            Self::position_for_wayland(window, &state);
        } else {
            Self::position_for_non_wayland(window);
        }

        #[cfg(target_os = "linux")]
        let is_wayland_session = is_wayland();

        #[cfg(not(target_os = "linux"))]
        let is_wayland_session = false;

        if is_wayland_session {
            // Wayland needs to be born "On Top" to be visible
            let _ = window.show();
            let _ = window.set_always_on_top(true);
            let _ = window.set_focus();
        } else {
            // X11 born as normal window.
            // We do NOT activate always_on_top to avoid focus blocking and glitch.
            let _ = window.show();
        }

        let window_clone = window.clone();
        let app_clone = app.clone();

        std::thread::spawn(move || {
            // For Wayland, we still need a small delay for the compositor
            // For X11, we use polling-based wait instead of fixed sleep
            #[cfg(target_os = "linux")]
            if is_wayland_session {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let _ = window_clone.set_always_on_top(false);
                let _ = window_clone.set_focus();
            } else {
                // Use EWMH _NET_ACTIVE_WINDOW protocol with polling instead of fixed sleep.
                // This waits for the window to actually appear in X11's client list
                // before attempting activation, solving the race condition.
                if let Err(e) = x11_robust_activate("Clipboard History") {
                    eprintln!("[WindowController] X11 activation failed: {}", e);
                    // Fallback: try xdotool as last resort
                    let _ = Self::x11_activate_window_xdotool();
                }
            }

            let _ = app_clone.emit("window-shown", ());
        });
    }

    /// Activate window on X11 using xdotool (fallback method)
    #[cfg(target_os = "linux")]
    fn x11_activate_window_xdotool() -> Result<(), String> {
        use std::process::Command;

        let output = Command::new("xdotool")
            .args(["search", "--name", "Clipboard History"])
            .output()
            .map_err(|e| format!("xdotool search failed: {}", e))?;

        let window_ids = String::from_utf8_lossy(&output.stdout);
        if let Some(window_id) = window_ids.lines().next() {
            Command::new("xdotool")
                .args(["windowactivate", "--sync", window_id])
                .output()
                .map_err(|e| format!("windowactivate failed: {}", e))?;
            Ok(())
        } else {
            Err("Window not found".to_string())
        }
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
        {
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

fn handle_window_moved_for_wayland(
    window: &WebviewWindow,
    state: &State<AppState>,
    pos: &PhysicalPosition<i32>,
) {
    if !is_wayland() || !window.is_visible().unwrap_or(false) {
        return;
    }

    let monitor_name = window
        .current_monitor()
        .ok()
        .flatten()
        .and_then(|m| m.name().map(|n| n.to_string()));

    let mut config = state.config_manager.lock();
    // UPDATE MEMORY ONLY (No Disk I/O here)
    config.update_state(monitor_name, pos.x, pos.y);
}

// --- Background Listeners ---

fn start_clipboard_watcher(app: AppHandle, clipboard_manager: Arc<Mutex<ClipboardManager>>) {
    std::thread::spawn(move || {
        let mut last_text_hash: Option<u64> = None;
        let mut last_image_hash: Option<u64> = None;

        loop {
            std::thread::sleep(Duration::from_millis(500));
            let mut manager = clipboard_manager.lock();

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
        println!();
        println!("SHORTCUTS:");
        println!("    Super+V          Open clipboard history");
        println!("    Ctrl+Alt+V       Alternative shortcut");
        return;
    }

    // Check if --background flag is present (start minimized to tray)
    let start_in_background = args.iter().any(|arg| arg == "--background");
    if start_in_background {
        println!("[Startup] Starting in background mode (system tray only)");
        STARTED_IN_BACKGROUND.store(true, Ordering::SeqCst);
    }

    // Check if --settings flag is present (for first instance startup)
    let open_settings_on_start = args.iter().any(|arg| arg == "--settings");

    // Clone for use in setup closure
    let start_in_background_clone = start_in_background;

    win11_clipboard_history_lib::session::init();

    let is_mouse_inside = Arc::new(AtomicBool::new(false));
    let base_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("win11-clipboard-history");

    // Ensure base directory exists
    if let Err(e) = std::fs::create_dir_all(&base_dir) {
        eprintln!("Failed to create base directory: {}", e);
    }

    let history_path = base_dir.join("history.json");
    let clipboard_manager = Arc::new(Mutex::new(ClipboardManager::new(history_path)));

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
            } else {
                println!("[SingleInstance] Secondary instance detected, toggling window...");
                WindowController::toggle(app);
            }
        }))
        .manage(AppState {
            clipboard_manager: clipboard_manager.clone(),
            emoji_manager: emoji_manager.clone(),
            config_manager: config_manager.clone(),
            is_mouse_inside: is_mouse_inside.clone(),
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // FIRST THING: If started in background mode, immediately hide the main window
            // This runs before anything else to prevent the window from appearing
            if start_in_background_clone {
                if let Some(main_window) = app.get_webview_window("main") {
                    let _ = main_window.hide();
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

            let icon = Image::from_bytes(include_bytes!("../icons/icon.png")).unwrap();

            // Get temp directory for tray icon (avoids permission issues with XDG_RUNTIME_DIR)
            let temp_dir = std::env::temp_dir().join("win11-clipboard-history");
            std::fs::create_dir_all(&temp_dir).ok();

            let _tray = TrayIconBuilder::new()
                .icon(icon)
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

            // Verify that settings window was created from config
            if app.get_webview_window("settings").is_none() {
                eprintln!("[Setup] FATAL: Settings window missing from config");
            } else {
                println!("[Setup] Settings window created successfully from config");
            }

            // Window Event Handlers (Focus & Move)
            let main_window = app.get_webview_window("main").unwrap();
            let w_clone = main_window.clone();
            let app_handle_for_event = app_handle.clone();

            main_window.on_window_event(move |event| match event {
                // Block any window show attempts when started in background mode
                // This catches cases where GTK/Tauri automatically shows the window
                WindowEvent::Focused(true) => {
                    // Load both flags atomically with SeqCst to avoid race conditions
                    let started_in_background = STARTED_IN_BACKGROUND.load(Ordering::SeqCst);
                    let initial_show_allowed = INITIAL_SHOW_ALLOWED.load(Ordering::SeqCst);

                    // If started in background and initial show hasn't been allowed yet,
                    // immediately hide the window
                    if started_in_background && !initial_show_allowed {
                        println!("[WindowController] Background mode: intercepted focus, hiding window");
                        let _ = w_clone.hide();
                    }
                }
                WindowEvent::Focused(false) => {
                    let state = w_clone.state::<AppState>();
                    if state.is_mouse_inside.load(Ordering::Relaxed) {
                        return;
                    }

                    // Don't hide if settings window is visible (for live preview)
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

            start_clipboard_watcher(app_handle.clone(), clipboard_manager);

            // Register global shortcut (Super+V) with the desktop environment
            // This runs in a background thread to avoid blocking startup
            #[cfg(target_os = "linux")]
            std::thread::spawn(|| {
                // Give the desktop environment a moment to settle
                std::thread::sleep(std::time::Duration::from_secs(2));
                win11_clipboard_history_lib::linux_shortcut_manager::register_global_shortcut();
            });

            // If --settings flag was passed on first startup, open the settings window
            if open_settings_on_start {
                SettingsController::show(&app_handle);
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
                                    let _ = window_clone.hide();
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
            set_mouse_state,
            get_user_settings,
            set_user_settings,
            is_settings_window_visible,
            copy_text_to_clipboard,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
