# Window Position and Size Persistence Implementation

## Overview

This document describes the implementation of window position and size persistence for web applications in jvlauncher. Web applications now remember their last window position and dimensions when reopened.

## Problem Statement

Previously, when a web application was launched from the launcher:
1. The window would always open in the center of the screen
2. Moving the window to a different position and closing it would not save the position
3. Reopening the application would reset it to the center

## Solution

The implementation adds persistent storage of window position and size for each web application, allowing them to reopen in their last known location and dimensions.

## Changes Made

### 1. Database Schema Updates (`src-tauri/src/database.rs`)

Added four new columns to the `webapp_details` table:
- `window_x` (INTEGER): X coordinate of the window
- `window_y` (INTEGER): Y coordinate of the window
- `window_width` (INTEGER): Width of the window
- `window_height` (INTEGER): Height of the window

The schema migration automatically adds these columns to existing databases using `ALTER TABLE` statements.

### 2. New Data Structure (`src-tauri/src/database.rs`)

Created `WindowState` struct to represent window position and size:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
```

### 3. Database Functions (`src-tauri/src/database.rs`)

Added two new functions:

#### `save_window_state(pool: &DbPool, app_id: i64, state: &WindowState) -> Result<()>`
- Saves the window position and size to the database
- Called when a webapp window is closed
- Updates the `webapp_details` table for the specific app

#### `load_window_state(pool: &DbPool, app_id: i64) -> Result<Option<WindowState>>`
- Loads the saved window state from the database
- Returns `None` if no state is saved or if any value is missing
- Called when launching a webapp

### 4. Launcher Updates (`src-tauri/src/launcher.rs`)

Modified `launch_webapp()` function to:

1. **Accept database pool parameter**: Updated function signature to include `pool: &DbPool`
2. **Load saved state**: Attempts to load the saved window state when launching
3. **Restore position and size**: If saved state exists, applies it to the new window
4. **Fallback to defaults**: If no saved state, uses default size (1200x800) and centers the window
5. **Save on close**: Attaches an event handler to save window state when the window closes

### 5. Command Updates (`src-tauri/src/commands.rs`)

Updated the `launch()` command to pass the database pool to `launcher::launch_app()`.

## How It Works

### Launch Flow
```
User launches webapp
    ↓
load_window_state(pool, app_id)
    ↓
If saved state exists:
    - Set window position to (x, y)
    - Set window size to (width, height)
Else:
    - Set default size (1200x800)
    - Center window on screen
    ↓
Window opens
```

### Close Flow
```
User closes webapp window
    ↓
on_window_event triggered (CloseRequested)
    ↓
Get current window position: outer_position()
Get current window size: outer_size()
    ↓
Create WindowState struct
    ↓
save_window_state(pool, app_id, state)
    ↓
Update database
```

## Technical Details

### Window Position Coordinates
- `x` and `y` are in screen coordinates (pixels from top-left of screen)
- Tauri's `outer_position()` returns the window's outer frame position
- Tauri's `position()` builder method sets the window position

### Window Size
- `width` and `height` are in logical pixels
- Tauri's `outer_size()` returns the window's outer frame size
- Tauri's `inner_size()` builder method sets the window's inner size

### Database Storage
- Window state is stored in the `webapp_details` table
- Each webapp has its own independent window state
- NULL values indicate no saved state (first launch)

## Testing

Unit tests have been added to verify:
- `WindowState` struct serialization
- `WindowState` struct cloning

Run tests with:
```bash
cd src-tauri
cargo test
```

## Behavior

### First Launch
- Window opens at default size (1200x800) centered on screen
- No saved state exists yet

### Subsequent Launches
- Window opens at the position and size from the last session
- If the saved position is off-screen, Tauri may adjust it

### Manual Window Resizing
- Users can resize the window normally
- The new size is saved when the window closes

### Manual Window Moving
- Users can move the window to any position
- The new position is saved when the window closes

## Compatibility

- Works with all web applications (webapps)
- Does not affect native applications or TUI applications
- Backward compatible: existing databases will have NULL values for window state
- First launch of existing webapps will use default positioning

## Future Enhancements

Possible improvements:
1. Save window state on minimize/maximize events
2. Restore maximized/fullscreen state
3. Per-monitor window positioning
4. Validation to ensure saved positions are on-screen
5. User preference to disable persistence

## Files Modified

1. `src-tauri/src/database.rs`
   - Added `WindowState` struct
   - Updated schema creation
   - Added `save_window_state()` function
   - Added `load_window_state()` function
   - Added unit tests

2. `src-tauri/src/launcher.rs`
   - Updated `launch_app()` signature
   - Updated `launch_webapp()` to load and save window state
   - Added event handler for window close

3. `src-tauri/src/commands.rs`
   - Updated `launch()` command to pass pool to launcher

## Build and Deployment

The implementation is fully integrated and compiles without errors:
```bash
cargo build --release
```

All tests pass:
```bash
cargo test
```

