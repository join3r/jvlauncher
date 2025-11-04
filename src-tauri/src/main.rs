// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod database;
mod icon_extractor;
mod icon_fetcher;
mod launcher;
mod shortcut_manager;
mod terminal;
mod updater;

#[cfg(target_os = "macos")]
mod macos_delegate;

use tauri::Listener;

use tauri::{Manager, menu::{MenuBuilder, MenuItemBuilder}, tray::{TrayIconBuilder, TrayIconEvent}};
use tauri_plugin_autostart::ManagerExt;

fn main() {
    println!("Starting...");
    
    // Initialize logger
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Set activation policy to Accessory on macOS
            // This makes the app a menu bar app that doesn't appear in the Dock
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Install custom macOS delegate to prevent Cmd+Q from quitting the app
            #[cfg(target_os = "macos")]
            macos_delegate::prevent_app_termination();

            // Get app data directory
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");
            
            // Ensure app data directory exists
            std::fs::create_dir_all(&app_data_dir)
                .expect("Failed to create app data directory");

            // Initialize database
            let db_path = app_data_dir.join("launcher.db");
            let pool = database::init_database(db_path)
                .expect("Failed to initialize database");

            // Store database pool in app state
            app.manage(pool.clone());

            // Initialize terminal state
            app.manage(terminal::TerminalState {
                windows: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            });

            // Get settings and register global shortcut
            if let Ok(settings) = database::get_settings(&pool) {
                let app_handle = app.handle().clone();
                if let Err(e) = shortcut_manager::register_global_shortcut(
                    &app_handle,
                    &settings.global_shortcut,
                ) {
                    eprintln!("Failed to register global shortcut: {}", e);
                }

                // Setup autostart if enabled
                if settings.start_at_login {
                    let autostart_manager = app.autolaunch();
                    let _ = autostart_manager.enable();
                }
            }

            // Register global shortcuts for all apps
            if let Ok(apps) = database::get_all_apps(&pool) {
                let app_handle = app.handle().clone();
                for app_item in apps {
                    if let Some(global_shortcut) = &app_item.global_shortcut {
                        if !global_shortcut.is_empty() {
                            if let Err(e) = shortcut_manager::register_app_shortcut(
                                &app_handle,
                                app_item.id,
                                global_shortcut,
                            ) {
                                eprintln!("Failed to register global shortcut for app {}: {}", app_item.id, e);
                            }
                        }
                    }
                }
            }

            // Setup event listener for launching apps via global shortcuts
            let pool_clone = pool.clone();
            let app_handle_clone = app.handle().clone();
            app.listen("launch-app-by-shortcut", move |event| {
                if let Some(app_id) = event.payload().parse::<i64>().ok() {
                    if let Ok(apps) = database::get_all_apps(&pool_clone) {
                        if let Some(app_to_launch) = apps.iter().find(|a| a.id == app_id) {
                            let _ = launcher::launch_app(app_to_launch, &app_handle_clone, &pool_clone);
                        }
                    }
                }
            });

            // Get main window and setup close handler
            let window = app.get_webview_window("main")
                .expect("Failed to get main window");

            // Apply macOS-specific window effects (vibrancy/glassmorphism)
            #[cfg(target_os = "macos")]
            {
                use tauri::window::Effect;
                use tauri::window::EffectState;

                // Apply vibrancy effect for liquid glass appearance
                let _ = window.set_effects(tauri::utils::config::WindowEffectsConfig {
                    effects: vec![Effect::HudWindow],
                    state: Some(EffectState::FollowsWindowActiveState),
                    radius: Some(12.0),
                    color: None,
                });
            }

            // Intercept close event to hide window instead of quitting
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        // Prevent default close behavior
                        api.prevent_close();
                        // Hide window instead
                        let _ = window_clone.hide();
                    }
                    tauri::WindowEvent::Focused(focused) => {
                        // Hide window when it loses focus
                        if !focused {
                            let _ = window_clone.hide();
                        }
                    }
                    _ => {}
                }
            });

            // Setup system tray
            let show_item = MenuItemBuilder::with_id("show", "Show Launcher").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            println!("Exiting");
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, .. } = event {
                        if button == tauri::tray::MouseButton::Left {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                    }
                })
                .build(app)?;

            // Note: Window will stay visible until user presses Escape or the global shortcut again
            // This prevents the window from disappearing when releasing the keyboard shortcut

            // Start background update check
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                updater::check_updates_on_startup(app_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_all_apps,
            commands::create_app,
            commands::update_app,
            commands::delete_app,
            commands::reorder_apps,
            commands::launch,
            commands::extract_icon_from_binary,
            commands::save_icon_from_file,
            commands::paste_icon_from_clipboard,
            commands::save_icon_from_file_temp,
            commands::paste_icon_from_clipboard_temp,
            commands::finalize_temp_icon,
            commands::cleanup_temp_icon,
            commands::fetch_web_icon,
            commands::get_settings,
            commands::update_setting,
            commands::update_global_shortcut,
            commands::check_shortcut_conflict,
            commands::check_global_shortcut_conflict,
            commands::toggle_main_window,
            commands::hide_main_window,
            commands::quit_app,
            commands::open_settings_window,
            commands::open_add_app_window,
            commands::open_edit_app_window,
            commands::resize_main_window,
            commands::auto_resize_window,
            commands::send_terminal_input,
            commands::resize_terminal,
            updater::check_for_updates,
            updater::download_and_install_update,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, _event| {
            // With ActivationPolicy::Accessory, the app becomes a menu bar app
            // The system tray keeps the app running even when all windows are closed
            // No need to handle ExitRequested - the tray icon keeps the app alive
        });
}

