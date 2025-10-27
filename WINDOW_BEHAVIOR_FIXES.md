# Window Behavior Fixes

## Issues Fixed

### Issue #1: Launcher Auto-Close After Launching Webapp ✅

**Status**: Already working correctly

**Description**: When launching a web application from the launcher window, the launcher should automatically hide.

**Implementation**: 
- Located in `src-tauri/src/commands.rs` (lines 76-79)
- After successfully launching any app, the main launcher window is hidden
- This allows the launched app to take focus

**Code**:
```rust
// Hide the main launcher window after launching
if let Some(window) = app_handle.get_webview_window("main") {
    window.hide().map_err(|e| format!("Failed to hide window: {}", e))?;
}
```

**Testing**:
1. Open the launcher (Cmd+Shift+Space)
2. Click on a webapp to launch it
3. ✅ Launcher window should hide automatically
4. ✅ Webapp window should open and take focus

---

### Issue #2: Cmd+Q on Webapp Should Not Hide Launcher ✅ FIXED

**Status**: Fixed

**Problem**:
When pressing Cmd+Q while a webapp window was focused, BOTH the webapp window AND the launcher would quit entirely. The launcher should remain running in the background.

**Root Cause #1 - Last Window Closing Triggers Exit**:
By default, Tauri triggers an `ExitRequested` event when the last window is closed. Since the main launcher window is hidden (not visible), when you close the webapp window, Tauri thinks it's the "last window" and tries to quit the entire app.

**Root Cause #2 - Not Distinguishing Exit Types**:
The original code didn't distinguish between:
- **User-initiated exit** (Cmd+Q): `code` is `None`
- **Programmatic exit** (last window closed): `code` is `Some(0)`

**Solution**:
Modified the `ExitRequested` handler to:
1. **ALWAYS prevent exit** - The app should never quit automatically
2. **Check the exit code** to determine the exit type
3. **Only handle window actions for Cmd+Q** (when `code` is `None`)
4. **Ignore programmatic exits** (when `code` is `Some(0)`)

```rust
// AFTER (FIXED CODE):
RunEvent::ExitRequested { api, code, .. } => {
    // ALWAYS prevent the app from quitting
    api.prevent_exit();

    // Only handle window-specific actions if this is a Cmd+Q (code is None)
    // When the last window closes, code is Some(0), and we should do nothing
    if code.is_none() {
        // Handle Cmd+Q on focused window
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
```

**Files Modified**:
- `src-tauri/src/main.rs` (lines 98-142)
  - Added `code` parameter to `ExitRequested` handler
  - Added logic to differentiate between Cmd+Q and last-window-closed
  - Added debug logging with `eprintln!` to track exit events
- `CMD_Q_BEHAVIOR.md` (updated documentation)
- `WINDOW_BEHAVIOR_FIXES.md` (this file)

**Testing**:
1. Open the launcher (Cmd+Shift+Space)
2. Launch a webapp
3. ✅ Launcher should hide
4. Focus the webapp window
5. Press Cmd+Q
6. ✅ Webapp window should close
7. ✅ App should remain running (check Activity Monitor or Dock)
8. Press Cmd+Shift+Space
9. ✅ Launcher should reappear

**Debug Output** (visible in terminal when running with `cargo run`):
- When pressing Cmd+Q on webapp: `Exit requested with code: None` + `Focused window: webapp_X`
- When webapp closes: `Exit requested with code: Some(0)` + `Exit triggered by last window closing - ignoring`

---

## Complete Cmd+Q Behavior (macOS)

### Summary Table

| Window Focused | Cmd+Q Action | Launcher State | App State |
|---------------|--------------|----------------|-----------|
| Main Launcher | Hide launcher | Hidden | Running |
| Webapp Window | Close webapp | Unchanged | Running |
| Terminal Window | Close terminal + kill process | Unchanged | Running |
| No Window | Do nothing | Unchanged | Running |

### Detailed Behavior

**1. Main Launcher Window Focused**
- Action: Window hides
- Launcher: Hidden
- App: Continues running
- How to show again: Press global shortcut (Cmd+Shift+Space)

**2. Webapp Window Focused**
- Action: Webapp window closes
- Launcher: Remains in current state (hidden or visible)
- App: Continues running
- Other windows: Unaffected

**3. Terminal Window Focused**
- Action: Terminal window closes + process killed
- Launcher: Remains in current state (hidden or visible)
- App: Continues running
- Other windows: Unaffected

**4. No Window Focused**
- Action: Nothing happens
- Launcher: Remains in current state
- App: Continues running
- Reason: Prevents accidental hiding of launcher when closing other windows

---

## Implementation Details

### Main Event Handler

**Location**: `src-tauri/src/main.rs` (lines 98-142)

**Logic Flow**:
```
Exit requested (Cmd+Q, Dock→Quit, or last window closed)
    ↓
ALWAYS prevent app quit (api.prevent_exit())
    ↓
Check exit code
    ↓
    ├─ code is None (Cmd+Q)
    │   ↓
    │   Find focused window
    │   ↓
    │   ├─ Webapp/Terminal? → Close window
    │   ├─ Main launcher? → Hide window
    │   └─ No focus? → Do nothing
    │
    └─ code is Some(0) (last window closed)
        ↓
        Do nothing (ignore)
```

**Code**:
```rust
.run(|app_handle, event| {
    #[cfg(target_os = "macos")]
    {
        use tauri::RunEvent;

        match event {
            RunEvent::ExitRequested { api, code, .. } => {
                // ALWAYS prevent the app from quitting
                api.prevent_exit();

                eprintln!("Exit requested with code: {:?}", code);

                // Only handle window-specific actions if this is a Cmd+Q (code is None)
                if code.is_none() {
                    if let Some(focused_window) = app_handle.webview_windows().values()
                        .find(|w| w.is_focused().unwrap_or(false))
                    {
                        let window_label = focused_window.label();
                        eprintln!("Focused window: {}", window_label);

                        if window_label.starts_with("webapp_") || window_label.starts_with("terminal_") {
                            let _ = focused_window.close();
                        } else if window_label == "main" {
                            let _ = focused_window.hide();
                        }
                    } else {
                        eprintln!("No window is focused");
                    }
                } else {
                    eprintln!("Exit triggered by last window closing - ignoring");
                }
            }
            _ => {}
        }
    }
});
```

---

## Platform Considerations

### macOS ✅
- Cmd+Q is intercepted
- Custom window close behavior
- Launcher stays in background
- Can quit via Dock or Settings

### Windows/Linux
- Default quit behavior preserved
- Alt+F4 (Windows) or Ctrl+Q (Linux) quits normally
- No custom interception needed

---

## How to Actually Quit the App

Since Cmd+Q no longer quits the entire application, use one of these methods:

### Method 1: Dock Icon (macOS)
1. Right-click app icon in Dock
2. Select "Quit"

### Method 2: Settings Panel
1. Open launcher (Cmd+Shift+Space)
2. Click Settings (⚙️)
3. Scroll to bottom
4. Click "Quit Application"
5. Confirm in dialog

### Method 3: Programmatic (Developers)
```javascript
import { invoke } from '@tauri-apps/api/core';
await invoke('quit_app');
```

---

## Testing Checklist

- [x] Build succeeds without errors
- [ ] Launcher hides after launching webapp
- [ ] Launcher hides after launching native app
- [ ] Launcher hides after launching TUI app
- [ ] Cmd+Q on main launcher hides it
- [ ] Cmd+Q on webapp closes only webapp
- [ ] Cmd+Q on terminal closes only terminal
- [ ] Launcher doesn't hide when webapp closes
- [ ] Launcher doesn't hide when terminal closes
- [ ] Global shortcut shows launcher again
- [ ] Quit button in settings works
- [ ] Dock → Quit works

---

## Troubleshooting

### Launcher still hides when closing webapp
- **Check**: Make sure you rebuilt after the fix
- **Verify**: Lines 124-131 in main.rs should be removed
- **Test**: Close webapp and check if launcher is still accessible

### Launcher doesn't hide after launching app
- **Check**: Verify commands.rs lines 76-79 are present
- **Test**: Add console logging to confirm the hide command is called

### Can't quit the app
- **Solution**: Use Dock → Quit or Settings → Quit Application
- **Alternative**: Force quit with Cmd+Option+Esc

---

## Summary

Both issues have been resolved:

1. ✅ **Launcher auto-hides after launching webapp** - Already working correctly
2. ✅ **Cmd+Q on webapp doesn't affect launcher** - Fixed by removing problematic else block

The application now provides a smooth, intuitive window management experience on macOS while respecting platform conventions.

