# Terminal Application Rendering Fixes

## Problem Summary

Terminal applications (like k9s) were launching but displaying blank/empty windows with no content. The terminal window would open successfully, but no output, UI, or interactive interface was visible.

## Root Causes Identified

### 1. **Line-based Reading Instead of Raw Bytes**
- **Issue**: The PTY output was being read using `BufReader::read_line()`, which only works for line-oriented output
- **Impact**: TUI applications use ANSI escape sequences, control characters, and cursor positioning that don't work with line-based reading
- **Location**: `src-tauri/src/terminal.rs:41-57`

### 2. **Missing Environment Variables**
- **Issue**: No environment variables were being set for the spawned terminal process
- **Impact**: Terminal applications need `TERM`, `COLORTERM`, etc. to know what capabilities are available
- **Location**: `src-tauri/src/terminal.rs:26-30`

### 3. **No Input Handling**
- **Issue**: The terminal window could only display output but couldn't send user input back to the PTY
- **Impact**: Interactive applications were impossible to use
- **Location**: No input mechanism existed

### 4. **Simple Text Display Instead of Terminal Emulator**
- **Issue**: The frontend just appended text to a div without interpreting ANSI codes
- **Impact**: No cursor positioning, colors, or terminal features were supported
- **Location**: `dist/terminal.html`

### 5. **No PTY Resize Support**
- **Issue**: PTY was created with fixed size (24x80) with no resize mechanism
- **Impact**: Terminal applications couldn't adapt to window size changes
- **Location**: `src-tauri/src/terminal.rs:19-24`

## Solutions Implemented

### 1. Raw Byte Reading (✅ Fixed)
**File**: `src-tauri/src/terminal.rs`

Changed from line-based reading to raw byte reading:
```rust
// OLD: Line-based reading
let mut buf_reader = BufReader::new(reader);
let mut line = String::new();
while buf_reader.read_line(&mut line).is_ok() { ... }

// NEW: Raw byte reading
let mut buffer = [0u8; 8192];
loop {
    match reader.read(&mut buffer) {
        Ok(0) => break,
        Ok(n) => {
            if let Ok(output) = String::from_utf8(buffer[..n].to_vec()) {
                // Emit to frontend
            }
        }
        Err(_) => break,
    }
}
```

### 2. Environment Variables (✅ Fixed)
**File**: `src-tauri/src/terminal.rs`

Added essential environment variables:
```rust
cmd.env("TERM", "xterm-256color");
cmd.env("COLORTERM", "truecolor");
cmd.env("TERM_PROGRAM", "jvlauncher");

// Preserve important variables
if let Ok(path) = std::env::var("PATH") {
    cmd.env("PATH", path);
}
// ... HOME, USER, SHELL
```

### 3. Bidirectional Communication (✅ Fixed)
**Files**: 
- `src-tauri/src/terminal.rs` - Added TerminalState and TerminalHandle
- `src-tauri/src/commands.rs` - Added `send_terminal_input` command
- `src-tauri/src/main.rs` - Registered new commands and initialized state

Added input handling:
```rust
// Backend: Store PTY master for writing
pub struct TerminalState {
    pub windows: Arc<Mutex<std::collections::HashMap<String, TerminalHandle>>>,
}

// Command to send input
#[tauri::command]
pub fn send_terminal_input(
    app_handle: AppHandle,
    window_label: String,
    data: String,
) -> Result<(), String> {
    // Write to PTY master
}
```

### 4. xterm.js Terminal Emulator (✅ Fixed)
**File**: `dist/terminal.html`

Replaced simple text display with full-featured xterm.js:
```html
<!-- Load xterm.js and addons -->
<script src="https://cdn.jsdelivr.net/npm/xterm@5.3.0/lib/xterm.js"></script>
<script src="https://cdn.jsdelivr.net/npm/xterm-addon-fit@0.8.0/lib/xterm-addon-fit.js"></script>

<script>
    // Initialize terminal with proper theme and settings
    const term = new Terminal({
        cursorBlink: true,
        fontSize: 14,
        theme: { ... },
    });
    
    // Add fit addon for auto-resizing
    const fitAddon = new FitAddon.FitAddon();
    term.loadAddon(fitAddon);
    
    // Listen for output from backend
    await listen('terminal-output', (event) => {
        term.write(event.payload);
    });
    
    // Send input to backend
    term.onData((data) => {
        invoke('send_terminal_input', {
            windowLabel: windowLabel,
            data: data
        });
    });
</script>
```

### 5. PTY Resize Support (✅ Fixed)
**Files**: 
- `src-tauri/src/commands.rs` - Added `resize_terminal` command
- `dist/terminal.html` - Added resize event handler

```rust
#[tauri::command]
pub fn resize_terminal(
    app_handle: AppHandle,
    window_label: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    // Resize PTY
}
```

```javascript
window.addEventListener('resize', () => {
    fitAddon.fit();
    invoke('resize_terminal', {
        windowLabel: windowLabel,
        rows: term.rows,
        cols: term.cols
    });
});
```

## Files Modified

1. **src-tauri/src/terminal.rs**
   - Changed from line-based to raw byte reading
   - Added environment variables
   - Added TerminalState and TerminalHandle structures
   - Added cleanup on window close

2. **src-tauri/src/commands.rs**
   - Added `send_terminal_input` command
   - Added `resize_terminal` command

3. **src-tauri/src/main.rs**
   - Initialized TerminalState
   - Registered new terminal commands

4. **dist/terminal.html**
   - Replaced simple text display with xterm.js
   - Added input handling
   - Added resize support
   - Added proper terminal theme

## Testing

To test the fixes:

1. Build the application:
   ```bash
   cd src-tauri && cargo build
   ```

2. Run the application:
   ```bash
   cargo tauri dev
   ```

3. Add a TUI application (e.g., k9s, htop, vim):
   - Click "Add Application"
   - Select "Terminal Application" type
   - Browse to the binary (e.g., `/usr/local/bin/k9s`)
   - Save and launch

4. Verify:
   - ✅ Terminal window opens
   - ✅ Application UI is visible with colors
   - ✅ Keyboard input works
   - ✅ Window can be resized
   - ✅ Application responds to commands

## Debugging Notes

During implementation, we discovered:
- The PTY was working correctly and reading output
- k9s was running and sending ANSI escape sequences
- The issue was with the frontend xterm.js integration
- Using `unpkg.com` CDN works better than `cdn.jsdelivr.net` with Tauri
- The global `window.__TAURI__` API is more reliable than ES module imports in Tauri webviews

## Expected Behavior

After these fixes, terminal applications should:
- Display their full interactive interface
- Show colors and formatting correctly
- Respond to keyboard input
- Handle window resizing properly
- Support all standard terminal features (cursor positioning, ANSI codes, etc.)

## Technical Details

### PTY Communication Flow

```
User Input → xterm.js → send_terminal_input → PTY Master → PTY Slave → Application
Application → PTY Slave → PTY Master → Raw Bytes → terminal-output event → xterm.js → Display
```

### Environment Variables Set

- `TERM=xterm-256color` - Terminal type with 256 color support
- `COLORTERM=truecolor` - True color support indicator
- `TERM_PROGRAM=jvlauncher` - Terminal program identifier
- `PATH`, `HOME`, `USER`, `SHELL` - Preserved from parent environment

### xterm.js Features Enabled

- Cursor blinking
- 256 color support
- Web links addon (clickable URLs)
- Fit addon (auto-resize)
- Proper color theme matching terminal standards

