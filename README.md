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
cd src-tauri
cargo tauri dev
```

## Usage

1. **Show Launcher**: Press the global shortcut (default: Cmd/Ctrl+Space)
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

# The installer will be in src-tauri/target/release/bundle/
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

[Your License Here]

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Roadmap

- [ ] Plugin system for custom launchers
- [ ] Cloud sync for settings and apps
- [ ] App usage statistics
- [ ] Search functionality
- [ ] Tags and categories
- [ ] Import/export configuration

