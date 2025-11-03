# Quick Start Guide

Get up and running with jvlauncher in under 5 minutes!

## Installation

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Tauri CLI
cargo install tauri-cli --version "^2.0.0"
```

### Platform-Specific Setup

**macOS:**
```bash
xcode-select --install
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Windows:**
- Install [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

## Build & Run

### Development Mode

```bash
./dev.sh
```

### Production Build

```bash
./build.sh
```

## First Steps

1. **Launch**: Press `Cmd+Shift+Space` (macOS) or `Ctrl+Shift+Space` (Windows/Linux)
2. **Add App**: Click the blue "+" button
3. **Configure**: Click the gear icon ⚙️ for settings

## Adding Your First App

### Example: Add Terminal

1. Click "+" button
2. Select "Application"
3. Name: "Terminal"
4. Binary Path: 
   - macOS: `/Applications/Utilities/Terminal.app`
   - Linux: `/usr/bin/gnome-terminal`
   - Windows: `C:\Windows\System32\cmd.exe`
5. Click "Save"

### Example: Add Web App

1. Click "+" button
2. Select "Web Application"
3. Name: "Gmail"
4. URL: `https://mail.google.com`
5. Choose an icon (optional)
6. Click "Save"

## Usage Tips

- **Navigate**: Arrow keys
- **Launch**: Click or press Enter
- **Edit**: Right-click → Edit
- **Delete**: Right-click → Delete
- **Reorder**: Drag and drop
- **Hide**: Press Escape

## Common Issues

**App won't launch?**
- Check the binary path is correct
- Ensure the file is executable

**Shortcut not working?**
- Another app might be using it
- Change it in Settings

**Icons not showing?**
- Try manually selecting an icon
- Icons extract automatically when possible

## Next Steps

- Read [README.md](README.md) for full feature list
- Check [SETUP.md](SETUP.md) for detailed setup instructions
- See [ARCHITECTURE.md](ARCHITECTURE.md) to understand the code

## Need Help?

Open an issue on GitHub with:
- Your OS and version
- Steps to reproduce the problem
- Error messages if any

Happy launching! 🚀

