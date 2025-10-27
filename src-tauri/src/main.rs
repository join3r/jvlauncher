// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod database;
mod icon_extractor;
mod launcher;
mod shortcut_manager;
mod terminal;

use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

fn main() {
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
        .setup(|app| {
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

            // Get main window
            let _window = app.get_webview_window("main")
                .expect("Failed to get main window");

            // Note: Window will stay visible until user presses Escape or the global shortcut again
            // This prevents the window from disappearing when releasing the keyboard shortcut

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
            commands::get_settings,
            commands::update_setting,
            commands::update_global_shortcut,
            commands::toggle_main_window,
            commands::hide_main_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

