use crate::database::{App, DbPool, NewApp, Settings};
use crate::{database, icon_extractor, launcher};
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

