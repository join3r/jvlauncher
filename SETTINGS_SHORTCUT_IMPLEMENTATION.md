# Settings Keyboard Shortcut Implementation

## Overview
Implemented a keyboard shortcut to open the settings window following macOS conventions.

## Implementation Date
October 28, 2025

## Feature Description
Added support for the standard macOS preferences keyboard shortcut:
- **macOS**: `Command + ,` (comma)
- **Windows/Linux**: `Ctrl + ,` (comma)

This follows the standard convention used by most macOS applications for accessing preferences/settings.

## Changes Made

### File Modified: `dist/app.js`

**Location**: Lines 434-482 (global keyboard event listener)

**What Changed**:
1. Added a new keyboard shortcut handler for `Command+,` / `Ctrl+,`
2. The handler checks for:
   - The comma key (`,`)
   - Either `metaKey` (Command on macOS) or `ctrlKey` (Ctrl on Windows/Linux)
   - No Shift or Alt modifiers (to ensure it's exactly Command+, or Ctrl+,)
3. When detected, it calls `showSettingsModal()` to open the settings window
4. Updated the typing detection logic to allow the comma key to be processed even when in input fields (for the settings shortcut)

**Code Added**:
```javascript
// Handle Command+, (macOS) or Ctrl+, (Windows/Linux) to open settings
// This is the standard preferences shortcut on macOS
if (e.key === ',' && (e.metaKey || e.ctrlKey) && !e.shiftKey && !e.altKey) {
    e.preventDefault();
    console.log('Settings shortcut pressed (Command/Ctrl+,), opening settings window');
    showSettingsModal();
    return;
}
```

## How It Works

1. **Event Capture**: The global keyboard listener in `dist/app.js` captures all keydown events
2. **Recording Mode Check**: First checks if the user is in shortcut recording mode (to avoid conflicts)
3. **Input Field Check**: Checks if the user is typing in an input field (but allows comma key for settings shortcut)
4. **Settings Shortcut Detection**: Detects `Command+,` or `Ctrl+,` combination
5. **Window Opening**: Calls the existing `showSettingsModal()` function which invokes the Tauri backend command `open_settings_window`
6. **Window Management**: The backend handles:
   - Checking if settings window already exists
   - If exists: brings it to focus
   - If not: creates a new settings window
   - Manages always-on-top behavior for proper window layering

## Testing Instructions

### Manual Testing
1. **Start the application**:
   ```bash
   cd /Users/join3r/Downloads/Temp/jvlauncher
   cargo tauri dev
   ```

2. **Open the launcher window** using the global shortcut (default: `Command+Shift+Space` on macOS)

3. **Test the settings shortcut**:
   - Press `Command + ,` (on macOS) or `Ctrl + ,` (on Windows/Linux)
   - The settings window should open

4. **Test when settings window is already open**:
   - Press `Command + ,` again
   - The existing settings window should come to focus (not create a duplicate)

5. **Test from different contexts**:
   - With launcher window focused
   - With settings window already open
   - After closing and reopening the launcher

### Expected Behavior
✅ Pressing `Command+,` (macOS) or `Ctrl+,` (Windows/Linux) opens the settings window
✅ If settings window is already open, it brings it to focus
✅ The shortcut works when the launcher window has focus
✅ The shortcut is prevented from interfering with text input (except when specifically pressed)
✅ Console logs confirm the shortcut was detected

## Integration with Existing Features

### Compatible With:
- ✅ **App Shortcuts**: The settings shortcut is checked before app-specific shortcuts
- ✅ **Escape Key**: Still works to hide the main window
- ✅ **Shortcut Recording**: Recording mode takes priority over the settings shortcut
- ✅ **Text Input**: Doesn't interfere with typing in input fields (except when Command/Ctrl is held)
- ✅ **Existing Settings Button**: The gear icon button still works as before

### Backend Integration:
The implementation uses the existing backend infrastructure:
- `open_settings_window` Tauri command (defined in `src-tauri/src/commands.rs`)
- Window management logic that handles window creation and focus
- Always-on-top behavior management for proper window layering

## Platform-Specific Behavior

### macOS
- Uses `Command + ,` (the standard macOS preferences shortcut)
- Detected via `event.metaKey`
- Follows macOS Human Interface Guidelines

### Windows/Linux
- Uses `Ctrl + ,` (equivalent to macOS Command)
- Detected via `event.ctrlKey`
- Provides consistent cross-platform experience

## Benefits

1. **Standard Convention**: Follows the macOS standard for preferences access
2. **Muscle Memory**: Users familiar with macOS apps will instinctively know how to access settings
3. **Keyboard-Driven**: Enhances the keyboard-first design of the launcher
4. **No Conflicts**: Carefully designed to avoid conflicts with existing shortcuts
5. **Cross-Platform**: Works consistently across macOS, Windows, and Linux

## Future Enhancements

Potential improvements for future versions:
- [ ] Add the shortcut to a help/documentation screen
- [ ] Consider adding a menu bar item showing the shortcut
- [ ] Add to the system tray menu as a hint
- [ ] Include in onboarding/tutorial if one is added

## Related Files

- **Frontend**: `dist/app.js` - Keyboard event handling
- **Backend**: `src-tauri/src/commands.rs` - `open_settings_window` command
- **Backend**: `src-tauri/src/main.rs` - Window setup and event handling

## Notes

- The shortcut only works when the launcher window has focus
- This is intentional to avoid conflicts with other applications
- The global shortcut (Command+Shift+Space) is still needed to show the launcher window
- Once the launcher is visible, Command+, can be used to access settings

## Verification

The implementation has been tested and verified to:
- ✅ Compile successfully
- ✅ Not introduce any TypeScript/JavaScript errors
- ✅ Follow the existing code style and patterns
- ✅ Integrate seamlessly with existing keyboard handling
- ✅ Work with the existing settings window infrastructure

