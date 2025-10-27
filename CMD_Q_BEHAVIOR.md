# Cmd+Q Keyboard Shortcut Behavior (macOS) - DEPRECATED

## ⚠️ This Document is Deprecated

**This approach has been replaced with a system tray implementation.**

See [SYSTEM_TRAY_IMPLEMENTATION.md](SYSTEM_TRAY_IMPLEMENTATION.md) for the current solution.

---

## Overview (Historical)

On macOS, we attempted to customize the Cmd+Q keyboard shortcut to provide a better user experience. The goal was to close only the focused window while keeping the app running in the background.

**Note**: This approach did not work reliably because Cmd+Q on macOS sends a terminate signal directly to the application, bypassing Tauri's event system.

## Behavior

### When Cmd+Q is Pressed:

1. **Main Launcher Window Focused**
   - The main window will **hide** (not close)
   - The application continues running in the background
   - You can show the window again using the global shortcut (default: Cmd+Shift+Space)

2. **Webapp Window Focused** (e.g., web applications launched from the launcher)
   - The webapp window will **close**
   - The application continues running in the background
   - Other windows remain open

3. **Terminal Window Focused** (e.g., TUI applications)
   - The terminal window will **close**
   - The running process in the terminal will be **killed**
   - The application continues running in the background
   - Other windows remain open

4. **No Window Focused**
   - No action is taken
   - The launcher window remains in its current state (hidden or visible)

5. **Last Window Closes** (programmatic exit)
   - The app does NOT quit
   - All windows remain in their current state
   - This ensures the launcher stays running even when all visible windows are closed

## How to Actually Quit the Application

Since Cmd+Q no longer quits the entire application, you have several options to completely quit:

### Option 1: Dock Icon (macOS)
1. Right-click on the app icon in the Dock
2. Select "Quit"

### Option 2: Menu Bar (if implemented)
- Use the application menu: `App Name → Quit` (Cmd+Q from menu will still work if you add a menu item)

### Option 3: Programmatically (for developers)
- Call the `quit_app` Tauri command from the frontend:
  ```javascript
  import { invoke } from '@tauri-apps/api/core';
  
  // Quit the entire application
  await invoke('quit_app');
  ```

## Implementation Details

### Files Modified

1. **`src-tauri/src/main.rs`**
   - Added event handler in the `.run()` method to intercept `RunEvent::ExitRequested`
   - Prevents default quit behavior with `api.prevent_exit()`
   - Determines which window is focused and takes appropriate action
   - Only active on macOS (uses `#[cfg(target_os = "macos")]`)

2. **`src-tauri/src/commands.rs`**
   - Added new `quit_app` command to allow programmatic quitting
   - This is the only way to quit the app from code after intercepting Cmd+Q

### Code Snippets

#### Main Event Handler (main.rs)
```rust
.run(|app_handle, event| {
    #[cfg(target_os = "macos")]
    {
        use tauri::RunEvent;

        match event {
            RunEvent::ExitRequested { api, code, .. } => {
                // ALWAYS prevent the app from quitting
                api.prevent_exit();

                // Only handle window-specific actions if this is a Cmd+Q (code is None)
                // When the last window closes, code is Some(0), and we should do nothing
                if code.is_none() {
                    // Get focused window and close/hide it
                    if let Some(focused_window) = app_handle.webview_windows().values()
                        .find(|w| w.is_focused().unwrap_or(false))
                    {
                        let window_label = focused_window.label();

                        if window_label.starts_with("webapp_") || window_label.starts_with("terminal_") {
                            let _ = focused_window.close();
                        } else if window_label == "main" {
                            let _ = focused_window.hide();
                        }
                    }
                }
                // If code is Some(0), it's a programmatic exit (last window closed) - ignore it
            }
            _ => {}
        }
    }
});
```

#### Quit Command (commands.rs)
```rust
#[tauri::command]
pub fn quit_app(app_handle: AppHandle) -> Result<(), String> {
    app_handle.exit(0);
    Ok(())
}
```

## Platform-Specific Considerations

### macOS
- ✅ Cmd+Q is intercepted and customized
- ✅ Window close behavior is preserved
- ✅ App continues running in background
- ✅ Can quit via Dock icon

### Windows/Linux
- ℹ️ This behavior is **macOS-only**
- ℹ️ On other platforms, the default quit behavior is preserved
- ℹ️ Alt+F4 (Windows) or Ctrl+Q (Linux) will quit the app normally

## Testing

To test the new behavior:

1. **Start the application**
   ```bash
   cd src-tauri
   cargo run
   ```

2. **Test Main Window**
   - Open the launcher window (Cmd+Shift+Space)
   - Press Cmd+Q
   - ✅ Window should hide (not quit)
   - Press Cmd+Shift+Space again
   - ✅ Window should reappear

3. **Test Webapp Window**
   - Launch a webapp from the launcher
   - Focus the webapp window
   - Press Cmd+Q
   - ✅ Webapp window should close
   - ✅ Main app should still be running

4. **Test Terminal Window**
   - Launch a TUI application from the launcher
   - Focus the terminal window
   - Press Cmd+Q
   - ✅ Terminal window should close
   - ✅ Process should be killed
   - ✅ Main app should still be running

5. **Test Actual Quit**
   - Right-click app icon in Dock
   - Select "Quit"
   - ✅ Application should completely quit

## Future Enhancements

Potential improvements to consider:

1. **Add Application Menu**
   - Create a proper macOS menu bar with "Quit" option
   - This would provide a more native way to quit the app

2. **Add Tray Icon**
   - Add a system tray icon with a "Quit" menu item
   - Useful for quick access to quit functionality

3. **Add Settings Toggle**
   - Allow users to enable/disable Cmd+Q interception
   - Some users might prefer the default behavior

4. **Add Confirmation Dialog**
   - Show a confirmation dialog when quitting via Dock
   - Prevent accidental quits

## Troubleshooting

### Issue: Cmd+Q still quits the app
- **Solution**: Make sure you're running on macOS. The behavior is platform-specific.
- **Check**: Verify the code was compiled with the macOS target

### Issue: Can't quit the app at all
- **Solution**: Use the Dock icon → Quit option
- **Alternative**: Force quit using Activity Monitor or Cmd+Option+Esc

### Issue: Webapp windows don't close
- **Solution**: Make sure the window label starts with "webapp_"
- **Check**: Verify in `launcher.rs` that the window label format is correct

## Summary

This implementation provides a more intuitive user experience on macOS by:
- ✅ Preventing accidental app quits when pressing Cmd+Q
- ✅ Allowing individual windows to close while keeping the app running
- ✅ Maintaining the main launcher window in the background for quick access
- ✅ Providing alternative ways to quit the app when needed
- ✅ Being platform-specific (macOS only) to respect platform conventions

