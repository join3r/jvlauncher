# Debugging Guide - Button Issues

## Changes Made

### Issue 1: Escape Key ✅ FIXED
**Added global Escape key listener** that works anywhere in the window, not just on the grid.

**File:** `dist/app.js`
- Added `document.addEventListener('keydown', ...)` for global Escape handling
- Now pressing Escape anywhere will hide the window

### Issue 2: Buttons Not Working 🔍 DEBUGGING
**Added extensive logging** to diagnose button click issues.

**File:** `dist/app.js`
- Added console.log before and after button click handlers
- Added error handling with try/catch blocks
- Added checks for missing DOM elements

## How to Test

### Step 1: Start the App
```bash
cd /Users/join3r/Downloads/Temp/test-impl
./dev.sh
```

### Step 2: Open Developer Tools
1. Press `Cmd+Shift+Space` to show the window
2. **Right-click anywhere** in the window
3. Select **"Inspect"** or **"Inspect Element"**
4. Click the **"Console"** tab

### Step 3: Check Console Output

You should see these messages when the app loads:
```
Initializing app...
Tauri API ready
Setting up event listeners...
Found add button: <button...>
Add button listener attached
Found settings button: <button...>
Settings button listener attached
Keyboard navigation listener attached
Global Escape key listener attached
App initialized successfully
```

### Step 4: Test Escape Key
1. Press **Escape** key
2. Console should show: `Escape pressed, hiding window`
3. Window should hide ✅

### Step 5: Test Buttons
1. Press `Cmd+Shift+Space` to show window again
2. Click the **blue "+" button** (bottom-right)
3. Console should show:
   ```
   Add button clicked! MouseEvent {...}
   showAddModal called
   Add modal created and appended
   ```
4. If modal appears ✅ buttons are working!
5. Close modal and click the **gear "⚙️" button** (top-right)
6. Console should show:
   ```
   Settings button clicked! MouseEvent {...}
   showSettingsModal called
   Settings modal created and appended
   ```

## Troubleshooting

### If You See Errors

#### Error: "Add button not found!" or "Settings button not found!"
**Problem:** HTML elements missing or IDs don't match

**Solution:** Check that `dist/index.html` has:
```html
<button id="add-btn">+</button>
<button id="settings-btn">⚙️</button>
```

#### Error: "Modals container not found!"
**Problem:** Missing modals div in HTML

**Solution:** Check that `dist/index.html` has:
```html
<div id="modals"></div>
```

#### Error: "Tauri API not available"
**Problem:** App not running in Tauri context

**Solution:** Make sure you're running with `./dev.sh`, not opening HTML directly

#### Button clicks logged but nothing happens
**Problem:** JavaScript error in modal creation

**Solution:** Look for red error messages in console after "showAddModal called"

#### No console messages at all
**Problem:** JavaScript not loading

**Solution:** 
1. Check Network tab in DevTools for 404 errors
2. Verify `dist/app.js` exists
3. Check for JavaScript syntax errors (red text in Console)

### If Buttons Still Don't Work

**Share these details:**
1. All console messages (copy from Console tab)
2. Any red error messages
3. Screenshot of the window
4. What happens when you click buttons

## What Should Work Now

- ✅ **Escape key**: Hides window from anywhere
- ✅ **Global shortcut** (`Cmd+Shift+Space`): Toggles window
- ✅ **Window stays visible** when releasing shortcut
- 🔍 **Buttons**: Should work (we're debugging if not)

## Next Steps

1. Run `./dev.sh`
2. Press `Cmd+Shift+Space`
3. Open DevTools (right-click → Inspect)
4. Check Console for messages
5. Try clicking buttons
6. Share console output if buttons don't work

The extensive logging will help us identify exactly what's failing!

