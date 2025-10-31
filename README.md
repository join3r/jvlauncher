# App Launcher

A cross-platform application launcher built with Tauri 2.0 and Rust. Launch applications, webapps, and terminal applications with customizable shortcuts and an intuitive grid interface.

## Features

- **Global Shortcut**: Show/hide the launcher with a customizable keyboard shortcut (default: Cmd/Ctrl+Space)
- **Multiple App Types**:
  - **Applications**: Launch native applications with command-line parameters
  - **Web Apps**: Open webapps in dedicated windows with persistent sessions
  - **Terminal Apps**: Run TUI applications in an embedded terminal
- **Grid Layout**: Organize your apps in a customizable grid
- **Keyboard Navigation**: Navigate with arrow keys and launch with Enter
- **Drag & Drop**: Reorder apps by dragging them to new positions
- **Icon Support**: 
  - Auto-extract icons from applications
  - Use custom icon images
- **Customization**:
  - Light/Dark/System theme
  - Adjustable grid size
  - Configure start at login
  - Per-app keyboard shortcuts
- **Platform-Specific Theming**:
  - **macOS**: Native liquid glass (glassmorphism) theme with vibrancy and backdrop blur
  - **Windows/Linux**: Material Design theme with solid backgrounds
- **Cross-Platform**: Works on macOS, Linux, and Windows

## Development

### Prerequisites

- Rust (latest stable version)
- Node.js and npm (for frontend dependencies if needed)
- Platform-specific requirements:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: Development packages for GTK and WebKit
  - **Windows**: Visual Studio Build Tools

### Building

```bash
# Install Tauri CLI
cargo install tauri-cli --version "^2.0.0"

# Build the application
cd src-tauri
cargo tauri build
```

### Running in Development

```bash
./dev.sh
```

## Usage

1. **Show Launcher**: Press the global shortcut (default: Cmd/Ctrl+Shift+Space)
2. **Add Application**: Click the + button in the bottom-right corner
3. **Navigate**: Use arrow keys to move between apps
4. **Launch**: Click an app or press Enter
5. **Edit/Delete**: Right-click an app to show options
6. **Reorder**: Drag and drop apps to rearrange them
7. **Settings**: Click the gear icon in the top-right corner

### Keyboard Shortcuts

- **Arrow Keys**: Navigate between apps
- **Enter**: Launch selected app
- **Escape**: Hide launcher window
- **Custom Shortcuts**: Assign per-app shortcuts for quick access

## Platform-Specific Theming

### macOS Liquid Glass Theme

On macOS, the application features a native "liquid glass" (glassmorphism) theme that integrates seamlessly with the macOS design language:

- **Translucent Backgrounds**: Semi-transparent windows with backdrop blur
- **Vibrancy Effects**: Native macOS vibrancy using the HudWindow effect
- **Adaptive Blur**: 40px blur in light mode, 60px in dark mode
- **System Integration**: Follows macOS Human Interface Guidelines
- **Native Colors**: Uses macOS system blue (#007aff) as accent color

The liquid glass theme automatically activates when running on macOS and supports both light and dark modes.

### Material Design Theme (Windows/Linux)

On Windows and Linux, the application uses a Material Design theme:

- **Solid Backgrounds**: No transparency for better performance
- **Material Colors**: Material Design color palette
- **Standard Shadows**: Material elevation shadows
- **Material Blue**: #2196f3 as accent color

For more details, see [MACOS_LIQUID_GLASS_THEME.md](MACOS_LIQUID_GLASS_THEME.md).

## Configuration

Settings are stored in a SQLite database in your application data directory:

- **macOS**: `~/Library/Application Support/com.applauncher.dev/`
- **Linux**: `~/.local/share/com.applauncher.dev/`
- **Windows**: `%APPDATA%\com.applauncher.dev\`

## Architecture

### Backend (Rust)

- **Database**: SQLite with rusqlite for data persistence
- **Icon Extraction**: Platform-specific icon extraction from binaries
- **Launcher**: Process management for different app types
- **Terminal**: Embedded terminal using portable-pty
- **Global Shortcuts**: System-wide keyboard shortcuts
- **Window Management**: Always-on-top, show/hide functionality

### Frontend (HTML/CSS/JavaScript)

- **Vanilla JavaScript**: No framework overhead
- **Tauri API**: Native integration with backend
- **Responsive Grid**: CSS Grid layout with dynamic sizing
- **Modern UI**: Clean, accessible interface with animations

## Building for Production

```bash
# Build release version
cd src-tauri
cargo tauri build

# The installer will be in target/release/bundle/
```

## Troubleshooting

### Global Shortcut Not Working

- Check if another application is using the same shortcut
- Try changing the shortcut in Settings
- Restart the application after changing shortcuts

### Icons Not Extracting

- Ensure the application path is correct
- Try manually selecting an icon image
- On Windows, icon extraction requires additional implementation

### Terminal Apps Not Starting

- Verify the binary path is correct
- Check that the application is executable
- Review terminal window for error messages

## License

No idea

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.
