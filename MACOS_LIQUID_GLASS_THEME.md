# macOS Liquid Glass Theme Implementation

## Overview

This document describes the implementation of a platform-specific "liquid glass" (glassmorphism) theme for macOS, while maintaining the Material Design theme for Windows and Linux platforms.

## What Was Implemented

### 1. Platform Detection

**File: `dist/app.js`**

Added automatic platform detection that runs on application startup:

```javascript
// Platform detection
let isMacOS = false;

async function detectPlatform() {
    const platform = await tauri.os.platform();
    isMacOS = platform === 'macos';
    
    // Apply platform-specific class to root element
    if (isMacOS) {
        document.documentElement.classList.add('platform-macos');
    } else {
        document.documentElement.classList.add('platform-other');
    }
}
```

This function is called during app initialization and adds a CSS class to the root `<html>` element, enabling platform-specific styling.

### 2. Window Configuration

**File: `src-tauri/tauri.conf.json`**

Updated the main window configuration to support transparency and native macOS styling:

```json
{
  "transparent": true,
  "titleBarStyle": "Overlay"
}
```

- `transparent: true` - Enables window transparency for glassmorphism effects
- `titleBarStyle: "Overlay"` - Uses native macOS title bar styling

### 3. Native macOS Window Effects

**File: `src-tauri/src/main.rs`**

Added native macOS vibrancy effects using Tauri's window effects API:

```rust
#[cfg(target_os = "macos")]
{
    use tauri::window::Effect;
    use tauri::window::EffectState;
    
    let _ = window.set_effects(tauri::utils::config::WindowEffectsConfig {
        effects: vec![Effect::HudWindow],
        state: Some(EffectState::FollowsWindowActiveState),
        radius: Some(12.0),
        color: None,
    });
}
```

This applies the `HudWindow` effect which provides:
- Native macOS vibrancy
- Backdrop blur that follows window active state
- Smooth integration with macOS system UI

### 4. CSS Liquid Glass Theme

**File: `dist/styles.css`**

Implemented comprehensive platform-specific theming:

#### CSS Variables for macOS (Light Mode)
```css
:root.platform-macos {
    --accent: #007aff; /* macOS system blue */
    --bg-primary: rgba(255, 255, 255, 0.65);
    --bg-secondary: rgba(255, 255, 255, 0.75);
    --bg-tertiary: rgba(255, 255, 255, 0.55);
    --text-primary: #1d1d1f;
    --text-secondary: #6e6e73;
    --border-color: rgba(0, 0, 0, 0.06);
    --blur-amount: 40px;
}
```

#### CSS Variables for macOS (Dark Mode)
```css
:root.platform-macos[data-theme="dark"] {
    --accent: #0a84ff;
    --bg-primary: rgba(30, 30, 30, 0.7);
    --bg-secondary: rgba(40, 40, 40, 0.8);
    --bg-tertiary: rgba(25, 25, 25, 0.65);
    --text-primary: #f5f5f7;
    --text-secondary: #a1a1a6;
    --border-color: rgba(255, 255, 255, 0.08);
    --blur-amount: 60px;
}
```

#### Glassmorphism Effects

**Main Container:**
```css
.platform-macos .app-container {
    background: var(--bg-primary);
    border-radius: 16px;
    -webkit-backdrop-filter: saturate(180%) blur(var(--blur-amount));
    backdrop-filter: saturate(180%) blur(var(--blur-amount));
    box-shadow: 
        0 0 0 0.5px rgba(255, 255, 255, 0.1) inset,
        0 20px 60px rgba(0, 0, 0, 0.3),
        0 0 1px rgba(0, 0, 0, 0.2);
}
```

**Icon Items:**
```css
.platform-macos .icon-item {
    background: var(--bg-secondary);
    border-radius: 12px;
    -webkit-backdrop-filter: saturate(180%) blur(20px);
    backdrop-filter: saturate(180%) blur(20px);
    box-shadow: 
        0 0 0 0.5px rgba(255, 255, 255, 0.1) inset,
        var(--shadow-sm);
}
```

**Modals and Dialogs:**
```css
.platform-macos .modal-content {
    background: var(--bg-secondary);
    border-radius: 16px;
    -webkit-backdrop-filter: saturate(180%) blur(60px);
    backdrop-filter: saturate(180%) blur(60px);
    box-shadow: 
        0 0 0 0.5px rgba(255, 255, 255, 0.1) inset,
        0 20px 60px rgba(0, 0, 0, 0.4);
}
```

### 5. Material Design Theme (Windows/Linux)

The Material Design theme is preserved for Windows and Linux:

```css
:root.platform-other {
    --accent: #2196f3; /* Material blue */
    --bg-primary: #ffffff;
    --bg-secondary: #f5f5f5;
    --bg-tertiary: #e0e0e0;
    --text-primary: #212121;
    --text-secondary: #757575;
    --border-color: rgba(0, 0, 0, 0.12);
}
```

## Key Features

### 1. Translucent/Frosted Glass Effects
- Semi-transparent backgrounds with backdrop blur
- Content behind windows shows through with blur effect
- Saturation boost (180%) for enhanced vibrancy

### 2. Native macOS Visual Styling
- Uses macOS system colors (#007aff for light, #0a84ff for dark)
- Follows macOS Human Interface Guidelines
- Proper window chrome with native title bar
- Smooth animations with cubic-bezier easing

### 3. Platform Detection
- Automatic detection on app startup
- CSS classes applied to root element
- No manual configuration required

### 4. Vibrancy Effects
- Native `HudWindow` effect on macOS
- Backdrop blur that follows window active state
- Proper integration with macOS system UI

### 5. Light and Dark Mode Support
- Automatic system theme detection
- Manual theme override support
- Smooth transitions between themes

## Visual Characteristics

### macOS Liquid Glass Theme
- **Transparency:** 65-75% opacity for backgrounds
- **Blur:** 40-60px backdrop blur
- **Borders:** Subtle inset borders with white/black tints
- **Shadows:** Multi-layered shadows for depth
- **Corners:** 12-16px border radius
- **Colors:** macOS system colors

### Material Design Theme (Windows/Linux)
- **Transparency:** Solid backgrounds
- **Blur:** No blur effects
- **Borders:** Standard Material borders
- **Shadows:** Material elevation shadows
- **Corners:** 8px border radius
- **Colors:** Material Design palette

## Files Modified

1. **src-tauri/tauri.conf.json** - Window transparency configuration
2. **src-tauri/src/main.rs** - Native macOS window effects
3. **dist/app.js** - Platform detection logic
4. **dist/styles.css** - Platform-specific theming

## Testing

To test the implementation:

1. **On macOS:**
   ```bash
   ./dev.sh
   ```
   - Verify liquid glass effect with backdrop blur
   - Test light and dark mode switching
   - Check that content behind window is visible with blur

2. **On Windows/Linux:**
   ```bash
   ./dev.sh
   ```
   - Verify Material Design theme is applied
   - Ensure no transparency or blur effects
   - Test theme switching

## Browser Compatibility

The implementation uses:
- `-webkit-backdrop-filter` for Safari/WebKit
- `backdrop-filter` for modern browsers
- Graceful degradation for unsupported browsers

## Performance Considerations

- Backdrop blur is GPU-accelerated on macOS
- Native window effects use system APIs
- CSS animations use hardware acceleration
- No performance impact on Windows/Linux

## Future Enhancements

Potential improvements:
- [ ] Add vibrancy material options (light, dark, ultra-dark)
- [ ] Support for reduced transparency accessibility setting
- [ ] Custom blur intensity settings
- [ ] Windows 11 Acrylic/Mica effects
- [ ] Linux compositor-specific effects

## References

- [macOS Human Interface Guidelines - Materials](https://developer.apple.com/design/human-interface-guidelines/materials)
- [Tauri Window Effects API](https://tauri.app/v1/api/js/window/#seteffects)
- [CSS backdrop-filter](https://developer.mozilla.org/en-US/docs/Web/CSS/backdrop-filter)
- [Glassmorphism Design Trend](https://uxdesign.cc/glassmorphism-in-user-interfaces-1f39bb1308c9)

