# Auto-Resize Window Implementation

## Overview

Implemented automatic window resizing to fit content for all modal windows (Settings, Add App, Edit App). This eliminates the need for manual size adjustments when adding new UI elements.

## Implementation Details

### 1. Backend: Tauri Command

**File**: `src-tauri/src/commands.rs`

Added a new Tauri command `auto_resize_window` that:
- Accepts window label and content dimensions from JavaScript
- Applies reasonable min/max constraints (400-1200px width, 200-1000px height)
- Resizes the window using logical pixels (handles high-DPI displays correctly)
- Centers the window after resizing

```rust
#[tauri::command]
pub fn auto_resize_window(
    app_handle: AppHandle,
    window_label: String,
    content_width: f64,
    content_height: f64,
) -> Result<(), String>
```

**Registered in**: `src-tauri/src/main.rs` (line 201)

### 2. Frontend: JavaScript Helper Function

**Files**: `dist/settings.js` and `dist/app-form.js`

Added `autoResizeWindow()` function that:
- Measures actual content dimensions using multiple DOM properties
- Gets the current window label
- Calls the backend `auto_resize_window` command
- Handles errors gracefully

The function measures content using:
- `body.scrollHeight` / `body.offsetHeight`
- `html.clientHeight` / `html.scrollHeight` / `html.offsetHeight`
- Takes the maximum value to ensure all content is visible

### 3. Integration Points

#### Settings Window (`dist/settings.js`)

Auto-resize is triggered:
1. **After initialization** (line 496) - 100ms delay to ensure content is rendered
2. **When update section is shown** (line 306) - after checking for updates
3. **When update section is hidden** (line 313) - when no update is available
4. **On error** (line 323) - when update check fails

#### App Form Window (`dist/app-form.js`)

Auto-resize is triggered:
1. **After initialization** (line 700) - 100ms delay to ensure content is rendered
2. **When icon preview changes** (line 320) - after selecting/pasting an icon
3. **When field visibility changes** (line 374) - when switching between app types (Application/Web App/Terminal)

### 4. Size Constraints

The auto-resize command applies these constraints:
- **Width**: 400px minimum, 1200px maximum
- **Height**: 200px minimum, 1000px maximum

These prevent windows from becoming too small or too large while still accommodating most content.

## Benefits

### Before
- **Settings window**: Fixed 600×550px - too small when update section appeared
- **Add app window**: Fixed 520×680px - borderline fit, scrolling when icon shown
- **Edit app window**: Fixed 520×620px - already too small for content

### After
- Windows automatically resize to fit their actual content
- No scrolling required in normal use cases
- Adding new UI elements doesn't require manual size adjustments
- Consistent user experience across all windows

## Testing Scenarios

Test the following to verify auto-resize works correctly:

1. **Settings Window**
   - Open settings → window should fit all content
   - Click "Check for Updates" → window should expand if update is available
   - Close update section → window should shrink back

2. **Add App Window**
   - Open "Add Application" → window should fit form
   - Switch between app types (Application/Web App/Terminal) → window should adjust
   - Select an icon → window should expand to show preview

3. **Edit App Window**
   - Edit an existing app → window should fit all fields
   - Change icon → window should adjust for preview

## Future Enhancements

Potential improvements:
1. Add animation/transition for smoother resize
2. Remember user's preferred window size if they manually resize
3. Add per-window size constraints based on content type
4. Implement debouncing for rapid content changes

## Technical Notes

- Uses 100ms delay (`setTimeout`) to ensure DOM is fully rendered before measuring
- Measures multiple DOM properties and takes maximum to ensure accuracy
- Uses logical pixels for high-DPI display compatibility
- Centers window after resize for better UX
- Graceful error handling - failures don't break the UI

