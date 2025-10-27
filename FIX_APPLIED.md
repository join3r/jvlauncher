# Fix Applied: Window Disappearing Issue

## Problem
The window was disappearing immediately when releasing the keyboard shortcut `Cmd+Shift+Space`.

## Root Cause
The global shortcut handler was firing on **BOTH** key press and key release events:
1. Press `Cmd+Shift+Space` → Handler fires → Window shows
2. Release `Cmd+Shift+Space` → Handler fires **AGAIN** → Window hides (toggle)

## Solution
Modified `src-tauri/src/shortcut_manager.rs` to only respond to key **PRESS** events, ignoring key **RELEASE** events.

**Code change:**
```rust
// Before: Handler fired on all events
app_handle.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
    // Always toggled, regardless of press/release
    ...
});

// After: Handler only fires on key PRESS
app_handle.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
    if event.state() == ShortcutState::Pressed {
        // Only toggle on key PRESS, not RELEASE
        ...
    }
});
```

## Test Now

```bash
./dev.sh
```

**Expected behavior:**
1. Press `Cmd+Shift+Space` → Window appears ✅
2. **Release the keys** → Window **stays visible** ✅
3. You can now:
   - Click buttons ("+", "⚙️")
   - Use keyboard navigation
   - Interact with the UI
4. Press `Cmd+Shift+Space` **again** → Window hides ✅
5. Or press `Escape` → Window hides ✅

## Changes Made

**Files modified:**
- ✅ `src-tauri/src/shortcut_manager.rs` - Filter for key press events only
- ✅ `src-tauri/src/main.rs` - Removed auto-hide on blur
- ✅ `src-tauri/src/database.rs` - Changed default shortcut to Cmd+Shift+Space
- ✅ `dist/app.js` - Fixed button initialization
- ✅ `dist/index.html` - Fixed script loading

## How It Works Now

**Window visibility logic:**
- Shortcut pressed (once) → Toggle window on
- Window stays visible until:
  - Shortcut pressed again → Toggle window off
  - Escape key → Hide window
  - Launch an app → Hide window (after launching)

**No more:**
- ❌ Auto-hide when releasing shortcut
- ❌ Auto-hide when losing focus
- ❌ Unexpected disappearing

This matches the behavior of professional launchers like Alfred and Raycast! 🎉

