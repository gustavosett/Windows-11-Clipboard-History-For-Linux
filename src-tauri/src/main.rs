// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! # Windows 11 Clipboard History — Application Entry Point
//!
//! This file is intentionally thin. All domain logic lives in the library
//! crates under `src-tauri/src/`. The `main()` function:
//!
//! 1. Initialises tracing and the rendering environment.
//! 2. Builds the Tauri application with plugins and shared state.
//! 3. Registers all Tauri commands (from `commands.rs`).
//! 4. Starts the clipboard watcher and theme listener.

use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

use windows_11_style_clipboard_history_manager::autostart_manager;
use windows_11_style_clipboard_history_manager::clipboard_manager::ClipboardManager;
use windows_11_style_clipboard_history_manager::commands;
use windows_11_style_clipboard_history_manager::config_manager::ConfigManager;
use windows_11_style_clipboard_history_manager::emoji_manager::EmojiManager;
use windows_11_style_clipboard_history_manager::permission_checker;
use windows_11_style_clipboard_history_manager::rendering_env;
use windows_11_style_clipboard_history_manager::session;
use windows_11_style_clipboard_history_manager::shortcut_setup;

use windows_11_style_clipboard_history_manager::user_settings::UserSettingsManager;
use windows_11_style_clipboard_history_manager::window_controller::{
    SettingsController, WindowController, STARTED_IN_BACKGROUND,
};
use windows_11_style_clipboard_history_manager::AppState;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    windows_11_style_clipboard_history_manager::init_tracing();

    let args: Vec<String> = std::env::args().collect();

    // Handle --version / -v
    if args.iter().any(|arg| arg == "--version" || arg == "-v") {
        println!("windows-11-style-clipboard-history-manager {VERSION}");
        return;
    }

    // Handle --help / -h
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }

    // MUST run before Tauri / WebKit init
    rendering_env::init();

    let start_in_background = args.iter().any(|arg| arg == "--background");
    if start_in_background {
        tracing::info!("[Startup] Starting in background mode (system tray only)");
        STARTED_IN_BACKGROUND.store(true, Ordering::SeqCst);
    }

    let open_settings_on_start = args.iter().any(|arg| arg == "--settings");
    let open_emoji_on_start = args.iter().any(|arg| arg == "--emoji");

    windows_11_style_clipboard_history_manager::session::init();

    let is_mouse_inside = Arc::new(AtomicBool::new(false));
    let base_dir = windows_11_style_clipboard_history_manager::clipboard_manager::data_dir();

    if let Err(e) = std::fs::create_dir_all(&base_dir) {
        tracing::error!("Failed to create base directory: {e}");
    }

    let user_settings = UserSettingsManager::new().load();

    // Construct the manager with the user's preferred encryption-key
    // backend (file | Secret Service). See ADR-0006.
    // ساخت مدیر با بک‌اند کلید ترجیحی کاربر (فایل | Secret Service). ADR-0006.
    let key_backend = windows_11_style_clipboard_history_manager::history_crypto::KeyBackend::from_setting(
        &user_settings.history_key_backend,
    );
    tracing::info!(
        "[Startup] History key backend: requested '{}'",
        key_backend.as_str()
    );
    let clipboard_manager = Arc::new(Mutex::new(ClipboardManager::new_with_key_backend(
        base_dir.clone(),
        user_settings.max_history_size,
        key_backend,
    )));
    {
        let mut manager = clipboard_manager.lock();
        manager.set_privacy_policy(user_settings.privacy_policy());
        manager.set_auto_delete_interval_minutes(user_settings.auto_delete_interval_in_minutes());
    }
    windows_11_style_clipboard_history_manager::linux_shortcut_manager::set_allow_wm_config_rewrite(
        user_settings.allow_wm_config_rewrite,
    );

    let emoji_manager = Arc::new(Mutex::new(EmojiManager::new(base_dir.clone())));
    let config_manager = Arc::new(Mutex::new(ConfigManager::new(base_dir)));

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if argv.iter().any(|arg| arg == "--settings") {
                tracing::info!("[SingleInstance] Opening settings...");
                SettingsController::show(app);
            } else if argv.iter().any(|arg| arg == "--emoji") {
                tracing::info!("[SingleInstance] Opening emoji picker...");
                WindowController::toggle_with_tab(app, Some("emoji"));
            } else {
                tracing::info!("[SingleInstance] Toggling window...");
                WindowController::toggle(app);
            }
        }))
        .manage(AppState {
            clipboard_manager: clipboard_manager.clone(),
            emoji_manager: emoji_manager.clone(),
            config_manager: config_manager.clone(),
            is_mouse_inside: is_mouse_inside.clone(),
            paste_gate: tokio::sync::Mutex::new(()),
            paste_ticket: Mutex::new(None),
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                if window.label() == "setup" {
                    if windows_11_style_clipboard_history_manager::permission_checker::first_run_pending() {
                        tracing::info!("[Setup] Setup window closed without completion. Exiting.");
                        window.app_handle().exit(0);
                    }
                }
            }
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();

            windows_11_style_clipboard_history_manager::input_simulator::init();
            windows_11_style_clipboard_history_manager::paste_sync::init();

            // Background mode: immediately hide the main window
            if start_in_background {
                if let Some(main_window) = app.get_webview_window("main") {
                    let _ = main_window.hide();
                    tracing::debug!("[Setup] Immediately hiding main window for background mode");
                }
            }

            // Auto-migrate old autostart entries
            match autostart_manager::migrate_native() {
                Ok(true) => tracing::info!("[Setup] Migrated autostart entry to use wrapper script"),
                Ok(false) => {}
                Err(e) => tracing::warn!("[Setup] Failed to migrate autostart: {e}"),
            }

            // Build system tray
            build_tray(app, &app_handle)?;

            // Verify settings window
            if app.get_webview_window("settings").is_none() {
                tracing::error!("[Setup] FATAL: Settings window missing from config");
            } else {
                tracing::info!("[Setup] Settings window created successfully from config");
            }

            // Register window event handlers
            if let Some(main_window) = app.get_webview_window("main") {
                windows_11_style_clipboard_history_manager::window_controller::register_window_events(
                    &main_window,
                    &app_handle,
                );

                // Start clipboard watcher
                windows_11_style_clipboard_history_manager::clipboard_watcher::start(
                    app_handle.clone(),
                    clipboard_manager.clone(),
                );

                // Start theme change listener
                let app_handle_for_theme = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) =
                        windows_11_style_clipboard_history_manager::theme_manager::start_theme_listener(
                            app_handle_for_theme,
                        )
                        .await
                    {
                        tracing::error!("[ThemeManager] Failed to start theme listener: {e}");
                    }
                });

                // Handle --settings flag
                if open_settings_on_start {
                    SettingsController::show(&app_handle);
                }

                // Handle --emoji flag
                if open_emoji_on_start {
                    let app_handle_for_emoji = app_handle.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        let _ = app_handle_for_emoji.emit("switch-tab", "emoji");
                    });
                }

                // Background mode enforcer
                if start_in_background {
                    windows_11_style_clipboard_history_manager::window_controller::spawn_background_enforcer(
                        &main_window,
                    );
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // History
            commands::get_history_page,
            commands::get_item,
            commands::clear_history,
            commands::delete_item,
            commands::toggle_pin,
            // Emoji
            commands::get_recent_emojis,
            // Mouse
            commands::set_mouse_state,
            // Settings
            commands::get_user_settings,
            commands::set_user_settings,
            commands::is_settings_window_visible,
            commands::get_default_settings,
            commands::set_app_language,
            // Theme
            commands::get_system_theme,
            commands::refresh_system_theme,
            commands::is_theme_listener_active,
            // Paste
            commands::paste_item,
            commands::paste_text,
            commands::paste_gif_from_url,
            commands::finish_paste,
            commands::copy_text_to_clipboard,
            commands::open_safe_url,
            commands::search_tenor,
            // Encryption key backend
            commands::get_history_key_backend_status,
            commands::migrate_history_key_to_secret_service,
            commands::migrate_history_key_to_file,
            // Setup
            commands::finish_setup,
            // Permissions
            permission_checker::check_permissions,
            permission_checker::fix_permissions_now,
            permission_checker::is_first_run,
            permission_checker::reset_first_run,
            // Shortcuts
            shortcut_setup::get_desktop_environment,
            shortcut_setup::register_de_shortcut,
            shortcut_setup::check_shortcut_tools,
            shortcut_setup::detect_conflicts,
            shortcut_setup::resolve_conflicts,
            // Autostart
            autostart_manager::autostart_enable,
            autostart_manager::autostart_disable,
            autostart_manager::autostart_is_enabled,
            autostart_manager::autostart_migrate,
            // Rendering
            rendering_env::get_rendering_environment,
            session::get_session_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Build the system tray icon and menu
fn build_tray(app: &tauri::App, app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let settings_manager = UserSettingsManager::new();
    let settings = settings_manager.load();
    // The native tray belongs to the always-English main surface. Language
    // selection is intentionally scoped to Settings and first-run Setup.
    // سینی native متعلق به سطح اصلی همیشه‌انگلیسی است؛ انتخاب زبان عمداً
    // فقط به تنظیمات و راه‌اندازی نخست محدود شده است.
    let show = MenuItem::with_id(app, "show", "Show Clipboard", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &settings_item, &quit])?;

    let temp_dir = std::env::temp_dir().join("windows-11-style-clipboard-history-manager");
    std::fs::create_dir_all(&temp_dir).ok();

    windows_11_style_clipboard_history_manager::theme_manager::update_dynamic_tray_flag(
        settings.enable_dynamic_tray_icon,
    );

    let (icon, use_template_icon) =
        windows_11_style_clipboard_history_manager::theme_manager::initial_tray_icon(&settings);

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

    // Update icon asynchronously if dynamic is enabled
    if settings.enable_dynamic_tray_icon {
        let app_handle_bg = app_handle.clone();
        let settings_bg = settings.clone();
        tauri::async_runtime::spawn(async move {
            windows_11_style_clipboard_history_manager::theme_manager::refresh_tray_icon(
                &app_handle_bg,
                &settings_bg,
            )
            .await;
        });
    }

    Ok(())
}

/// Print help message
fn print_help() {
    println!("windows-11-style-clipboard-history-manager {VERSION}");
    println!();
    println!("USAGE:");
    println!("    windows-11-style-clipboard-history-manager [OPTIONS]");
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
}
