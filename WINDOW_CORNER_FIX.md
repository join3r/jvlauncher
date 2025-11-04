# Window Corner Layering Fix

## Issue Description

The launcher window had a visual rendering issue where the corners/edges appeared to have layering problems. Specifically, at the window corners, there were two overlapping elements with different border-radius values creating a visual artifact where both layers were visible.

## Root Cause

The issue was caused by **mismatched border-radius values** between two layers:

1. **Native macOS window effect** (Tauri backend): `radius: 12.0` pixels
   - Location: `src-tauri/src/main.rs` line 97
   - Applied via `window.set_effects()` with `Effect::HudWindow`

2. **CSS app-container** (Frontend): `border-radius: 22px`
   - Location: `dist/styles.css` line 154
   - Applied to `.platform-macos .app-container`

The native window effect creates a blurred, vibrancy background with 12px rounded corners, while the CSS container on top had 22px rounded corners. This mismatch caused both corner radii to be visible, creating an unsightly layering artifact.

## Solution

Changed the CSS border-radius to match the native window effect radius:

**File: `dist/styles.css`**

```css
/* macOS Liquid Glass Effect */
.platform-macos .app-container {
    background: var(--bg-primary);
    border: none; /* Remove border for cleaner look */
    border-radius: 12px; /* Match native window effect radius */
    -webkit-backdrop-filter: saturate(180%) blur(var(--blur-amount));
    backdrop-filter: saturate(180%) blur(var(--blur-amount));
    box-shadow:
        0 0 0 1px rgba(255, 255, 255, 0.18) inset,
        0 0 0 1px rgba(0, 0, 0, 0.1),
        0 20px 60px rgba(0, 0, 0, 0.25);
}
```

**Changed:** `border-radius: 22px` → `border-radius: 12px`

## Why 12px?

- **Native macOS standard**: 12px is the standard corner radius for macOS HUD windows
- **Consistency**: Matches the Tauri window effect radius exactly
- **Visual harmony**: Ensures the CSS layer and native window effect layer have identical corner rounding
- **No artifacts**: Eliminates the visible layering issue at the corners

## Technical Details

### Tauri Window Effects
The native window effect is applied in the Rust backend:

```rust
#[cfg(target_os = "macos")]
{
    use tauri::window::Effect;
    use tauri::window::EffectState;

    let _ = window.set_effects(tauri::utils::config::WindowEffectsConfig {
        effects: vec![Effect::HudWindow],
        state: Some(EffectState::FollowsWindowActiveState),
        radius: Some(12.0),  // ← This must match CSS border-radius
        color: None,
    });
}
```

### CSS Styling
The CSS must match this radius to prevent layering artifacts:

```css
.platform-macos .app-container {
    border-radius: 12px; /* Must match Tauri window effect radius */
    /* ... other styles ... */
}
```

## Verification

After applying this fix:
- ✅ Window corners appear clean and uniform
- ✅ No visible layering artifacts
- ✅ Consistent rounded corners throughout
- ✅ Native macOS appearance maintained

## Related Files

- `src-tauri/src/main.rs` - Native window effect configuration
- `dist/styles.css` - CSS styling for app container
- `src-tauri/tauri.conf.json` - Window transparency and decoration settings

## Notes

- This fix only affects macOS builds (`.platform-macos` class)
- Windows/Linux builds use different styling and are unaffected
- The 12px radius is consistent with macOS HUD window standards
- Other internal elements (icon items, etc.) can have different border-radius values as they don't interact with the window chrome

