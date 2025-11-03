# jvlauncher

A cross-platform application launcher built with Tauri 2.0 and Rust.

## Build

```bash
./build.sh
```

Build outputs:
- **macOS**: `.dmg` file and `.app` bundle in `./target/release/bundle/`
- **Linux**: `.AppImage` or `.deb` in `./target/release/bundle/`
- **Windows**: `.msi` installer in `./target/release/bundle/`

## Development

```bash
./dev.sh
```

## Quick Start

1. **Build**: Run `./build.sh`
2. **Launch**: Press `Cmd/Ctrl+Shift+Space` to show the launcher
3. **Add Apps**: Click the `+` button to add applications, webapps, or terminal apps
4. **Settings**: Click the gear icon to configure theme, grid size, and shortcuts

## Features

- **Global Shortcut**: Show/hide launcher with customizable keyboard shortcut
- **Multiple App Types**: Launch native apps, webapps, and terminal applications
- **Grid Layout**: Organize apps in a customizable grid
- **Keyboard Navigation**: Navigate with arrow keys, launch with Enter
- **Drag & Drop**: Reorder apps by dragging
- **Custom Icons**: Auto-extract or use custom icons
- **Cross-Platform**: macOS, Linux, and Windows

## Usage

- **Arrow Keys**: Navigate between apps
- **Enter**: Launch selected app
- **Escape**: Hide launcher window
- **Right-click**: Edit or delete apps
- **Drag & Drop**: Reorder apps

## Configuration

Settings are stored in:
- **macOS**: `~/Library/Application Support/com.jvlauncher.dev/`
- **Linux**: `~/.local/share/com.jvlauncher.dev/`
- **Windows**: `%APPDATA%\com.jvlauncher.dev\`

## License

No idea
