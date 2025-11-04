# Visual Testing Guide - Window Corner Fix

## Quick Test

### 1. Launch the Application
```bash
bun run tauri dev
```

### 2. Open the Launcher Window
- Use your global shortcut (default: Cmd+Space or configured shortcut)
- OR click the system tray icon

### 3. Visual Inspection Checklist

#### ✅ Window Corners
Look at all four corners of the window:
- [ ] Corners are uniformly rounded
- [ ] No double-layer effect visible
- [ ] No "ghost" outline with different radius
- [ ] Clean, single border-radius appearance
- [ ] Smooth transition from window to background

#### ✅ Window Edges
Inspect the edges between corners:
- [ ] Straight edges are clean
- [ ] No layering artifacts along edges
- [ ] Consistent border appearance
- [ ] No visible seams or gaps

#### ✅ Liquid Glass Effect
Verify the macOS liquid glass effect is working:
- [ ] Background blur is visible
- [ ] Semi-transparent appearance
- [ ] Vibrancy effect active
- [ ] Content behind window is blurred

#### ✅ Different Backgrounds
Test with different desktop backgrounds:
- [ ] Solid color background
- [ ] Complex image background
- [ ] Light background (test dark mode)
- [ ] Dark background (test light mode)

### 4. Theme Testing

#### Light Mode
1. Ensure system is in light mode
2. Open launcher
3. Check corners and edges
4. Verify glass effect

#### Dark Mode
1. Switch system to dark mode
2. Open launcher
3. Check corners and edges
4. Verify glass effect

### 5. Movement Testing
- [ ] Move window around screen
- [ ] Check corners at different screen positions
- [ ] Verify no artifacts appear during movement
- [ ] Test near screen edges

## What to Look For

### ❌ BEFORE (Problem)
```
┌─────────────────┐
│  ╭─────────────╮ │  ← Two visible corner radii
│  │             │ │     (22px CSS over 12px native)
│  │   Content   │ │
│  │             │ │
│  ╰─────────────╯ │
└─────────────────┘
```

### ✅ AFTER (Fixed)
```
╭─────────────────╮
│                 │  ← Single, clean corner radius
│    Content      │     (12px matching both layers)
│                 │
╰─────────────────╯
```

## Common Issues to Watch For

### Issue: Still seeing double corners
**Possible causes:**
- Browser cache (hard reload: Cmd+Shift+R)
- CSS not reloaded (restart dev server)
- Wrong platform class applied

**Solution:**
1. Kill the dev server (Ctrl+C)
2. Clear any caches
3. Restart: `bun run tauri dev`

### Issue: No blur effect
**Possible causes:**
- Not on macOS
- Transparency not enabled
- System settings blocking effects

**Solution:**
- Verify you're on macOS
- Check `tauri.conf.json` has `"transparent": true`
- Check System Preferences > Accessibility > Display

### Issue: Corners look pixelated
**Possible causes:**
- Retina display scaling
- Anti-aliasing issues

**Solution:**
- This is expected on non-retina displays
- On retina displays, should be smooth

## Screenshot Comparison

### How to Take Screenshots for Comparison

1. **Before Fix:**
   - Checkout previous commit
   - Launch app
   - Screenshot window corners

2. **After Fix:**
   - Checkout current commit
   - Launch app
   - Screenshot window corners

3. **Compare:**
   - Place screenshots side-by-side
   - Zoom in on corners
   - Look for double-layer effect in "before"
   - Verify single layer in "after"

## Automated Visual Testing (Future)

Consider implementing:
- Screenshot-based regression tests
- Pixel-perfect comparison tools
- Automated corner detection
- Visual diff tools

## Platform-Specific Notes

### macOS (Primary Target)
- This fix specifically targets macOS
- Uses native HudWindow effect
- 12px radius is macOS standard

### Windows
- Uses different styling
- No native vibrancy effect
- 8px border-radius (Material Design)
- Should be unaffected by this fix

### Linux
- Uses different styling
- No native vibrancy effect
- 8px border-radius (Material Design)
- Should be unaffected by this fix

## Performance Check

While testing, also verify:
- [ ] No performance degradation
- [ ] Smooth animations
- [ ] No lag when opening/closing
- [ ] Blur effect renders smoothly

## Reporting Issues

If you still see corner layering issues after this fix:

1. **Capture:**
   - Screenshot of the issue
   - System information (macOS version)
   - Display information (retina/non-retina)

2. **Verify:**
   - CSS file has `border-radius: 12px`
   - Rust file has `radius: Some(12.0)`
   - Platform class is correctly applied

3. **Report:**
   - Include all captured information
   - Describe exact visual issue
   - Steps to reproduce

## Success Criteria

The fix is successful when:
- ✅ No double-layer corner effect visible
- ✅ Clean, uniform 12px rounded corners
- ✅ Liquid glass effect still working
- ✅ No visual artifacts at any corner or edge
- ✅ Consistent appearance in light and dark modes
- ✅ No performance issues introduced

