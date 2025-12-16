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
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use win11_clipboard_history_lib::clipboard_manager::{ClipboardItem, ClipboardManager};
use win11_clipboard_history_lib::config_manager::{resolve_window_position, ConfigManager};
use win11_clipboard_history_lib::emoji_manager::{EmojiManager, EmojiUsage};
use win11_clipboard_history_lib::focus_manager::{restore_focused_window, save_focused_window};
use win11_clipboard_history_lib::input_simulator::simulate_paste_keystroke;
use win11_clipboard_history_lib::session::is_wayland;

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
    state.clipboard_manager.lock().toggle_pin(&id)
}

#[tauri::command]
fn get_recent_emojis(state: State<AppState>) -> Vec<EmojiUsage> {
    state.emoji_manager.lock().get_recent()
}

#[tauri::command]
fn set_mouse_state(state: State<AppState>, inside: bool) {
    state.is_mouse_inside.store(inside, Ordering::Relaxed);
}

#[tauri::command]
async fn paste_item(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    // 1. Get Item (Scope lock tightly)
    let item = {
        let manager = state.clipboard_manager.lock();
        manager.get_item(&id).cloned()
    };

    if let Some(item) = item {
        // 2. Prepare Environment (Hide Window -> Restore Focus)
        WindowController::hide(&app);
        PasteHelper::prepare_target_window().await?;

        // 3. Perform Paste
        let mut manager = state.clipboard_manager.lock();
        manager.paste_item(&item).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn paste_emoji(
    app: AppHandle,
    state: State<'_, AppState>,
    char: String,
) -> Result<(), String> {
    state.emoji_manager.lock().record_usage(&char);

    // 1. Prepare Environment
    WindowController::hide(&app);
    PasteHelper::prepare_target_window().await?;

    // 2. Set Clipboard & Mark
    {
        let mut manager = state.clipboard_manager.lock();
        manager.mark_text_as_pasted(&char);

        use arboard::Clipboard;
        Clipboard::new()
            .map_err(|e| e.to_string())?
            .set_text(&char)
            .map_err(|e| e.to_string())?;
    }

    // 3. Simulate Paste (Manual trigger required for emoji)
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

// --- Helper for Paste Logic ---

struct PasteHelper;

impl PasteHelper {
    /// Restores focus to the previous window and waits for it to settle.
    /// This ensures keystrokes are sent to the correct application.
    async fn prepare_target_window() -> Result<(), String> {
        if let Err(e) = restore_focused_window() {
            eprintln!("[PasteHelper] Warning: Focus restoration failed: {}", e);
        }
        // Wait for OS window manager (especially Wayland/GNOME) to process focus change
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(())
    }
}

// --- Window Controller (Visibility & Positioning) ---

struct WindowController;

impl WindowController {
    pub fn toggle(app: &AppHandle) {
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

        let _ = window.show();
        let _ = window.set_focus();
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
                let _ = window.center();
                let _ = window.show();
                let _ = window.set_focus();
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
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    std::hash::Hash::hash(&text, &mut hasher);
                    let text_hash = std::hash::Hasher::finish(&hasher);

                    if Some(text_hash) != last_text_hash {
                        last_text_hash = Some(text_hash);
                        last_image_hash = None;
                        if let Some(item) = manager.add_text(text) {
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

/// Register global shortcuts using tauri-plugin-global-shortcut
fn register_global_shortcuts(app: &AppHandle) {
    // Super+V shortcut (Windows key + V)
    let super_v = Shortcut::new(Some(Modifiers::SUPER), Code::KeyV);

    // Ctrl+Alt+V as fallback (for systems where Super is used by DE)
    let ctrl_alt_v = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyV);

    let app_handle = app.clone();

    if let Err(e) = app
        .global_shortcut()
        .on_shortcut(super_v, move |_app, shortcut, event| {
            if event.state == ShortcutState::Pressed {
                eprintln!("[GlobalShortcut] Super+V triggered: {:?}", shortcut);
                WindowController::toggle(&app_handle);
            }
        })
    {
        eprintln!("[GlobalShortcut] Failed to register Super+V: {}", e);
    }

    let app_handle2 = app.clone();

    if let Err(e) = app
        .global_shortcut()
        .on_shortcut(ctrl_alt_v, move |_app, shortcut, event| {
            if event.state == ShortcutState::Pressed {
                eprintln!("[GlobalShortcut] Ctrl+Alt+V triggered: {:?}", shortcut);
                WindowController::toggle(&app_handle2);
            }
        })
    {
        eprintln!("[GlobalShortcut] Failed to register Ctrl+Alt+V: {}", e);
    }

    eprintln!("[GlobalShortcut] Attempted to register shortcuts: Super+V, Ctrl+Alt+V");
}

// --- Main ---

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("win11-clipboard-history {}", VERSION);
        return;
    }

    win11_clipboard_history_lib::session::init();

    let is_mouse_inside = Arc::new(AtomicBool::new(false));
    let clipboard_manager = Arc::new(Mutex::new(ClipboardManager::new()));

    let base_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("win11-clipboard-history");

    let emoji_manager = Arc::new(Mutex::new(EmojiManager::new(base_dir.clone())));

    let config_manager = Arc::new(Mutex::new(ConfigManager::new(base_dir)));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            clipboard_manager: clipboard_manager.clone(),
            emoji_manager: emoji_manager.clone(),
            config_manager: config_manager.clone(),
            is_mouse_inside: is_mouse_inside.clone(),
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();

            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            // Load the tray icon - use 32x32 PNG for best compatibility with all DEs
            let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .or_else(|_| Image::from_bytes(include_bytes!("../icons/icon.png")))
                .expect("Failed to load tray icon");

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .tooltip("Clipboard History - Super+V")
                .title("Clipboard History")
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => WindowController::toggle(app),
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

            // Window Event Handlers (Focus & Move)
            let main_window = app.get_webview_window("main").unwrap();
            let w_clone = main_window.clone();

            main_window.on_window_event(move |event| match event {
                WindowEvent::Focused(false) => {
                    let state = w_clone.state::<AppState>();
                    if state.is_mouse_inside.load(Ordering::Relaxed) {
                        return;
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

            // Register global shortcuts using tauri-plugin-global-shortcut
            register_global_shortcuts(&app_handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_history,
            clear_history,
            delete_item,
            toggle_pin,
            paste_item,
            get_recent_emojis,
            paste_emoji,
            paste_gif_from_url,
            finish_paste,
            set_mouse_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
