# Keyboard Shortcut Recording Bug Fix

## Problem Description

When attempting to record complex keyboard shortcuts with multiple modifiers (especially the "hyperkey" combination: Command + Option + Control + Shift), the shortcut recorder had two critical bugs:

### Bug 1: Missing Shift Modifier and Premature Recording Stop
- **Issue**: When pressing Command + Option + Control + Shift (without a final key), the recorder would stop immediately
- **Cause**: In `settings.js` and `app-form.js`, the condition `shortcut.includes('+')` would return true even when only modifiers were pressed, causing premature recording termination
- **Result**: The recorder never waited for the final key (like "R") to be pressed

### Bug 2: Incomplete Hyperkey Support
- **Issue**: When both Command AND Control were pressed simultaneously (hyperkey scenario on macOS), only "CommandOrControl" was recorded
- **Cause**: The logic used `event.ctrlKey || event.metaKey` which only checked if EITHER was pressed, not both
- **Result**: The full hyperkey combination was not captured correctly

## Example of the Bug

**User Action**: Press Command + Option + Control + Shift, then press "R"

**Expected Result**: `Command+Control+Alt+Shift+R`

**Actual Result (Before Fix)**: `CommandOrControl+Alt` (recording stopped prematurely, missing Shift and R)

## Root Cause Analysis

### Issue in `formatShortcut()` Function

The original code in all three files (`app.js`, `settings.js`, `app-form.js`):

```javascript
// Original buggy code
function formatShortcut(event) {
    const parts = [];
    if (event.ctrlKey || event.metaKey) parts.push('CommandOrControl');  // ❌ Only adds once even if both are pressed
    if (event.altKey) parts.push('Alt');
    if (event.shiftKey) parts.push('Shift');
    // ...
}
```

**Problem**: When both `event.ctrlKey` AND `event.metaKey` are true (hyperkey scenario), the `||` operator only adds "CommandOrControl" once, losing information about the full combination.

### Issue in `handleRecordingKeyDown()` Function

The original code in `settings.js` and `app-form.js`:

```javascript
// Original buggy code
function handleRecordingKeyDown(event) {
    // ...
    const shortcut = formatShortcut(event);
    if (shortcut && shortcut.includes('+')) {  // ❌ Stops recording even with only modifiers
        currentRecordingInput.value = shortcut;
        stopRecording();
    }
}
```

**Problem**: When modifiers are pressed (e.g., "CommandOrControl+Alt+Shift"), the condition `shortcut.includes('+')` is true, so recording stops before the user presses a non-modifier key.

## The Fix

### Fix 1: Enhanced `formatShortcut()` Function

Updated code in all three files:

```javascript
function formatShortcut(event) {
    const parts = [];
    
    // Add modifiers - handle the case where both Ctrl and Meta (Command) are pressed
    // This is important for "hyperkey" combinations (Cmd+Ctrl+Alt+Shift)
    if (event.ctrlKey && event.metaKey) {
        // Both Control and Command are pressed (hyperkey scenario on macOS)
        // Add them separately to capture the full combination
        parts.push('Command');
        parts.push('Control');
    } else if (event.ctrlKey || event.metaKey) {
        // Only one of them is pressed - use cross-platform abstraction
        parts.push('CommandOrControl');
    }
    
    if (event.altKey) parts.push('Alt');
    if (event.shiftKey) parts.push('Shift');
    
    // ... rest of the function
}
```

**Benefits**:
- Detects when both Command AND Control are pressed simultaneously
- Adds them as separate modifiers ("Command+Control") for hyperkey scenarios
- Falls back to "CommandOrControl" for normal single-modifier usage (cross-platform compatibility)

### Fix 2: Wait for Non-Modifier Key

Updated code in `settings.js` and `app-form.js`:

```javascript
function handleRecordingKeyDown(event) {
    if (!isRecording || !currentRecordingInput) return;
    
    event.preventDefault();
    event.stopPropagation();
    
    // Only stop recording if a non-modifier key is pressed
    const key = event.key;
    if (key && key !== 'Control' && key !== 'Meta' && key !== 'Alt' && key !== 'Shift') {
        const shortcut = formatShortcut(event);
        if (shortcut) {
            currentRecordingInput.value = shortcut;
            stopRecording();
        }
    }
}
```

**Benefits**:
- Recording only stops when a non-modifier key is pressed
- Allows users to press all modifiers first, then the final key
- Prevents premature recording termination

Note: `app.js` already had the correct logic using an early return for modifier-only keys.

## Files Modified

1. **dist/app.js** - Enhanced `formatShortcut()` function (lines 108-165)
2. **dist/settings.js** - Enhanced `formatShortcut()` function (lines 25-49) and fixed `handleRecordingKeyDown()` (lines 85-101)
3. **dist/app-form.js** - Enhanced `formatShortcut()` function (lines 55-79) and fixed `handleRecordingKeyDown()` (lines 115-131)

## Testing Instructions

### Test Case 1: Hyperkey + Letter
1. Open the app and click Settings (⚙️)
2. Click the "Record" button next to "Global Shortcut"
3. Press and hold: Command + Option + Control + Shift
4. While holding all modifiers, press "R"
5. **Expected**: Input shows `Command+Control+Alt+Shift+R`

### Test Case 2: Standard Shortcut
1. Click the "Record" button
2. Press Command + Shift + K (or Ctrl + Shift + K on Windows/Linux)
3. **Expected**: Input shows `CommandOrControl+Shift+K`

### Test Case 3: Simple Shortcut
1. Click the "Record" button
2. Press Command + A (or Ctrl + A on Windows/Linux)
3. **Expected**: Input shows `CommandOrControl+A`

### Test Case 4: Per-App Shortcut
1. Add or edit an application
2. Click "Record" button next to "Keyboard Shortcut"
3. Press Command + Control + Option + Shift + F
4. **Expected**: Input shows `Command+Control+Alt+Shift+F`

## Technical Notes

### Cross-Platform Compatibility

The fix maintains cross-platform compatibility:
- **Normal usage**: "CommandOrControl" is used when only Cmd (macOS) or Ctrl (Windows/Linux) is pressed
- **Hyperkey usage**: "Command+Control" is used when both are pressed simultaneously (primarily a macOS scenario)

### Tauri Shortcut Format

Tauri's global shortcut plugin uses the following format:
- Modifiers: `CommandOrControl`, `Command`, `Control`, `Alt`, `Shift`
- Keys: Letter keys (A-Z), numbers (0-9), function keys (F1-F12), special keys (Space, Enter, etc.)
- Format: `Modifier+Modifier+Key` (e.g., `CommandOrControl+Shift+A`)

### Why "Command+Control" for Hyperkey?

On macOS, the hyperkey is typically implemented as Command + Option + Control + Shift all pressed together. The fix now properly captures this as:
- `Command+Control+Alt+Shift+R`

This ensures that:
1. All four modifiers are captured
2. The final key is included
3. The shortcut can be properly registered with Tauri's global shortcut system

## Verification

After applying this fix:
- ✅ Hyperkey combinations are properly captured
- ✅ All modifiers (including Shift) are recorded
- ✅ Recording waits for a non-modifier key before stopping
- ✅ Standard shortcuts still work correctly
- ✅ Cross-platform compatibility is maintained

## Related Documentation

- [Tauri Global Shortcut Plugin](https://v2.tauri.app/plugin/global-shortcut/)
- [Keyboard Event API](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent)

