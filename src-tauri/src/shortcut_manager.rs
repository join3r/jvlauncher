use anyhow::Result;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use std::collections::HashMap;
use std::sync::Mutex;

/// Global state to track registered app shortcuts
static APP_SHORTCUTS: Mutex<Option<HashMap<String, i64>>> = Mutex::new(None);

/// Global state to track app window labels by app ID
static APP_WINDOWS: Mutex<Option<HashMap<i64, String>>> = Mutex::new(None);

/// Global state to track the previously focused application (by bundle ID)
/// This allows us to restore focus when hiding jvlauncher windows
static PREVIOUS_APP: Mutex<Option<String>> = Mutex::new(None);

/// Initialize the app shortcuts map
fn init_app_shortcuts() {
    let mut shortcuts = APP_SHORTCUTS.lock().unwrap();
    if shortcuts.is_none() {
        *shortcuts = Some(HashMap::new());
    }
}

/// Initialize the app windows map
fn init_app_windows() {
    let mut windows = APP_WINDOWS.lock().unwrap();
    if windows.is_none() {
        *windows = Some(HashMap::new());
    }
}

/// Register a window for an app (called when launching)
pub fn register_app_window(app_id: i64, window_label: String) {
    init_app_windows();
    let mut windows = APP_WINDOWS.lock().unwrap();
    if let Some(map) = windows.as_mut() {
        map.insert(app_id, window_label);
    }
}

/// Unregister a window for an app (called when window is closed)
pub fn unregister_app_window(app_id: i64) {
    init_app_windows();
    let mut windows = APP_WINDOWS.lock().unwrap();
    if let Some(map) = windows.as_mut() {
        map.remove(&app_id);
    }
}

/// Capture the currently focused application before showing a jvlauncher window
/// This allows us to restore focus later when hiding the window
fn capture_previous_app() {
    if let Some(bundle_id) = crate::macos_delegate::get_frontmost_app_bundle_id() {
        let mut previous = PREVIOUS_APP.lock().unwrap();
        *previous = Some(bundle_id);
    }
}

/// Public function to capture the currently focused application
/// This can be called from other modules before showing windows
pub fn capture_current_app() {
    capture_previous_app();
}

/// Restore focus to the previously focused application
/// Returns true if focus was restored, false otherwise
fn restore_previous_app() -> bool {
    let bundle_id = {
        let previous = PREVIOUS_APP.lock().unwrap();
        previous.clone()
    };

    if let Some(bundle_id) = bundle_id {
        crate::macos_delegate::activate_app_by_bundle_id(&bundle_id)
    } else {
        false
    }
}

/// Public function to restore focus to the previously focused application
/// This can be called from other modules when hiding windows
pub fn restore_focus_to_previous_app() {
    restore_previous_app();
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
                    // Window is visible, hide it and restore focus to previous app
                    let _ = window.hide();
                    restore_previous_app();
                } else {
                    // Window is hidden, capture current app before showing
                    capture_previous_app();
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
    init_app_windows();

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
            // Check if the app's window is already open and focused
            let window_label = {
                let windows = APP_WINDOWS.lock().unwrap();
                windows.as_ref()
                    .and_then(|map| map.get(&app_id))
                    .cloned()
            };

            if let Some(label) = window_label {
                // Window exists, check if it's focused
                if let Some(window) = app_handle_clone.get_webview_window(&label) {
                    let is_focused = window.is_focused().unwrap_or(false);
                    let is_visible = window.is_visible().unwrap_or(false);

                    if is_focused && is_visible {
                        // Window is focused, hide it (toggle off) and restore previous app
                        let _ = window.hide();
                        restore_previous_app();
                        return;
                    } else if is_visible {
                        // Window is visible but not focused, capture current app then focus it
                        capture_previous_app();
                        let _ = window.set_focus();
                        return;
                    }
                    // If window exists but is hidden, fall through to launch
                }
            }

            // Window doesn't exist or is hidden, capture current app then launch
            capture_previous_app();
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

