# System Tray Implementation

## Overview

This document describes the system tray implementation that solves the Cmd+Q behavior issue on macOS.

## The Problem

On macOS, pressing **Cmd+Q** sends a terminate signal directly to the application at the OS level, completely bypassing Tauri's event system. This means:

1. `RunEvent::ExitRequested` is **NOT triggered** by Cmd+Q
2. Window close handlers are **NOT triggered** by Cmd+Q
3. The app **terminates immediately** without any chance to intercept

This is a known limitation of Tauri on macOS (see GitHub issues #9198 and #13511).

## The Solution

Instead of trying to intercept Cmd+Q (which is impossible), we implemented a **system tray icon** that keeps the app running in the background. This is the standard approach for launcher-style applications on macOS (like Raycast, Alfred, Spotlight alternatives, etc.).

### How It Works

1. **App runs in the background**: The app never quits automatically
2. **System tray icon**: Always visible in the menu bar
3. **Tray menu**: Provides "Show Launcher" and "Quit" options
4. **Click behavior**: Left-clicking the tray icon toggles the launcher window
5. **Window close**: Closing windows (X button or Cmd+Q) only hides them, doesn't quit the app
6. **Explicit quit**: Users must quit from the tray menu

## Implementation Details

### Files Modified

1. **`src-tauri/Cargo.toml`**
   - Added `tray-icon` feature to Tauri

2. **`src-tauri/src/main.rs`**
   - Added tray icon imports
   - Created tray menu with "Show Launcher" and "Quit" items
   - Added tray icon click handler (toggles launcher visibility)
   - Added menu item click handlers

### Code Structure

```rust
// Import tray-related modules
use tauri::{
    Manager, 
    menu::{MenuBuilder, MenuItemBuilder}, 
    tray::{TrayIconBuilder, TrayIconEvent}
};

// In setup function:
// 1. Create menu items
let show_item = MenuItemBuilder::with_id("show", "Show Launcher").build(app)?;
let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

// 2. Build menu
let menu = MenuBuilder::new(app)
    .item(&show_item)
    .separator()
    .item(&quit_item)
    .build()?;

// 3. Create tray icon
let _tray = TrayIconBuilder::new()
    .menu(&menu)
    .icon(app.default_window_icon().unwrap().clone())
    .on_menu_event(|app, event| {
        // Handle menu clicks
    })
    .on_tray_icon_event(|tray, event| {
        // Handle tray icon clicks
    })
    .build(app)?;
```

### Tray Menu Items

| Menu Item | Action |
|-----------|--------|
| **Show Launcher** | Shows and focuses the launcher window |
| **Quit** | Exits the application completely |

### Tray Icon Click Behavior

- **Left Click**: Toggles launcher window visibility
  - If visible → Hide
  - If hidden → Show and focus

## User Experience

### Before (Without System Tray)

❌ **Problem**: Pressing Cmd+Q on any window would quit the entire app
❌ **Problem**: No way to prevent app from quitting
❌ **Problem**: Had to restart the app after accidentally quitting

### After (With System Tray)

✅ **Solution**: App runs in the background with tray icon
✅ **Solution**: Cmd+Q only closes/hides windows, doesn't quit app
✅ **Solution**: Explicit quit from tray menu
✅ **Solution**: Quick access to launcher via tray icon click

## Testing Instructions

### Test 1: System Tray Icon Visibility
1. Start the app
2. **Expected**: Tray icon appears in the menu bar (top-right on macOS)
3. **Expected**: Icon uses the app's default icon

### Test 2: Tray Menu
1. Click the tray icon
2. **Expected**: Menu appears with "Show Launcher" and "Quit" options
3. Click "Show Launcher"
4. **Expected**: Launcher window appears and gets focus

### Test 3: Tray Icon Click (Toggle)
1. Left-click the tray icon
2. **Expected**: Launcher window shows
3. Left-click the tray icon again
4. **Expected**: Launcher window hides

### Test 4: Cmd+Q on Launcher (Main Window)
1. Show the launcher window
2. Press **Cmd+Q** while launcher is focused
3. **Expected**: Launcher window hides
4. **Expected**: App stays running (tray icon still visible)
5. Click tray icon
6. **Expected**: Launcher reappears

### Test 5: Cmd+Q on Webapp Window
1. Launch a webapp from the launcher
2. Press **Cmd+Q** while webapp is focused
3. **Expected**: Webapp window closes
4. **Expected**: App stays running (tray icon still visible)
5. Press **Cmd+Shift+Space** (or your global shortcut)
6. **Expected**: Launcher reappears

### Test 6: Close Button (X) on Launcher
1. Show the launcher window
2. Click the close button (X) or red dot on macOS
3. **Expected**: Launcher window hides
4. **Expected**: App stays running (tray icon still visible)

### Test 7: Close Button (X) on Webapp
1. Launch a webapp
2. Click the close button (X) on the webapp window
3. **Expected**: Webapp window closes
4. **Expected**: App stays running (tray icon still visible)

### Test 8: Quit from Tray Menu
1. Click the tray icon
2. Click "Quit"
3. **Expected**: All windows close
4. **Expected**: Tray icon disappears
5. **Expected**: App quits completely

## Behavior Summary

| Action | Launcher Window | Webapp Window | App Status |
|--------|----------------|---------------|------------|
| **Cmd+Q** | Hides | Closes | Running |
| **Close button (X)** | Hides | Closes | Running |
| **Tray → Quit** | Closes | Closes | **Quits** |
| **Tray icon click** | Toggles | - | Running |
| **Tray → Show Launcher** | Shows | - | Running |

## Technical Notes

### Why This Approach?

1. **macOS Standard**: Menu bar apps are the standard for launcher-style applications on macOS
2. **Reliable**: Doesn't rely on intercepting OS-level signals
3. **User-Friendly**: Clear visual indicator that app is running
4. **Explicit Control**: Users have full control over when to quit

### Alternative Approaches Considered

1. **Intercept Cmd+Q with native code**: Requires Objective-C plugin, complex
2. **Hide all windows**: Confusing UX, no visual indicator app is running
3. **Prevent all window closes**: Frustrating for users, non-standard behavior

### Platform Differences

- **macOS**: Tray icon appears in menu bar (top-right)
- **Windows**: Tray icon appears in system tray (bottom-right)
- **Linux**: Tray icon appears in system tray (varies by desktop environment)

## Future Enhancements

Potential improvements:

1. **Custom Tray Icon**: Create a smaller, monochrome icon optimized for menu bar
2. **Recent Apps**: Add recently launched apps to tray menu
3. **Quick Launch**: Add favorite apps to tray menu for quick access
4. **Keyboard Shortcut**: Show keyboard shortcut in tray menu items
5. **Notifications**: Show notifications for app launches

## Troubleshooting

### Tray icon not appearing

1. Check if the app is running (Activity Monitor on macOS)
2. Restart the app
3. Check system tray settings (some systems hide tray icons)

### Can't quit the app

1. Use the tray menu → Quit
2. If tray icon is not visible, use Activity Monitor to force quit
3. Check if multiple instances are running

### Tray icon click not working

1. Try right-clicking the tray icon (shows menu)
2. Use the tray menu instead of clicking the icon
3. Restart the app

## Related Documentation

- [CMD_Q_BEHAVIOR.md](CMD_Q_BEHAVIOR.md) - Previous Cmd+Q implementation attempts
- [WINDOW_BEHAVIOR_FIXES.md](WINDOW_BEHAVIOR_FIXES.md) - Window behavior documentation
- [Tauri Tray Icon Documentation](https://v2.tauri.app/reference/javascript/api/namespacetray/)

## Conclusion

The system tray implementation provides a reliable, user-friendly solution to the Cmd+Q behavior issue on macOS. It follows platform conventions and gives users explicit control over the application lifecycle.

