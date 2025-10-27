# Setup Guide

This guide will help you set up and run the App Launcher on your system.

## Prerequisites

### All Platforms

1. **Rust**: Install from [rustup.rs](https://rustup.rs/)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Tauri CLI**: Install via cargo
   ```bash
   cargo install tauri-cli --version "^2.0.0"
   ```

### Platform-Specific Requirements

#### macOS

- Xcode Command Line Tools:
  ```bash
  xcode-select --install
  ```

#### Linux (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

#### Linux (Fedora)

```bash
sudo dnf install webkit2gtk4.1-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel
```

#### Linux (Arch)

```bash
sudo pacman -Syu
sudo pacman -S webkit2gtk-4.1 \
  base-devel \
  curl \
  wget \
  file \
  openssl \
  appmenu-gtk-module \
  gtk3 \
  libappindicator-gtk3 \
  librsvg \
  libvips
```

#### Windows

- Install [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually pre-installed on Windows 11)

## Building the Application

### Development Build

```bash
# Using the provided script
./dev.sh

# Or manually
cd src-tauri
cargo tauri dev
```

### Production Build

```bash
# Using the provided script
./build.sh

# Or manually
cd src-tauri
cargo tauri build
```

The built application will be located in:
- **macOS**: `src-tauri/target/release/bundle/dmg/`
- **Linux**: `src-tauri/target/release/bundle/appimage/` or `deb/`
- **Windows**: `src-tauri/target/release/bundle/msi/`

## Running the Application

After building, you can:

1. Install the generated package/installer
2. Or run directly: `./src-tauri/target/release/app-launcher`

## First Run

1. The application will start minimized in the system tray
2. Press `Cmd+Space` (macOS) or `Ctrl+Space` (Linux/Windows) to show the launcher
3. Click the "+" button to add your first application
4. Click the gear icon to configure settings

## Adding Applications

### Native Application

1. Click the "+" button
2. Select "Application" as the type
3. Fill in:
   - **Name**: Display name for the app
   - **Binary Path**: Click "Browse" to select the executable
     - macOS: `.app` bundle or binary
     - Linux: Binary file (usually in `/usr/bin/` or `/usr/local/bin/`)
     - Windows: `.exe` file
   - **Parameters**: Optional command-line arguments
   - **Shortcut**: Optional keyboard shortcut (e.g., "Ctrl+1")
4. Click "Save"

### Web Application

1. Click the "+" button
2. Select "Web Application" as the type
3. Fill in:
   - **Name**: Display name
   - **URL**: Full URL (e.g., `https://example.com`)
   - **Icon**: Choose a custom icon image
   - **Shortcut**: Optional keyboard shortcut
4. Click "Save"

Note: Each webapp has its own persistent session, keeping you logged in.

### Terminal Application

1. Click the "+" button
2. Select "Terminal Application" as the type
3. Fill in:
   - **Name**: Display name
   - **Binary Path**: Path to the TUI application
   - **Parameters**: Optional command-line arguments
   - **Shortcut**: Optional keyboard shortcut
4. Click "Save"

The app will launch in an embedded terminal window.

## Configuration

### Settings

Access settings via the gear icon in the top-right:

- **Theme**: Choose between System, Light, or Dark mode
- **Grid Size**: Set the number of columns (2-10)
- **Global Shortcut**: Change the shortcut to show/hide the launcher
- **Start at Login**: Enable/disable automatic startup

### Data Location

All data is stored in your application data directory:

- **macOS**: `~/Library/Application Support/com.applauncher.dev/`
- **Linux**: `~/.local/share/com.applauncher.dev/`
- **Windows**: `%APPDATA%\com.applauncher.dev\`

Contents:
- `launcher.db`: SQLite database with apps and settings
- `icons/`: Extracted and custom icon files
- `webapps/`: Session data for web applications

## Keyboard Shortcuts

### Global
- **Cmd/Ctrl+Space**: Toggle launcher visibility (configurable)

### In Launcher
- **Arrow Keys**: Navigate between apps
- **Enter**: Launch selected app
- **Escape**: Hide launcher

### Per-App
Configure custom shortcuts for individual apps in the add/edit dialog.

## Troubleshooting

### Launcher doesn't appear

1. Check if the app is running (look for it in system tray/menu bar)
2. Try changing the global shortcut in settings
3. Restart the application

### Icons not showing

1. Icons may take a moment to extract on first add
2. Try manually selecting an icon image
3. Check file permissions in the icons directory

### Global shortcut conflicts

If your global shortcut doesn't work:
1. Another app may be using the same shortcut
2. Change it in Settings
3. Restart the application for changes to take effect

### Building fails

1. Ensure all prerequisites are installed
2. Update Rust: `rustup update`
3. Clean build: `cargo clean && cargo build`

### Linux: DBus errors

If you see DBus-related errors:
```bash
sudo apt install dbus-x11
```

### macOS: Code signing issues

For development builds that won't open:
```bash
xattr -cr src-tauri/target/release/bundle/macos/app-launcher.app
```

## Uninstalling

### macOS
1. Quit the application
2. Move `App Launcher.app` from Applications to Trash
3. Remove data: `rm -rf ~/Library/Application\ Support/com.applauncher.dev/`

### Linux
```bash
# If installed via package manager
sudo apt remove app-launcher  # or equivalent

# Manual installation
rm -rf ~/.local/share/applications/app-launcher.desktop
rm -rf ~/.local/share/com.applauncher.dev/
```

### Windows
1. Uninstall via "Add or Remove Programs"
2. Remove data folder: `%APPDATA%\com.applauncher.dev\`

## Development

### Project Structure

```
test-impl/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs      # Entry point
│   │   ├── commands.rs  # Tauri commands
│   │   ├── database.rs  # SQLite operations
│   │   ├── launcher.rs  # App launching logic
│   │   ├── terminal.rs  # Terminal emulator
│   │   ├── icon_extractor.rs
│   │   └── shortcut_manager.rs
│   └── Cargo.toml
├── dist/                # Frontend files
│   ├── index.html
│   ├── app.js
│   ├── styles.css
│   └── terminal.html
└── README.md
```

### Adding Features

1. **Backend**: Add Rust functions in appropriate modules
2. **Commands**: Expose via `commands.rs` with `#[tauri::command]`
3. **Frontend**: Call using `invoke()` from JavaScript
4. **Database**: Update schema in `database.rs`

### Testing

```bash
# Run tests
cd src-tauri
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## Getting Help

If you encounter issues:

1. Check the logs in the data directory
2. Review this documentation
3. Check for similar issues in the repository
4. Open a new issue with details about your system and the problem

## License

[Your License Here]

