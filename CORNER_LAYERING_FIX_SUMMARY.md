# Window Corner Layering Fix - Summary

## Date: 2025-11-04

## Issue Reported
Visual rendering issue with the launcher window where corners/edges appeared to have layering problems. Two overlapping elements with different border-radius values created visible artifacts at the window corners.

## Investigation Results

### Root Cause Identified
Mismatch between two layered elements with different corner rounding:

1. **Native macOS Window Effect Layer** (Tauri/Rust backend)
   - File: `src-tauri/src/main.rs`
   - Line: 97
   - Radius: `12.0` pixels
   - Applied via: `window.set_effects()` with `Effect::HudWindow`

2. **CSS Container Layer** (Frontend)
   - File: `dist/styles.css`
   - Line: 154 (before fix)
   - Radius: `22px`
   - Applied to: `.platform-macos .app-container`

### Visual Problem
The native window effect (12px radius) created a blurred vibrancy background, while the CSS container (22px radius) sat on top with different corner rounding. This caused both corner radii to be visible simultaneously, creating an unsightly double-layer effect.

## Changes Made

### 1. Fixed Border Radius Mismatch
**File:** `dist/styles.css`
**Line:** 154

**Before:**
```css
.platform-macos .app-container {
    background: var(--bg-primary);
    border: none;
    border-radius: 22px;
    /* ... */
}
```

**After:**
```css
.platform-macos .app-container {
    background: var(--bg-primary);
    border: none;
    border-radius: 12px; /* Match native window effect radius */
    /* ... */
}
```

### 2. Cleaned Up Duplicate CSS Property
**File:** `dist/styles.css`
**Lines:** 135-148

**Before:**
```css
.app-container {
    /* ... */
    position: relative;
    position: relative;  /* Duplicate */
    /* ... */
}
```

**After:**
```css
.app-container {
    /* ... */
    position: relative;
    /* ... */
}
```

## Technical Rationale

### Why 12px?
1. **Native macOS Standard**: 12px is the standard corner radius for macOS HUD windows
2. **Exact Match**: Matches the Tauri window effect radius precisely
3. **Visual Consistency**: Ensures CSS and native layers have identical corner rounding
4. **Eliminates Artifacts**: Removes the visible layering issue completely

### Architecture Understanding
```
┌─────────────────────────────────────────┐
│  Native macOS Window (Tauri)            │
│  - Transparent window                   │
│  - HudWindow effect with 12px radius    │
│  - Vibrancy/blur background             │
│  └─────────────────────────────────────┐│
│    │  CSS .app-container               ││
│    │  - Must match 12px radius         ││
│    │  - Semi-transparent background    ││
│    │  - Backdrop filter blur           ││
│    └───────────────────────────────────┘│
└─────────────────────────────────────────┘
```

## Verification Checklist

After applying this fix, verify:
- ✅ Window corners appear clean and uniform
- ✅ No visible layering artifacts at edges
- ✅ Consistent rounded corners throughout
- ✅ Native macOS appearance maintained
- ✅ Liquid glass effect still working
- ✅ No content bleeding outside corners

## Testing Instructions

1. **Launch the application:**
   ```bash
   bun run tauri dev
   ```

2. **Open the launcher window** (via global shortcut or system tray)

3. **Inspect the corners** - Look for:
   - Clean, single-layer rounded corners
   - No visible double-rounding effect
   - Consistent corner radius all around
   - Proper blur/vibrancy effect

4. **Test in different modes:**
   - Light mode
   - Dark mode
   - Different screen positions
   - Different window sizes (if resizable)

## Platform Impact

- **macOS**: Fixed (primary target of this fix)
- **Windows**: Unaffected (uses different styling)
- **Linux**: Unaffected (uses different styling)

## Related Files

### Modified
- `dist/styles.css` - Fixed border-radius and removed duplicate property

### Referenced (No Changes)
- `src-tauri/src/main.rs` - Contains native window effect configuration
- `src-tauri/tauri.conf.json` - Window transparency settings
- `dist/index.html` - HTML structure

## Additional Notes

### Other Border Radius Values in Codebase
The following elements have different border-radius values, which is correct:
- `.platform-macos .icon-item`: 16px (internal UI element)
- Icon images: 14px (internal UI element)
- Context menus: 12px (matches window)
- Modals: 12px (matches window)

These don't cause layering issues because they're internal to the window and don't interact with the window chrome.

### Future Considerations

If the native window effect radius is ever changed in `main.rs`, the CSS must be updated to match:

```rust
// main.rs
radius: Some(12.0),  // ← If this changes...
```

```css
/* styles.css */
border-radius: 12px; /* ← This must change too */
```

Consider creating a constant or configuration value that's shared between Rust and CSS to prevent future mismatches.

## Documentation Created

- `WINDOW_CORNER_FIX.md` - Detailed technical documentation
- `CORNER_LAYERING_FIX_SUMMARY.md` - This summary document

