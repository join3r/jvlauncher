use anyhow::Result;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

/// Register a global shortcut to toggle the main window
pub fn register_global_shortcut(
    app_handle: &AppHandle,
    shortcut_str: &str,
) -> Result<()> {
    // Parse the shortcut string
    let shortcut: Shortcut = shortcut_str.parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse shortcut: {:?}", e))?;

    // Unregister any existing shortcuts first
    let _ = app_handle.global_shortcut().unregister_all();

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

/// Unregister all global shortcuts
pub fn unregister_all_shortcuts(app_handle: &AppHandle) -> Result<()> {
    app_handle.global_shortcut().unregister_all()
        .map_err(|e| anyhow::anyhow!("Failed to unregister shortcuts: {:?}", e))?;
    Ok(())
}

/// Update the global shortcut
pub fn update_global_shortcut(
    app_handle: &AppHandle,
    new_shortcut: &str,
) -> Result<()> {
    unregister_all_shortcuts(app_handle)?;
    register_global_shortcut(app_handle, new_shortcut)?;
    Ok(())
}

