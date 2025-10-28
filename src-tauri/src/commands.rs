use crate::database::{App, DbPool, NewApp, Settings};
use crate::{database, icon_extractor, launcher, terminal};
use tauri::{AppHandle, Manager, State};

/// Get all apps from the database
#[tauri::command]
pub fn get_all_apps(pool: State<DbPool>) -> Result<Vec<App>, String> {
    database::get_all_apps(&pool)
        .map_err(|e| format!("Failed to get apps: {}", e))
}

/// Create a new app
#[tauri::command]
pub fn create_app(
    pool: State<DbPool>,
    app_handle: AppHandle,
    new_app: NewApp,
) -> Result<i64, String> {
    // Generate session directory for webapps
    let session_dir = if new_app.app_type == database::AppType::Webapp {
        let app_data = app_handle.path().app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;
        
        let session_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let session_path = app_data.join("webapps").join(format!("session_{}", session_id));
        std::fs::create_dir_all(&session_path)
            .map_err(|e| format!("Failed to create session directory: {}", e))?;
        
        Some(session_path)
    } else {
        None
    };

    database::create_app(&pool, new_app, session_dir)
        .map_err(|e| format!("Failed to create app: {}", e))
}

/// Update an existing app
#[tauri::command]
pub fn update_app(pool: State<DbPool>, app: App) -> Result<(), String> {
    database::update_app(&pool, app)
        .map_err(|e| format!("Failed to update app: {}", e))
}

/// Delete an app
#[tauri::command]
pub fn delete_app(pool: State<DbPool>, app_id: i64) -> Result<(), String> {
    database::delete_app(&pool, app_id)
        .map_err(|e| format!("Failed to delete app: {}", e))
}

/// Reorder apps
#[tauri::command]
pub fn reorder_apps(pool: State<DbPool>, app_ids: Vec<i64>) -> Result<(), String> {
    database::reorder_apps(&pool, app_ids)
        .map_err(|e| format!("Failed to reorder apps: {}", e))
}

/// Launch an app
#[tauri::command]
pub fn launch(pool: State<DbPool>, app_handle: AppHandle, app_id: i64) -> Result<(), String> {
    let apps = database::get_all_apps(&pool)
        .map_err(|e| format!("Failed to get apps: {}", e))?;
    
    let app = apps.iter()
        .find(|a| a.id == app_id)
        .ok_or_else(|| format!("App with id {} not found", app_id))?;

    launcher::launch_app(app, &app_handle)
        .map_err(|e| format!("Failed to launch app: {}", e))?;

    // Hide the main launcher window after launching
    if let Some(window) = app_handle.get_webview_window("main") {
        window.hide().map_err(|e| format!("Failed to hide window: {}", e))?;
    }

    Ok(())
}

/// Extract icon from a binary file
#[tauri::command]
pub fn extract_icon_from_binary(
    app_handle: AppHandle,
    binary_path: String,
) -> Result<String, String> {
    let app_data = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    
    let icons_dir = app_data.join("icons");
    icon_extractor::ensure_icons_dir(&icons_dir)
        .map_err(|e| format!("Failed to create icons directory: {}", e))?;

    icon_extractor::extract_icon_from_binary(&binary_path, &icons_dir)
        .map_err(|e| format!("Failed to extract icon: {}", e))
}

/// Save an icon from a user-provided file
#[tauri::command]
pub fn save_icon_from_file(
    app_handle: AppHandle,
    source_path: String,
    app_name: String,
) -> Result<String, String> {
    let app_data = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    
    let icons_dir = app_data.join("icons");
    icon_extractor::ensure_icons_dir(&icons_dir)
        .map_err(|e| format!("Failed to create icons directory: {}", e))?;

    icon_extractor::save_icon_from_file(&source_path, &icons_dir, &app_name)
        .map_err(|e| format!("Failed to save icon: {}", e))
}

/// Get application settings
#[tauri::command]
pub fn get_settings(pool: State<DbPool>) -> Result<Settings, String> {
    database::get_settings(&pool)
        .map_err(|e| format!("Failed to get settings: {}", e))
}

/// Update a single setting
#[tauri::command]
pub fn update_setting(pool: State<DbPool>, key: String, value: String) -> Result<(), String> {
    database::update_setting(&pool, &key, &value)
        .map_err(|e| format!("Failed to update setting: {}", e))
}

/// Update the global shortcut and re-register it
#[tauri::command]
pub fn update_global_shortcut(app_handle: AppHandle, shortcut: String) -> Result<(), String> {
    crate::shortcut_manager::update_global_shortcut(&app_handle, &shortcut)
        .map_err(|e| format!("Failed to update global shortcut: {}", e))
}

/// Toggle the main launcher window
#[tauri::command]
pub fn toggle_main_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|e| format!("Failed to hide window: {}", e))?;
        } else {
            window.show().map_err(|e| format!("Failed to show window: {}", e))?;
            window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;
        }
    }
    Ok(())
}

/// Hide the main launcher window
#[tauri::command]
pub fn hide_main_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.hide().map_err(|e| format!("Failed to hide window: {}", e))?;
    }
    Ok(())
}

/// Quit the application completely
/// This is the only way to actually quit the app since Cmd+Q is intercepted on macOS
#[tauri::command]
pub fn quit_app(app_handle: AppHandle) -> Result<(), String> {
    app_handle.exit(0);
    Ok(())
}

/// Resize the main window based on grid dimensions
#[tauri::command]
pub fn resize_main_window(app_handle: AppHandle, grid_cols: i32, grid_rows: i32) -> Result<(), String> {
    use tauri::Manager;

    // Calculate window dimensions based on grid
    // Icon item dimensions:
    // - Icon: 52px (reduced from 64px for better density)
    // - Padding: 12px * 2 = 24px
    // - Text height: ~46px (name + shortcut)
    // - Total per item: ~122px height, ~100px width
    const ITEM_WIDTH: f64 = 100.0;
    const ITEM_HEIGHT: f64 = 122.0;
    const GRID_GAP: f64 = 16.0;
    const PADDING_HORIZONTAL: f64 = 48.0; // 24px left + 24px right
    const PADDING_TOP: f64 = 24.0; // 24px top padding
    const PADDING_BOTTOM: f64 = 24.0; // 24px bottom padding
    const FLOATING_BUTTONS_HEIGHT: f64 = 72.0; // Space for floating buttons (48px button + 24px margin)

    // Calculate dimensions
    let width = (ITEM_WIDTH * grid_cols as f64) + (GRID_GAP * (grid_cols - 1) as f64) + PADDING_HORIZONTAL;
    let height = (ITEM_HEIGHT * grid_rows as f64) + (GRID_GAP * (grid_rows - 1) as f64) + PADDING_TOP + PADDING_BOTTOM + FLOATING_BUTTONS_HEIGHT;

    println!("Calculated window size: {}x{} for grid {}x{}", width, height, grid_cols, grid_rows);

    // Apply min/max constraints
    let width = width.max(400.0).min(2000.0);
    let height = height.max(200.0).min(1400.0);

    println!("After constraints: {}x{}", width, height);

    // Get main window and resize
    if let Some(window) = app_handle.get_webview_window("main") {
        // Use Logical size instead of Physical to handle high-DPI displays correctly
        window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: width,
            height: height,
        }))
        .map_err(|e| format!("Failed to resize window: {}", e))?;

        // Center the window after resize
        window.center()
            .map_err(|e| format!("Failed to center window: {}", e))?;
    }

    Ok(())
}

/// Open the settings window
#[tauri::command]
pub fn open_settings_window(app_handle: AppHandle) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

    let window_label = "settings";

    // Disable always-on-top for main window to allow modal to appear on top
    if let Some(main_window) = app_handle.get_webview_window("main") {
        let _ = main_window.set_always_on_top(false);
    }

    // Check if window already exists
    if let Some(window) = app_handle.get_webview_window(window_label) {
        window.show().map_err(|e| format!("Failed to show window: {}", e))?;
        window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;
        return Ok(());
    }

    // Create new window
    let window = WebviewWindowBuilder::new(
        &app_handle,
        window_label,
        WebviewUrl::App("settings.html".into())
    )
    .title("Settings")
    .inner_size(540.0, 680.0)
    .resizable(false)
    .center()
    .always_on_top(true)
    .skip_taskbar(false)
    .build()
    .map_err(|e| format!("Failed to create settings window: {}", e))?;

    // Set up close handler to restore main window always-on-top
    let app_handle_clone = app_handle.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            if let Some(main_window) = app_handle_clone.get_webview_window("main") {
                let _ = main_window.set_always_on_top(true);
            }
        }
    });

    Ok(())
}

/// Open the add app window
#[tauri::command]
pub fn open_add_app_window(app_handle: AppHandle) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

    let window_label = "add-app";

    // Disable always-on-top for main window to allow modal to appear on top
    if let Some(main_window) = app_handle.get_webview_window("main") {
        let _ = main_window.set_always_on_top(false);
    }

    // Check if window already exists
    if let Some(window) = app_handle.get_webview_window(window_label) {
        window.show().map_err(|e| format!("Failed to show window: {}", e))?;
        window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;
        return Ok(());
    }

    // Create new window
    let window = WebviewWindowBuilder::new(
        &app_handle,
        window_label,
        WebviewUrl::App("app-form.html".into())
    )
    .title("Add Application")
    .inner_size(560.0, 680.0)
    .resizable(false)
    .center()
    .always_on_top(true)
    .skip_taskbar(false)
    .build()
    .map_err(|e| format!("Failed to create add app window: {}", e))?;

    // Set up close handler to restore main window always-on-top
    let app_handle_clone = app_handle.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            if let Some(main_window) = app_handle_clone.get_webview_window("main") {
                let _ = main_window.set_always_on_top(true);
            }
        }
    });

    Ok(())
}

/// Open the edit app window
#[tauri::command]
pub fn open_edit_app_window(app_handle: AppHandle, app_id: i64) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

    let window_label = format!("edit-app-{}", app_id);

    // Disable always-on-top for main window to allow modal to appear on top
    if let Some(main_window) = app_handle.get_webview_window("main") {
        let _ = main_window.set_always_on_top(false);
    }

    // Check if window already exists
    if let Some(window) = app_handle.get_webview_window(&window_label) {
        window.show().map_err(|e| format!("Failed to show window: {}", e))?;
        window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;
        return Ok(());
    }

    // Create new window
    let window = WebviewWindowBuilder::new(
        &app_handle,
        &window_label,
        WebviewUrl::App("app-form.html".into())
    )
    .title("Edit Application")
    .inner_size(560.0, 680.0)
    .resizable(false)
    .center()
    .always_on_top(true)
    .skip_taskbar(false)
    .build()
    .map_err(|e| format!("Failed to create edit app window: {}", e))?;

    // Set up close handler to restore main window always-on-top
    let app_handle_clone = app_handle.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            if let Some(main_window) = app_handle_clone.get_webview_window("main") {
                let _ = main_window.set_always_on_top(true);
            }
        }
    });

    Ok(())
}

/// Send input to a terminal window
#[tauri::command]
pub fn send_terminal_input(
    app_handle: AppHandle,
    window_label: String,
    data: String,
) -> Result<(), String> {
    use std::io::Write;

    if let Some(state) = app_handle.try_state::<terminal::TerminalState>() {
        if let Ok(windows) = state.windows.lock() {
            if let Some(handle) = windows.get(&window_label) {
                if let Ok(mut writer) = handle.writer.lock() {
                    writer.write_all(data.as_bytes())
                        .map_err(|e| format!("Failed to write to terminal: {}", e))?;
                    writer.flush()
                        .map_err(|e| format!("Failed to flush terminal: {}", e))?;
                    return Ok(());
                }
            }
        }
    }
    Err("Terminal window not found".to_string())
}

/// Resize a terminal PTY
#[tauri::command]
pub fn resize_terminal(
    app_handle: AppHandle,
    window_label: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    if let Some(state) = app_handle.try_state::<terminal::TerminalState>() {
        if let Ok(windows) = state.windows.lock() {
            if let Some(handle) = windows.get(&window_label) {
                if let Ok(master) = handle.master.lock() {
                    master.resize(portable_pty::PtySize {
                        rows,
                        cols,
                        pixel_width: 0,
                        pixel_height: 0,
                    })
                    .map_err(|e| format!("Failed to resize terminal: {}", e))?;
                    return Ok(());
                }
            }
        }
    }
    Err("Terminal window not found".to_string())
}

