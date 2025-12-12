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
use win11_clipboard_history_lib::emoji_manager::{EmojiManager, EmojiUsage};
use win11_clipboard_history_lib::focus_manager::{restore_focused_window, save_focused_window};
use win11_clipboard_history_lib::hotkey_manager::{HotkeyAction, HotkeyManager};
use win11_clipboard_history_lib::input_simulator::simulate_paste_keystroke;

/// Application state shared across all handlers
pub struct AppState {
    clipboard_manager: Arc<Mutex<ClipboardManager>>,
    emoji_manager: Arc<Mutex<EmojiManager>>,
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
    eprintln!("[PasteItem] Starting paste for id: {}", id);

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
        eprintln!("[PasteItem] Found item");

        // Restore focus to the previously active window
        if let Err(e) = restore_focused_window() {
            eprintln!("[PasteItem] Failed to restore focus: {}", e);
        }

        // Wait for focus to be restored
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Write to clipboard and simulate paste
        eprintln!("[PasteItem] About to call paste_item on manager");
        let mut manager = state.clipboard_manager.lock();
        manager
            .paste_item(&item)
            .map_err(|e| format!("Failed to paste: {}", e))?;
        eprintln!("[PasteItem] Paste complete");
    } else {
        eprintln!("[PasteItem] Item not found!");
    }

    Ok(())
}

/// Get recent emojis from LRU cache
#[tauri::command]
fn get_recent_emojis(state: State<AppState>) -> Vec<EmojiUsage> {
    state.emoji_manager.lock().get_recent()
}

/// Helper to paste text via clipboard pipeline
/// Pipeline: Mark as pasted -> Write to clipboard -> Hide window -> Restore focus -> Simulate Ctrl+V
async fn paste_text_via_clipboard(
    app: &AppHandle,
    state: &State<'_, AppState>,
    text: &str,
) -> Result<(), String> {
    // Step 1: Mark text as "pasted by us" so clipboard watcher ignores it
    {
        let mut clipboard_manager = state.clipboard_manager.lock();
        clipboard_manager.mark_text_as_pasted(text);
    }

    // Step 2: Write to system clipboard (transport only)
    {
        use arboard::Clipboard;
        let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard error: {}", e))?;
        clipboard
            .set_text(text)
            .map_err(|e| format!("Failed to set clipboard: {}", e))?;
    }

    // Step 3: Hide window
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    // Step 4: Restore focus
    if let Err(e) = restore_focused_window() {
        eprintln!("Warning: Failed to restore focus: {}", e);
    }

    // Step 5: Wait for focus to be fully restored
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Step 6: Simulate Ctrl+V
    simulate_paste_keystroke()?;

    Ok(())
}

/// Paste an emoji character
/// Pipeline: Record usage -> Delegate to paste_text_via_clipboard
#[tauri::command]
async fn paste_emoji(
    app: AppHandle,
    state: State<'_, AppState>,
    char: String,
) -> Result<(), String> {
    eprintln!("[PasteEmoji] Starting paste for emoji: {}", char);

    // Record usage in LRU cache
    {
        let mut emoji_manager = state.emoji_manager.lock();
        emoji_manager.record_usage(&char);
    }

    // Delegate to shared pipeline
    paste_text_via_clipboard(&app, &state, &char).await?;

    eprintln!("[PasteEmoji] Paste complete");
    Ok(())
}

/// Paste a GIF from URL
#[tauri::command]
async fn paste_gif_from_url(
    _app: AppHandle,
    state: State<'_, AppState>,
    url: String,
) -> Result<(), String> {
    eprintln!("[PasteGif] Starting paste for URL: {}", url);

    // Step 1: Download and copy to clipboard (blocking operation, run in spawn_blocking)
    let url_clone = url.clone();
    let file_uri = tokio::task::spawn_blocking(move || {
        win11_clipboard_history_lib::gif_manager::paste_gif_to_clipboard_with_uri(&url_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Failed to paste GIF: {}", e))?;

    // Step 2: Mark the file URI as pasted so clipboard watcher ignores it
    if let Some(uri) = file_uri {
        eprintln!("[PasteGif] Marking URI as pasted: {}", uri);
        let mut clipboard_manager = state.clipboard_manager.lock();
        clipboard_manager.mark_text_as_pasted(&uri);
        // Also mark without trailing newline in case wl-copy strips it
        let uri_trimmed = uri.trim();
        if uri_trimmed != uri {
            clipboard_manager.mark_text_as_pasted(uri_trimmed);
        }
    }

    // Step 3: Small delay to ensure clipboard is fully ready
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    Ok(())
}

/// Finish the paste sequence: Hide window -> Restore focus -> Simulate Ctrl+V
#[tauri::command]
async fn finish_paste(app: AppHandle) -> Result<(), String> {
    // Step 2: Hide window
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    // Step 3: Restore focus
    if let Err(e) = restore_focused_window() {
        eprintln!("Warning: Failed to restore focus: {}", e);
    }

    // Step 4: Wait for focus to be fully restored
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Step 5: Simulate Ctrl+V
    simulate_paste_keystroke()?;

    eprintln!("[PasteGif] Paste sequence complete");
    Ok(())
}

/// Show the clipboard window at cursor position
fn show_window_at_cursor(window: &WebviewWindow) {
    use tauri::{PhysicalPosition, PhysicalSize};

    // Try multiple methods to get cursor position
    let cursor_pos = get_cursor_position_multi(window);

    match cursor_pos {
        Some((x, y)) => {
            // We got cursor position - position window near cursor
            if let Ok(Some(monitor)) = window.current_monitor() {
                let monitor_size = monitor.size();
                let window_size = window.outer_size().unwrap_or(PhysicalSize::new(360, 480));

                // Calculate position, keeping window within screen bounds
                let mut pos_x = x;
                let mut pos_y = y;

                // Adjust if window would go off-screen
                if pos_x + window_size.width as i32 > monitor_size.width as i32 {
                    pos_x = monitor_size.width as i32 - window_size.width as i32 - 10;
                }
                if pos_y + window_size.height as i32 > monitor_size.height as i32 {
                    pos_y = monitor_size.height as i32 - window_size.height as i32 - 10;
                }

                // Ensure not negative
                pos_x = pos_x.max(10);
                pos_y = pos_y.max(10);

                eprintln!("[Window] Positioning at ({}, {})", pos_x, pos_y);
                if let Err(e) = window.set_position(PhysicalPosition::new(pos_x, pos_y)) {
                    eprintln!("[Window] Failed to set position: {:?}", e);
                    let _ = window.center();
                }
            }
        }
        None => {
            // No cursor position available, center the window
            eprintln!("[Window] Cursor position not available, centering");
            if let Err(e) = window.center() {
                eprintln!("[Window] Failed to center: {:?}", e);
            }
        }
    }

    let _ = window.show();
    let _ = window.set_focus();
}

/// Try multiple methods to get cursor position
fn get_cursor_position_multi(window: &WebviewWindow) -> Option<(i32, i32)> {
    // Method 1: Tauri's cursor_position (works in X11 mode)
    if let Ok(pos) = window.cursor_position() {
        eprintln!("[Cursor] Got position via Tauri: ({}, {})", pos.x, pos.y);
        return Some((pos.x as i32, pos.y as i32));
    }

    // Method 2: Use xdotool (X11)
    if let Some(pos) = get_cursor_via_xdotool() {
        return Some(pos);
    }

    // Method 3: Query X11 directly via x11rb
    #[cfg(target_os = "linux")]
    if let Some(pos) = get_cursor_via_x11() {
        return Some(pos);
    }

    None
}

/// Get cursor position using xdotool
fn get_cursor_via_xdotool() -> Option<(i32, i32)> {
    let output = std::process::Command::new("xdotool")
        .args(["getmouselocation", "--shell"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut x: Option<i32> = None;
    let mut y: Option<i32> = None;

    for line in stdout.lines() {
        if let Some(val) = line.strip_prefix("X=") {
            x = val.parse().ok();
        } else if let Some(val) = line.strip_prefix("Y=") {
            y = val.parse().ok();
        }
    }

    if let (Some(x), Some(y)) = (x, y) {
        eprintln!("[Cursor] Got position via xdotool: ({}, {})", x, y);
        return Some((x, y));
    }

    None
}

/// Get cursor position via X11 directly
#[cfg(target_os = "linux")]
fn get_cursor_via_x11() -> Option<(i32, i32)> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::ConnectionExt;

    let (conn, screen_num) = x11rb::connect(None).ok()?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let reply = conn.query_pointer(root).ok()?.reply().ok()?;
    let x = reply.root_x as i32;
    let y = reply.root_y as i32;

    eprintln!("[Cursor] Got position via x11rb: ({}, {})", x, y);
    Some((x, y))
}

/// Toggle window visibility
fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            // Save the currently focused window before showing our window
            save_focused_window();
            show_window_at_cursor(&window);
        }
    }
}

/// Start clipboard monitoring in background thread
fn start_clipboard_watcher(app: AppHandle, clipboard_manager: Arc<Mutex<ClipboardManager>>) {
    std::thread::spawn(move || {
        let mut last_text_hash: Option<u64> = None;
        let mut last_image_hash: Option<u64> = None;

        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));

            let mut manager = clipboard_manager.lock();

            // Check for text changes using hash to detect duplicates reliably
            if let Ok(text) = manager.get_current_text() {
                if !text.is_empty() {
                    // Hash the text for comparison
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    text.hash(&mut hasher);
                    let text_hash = hasher.finish();

                    if Some(text_hash) != last_text_hash {
                        last_text_hash = Some(text_hash);
                        // Clear image hash when text is copied
                        last_image_hash = None;

                        // add_text handles duplicate detection internally
                        if let Some(item) = manager.add_text(text) {
                            // Emit event to frontend
                            let _ = app.emit("clipboard-changed", &item);
                        }
                    }
                }
            }

            // Check for image changes
            if let Ok(Some((image_data, hash))) = manager.get_current_image() {
                if Some(hash) != last_image_hash {
                    last_image_hash = Some(hash);
                    // Clear text hash when image is copied
                    last_text_hash = None;
                    if let Some(item) = manager.add_image(image_data, hash) {
                        let _ = app.emit("clipboard-changed", &item);
                    }
                }
            }

            // Release the lock before sleeping
            drop(manager);
        }
    });
}

/// Start global hotkey listener
fn start_hotkey_listener(app: AppHandle) -> HotkeyManager {
    let app_clone = app.clone();

    HotkeyManager::new(move |action| match action {
        HotkeyAction::Toggle => toggle_window(&app_clone),
        HotkeyAction::Close => {
            if let Some(window) = app_clone.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                }
            }
        }
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

    // Initialize session detection early (caches Wayland/X11 detection)
    win11_clipboard_history_lib::session::init();

    let clipboard_manager = Arc::new(Mutex::new(ClipboardManager::new()));

    // Initialize emoji manager with app data directory
    let emoji_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("win11-clipboard-history");
    let emoji_manager = Arc::new(Mutex::new(EmojiManager::new(emoji_data_dir)));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            clipboard_manager: clipboard_manager.clone(),
            emoji_manager: emoji_manager.clone(),
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
            get_recent_emojis,
            paste_emoji,
            paste_gif_from_url,
            finish_paste,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
