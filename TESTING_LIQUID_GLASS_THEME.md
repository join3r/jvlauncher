# Testing the macOS Liquid Glass Theme

## Quick Test Guide

### Prerequisites
- macOS system (for testing liquid glass theme)
- Windows or Linux system (for testing Material Design theme)

### Running the Application

```bash
./dev.sh
```

Or for production build:
```bash
./build.sh
```

## What to Test

### 1. Platform Detection

**Expected Behavior:**
- On macOS: Console should show "Platform detected: macos (isMacOS: true)"
- On Windows/Linux: Console should show "Platform detected: windows/linux (isMacOS: false)"

**How to Verify:**
1. Open the application
2. Press `Cmd+Shift+Space` (macOS) or `Ctrl+Shift+Space` (Windows/Linux)
3. Open Developer Tools (if available in dev mode)
4. Check console for platform detection message

### 2. macOS Liquid Glass Theme

**Visual Characteristics to Verify:**

#### Main Window
- [ ] Window has translucent background
- [ ] Content behind window is visible with blur effect
- [ ] Window has rounded corners (16px radius)
- [ ] Subtle border with slight glow
- [ ] Multi-layered shadow for depth

#### App Icons/Items
- [ ] Semi-transparent background
- [ ] Backdrop blur effect visible
- [ ] Hover effect: slight lift with increased shadow
- [ ] Selected state: blue glow around item
- [ ] Smooth animations (200ms cubic-bezier)

#### Settings Button (⚙️)
- [ ] Circular button with glass effect
- [ ] Hover: slight scale up (1.05x)
- [ ] Backdrop blur visible through button

#### Context Menu (Right-click on app)
- [ ] Translucent background with strong blur
- [ ] Rounded corners (12px)
- [ ] Subtle inset border
- [ ] Hover: items highlight with glass effect

#### Modals (Settings, Add App, Edit App)
- [ ] Strong backdrop blur (60px)
- [ ] Translucent background
- [ ] Large rounded corners (16px)
- [ ] Multi-layered shadow
- [ ] Form inputs have glass effect

#### Add Button (+)
- [ ] Blue accent color (#007aff)
- [ ] Glow effect around button
- [ ] Hover: scale up with enhanced glow
- [ ] Smooth animation

### 3. Light/Dark Mode Switching

**Test Steps:**
1. Open Settings (⚙️ button)
2. Change theme to "Light"
3. Verify:
   - [ ] Light backgrounds (white with transparency)
   - [ ] Dark text (#1d1d1f)
   - [ ] Proper contrast
   - [ ] Blur effect still visible

4. Change theme to "Dark"
5. Verify:
   - [ ] Dark backgrounds (dark gray with transparency)
   - [ ] Light text (#f5f5f7)
   - [ ] Proper contrast
   - [ ] Enhanced blur effect (60px)

6. Change theme to "System"
7. Verify:
   - [ ] Follows macOS system theme
   - [ ] Changes when system theme changes

### 4. Windows/Linux Material Design Theme

**Visual Characteristics to Verify:**

#### Main Window
- [ ] Solid background (no transparency)
- [ ] No blur effects
- [ ] Standard rounded corners (8px)
- [ ] Material Design shadows
- [ ] Material blue accent (#2196f3)

#### App Icons/Items
- [ ] Solid backgrounds
- [ ] No blur effects
- [ ] Standard Material Design hover states
- [ ] Material elevation shadows

#### All UI Elements
- [ ] No transparency or blur
- [ ] Material Design color palette
- [ ] Standard Material animations
- [ ] Proper contrast ratios

### 5. Performance Testing

**macOS:**
- [ ] Smooth animations (60fps)
- [ ] No lag when moving window
- [ ] Blur updates in real-time
- [ ] Low CPU usage (<5% idle)
- [ ] Low memory usage (~50MB)

**Windows/Linux:**
- [ ] Smooth animations
- [ ] No performance issues
- [ ] Low resource usage

### 6. Accessibility Testing

**macOS:**
- [ ] Text is readable over blurred backgrounds
- [ ] Sufficient contrast in light mode
- [ ] Sufficient contrast in dark mode
- [ ] Focus indicators visible
- [ ] Keyboard navigation works

**Windows/Linux:**
- [ ] All Material Design accessibility standards met
- [ ] Proper contrast ratios
- [ ] Focus indicators visible

## Visual Comparison

### macOS Liquid Glass Theme

**Light Mode:**
- Background: rgba(255, 255, 255, 0.65) with 40px blur
- Text: #1d1d1f (dark)
- Accent: #007aff (macOS blue)
- Borders: Subtle with slight glow
- Shadows: Multi-layered, soft

**Dark Mode:**
- Background: rgba(30, 30, 30, 0.7) with 60px blur
- Text: #f5f5f7 (light)
- Accent: #0a84ff (bright macOS blue)
- Borders: Subtle white tint
- Shadows: Multi-layered, strong

### Material Design Theme (Windows/Linux)

**Light Mode:**
- Background: #ffffff (solid white)
- Text: #212121 (dark)
- Accent: #2196f3 (Material blue)
- Borders: rgba(0, 0, 0, 0.12)
- Shadows: Material elevation

**Dark Mode:**
- Background: #1e1e1e (solid dark)
- Text: #e0e0e0 (light)
- Accent: #2196f3 (Material blue)
- Borders: rgba(255, 255, 255, 0.12)
- Shadows: Material elevation

## Common Issues and Solutions

### Issue: Blur effect not visible on macOS

**Possible Causes:**
1. macOS Private API not enabled
2. Window not set to transparent
3. Browser doesn't support backdrop-filter

**Solutions:**
1. Check `tauri.conf.json` has `"macOSPrivateApi": true`
2. Check `tauri.conf.json` has `"transparent": true`
3. Use Safari/WebKit-based browser

### Issue: Platform detection not working

**Possible Causes:**
1. Tauri OS API not available
2. JavaScript error preventing detection

**Solutions:**
1. Check console for errors
2. Verify `window.__TAURI__.os.platform()` is available
3. Check that `detectPlatform()` is called in `init()`

### Issue: Theme not switching

**Possible Causes:**
1. CSS variables not updating
2. Data attribute not being set
3. CSS specificity issues

**Solutions:**
1. Check `applyTheme()` function is called
2. Verify `data-theme` attribute on `<html>` element
3. Check browser DevTools for CSS conflicts

### Issue: Performance problems on macOS

**Possible Causes:**
1. Too much blur
2. Too many layers with blur
3. GPU not being used

**Solutions:**
1. Reduce blur amount in CSS variables
2. Limit blur to main container only
3. Check GPU acceleration is enabled

## Automated Testing Checklist

- [ ] Platform detection works correctly
- [ ] CSS classes applied to root element
- [ ] Theme switching works in all modes
- [ ] All UI elements styled correctly
- [ ] No console errors
- [ ] No visual glitches
- [ ] Performance is acceptable
- [ ] Accessibility standards met

## Manual Testing Checklist

### macOS
- [ ] Launch application
- [ ] Verify liquid glass effect
- [ ] Test light mode
- [ ] Test dark mode
- [ ] Test system mode
- [ ] Test all UI interactions
- [ ] Test window movement
- [ ] Test with different backgrounds
- [ ] Test accessibility features

### Windows/Linux
- [ ] Launch application
- [ ] Verify Material Design theme
- [ ] Test light mode
- [ ] Test dark mode
- [ ] Test system mode
- [ ] Test all UI interactions
- [ ] Verify no transparency/blur
- [ ] Test accessibility features

## Screenshots

To document the implementation, take screenshots of:

1. **macOS Light Mode:**
   - Main window with apps
   - Settings modal
   - Context menu
   - Add app modal

2. **macOS Dark Mode:**
   - Main window with apps
   - Settings modal
   - Context menu
   - Add app modal

3. **Windows/Linux Light Mode:**
   - Main window with apps
   - Settings modal

4. **Windows/Linux Dark Mode:**
   - Main window with apps
   - Settings modal

## Reporting Issues

If you find any issues, please report:

1. Platform (macOS version, Windows version, Linux distro)
2. Theme mode (Light/Dark/System)
3. Expected behavior
4. Actual behavior
5. Screenshots if applicable
6. Console errors if any

## Success Criteria

The implementation is successful if:

✅ Platform detection works automatically
✅ macOS shows liquid glass theme with blur
✅ Windows/Linux shows Material Design theme
✅ Theme switching works correctly
✅ Performance is smooth (60fps)
✅ No visual glitches
✅ Accessibility standards met
✅ No console errors

