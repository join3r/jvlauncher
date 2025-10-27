# Architecture

This document describes the technical architecture of App Launcher.

## Overview

App Launcher is built using:
- **Tauri 2.0**: Cross-platform desktop application framework
- **Rust**: Backend logic and system integration
- **HTML/CSS/JavaScript**: User interface
- **SQLite**: Data persistence

## System Architecture

```
┌─────────────────────────────────────────────┐
│              Frontend (WebView)              │
│  ┌────────────┐  ┌──────────┐  ┌─────────┐ │
│  │ HTML/CSS   │  │JavaScript│  │  Tauri  │ │
│  │   UI       │  │  Logic   │  │   API   │ │
│  └────────────┘  └──────────┘  └─────────┘ │
└───────────────────────┬─────────────────────┘
                        │ IPC (invoke/emit)
┌───────────────────────┴─────────────────────┐
│          Backend (Rust/Tauri)                │
│  ┌──────────────────────────────────────┐  │
│  │         Tauri Commands                │  │
│  │  (get_apps, create_app, launch, etc)  │  │
│  └──────────────────────────────────────┘  │
│                      │                       │
│  ┌─────────┐  ┌──────────┐  ┌────────────┐ │
│  │Database │  │ Launcher │  │  Terminal  │ │
│  │ Module  │  │  Module  │  │   Module   │ │
│  └─────────┘  └──────────┘  └────────────┘ │
│                      │                       │
│  ┌─────────┐  ┌──────────┐  ┌────────────┐ │
│  │  Icon   │  │ Shortcut │  │   Window   │ │
│  │Extractor│  │ Manager  │  │  Manager   │ │
│  └─────────┘  └──────────┘  └────────────┘ │
└───────────────────────────────────────────┘
                        │
┌───────────────────────┴─────────────────────┐
│           System Integration                 │
│  ┌─────────┐  ┌──────────┐  ┌────────────┐ │
│  │ OS APIs │  │File System│  │  Processes │ │
│  └─────────┘  └──────────┘  └────────────┘ │
└─────────────────────────────────────────────┘
```

## Backend Modules

### 1. Database Module (`database.rs`)

**Purpose**: Manage persistent data storage

**Responsibilities**:
- Initialize SQLite database with schema
- CRUD operations for apps
- Settings management
- App reordering

**Tables**:
```sql
apps (
    id INTEGER PRIMARY KEY,
    app_type TEXT,
    name TEXT,
    icon_path TEXT,
    position INTEGER,
    shortcut TEXT
)

app_details (
    app_id INTEGER PRIMARY KEY,
    binary_path TEXT,
    cli_params TEXT
)

webapp_details (
    app_id INTEGER PRIMARY KEY,
    url TEXT,
    session_data_path TEXT
)

settings (
    key TEXT PRIMARY KEY,
    value TEXT
)
```

### 2. Commands Module (`commands.rs`)

**Purpose**: Expose backend functions to frontend

**Key Commands**:
- `get_all_apps()`: Retrieve all applications
- `create_app(NewApp)`: Add new application
- `update_app(App)`: Modify existing application
- `delete_app(app_id)`: Remove application
- `launch(app_id)`: Start application
- `reorder_apps(app_ids)`: Update positions
- `extract_icon_from_binary(path)`: Extract icon
- `get_settings()`: Retrieve settings
- `update_setting(key, value)`: Modify setting

### 3. Launcher Module (`launcher.rs`)

**Purpose**: Launch applications based on type

**App Types**:

**Native Applications**:
- Execute binaries with optional CLI parameters
- Platform-specific handling (`.app` bundles on macOS)
- Process spawning and management

**Web Applications**:
- Create isolated webview windows
- Persistent session storage per webapp
- Separate data directories for cookies/localStorage

**Terminal Applications**:
- Spawn command in embedded terminal
- PTY management via `portable-pty`
- Terminal window lifecycle

### 4. Terminal Module (`terminal.rs`)

**Purpose**: Embedded terminal emulator

**Features**:
- PTY (pseudo-terminal) creation
- Command execution
- Output streaming to frontend
- Process lifecycle management
- Window cleanup on close

### 5. Icon Extractor Module (`icon_extractor.rs`)

**Purpose**: Extract and manage application icons

**Platform-Specific Extraction**:

**macOS**:
- Extract from `.app` bundle Resources
- Convert `.icns` to PNG using `sips`
- Search in `Contents/Resources/`

**Windows**:
- Parse PE executable format
- Extract embedded icon resources
- Convert to standard format

**Linux**:
- Parse `.desktop` files
- Search icon theme directories
- Support for hicolor theme structure

**Common**:
- Resize icons to standard size (256x256)
- Save to icons directory
- Support for custom icon files

### 6. Shortcut Manager Module (`shortcut_manager.rs`)

**Purpose**: Global keyboard shortcut handling

**Features**:
- Register system-wide shortcuts
- Toggle window visibility on shortcut press
- Parse shortcut strings (e.g., "Cmd+Space")
- Update shortcuts at runtime
- Handle conflicts gracefully

### 7. Main Module (`main.rs`)

**Purpose**: Application entry point and initialization

**Initialization**:
1. Setup logging
2. Initialize database
3. Register Tauri plugins
4. Configure global shortcuts
5. Setup autostart if enabled
6. Configure main window
7. Register command handlers

**Window Configuration**:
- Always on top when visible
- Center on screen
- Fixed size based on settings
- Hide on focus loss
- Skip taskbar

## Frontend Architecture

### HTML (`index.html`)

Simple, semantic structure:
- App container
- Settings button (top-right)
- App grid (center)
- Add button (bottom-right)
- Modal placeholder

### CSS (`styles.css`)

Modern, responsive styling:
- CSS Grid for app layout
- Flexbox for modals
- CSS animations
- Dark mode support via media queries
- Accessible focus states

### JavaScript (`app.js`)

**State Management**:
```javascript
let apps = [];
let settings = { grid_size: 4, ... };
let selectedIndex = 0;
```

**Key Functions**:
- `loadApps()`: Fetch apps from backend
- `renderApps()`: Update DOM with apps
- `handleKeyDown()`: Keyboard navigation
- `launchApp()`: Launch via backend
- `showAddModal()`: Display add form
- `showEditModal()`: Display edit form
- `handleDrop()`: Drag and drop reordering

**Tauri Integration**:
```javascript
const { invoke } = window.__TAURI__.core;
await invoke('command_name', { arg: value });
```

## Data Flow

### Loading Apps

```
User opens app
    ↓
init() called
    ↓
loadApps() → invoke('get_all_apps')
    ↓
Backend queries database
    ↓
Returns Vec<App>
    ↓
Frontend renders grid
```

### Launching App

```
User clicks/enters app
    ↓
launchApp(appId)
    ↓
invoke('launch', { appId })
    ↓
Backend finds app in DB
    ↓
launcher::launch_app() based on type
    ↓
Spawn process/window/terminal
    ↓
Hide main window
```

### Adding App

```
User clicks + button
    ↓
showAddModal()
    ↓
User fills form & saves
    ↓
invoke('create_app', { newApp })
    ↓
Backend validates data
    ↓
Insert into database
    ↓
Return app ID
    ↓
Reload and render apps
```

## Window Management

### Main Launcher Window

**Properties**:
- Always on top when visible
- Centered on screen
- Fixed size (800x600 default)
- Skip taskbar
- Hide on blur

**Lifecycle**:
1. Created at startup (hidden)
2. Shown on global shortcut
3. Hidden on launch/escape/blur
4. Never destroyed (just hidden)

### Webapp Windows

**Properties**:
- Normal window decorations
- Resizable (1200x800 default)
- Independent of main window
- Persistent session storage

**Session Management**:
- Separate data directory per webapp
- Cookies and localStorage persist
- Isolated from other webapps

### Terminal Windows

**Properties**:
- Standard window (800x600)
- Contains PTY output
- Auto-scrolling
- Closes on process exit

## Security Considerations

1. **Input Validation**: All user inputs validated before database insertion
2. **Path Sanitization**: File paths checked before execution
3. **CSP**: Content Security Policy configured in Tauri
4. **IPC**: Only whitelisted commands exposed to frontend
5. **Process Isolation**: Child processes spawned safely

## Performance Optimizations

1. **Lazy Loading**: Apps loaded on demand
2. **Icon Caching**: Icons stored locally, not re-extracted
3. **Connection Pooling**: Database connections pooled
4. **Async Operations**: All I/O operations async
5. **Minimal Rerendering**: Only update DOM when needed

## Error Handling

**Backend**:
- Use `Result<T, E>` for all fallible operations
- Return descriptive error messages
- Log errors for debugging

**Frontend**:
- Catch all promise rejections
- Display user-friendly error messages
- Console log detailed errors for debugging

## Testing Strategy

**Unit Tests**:
- Database operations
- Icon extraction logic
- Shortcut parsing

**Integration Tests**:
- Command invocations
- Launch workflows
- Settings persistence

**Manual Testing**:
- UI interactions
- Cross-platform behavior
- Edge cases

## Platform-Specific Notes

### macOS
- Use `.app` bundles for apps
- `open` command for launching
- Code signing for distribution

### Linux
- Multiple package formats (deb, AppImage)
- Desktop file integration
- GTK theming support

### Windows
- MSI installer
- Windows shortcuts (.lnk)
- Registry integration for autostart

## Future Enhancements

1. **Plugin System**: Allow custom launchers
2. **Cloud Sync**: Sync settings across devices
3. **Search**: Quick search/filter functionality
4. **Categories**: Organize apps in groups
5. **Statistics**: Track app usage
6. **Themes**: Customizable color schemes

