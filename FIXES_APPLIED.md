# Fixes Applied

## Date: October 26, 2025

### Issue 1: Theme Switching Not Working ✅ FIXED

**Problem:** Changing theme in settings (Light/Dark/System) didn't apply to the UI - it remained in dark mode regardless of selection.

**Root Cause:** The CSS had hard-coded colors (like `#333`, `#666`, `#f8f9fa`) instead of using CSS variables, which overrode the theme system.

**Solution:**
1. Replaced all hard-coded colors in CSS with CSS variables:
   - `.settings-button` - now uses `var(--bg-secondary)` and `var(--border-color)`
   - `.icon-item .app-name` - now uses `var(--text-primary)`
   - `.icon-item .app-shortcut` - now uses `var(--text-secondary)` and `var(--bg-tertiary)`
   - `.context-menu button` - now uses `var(--text-primary)` and `var(--bg-tertiary)`

2. Removed redundant hard-coded dark theme media queries that were conflicting with the CSS variable system

**Files Modified:**
- `dist/styles.css` - Updated color values to use CSS variables

**How to Test:**
1. Run the app: `./dev.sh` or `docker compose up -d --build`
2. Click the settings button (⚙️) in the top-right
3. Change theme to "Light" and click Save
   - ✅ UI should immediately switch to light colors
4. Change theme to "Dark" and click Save
   - ✅ UI should immediately switch to dark colors
5. Change theme to "System" and click Save
   - ✅ UI should match your system theme

---

### Issue 2: Shortcut Recording Not Working ✅ FIXED

**Problem:** Users had to manually type keyboard shortcuts (like "CommandOrControl+Shift+A") which was error-prone and confusing.

**Requested Feature:** Click a "Record" button, then press the desired key combination, and it gets automatically formatted and written to the field.

**Solution:**
1. Added CSS styles for the record button with visual feedback:
   - `.btn-record` - Gray button that turns red when recording
   - Pulse animation when recording is active

2. Added JavaScript functionality:
   - `formatShortcut(event)` - Converts keyboard events to Tauri-compatible format
   - `startRecording(inputElement, buttonElement)` - Begins recording mode
   - `stopRecording()` - Ends recording mode
   - `handleRecordingKeyDown(event)` - Captures and formats the pressed keys
   - Global keydown listener that intercepts keys during recording mode

3. Updated Settings Modal:
   - Added "Record" button next to Global Shortcut input
   - Button changes to red and says "Press keys..." when active
   - Automatically stops recording and fills input when keys are pressed

4. Updated Add/Edit App Modals:
   - Added "Record" button next to per-app shortcut input
   - Same recording behavior as global shortcut

**Files Modified:**
- `dist/styles.css` - Added `.btn-record` styles and pulse animation
- `dist/app.js` - Added recording functionality and updated modals

**Key Features:**
- ✅ Converts modifier keys properly (Ctrl/Cmd → CommandOrControl, Alt → Alt, Shift → Shift)
- ✅ Formats single keys as uppercase (a → A, space → Space)
- ✅ Red pulsing button indicates recording is active
- ✅ Recording stops automatically after pressing a key combination
- ✅ Escape stops recording without setting a shortcut
- ✅ Closing modal cancels recording

**How to Test:**

**Global Shortcut:**
1. Run the app: `./dev.sh` or `docker compose up -d --build`
2. Click the settings button (⚙️)
3. In the "Global Shortcut" field, click the "Record" button
   - ✅ Button should turn red and pulse
   - ✅ Button text changes to "Press keys..."
4. Press a key combination (e.g., Cmd+Shift+K on Mac or Ctrl+Shift+K on Windows/Linux)
   - ✅ The input field should automatically fill with "CommandOrControl+Shift+K"
   - ✅ Button returns to gray and says "Record"
5. Click Save
   - ✅ Shortcut should be saved

**Per-App Shortcut:**
1. Click the "+" button to add an app (or right-click an existing app → Edit)
2. In the "Keyboard Shortcut" field, click the "Record" button
   - ✅ Button should turn red and pulse
3. Press a key combination (e.g., Cmd+1 or Ctrl+Alt+A)
   - ✅ The input field should automatically fill with the formatted shortcut
4. Fill in other fields and click Save
   - ✅ App should be saved with the shortcut

**Edge Cases:**
- Pressing Escape during recording cancels it without changing the value
- Closing the modal cancels recording
- Only one shortcut can be recorded at a time
- Pressing only modifier keys (Ctrl, Alt, Shift) doesn't stop recording - you need to press an actual key

---

### Issue 3: Keyboard Shortcuts with Option/Alt Showing Special Characters ✅ FIXED

**Problem:** When pressing Option+F on macOS, it showed "Alt+Ƒ" instead of "Alt+F" (Ƒ is the special character produced by Option+F on macOS).

**Root Cause:** The code was using `event.key` which returns the actual character produced, including special characters when Option/Alt is pressed. On macOS, Option acts as a modifier to type special characters.

**Solution:**
Updated `formatShortcut()` function to use `event.code` instead of `event.key`:
- `event.code` returns the physical key identifier (e.g., "KeyF", "Digit1")
- This works consistently across all platforms and modifier combinations
- Properly extracts the base key regardless of what character the combination produces

**Handled Cases:**
- Letter keys: `KeyA` → `A`, `KeyF` → `F`
- Number keys: `Digit0` → `0`, `Digit9` → `9`
- Arrow keys: `ArrowUp` → `Up`, `ArrowDown` → `Down`
- Special keys: Space, Enter, Tab, Escape, F1-F12, etc.

**Files Modified:**
- `dist/app.js` - Updated `formatShortcut()` function

**How to Test:**
1. Run the app and open Settings (⚙️)
2. Click "Record" next to Global Shortcut
3. Press Option+F (Mac) or Alt+F (Windows/Linux)
   - ✅ Should show "Alt+F" (not "Alt+Ƒ")
4. Try other combinations like Option+A, Option+1, etc.
   - ✅ All should display correctly as "Alt+A", "Alt+1"

---

### Issue 4: Per-App Shortcuts Not Working ✅ FIXED

**Problem:** Setting a keyboard shortcut for an individual app (e.g., Control+F) didn't launch the app when pressed in the launcher window.

**Root Cause:** The frontend keyboard handler only responded to arrow keys, Enter, and Escape. There was no code to check if pressed keys matched any app's shortcut.

**Solution:**
Added shortcut matching logic to the global keyboard event handler:
1. When any key is pressed, format it as a shortcut string (e.g., "CommandOrControl+F")
2. Check if it matches any app's shortcut in the apps array
3. If a match is found, launch that app immediately
4. Prevents default behavior to avoid conflicts

**How It Works:**
- When launcher window is open, pressing an app's shortcut launches it directly
- Works with any modifier combination: Ctrl, Alt, Shift, Cmd
- Examples: `CommandOrControl+F`, `Alt+1`, `CommandOrControl+Shift+A`

**Files Modified:**
- `dist/app.js` - Added shortcut matching in global keydown handler

**How to Test:**
1. Run the app and add/edit an app
2. Set a shortcut like Control+F (it will show as "CommandOrControl+F")
3. Click Save
4. Open the launcher window (using the global shortcut)
5. Press Control+F
   - ✅ The app should launch immediately
6. Try with other shortcuts like Alt+1, Cmd+Shift+A, etc.
   - ✅ All should work

**Note:** These are "local" shortcuts that only work when the launcher window is open. To make shortcuts work globally (even when launcher is closed), you would need to register them as system-wide global shortcuts in the backend, which requires additional implementation.

---

### Issue 5: Global Shortcut Changes Require App Restart ✅ FIXED

**Problem:** When changing the global shortcut in settings, the change was only saved to the database but not applied to the system. You had to restart the app for the new shortcut to work.

**Root Cause:** The `update_global_shortcut` function existed in `shortcut_manager.rs` but was not exposed as a Tauri command, so the frontend couldn't call it to re-register the shortcut immediately.

**Solution:**
1. Added new Tauri command `update_global_shortcut` that calls the backend shortcut manager
2. Registered the command in `main.rs`
3. Frontend now calls this command after saving the shortcut to database
4. The backend unregisters the old shortcut and registers the new one immediately

**Files Modified:**
- `src-tauri/src/commands.rs` - Added `update_global_shortcut` command
- `src-tauri/src/main.rs` - Registered the new command
- `dist/app.js` - Call backend command after saving shortcut

**How to Test:**
1. Run the app
2. Open Settings (⚙️) and click "Record" next to Global Shortcut
3. Press a new shortcut (e.g., Cmd+Shift+L)
4. Click Save
5. Close the launcher window
6. Press your new shortcut immediately
   - ✅ Launcher should open with the new shortcut (no restart needed)

---

### Issue 6: Shortcut Fields Should Be Read-Only ✅ FIXED

**Problem:** Users could manually type into shortcut input fields, which could lead to incorrect format and shortcuts not working.

**Requested Feature:** Shortcut fields should only be editable via the "Record" button.

**Solution:**
1. Added `readonly` attribute to all shortcut input fields
2. Added CSS styling for readonly inputs:
   - Slightly dimmed appearance to indicate they're not directly editable
   - Remove focus highlight when clicked
   - Cursor shows as default, not text insertion

**Files Modified:**
- `dist/app.js` - Added `readonly` to shortcut inputs in settings and app modals
- `dist/styles.css` - Added styling for `input[readonly]`

**How to Test:**
1. Open Settings or Add/Edit App modal
2. Try clicking in the shortcut input field and typing
   - ✅ Should not be editable by typing
3. Click the "Record" button and press keys
   - ✅ Should still work perfectly
4. Visual feedback: readonly field looks slightly different (dimmed)

---

### Issue 7: Icon Loading Error - Unsupported URL ✅ FIXED

**Problem:** When adding an app without manually selecting an icon (letting the system extract it from the binary), the icon wouldn't load and showed this error:
```
Failed to load resource: unsupported URL
asset://localhost/Users/join3r/Library/Application%20Support/com.applauncher.dev/icons/Slack.png
```

**Root Cause:** Two issues:
1. The `asset://` protocol is only for bundled assets, not arbitrary filesystem paths
2. Tauri wasn't configured to allow serving files from the app data directory

The icons are stored in the app's data directory (`$APPDATA`), which requires:
- Using Tauri's `convertFileSrc()` API to create proper URLs
- Configuring the asset protocol scope to allow `$APPDATA` access

**Solution:**
1. Added `convertFileSrc()` wrapper function from Tauri API
2. Updated `toAssetUrl()` to use `convertFileSrc()` instead of manually constructing URLs
3. Configured `assetProtocol` in `tauri.conf.json`:
   - Enabled the asset protocol
   - Added `$APPDATA/**` to allowed scopes
   - Added `core:path:default` permission

**Files Modified:**
- `dist/app.js` - Added `convertFileSrc()` wrapper and updated `toAssetUrl()`
- `src-tauri/tauri.conf.json` - Configured asset protocol scope

**How It Works:**
```javascript
// Before:
img.src = `asset://localhost/${app.icon_path}`;  // ❌ Wrong protocol

// After:
img.src = convertFileSrc(app.icon_path);  // ✅ Proper Tauri conversion
// Produces: https://asset.localhost/[hash]/Users/join3r/Library/...
```

**How to Test:**
1. Add a new application (like iTerm, VS Code, Chrome)
2. Click "Browse" for Binary Path and select the app
3. Don't manually select an icon - let it auto-extract
   - ✅ Icon should display correctly in the grid
   - ✅ No console errors about "unsupported URL"
4. Right-click and edit the app
   - ✅ Icon preview should display correctly in the modal

---

### Issue 8: Shortcuts Interfere with Text Input ✅ FIXED

**Problem:** App shortcuts were triggering even when typing in input fields. For example, if you set shortcut "S" for Slack, you couldn't type the letter "s" when naming an application - it would launch Slack and close the modal.

**Root Cause:** The global keyboard listener was processing all keydown events without checking if the user was actively typing in an input field, textarea, or select element.

**Solution:**
Added input field detection before processing shortcuts:
1. Check if `document.activeElement` is an input/textarea/select/contentEditable element
2. If user is typing, skip shortcut processing (except Escape key)
3. Only process app shortcuts when user is NOT in a text input

**Logic:**
```javascript
// Check if user is currently typing
const activeElement = document.activeElement;
const isTyping = activeElement && (
    activeElement.tagName === 'INPUT' ||
    activeElement.tagName === 'TEXTAREA' ||
    activeElement.tagName === 'SELECT' ||
    activeElement.isContentEditable
);

// Skip shortcut processing if typing
if (isTyping && e.key !== 'Escape') {
    return;
}
```

**Files Modified:**
- `dist/app.js` - Added input field detection to keyboard handler

**How to Test:**
1. Set a simple shortcut like "S" for Slack
2. Open Settings or Add Application modal
3. Type in the "Name" field and include the letter "s"
   - ✅ Should type normally without triggering Slack
4. Close the modal and press "S" while NOT in any input
   - ✅ Should launch Slack

**Special Cases:**
- **Escape key**: Still works even when typing (to close modals)
- **Recording mode**: Still captures all keys for shortcut recording
- **Grid navigation**: Arrow keys only work when app grid is focused, not when typing

---

### Issue 9: Close Button Should Hide Window, Not Quit App ✅ FIXED

**Problem:** Clicking the close button (X) on the window would quit the entire application. For a launcher app that should run in the background, this is incorrect behavior.

**Expected Behavior:** The close button should hide the window (like pressing Escape), keeping the app running in the background so you can reopen it with the global shortcut.

**Root Cause:** Tauri's default behavior is to quit the application when the last window is closed.

**Solution:**
Added a window event handler that intercepts the close request:
1. Listen for `CloseRequested` window events
2. Prevent the default close behavior with `api.prevent_close()`
3. Hide the window instead using `window.hide()`

**Implementation:**
```rust
window.on_window_event(move |event| {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();  // Don't quit
        let _ = window_clone.hide();  // Hide instead
    }
});
```

**Files Modified:**
- `src-tauri/src/main.rs` - Added close event handler

**How to Test:**
1. Start the app and open the launcher window
2. Click the close button (X) or red dot on macOS
   - ✅ Window should hide (not quit)
3. Press the global shortcut (e.g., Cmd+Shift+Space)
   - ✅ Window should reappear
4. The app should still be running in the background

**Quitting the App:**
Since the close button now only hides the window, to actually quit the app:
- **macOS**: Cmd+Q or right-click dock icon → Quit
- **Windows/Linux**: System tray icon → Quit (if implemented) or Task Manager

---

## Summary

All issues are now resolved:

1. **Theme Switching**: Works correctly for Light, Dark, and System themes
2. **Shortcut Recording**: Interactive recording with visual feedback for both global and per-app shortcuts
3. **Option/Alt Key Handling**: Correctly captures keyboard shortcuts with Option/Alt keys without special characters
4. **Per-App Shortcuts**: App shortcuts now work when launcher window is open
5. **Global Shortcut Live Update**: Changes to global shortcut apply immediately without restart
6. **Read-Only Shortcut Fields**: Shortcut inputs only editable via Record button
7. **Icon Loading**: Auto-extracted icons from binaries now load correctly
8. **Input Field Protection**: Shortcuts don't interfere with typing in text fields
9. **Close Button Behavior**: Close button hides window instead of quitting app

## Testing Commands

Start the development server:
```bash
./dev.sh
```

Or with Docker:
```bash
docker compose up -d --build
```

The app will open automatically. Test both features to verify they work as expected.

