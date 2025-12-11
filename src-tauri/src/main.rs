// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WebviewWindow,
};
use win11_clipboard_history_lib::clipboard_manager::{ClipboardItem, ClipboardManager};
use win11_clipboard_history_lib::hotkey_manager::HotkeyManager;

/// Application state shared across all handlers
pub struct AppState {
    clipboard_manager: Arc<Mutex<ClipboardManager>>,
    hotkey_manager: Arc<Mutex<Option<HotkeyManager>>>,
}

/// Get clipboard history
#[tauri::command]
fn get_history(state: State<AppState>) -> Vec<ClipboardItem> {
    state.clipboard_manager.lock().get_history()
}

/// Clear all clipboard history
#[tauri::command]
fn clear_history(state: State<AppState>) {
    state.clipboard_manager.lock().clear();
}

/// Delete a specific item from history
#[tauri::command]
fn delete_item(state: State<AppState>, id: String) {
    state.clipboard_manager.lock().remove_item(&id);
}

/// Pin/unpin an item
#[tauri::command]
fn toggle_pin(state: State<AppState>, id: String) -> Option<ClipboardItem> {
    state.clipboard_manager.lock().toggle_pin(&id)
}

/// Paste an item from history
#[tauri::command]
async fn paste_item(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    // First hide the window
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    // Get the item and paste it
    let item = {
        let manager = state.clipboard_manager.lock();
        manager.get_item(&id).cloned()
    };

    if let Some(item) = item {
        // Small delay to ensure window is hidden and previous app has focus
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Write to clipboard and simulate paste
        let manager = state.clipboard_manager.lock();
        manager
            .paste_item(&item)
            .map_err(|e| format!("Failed to paste: {}", e))?;
    }

    Ok(())
}

/// Show the clipboard window at cursor position
fn show_window_at_cursor(window: &WebviewWindow) {
    use tauri::{PhysicalPosition, PhysicalSize};

    // Get cursor position - fallback to center if not available
    let cursor_pos = window
        .cursor_position()
        .unwrap_or_else(|_| PhysicalPosition::new(100.0, 100.0));

    // Get monitor info for bounds checking
    if let Ok(Some(monitor)) = window.current_monitor() {
        let monitor_size = monitor.size();
        let window_size = window.outer_size().unwrap_or(PhysicalSize::new(360, 480));

        // Calculate position, keeping window within screen bounds
        let mut x = cursor_pos.x as i32;
        let mut y = cursor_pos.y as i32;

        // Adjust if window would go off-screen
        if x + window_size.width as i32 > monitor_size.width as i32 {
            x = monitor_size.width as i32 - window_size.width as i32 - 10;
        }
        if y + window_size.height as i32 > monitor_size.height as i32 {
            y = monitor_size.height as i32 - window_size.height as i32 - 10;
        }

        let _ = window.set_position(PhysicalPosition::new(x, y));
    }

    let _ = window.show();
    let _ = window.set_focus();
}

/// Toggle window visibility
fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            show_window_at_cursor(&window);
        }
    }
}

/// Start clipboard monitoring in background thread
fn start_clipboard_watcher(app: AppHandle, clipboard_manager: Arc<Mutex<ClipboardManager>>) {
    std::thread::spawn(move || {
        let mut last_text: Option<String> = None;
        let mut last_image_hash: Option<u64> = None;

        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));

            let mut manager = clipboard_manager.lock();

            // Check for text changes
            if let Ok(text) = manager.get_current_text() {
                if Some(&text) != last_text.as_ref() && !text.is_empty() {
                    last_text = Some(text.clone());
                    if let Some(item) = manager.add_text(text) {
                        // Emit event to frontend
                        let _ = app.emit("clipboard-changed", &item);
                    }
                }
            }

            // Check for image changes
            if let Ok(Some((image_data, hash))) = manager.get_current_image() {
                if Some(hash) != last_image_hash {
                    last_image_hash = Some(hash);
                    if let Some(item) = manager.add_image(image_data) {
                        let _ = app.emit("clipboard-changed", &item);
                    }
                }
            }
        }
    });
}

/// Start global hotkey listener
fn start_hotkey_listener(app: AppHandle) -> HotkeyManager {
    let app_clone = app.clone();

    HotkeyManager::new(move || {
        toggle_window(&app_clone);
    })
}

/// Application version from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    // Handle command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" | "-V" => {
                println!("win11-clipboard-history {}", VERSION);
                return;
            }
            "--help" | "-h" => {
                println!("Windows 11 Clipboard History for Linux v{}", VERSION);
                println!();
                println!("USAGE:");
                println!("    win11-clipboard-history [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("    -h, --help       Print help information");
                println!("    -v, --version    Print version information");
                println!();
                println!("HOTKEYS:");
                println!("    Super+V          Open clipboard history");
                println!("    Ctrl+Alt+V       Alternative hotkey");
                println!("    Esc              Close window");
                return;
            }
            _ => {
                eprintln!("Unknown option: {}", args[1]);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
    }

    let clipboard_manager = Arc::new(Mutex::new(ClipboardManager::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            clipboard_manager: clipboard_manager.clone(),
            hotkey_manager: Arc::new(Mutex::new(None)),
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Setup system tray
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_item = MenuItem::with_id(app, "show", "Show Clipboard", true, None::<&str>)?;
            let clear_item = MenuItem::with_id(app, "clear", "Clear History", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_item, &clear_item, &quit_item])?;

            // Load tray icon
            let icon =
                Image::from_bytes(include_bytes!("../icons/icon.png")).unwrap_or_else(|_| {
                    Image::from_bytes(include_bytes!("../icons/32x32.png")).unwrap()
                });

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("Clipboard History (Super+V)")
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        toggle_window(app);
                    }
                    "clear" => {
                        if let Some(state) = app.try_state::<AppState>() {
                            state.clipboard_manager.lock().clear();
                            let _ = app.emit("history-cleared", ());
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // Setup window blur handler (close on focus loss)
            let main_window = app.get_webview_window("main").unwrap();
            let window_clone = main_window.clone();

            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    let _ = window_clone.hide();
                }
            });

            // Start clipboard watcher
            start_clipboard_watcher(app_handle.clone(), clipboard_manager.clone());

            // Start global hotkey listener
            let hotkey_manager = start_hotkey_listener(app_handle.clone());

            // Store hotkey manager in state
            if let Some(state) = app_handle.try_state::<AppState>() {
                *state.hotkey_manager.lock() = Some(hotkey_manager);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_history,
            clear_history,
            delete_item,
            toggle_pin,
            paste_item,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
