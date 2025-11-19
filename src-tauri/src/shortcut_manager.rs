use anyhow::Result;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use std::collections::HashMap;
use std::sync::Mutex;

/// Global state to track registered app shortcuts
static APP_SHORTCUTS: Mutex<Option<HashMap<String, i64>>> = Mutex::new(None);

/// Global state to track app window labels by app ID
static APP_WINDOWS: Mutex<Option<HashMap<i64, String>>> = Mutex::new(None);

/// Global state to track hide_on_shortcut setting by app ID
static APP_HIDE_ON_SHORTCUT: Mutex<Option<HashMap<i64, bool>>> = Mutex::new(None);

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

/// Initialize the app hide_on_shortcut map
fn init_app_hide_on_shortcut() {
    let mut hide_settings = APP_HIDE_ON_SHORTCUT.lock().unwrap();
    if hide_settings.is_none() {
        *hide_settings = Some(HashMap::new());
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
    hide_on_shortcut: bool,
) -> Result<()> {
    init_app_shortcuts();
    init_app_windows();
    init_app_hide_on_shortcut();

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

    // Store the hide_on_shortcut setting
    {
        let mut hide_settings = APP_HIDE_ON_SHORTCUT.lock().unwrap();
        if let Some(map) = hide_settings.as_mut() {
            map.insert(app_id, hide_on_shortcut);
        }
    }

    // Register the shortcut
    let app_handle_clone = app_handle.clone();
    app_handle.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
        use tauri_plugin_global_shortcut::ShortcutState;

        if event.state() == ShortcutState::Pressed {
            // Get the hide_on_shortcut setting for this app
            let should_hide = {
                let hide_settings = APP_HIDE_ON_SHORTCUT.lock().unwrap();
                hide_settings.as_ref()
                    .and_then(|map| map.get(&app_id))
                    .copied()
                    .unwrap_or(false)
            };

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
                        if should_hide {
                            // Hide mode: hide the window and restore previous app
                            let _ = window.hide();
                            restore_previous_app();
                        } else {
                            // Close mode: close the window
                            let _ = window.close();
                        }
                        return;
                    } else if is_visible {
                        // Window is visible but not focused, focus it
                        // On macOS, use native APIs to reliably bring window to front
                        // This handles unminimizing through native APIs
                        crate::macos_delegate::bring_window_to_front(&window);
                        let _ = window.set_focus();

                        // Hide the launcher window AFTER focusing the target
                        // Use the helper to handle the delicate timing on macOS
                        crate::macos_delegate::switch_focus_and_hide_launcher(&app_handle_clone, &window);

                        return;
                    } else if should_hide {
                        // Window exists but is hidden, show and focus it
                        // On macOS, use native APIs to reliably bring window to front
                        // This handles visibility and unminimizing through native APIs
                        crate::macos_delegate::bring_window_to_front(&window);
                        let _ = window.set_focus();

                        // Hide the launcher window AFTER focusing the target
                        // Use the helper to handle the delicate timing on macOS
                        crate::macos_delegate::switch_focus_and_hide_launcher(&app_handle_clone, &window);

                        return;
                    }
                    // If window exists but is hidden and not in hide mode, fall through to launch
                }
            }

            // Window doesn't exist or is hidden (in close mode), hide launcher then launch
            if let Some(main_window) = app_handle_clone.get_webview_window("main") {
                let _ = main_window.hide();
            }
            let _ = app_handle_clone.emit("launch-app-by-shortcut", app_id);
        }
    })?;

    Ok(())
}

/// Unregister a global shortcut for an app
pub fn unregister_app_shortcut(
    app_handle: &AppHandle,
    app_id: i64,
    shortcut_str: &str,
) -> Result<()> {
    init_app_shortcuts();
    init_app_hide_on_shortcut();

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

    // Remove hide_on_shortcut setting
    {
        let mut hide_settings = APP_HIDE_ON_SHORTCUT.lock().unwrap();
        if let Some(map) = hide_settings.as_mut() {
            map.remove(&app_id);
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

