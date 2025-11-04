use anyhow::Result;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use std::collections::HashMap;
use std::sync::Mutex;

/// Global state to track registered app shortcuts
static APP_SHORTCUTS: Mutex<Option<HashMap<String, i64>>> = Mutex::new(None);

/// Initialize the app shortcuts map
fn init_app_shortcuts() {
    let mut shortcuts = APP_SHORTCUTS.lock().unwrap();
    if shortcuts.is_none() {
        *shortcuts = Some(HashMap::new());
    }
}

/// Register a global shortcut to toggle the main window
pub fn register_global_shortcut(
    app_handle: &AppHandle,
    shortcut_str: &str,
) -> Result<()> {
    // Parse the shortcut string
    let shortcut: Shortcut = shortcut_str.parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse shortcut: {:?}", e))?;

    // Unregister any existing launcher shortcut first
    // Note: We don't unregister ALL shortcuts here to preserve app shortcuts
    let _ = app_handle.global_shortcut().unregister(shortcut.clone());

    // Register the new shortcut
    let app_handle_clone = app_handle.clone();
    app_handle.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
        // Only respond to key DOWN events (state == Pressed), not key UP events (state == Released)
        // This prevents the window from toggling when releasing the shortcut
        use tauri_plugin_global_shortcut::ShortcutState;

        if event.state() == ShortcutState::Pressed {
            if let Some(window) = app_handle_clone.get_webview_window("main") {
                let is_visible = window.is_visible().unwrap_or(false);

                if is_visible {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.center();
                }
            }
        }
    })?;

    Ok(())
}

/// Register a global shortcut for an app
pub fn register_app_shortcut(
    app_handle: &AppHandle,
    app_id: i64,
    shortcut_str: &str,
) -> Result<()> {
    init_app_shortcuts();

    // Parse the shortcut string
    let shortcut: Shortcut = shortcut_str.parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse shortcut: {:?}", e))?;

    // Store the mapping
    {
        let mut shortcuts = APP_SHORTCUTS.lock().unwrap();
        if let Some(map) = shortcuts.as_mut() {
            map.insert(shortcut_str.to_string(), app_id);
        }
    }

    // Register the shortcut
    let app_handle_clone = app_handle.clone();
    app_handle.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
        use tauri_plugin_global_shortcut::ShortcutState;

        if event.state() == ShortcutState::Pressed {
            // Launch the app
            let _ = app_handle_clone.emit("launch-app-by-shortcut", app_id);
        }
    })?;

    Ok(())
}

/// Unregister a global shortcut for an app
pub fn unregister_app_shortcut(
    app_handle: &AppHandle,
    shortcut_str: &str,
) -> Result<()> {
    init_app_shortcuts();

    // Parse the shortcut string
    let shortcut: Shortcut = shortcut_str.parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse shortcut: {:?}", e))?;

    // Remove from mapping
    {
        let mut shortcuts = APP_SHORTCUTS.lock().unwrap();
        if let Some(map) = shortcuts.as_mut() {
            map.remove(shortcut_str);
        }
    }

    // Unregister the shortcut
    app_handle.global_shortcut().unregister(shortcut)
        .map_err(|e| anyhow::anyhow!("Failed to unregister shortcut: {:?}", e))?;

    Ok(())
}

/// Update the global shortcut
pub fn update_global_shortcut(
    app_handle: &AppHandle,
    new_shortcut: &str,
) -> Result<()> {
    // Don't unregister all shortcuts, just update the launcher shortcut
    register_global_shortcut(app_handle, new_shortcut)?;
    Ok(())
}

