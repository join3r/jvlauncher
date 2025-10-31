// Tauri API - using window.__TAURI__ directly
const getTauriAPI = () => {
    return window.__TAURI__;
};

// Wrapper functions for Tauri API
const invoke = async (cmd, args = {}) => {
    const tauri = getTauriAPI();
    if (!tauri) {
        throw new Error('Tauri API not available');
    }
    return await tauri.core.invoke(cmd, args);
};

const openDialog = async (options = {}) => {
    const tauri = getTauriAPI();
    if (!tauri) {
        throw new Error('Tauri API not available');
    }
    return await tauri.dialog.open(options);
};

const convertFileSrc = (filePath) => {
    const tauri = getTauriAPI();
    if (!tauri || !tauri.core.convertFileSrc) {
        console.error('convertFileSrc not available');
        return filePath;
    }
    return tauri.core.convertFileSrc(filePath);
};

// Platform detection
let isMacOS = false;

// Detect platform
async function detectPlatform() {
    const debugInfo = document.getElementById('debug-info');
    const debugPlatform = document.getElementById('debug-platform');

    try {
        const tauri = getTauriAPI();
        if (tauri && tauri.os && tauri.os.platform) {
            // Use Tauri's OS plugin (available when withGlobalTauri is true)
            const platform = tauri.os.platform();
            isMacOS = platform === 'macos';
            console.log('Platform detected:', platform, '(isMacOS:', isMacOS, ')');

            // Apply platform-specific class to root element
            if (isMacOS) {
                document.documentElement.classList.add('platform-macos');
                if (debugPlatform) debugPlatform.textContent = 'macOS (via Tauri OS plugin)';
            } else {
                document.documentElement.classList.add('platform-other');
                if (debugPlatform) debugPlatform.textContent = platform + ' (via Tauri OS plugin)';
            }

            // Show debug info for 5 seconds
            if (debugInfo) {
                debugInfo.style.display = 'block';
                setTimeout(() => { debugInfo.style.display = 'none'; }, 5000);
            }
        } else {
            console.error('Tauri OS plugin not available');
            console.log('Available Tauri modules:', tauri ? Object.keys(tauri) : 'none');
            if (debugPlatform) debugPlatform.textContent = 'Tauri OS plugin not available';
            if (debugInfo) debugInfo.style.display = 'block';
        }
    } catch (error) {
        console.error('Failed to detect platform:', error);
        // Fallback: try to detect from user agent
        const userAgent = navigator.userAgent.toLowerCase();
        if (userAgent.includes('mac')) {
            isMacOS = true;
            document.documentElement.classList.add('platform-macos');
            console.log('Platform detected from user agent: macOS');
            if (debugPlatform) debugPlatform.textContent = 'macOS (via user agent)';
        } else {
            document.documentElement.classList.add('platform-other');
            console.log('Platform detected from user agent: other');
            if (debugPlatform) debugPlatform.textContent = 'other (via user agent)';
        }

        // Show debug info for 5 seconds
        if (debugInfo) {
            debugInfo.style.display = 'block';
            setTimeout(() => { debugInfo.style.display = 'none'; }, 5000);
        }
    }
}

// State
let apps = [];
let settings = { grid_cols: 4, grid_rows: 3, theme: 'system', global_shortcut: 'CommandOrControl+Shift+Space', start_at_login: false };
let selectedIndex = null; // Start with no selection - highlight only appears after arrow/enter key press
let isDragging = false; // Track if a drag operation is in progress
let draggedIndex = null; // Track which item is being dragged
let draggedElement = null; // Track the dragged DOM element
let dragOverElement = null; // Track the element currently being dragged over
let dragGhost = null; // Visual drag ghost element

// Shortcut recording state
let isRecording = false;
let currentRecordingInput = null;
let currentRecordingButton = null;

// Convert file path to Tauri-compatible URL
function toAssetUrl(filePath) {
    if (!filePath) return '';
    // Use Tauri's convertFileSrc for filesystem paths
    return convertFileSrc(filePath);
}

// Format keyboard shortcut from event
function formatShortcut(event) {
    const parts = [];

    // Add modifiers - handle the case where both Ctrl and Meta (Command) are pressed
    // This is important for "hyperkey" combinations (Cmd+Ctrl+Alt+Shift)
    if (event.ctrlKey && event.metaKey) {
        // Both Control and Command are pressed (hyperkey scenario on macOS)
        // Add them separately to capture the full combination
        parts.push('Command');
        parts.push('Control');
    } else if (event.ctrlKey || event.metaKey) {
        // Only one of them is pressed - use cross-platform abstraction
        parts.push('CommandOrControl');
    }

    if (event.altKey) {
        parts.push('Alt');
    }
    if (event.shiftKey) {
        parts.push('Shift');
    }

    // Add key (skip if it's just a modifier)
    const key = event.key;
    const code = event.code;

    if (!['Control', 'Meta', 'Alt', 'Shift'].includes(key)) {
        let formattedKey;

        // Use event.code for letter keys to avoid special characters from Alt/Option
        if (code && code.startsWith('Key')) {
            // KeyA -> A, KeyF -> F, etc.
            formattedKey = code.substring(3).toUpperCase();
        } else if (code && code.startsWith('Digit')) {
            // Digit0 -> 0, Digit1 -> 1, etc.
            formattedKey = code.substring(5);
        } else if (code === 'Space') {
            formattedKey = 'Space';
        } else if (code && code.startsWith('Arrow')) {
            // ArrowUp -> Up, ArrowDown -> Down, etc.
            formattedKey = code.substring(5);
        } else if (key === ' ') {
            formattedKey = 'Space';
        } else if (key.length === 1 && !event.altKey) {
            // Single character without Alt modifier
            formattedKey = key.toUpperCase();
        } else {
            // Special keys like Enter, Escape, Tab, F1-F12, etc.
            formattedKey = key;
        }

        parts.push(formattedKey);
    }

    // Return formatted shortcut string
    return parts.length > 0 ? parts.join('+') : '';
}

// Start recording a shortcut
function startRecording(inputElement, buttonElement) {
    // Stop any existing recording
    stopRecording();
    
    isRecording = true;
    currentRecordingInput = inputElement;
    currentRecordingButton = buttonElement;
    
    // Update button state
    buttonElement.classList.add('recording');
    buttonElement.textContent = 'Press keys...';
    
    // Update input placeholder
    inputElement.value = '';
    inputElement.placeholder = 'Press your shortcut...';
    inputElement.focus();
}

// Stop recording a shortcut
function stopRecording() {
    if (!isRecording) return;
    
    isRecording = false;
    
    if (currentRecordingButton) {
        currentRecordingButton.classList.remove('recording');
        currentRecordingButton.textContent = 'Record';
    }
    
    if (currentRecordingInput) {
        currentRecordingInput.placeholder = 'CommandOrControl+Shift+Space';
    }
    
    currentRecordingInput = null;
    currentRecordingButton = null;
}

// Handle keydown during recording
function handleRecordingKeyDown(event) {
    if (!isRecording || !currentRecordingInput) return;
    
    event.preventDefault();
    event.stopPropagation();
    
    // Ignore if only modifier keys are pressed
    if (['Control', 'Meta', 'Alt', 'Shift'].includes(event.key)) {
        return;
    }
    
    // Format and set the shortcut
    const shortcut = formatShortcut(event);
    if (shortcut) {
        currentRecordingInput.value = shortcut;
        stopRecording();
    }
}

// Initialize app
async function init() {
    console.log('Initializing app...');

    // Check if Tauri API is available
    if (!window.__TAURI__) {
        console.error('Tauri API not available! window.__TAURI__ is undefined');
        console.log('Available window properties:', Object.keys(window));
        // Show error to user
        document.body.innerHTML = '<div style="padding: 20px; color: red;">Error: Tauri API not loaded. Please restart the application.</div>';
        return;
    }

    console.log('Tauri API ready');
    console.log('Available Tauri modules:', Object.keys(window.__TAURI__));

    try {
        await detectPlatform();
        await loadSettings();
        await loadApps();
        setupEventListeners();
        applyTheme();
        focusGrid();

        // Listen for window focus events to reload data
        const tauri = getTauriAPI();
        if (tauri && tauri.event) {
            // Listen for window focus to reload apps and settings
            tauri.event.listen('tauri://focus', async () => {
                console.log('Window focused, reloading data...');
                selectedIndex = null; // Reset selection when window is focused
                await loadSettings(); // This will also resize the window
                await loadApps();
                applyTheme();
            });
        }

        console.log('App initialized successfully');
    } catch (error) {
        console.error('Error during initialization:', error);
        alert('Failed to initialize app: ' + error.message);
    }
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}

// Load settings from backend
async function loadSettings() {
    try {
        settings = await invoke('get_settings');
        updateGridSize();
        // Resize window to match grid dimensions
        await resizeWindow();
    } catch (error) {
        console.error('Failed to load settings:', error);
    }
}

// Resize main window based on current grid settings
async function resizeWindow() {
    try {
        await invoke('resize_main_window', {
            gridCols: settings.grid_cols,
            gridRows: settings.grid_rows
        });
    } catch (error) {
        console.error('Failed to resize window:', error);
    }
}

// Load apps from backend
async function loadApps() {
    try {
        apps = await invoke('get_all_apps');
        renderApps();
    } catch (error) {
        console.error('Failed to load apps:', error);
    }
}

// Render apps grid
function renderApps() {
    const grid = document.getElementById('app-grid');
    grid.innerHTML = '';
    
    apps.forEach((app, index) => {
        const item = document.createElement('div');
        item.className = 'icon-item';
        item.dataset.index = index;
        item.dataset.appId = app.id;
        // Don't use HTML5 draggable - we'll use custom mouse event drag instead
        item.draggable = false;
        
        if (index === selectedIndex) {
            item.classList.add('selected');
        }
        
        // Icon
        if (app.icon_path) {
            const img = document.createElement('img');
            img.className = 'icon-image';
            img.src = toAssetUrl(app.icon_path);
            img.alt = app.name;
            img.draggable = false; // Prevent image from interfering with drag
            item.appendChild(img);
        } else {
            const placeholder = document.createElement('div');
            placeholder.className = 'icon-placeholder';
            placeholder.textContent = app.name.charAt(0).toUpperCase();
            placeholder.draggable = false; // Prevent placeholder from interfering with drag
            item.appendChild(placeholder);
        }
        
        // Name
        const name = document.createElement('div');
        name.className = 'app-name';
        name.textContent = app.name;
        name.draggable = false; // Prevent text from interfering with drag
        item.appendChild(name);
        
        // Shortcut
        if (app.shortcut) {
            const shortcut = document.createElement('div');
            shortcut.className = 'app-shortcut';
            shortcut.textContent = app.shortcut;
            shortcut.draggable = false; // Prevent shortcut from interfering with drag
            item.appendChild(shortcut);
        }
        
        // Event listeners
        let wasDragging = false;
        item.addEventListener('click', (e) => {
            // Prevent launching if we just finished a drag operation
            if (wasDragging || isDragging) {
                wasDragging = false;
                e.preventDefault();
                e.stopPropagation();
                return;
            }
            launchApp(app.id);
        });
        
        // Track if drag happened to prevent click
        const originalMouseDown = item.onmousedown;
        item.addEventListener('mousedown', (e) => {
            wasDragging = false;
        });
        item.addEventListener('contextmenu', (e) => showContextMenu(e, app));
        
        // Custom drag implementation using mouse events (more reliable than HTML5 drag in Tauri)
        let dragStartX = 0;
        let dragStartY = 0;
        let isMouseDown = false;
        
        item.addEventListener('mousedown', (e) => {
            // Only start drag on left mouse button
            if (e.button !== 0) return;
            
            // Don't start drag if clicking on a button or interactive element
            if (e.target.closest('button') || e.target.closest('input')) return;
            
            isMouseDown = true;
            dragStartX = e.clientX;
            dragStartY = e.clientY;
            
            // Small delay to distinguish between click and drag
            const dragThreshold = 5; // pixels
            
            const handleMouseMove = (moveEvent) => {
                if (!isMouseDown) return;
                
                const deltaX = Math.abs(moveEvent.clientX - dragStartX);
                const deltaY = Math.abs(moveEvent.clientY - dragStartY);
                
                // Start drag if mouse moved beyond threshold
                if (deltaX > dragThreshold || deltaY > dragThreshold) {
                    if (!isDragging) {
                        // Start drag
                        isDragging = true;
                        draggedIndex = index;
                        draggedElement = item;
                        wasDragging = true; // Mark that drag happened
                        
                        item.classList.add('dragging');
                        moveEvent.preventDefault(); // Prevent text selection
                        
                        // Create drag ghost element
                        createDragGhost(item, moveEvent);
                        
                        // Set up global mouse tracking
                        setupCustomDragTracking(moveEvent);
                    }
                }
            };
            
            const handleMouseUp = (upEvent) => {
                isMouseDown = false;
                document.removeEventListener('mousemove', handleMouseMove);
                document.removeEventListener('mouseup', handleMouseUp);
                
                // If we were dragging, handle the drop
                if (isDragging) {
                    cleanupCustomDragTracking(upEvent);
                }
            };
            
            document.addEventListener('mousemove', handleMouseMove);
            document.addEventListener('mouseup', handleMouseUp);
        });
        
        // Handle dragover - must use capture phase to catch events on child elements
        // and ALWAYS preventDefault to allow drop events
        item.addEventListener('dragover', (e) => {
            // CRITICAL: Always prevent default to allow drop events to fire
            e.preventDefault();
            e.dataTransfer.dropEffect = 'move';
            
            // Only add drag-over visual feedback if not the dragging item itself
            const currentIndex = parseInt(item.dataset.index);
            if (draggedIndex !== null && draggedIndex !== currentIndex) {
                // Remove drag-over from all items first
                document.querySelectorAll('.icon-item').forEach(el => {
                    const elIndex = parseInt(el.dataset.index);
                    if (elIndex !== draggedIndex) {
                        el.classList.remove('drag-over');
                    }
                });
                // Add drag-over to current item
                item.classList.add('drag-over');
            }
        }, true); // Capture phase to catch events on child elements
        
        item.addEventListener('dragleave', (e) => {
            // Check if we're actually leaving this item (not just moving to a child)
            const rect = item.getBoundingClientRect();
            const x = e.clientX;
            const y = e.clientY;
            // Remove drag-over if mouse is outside the item bounds
            if (x < rect.left || x > rect.right || y < rect.top || y > rect.bottom) {
                item.classList.remove('drag-over');
            }
        });
        
        // Handle drop - use capture phase to catch events on child elements
        item.addEventListener('drop', (e) => {
            e.preventDefault();
            e.stopPropagation(); // Stop propagation to prevent grid handler from interfering
            
            // Remove drag-over from all items
            document.querySelectorAll('.icon-item').forEach(el => {
                el.classList.remove('drag-over');
            });
            
            // Only process drop if we have a valid drag
            const currentIndex = parseInt(item.dataset.index);
            if (draggedIndex !== null && draggedIndex !== currentIndex) {
                handleDrop(e, currentIndex);
            }
        }, true); // Capture phase to catch events on child elements
        
        grid.appendChild(item);
    });
}

// Update grid size
function updateGridSize() {
    const grid = document.getElementById('app-grid');
    grid.style.gridTemplateColumns = `repeat(${settings.grid_cols}, 1fr)`;
    grid.style.gridTemplateRows = `repeat(${settings.grid_rows}, auto)`;
    // Resizing handled by backend upon startup and settings change
    console.log(`Grid updated to ${settings.grid_cols} × ${settings.grid_rows}`);
}

// Apply theme
function applyTheme() {
    const theme = settings.theme;
    const root = document.documentElement;
    
    if (theme === 'light') {
        root.setAttribute('data-theme', 'light');
    } else if (theme === 'dark') {
        root.setAttribute('data-theme', 'dark');
    } else {
        // System theme - remove attribute to use CSS media queries
        root.removeAttribute('data-theme');
    }
    
    console.log('Theme applied:', theme);
}

// Focus grid for keyboard navigation
function focusGrid() {
    document.getElementById('app-grid').focus();
}

// Setup event listeners
function setupEventListeners() {
    console.log('Setting up event listeners...');
    
    // Add button with detailed logging
    const addBtn = document.getElementById('add-btn');
    if (addBtn) {
        console.log('Found add button:', addBtn);
        addBtn.addEventListener('click', (e) => {
            console.log('Add button clicked!', e);
            showAddModal();
        });
        console.log('Add button listener attached');
    } else {
        console.error('Add button not found!');
    }
    
    // Settings button with detailed logging
    const settingsBtn = document.getElementById('settings-btn');
    if (settingsBtn) {
        console.log('Found settings button:', settingsBtn);
        settingsBtn.addEventListener('click', (e) => {
            console.log('Settings button clicked!', e);
            showSettingsModal();
        });
        console.log('Settings button listener attached');
    } else {
        console.error('Settings button not found!');
    }
    
    // Keyboard navigation on grid
    const appGrid = document.getElementById('app-grid');
    if (appGrid) {
        appGrid.addEventListener('keydown', handleKeyDown);
        
        // Prevent default drag behavior on grid to allow drops
        // But don't stop propagation so items can also handle it
        appGrid.addEventListener('dragover', (e) => {
            e.preventDefault();
            e.dataTransfer.dropEffect = 'move';
            // Don't stop propagation - let items handle it too
        });
        
        // Handle drop on empty grid space - use capture phase to check first
        // but don't prevent item drops from working
        appGrid.addEventListener('drop', (e) => {
            // Only handle if drop is directly on grid element itself, not on any child
            const isDirectDropOnGrid = e.target === appGrid;
            const isDropOnItem = e.target.closest('.icon-item') !== null;
            
            if (isDirectDropOnGrid && !isDropOnItem) {
                e.preventDefault();
                e.stopPropagation();
                // If drop happens on grid but not on an item, drop at the end
                if (draggedIndex !== null && draggedIndex < apps.length) {
                    handleDrop(e, apps.length - 1);
                }
            }
            // Otherwise let the item handle it - don't prevent propagation
        }, false); // Use bubbling phase so items can handle first
        
        console.log('Keyboard navigation listener attached');
    }
    
    // Prevent default drag behavior on document body to avoid browser's default drag image
    document.body.addEventListener('dragover', (e) => {
        // Allow dragover to propagate to icon-items and grid
        if (e.target.closest('.icon-item') || e.target.closest('.app-grid')) {
            return; // Let icon-items handle it
        }
        e.preventDefault();
    });
    
    // Global keyboard listener for Escape, recording, and app shortcuts
    document.addEventListener('keydown', (e) => {
        // Handle recording mode first
        if (isRecording) {
            handleRecordingKeyDown(e);
            return;
        }

        // Check if user is typing in an input field, textarea, or select
        const activeElement = document.activeElement;
        const isTyping = activeElement && (
            activeElement.tagName === 'INPUT' ||
            activeElement.tagName === 'TEXTAREA' ||
            activeElement.tagName === 'SELECT' ||
            activeElement.isContentEditable
        );

        // Don't process shortcuts if user is typing (except Escape and Command+,)
        if (isTyping && e.key !== 'Escape' && e.key !== ',') {
            return;
        }

        // Handle Command+, (macOS) or Ctrl+, (Windows/Linux) to open settings
        // This is the standard preferences shortcut on macOS
        if (e.key === ',' && (e.metaKey || e.ctrlKey) && !e.shiftKey && !e.altKey) {
            e.preventDefault();
            console.log('Settings shortcut pressed (Command/Ctrl+,), opening settings window');
            showSettingsModal();
            return;
        }

        // Handle Command+W (macOS) or Ctrl+W (Windows/Linux) to hide window
        // This is the standard close window shortcut
        if ((e.key === 'w' || e.key === 'W') && (e.metaKey || e.ctrlKey) && !e.shiftKey && !e.altKey) {
            e.preventDefault();
            console.log('Close window shortcut pressed (Command/Ctrl+W), hiding window');
            hideWindow();
            return;
        }

        // Check if pressed keys match any app shortcut
        const pressedShortcut = formatShortcut(e);
        if (pressedShortcut) {
            const matchingApp = apps.find(app => app.shortcut === pressedShortcut);
            if (matchingApp) {
                e.preventDefault();
                console.log(`Launching app via shortcut: ${matchingApp.name} (${pressedShortcut})`);
                launchApp(matchingApp.id);
                return;
            }
        }

        // Handle Escape key
        if (e.key === 'Escape') {
            console.log('Escape pressed, hiding window');
            hideWindow();
        }
    });
    console.log('Global keyboard listeners attached');
    
    // Close context menu on click
    document.addEventListener('click', closeContextMenu);
}

// Handle keyboard navigation
function handleKeyDown(e) {
    if (apps.length === 0) return;

    const gridCols = settings.grid_cols;  // Use columns for horizontal navigation
    const totalApps = apps.length;

    switch (e.key) {
        case 'ArrowRight':
            e.preventDefault();
            // Initialize selection on first arrow key press
            if (selectedIndex === null) {
                selectedIndex = 0;
            } else {
                selectedIndex = (selectedIndex + 1) % totalApps;
            }
            renderApps();
            break;
        case 'ArrowLeft':
            e.preventDefault();
            // Initialize selection on first arrow key press
            if (selectedIndex === null) {
                selectedIndex = 0;
            } else {
                selectedIndex = selectedIndex === 0 ? totalApps - 1 : selectedIndex - 1;
            }
            renderApps();
            break;
        case 'ArrowDown':
            e.preventDefault();
            // Initialize selection on first arrow key press
            if (selectedIndex === null) {
                selectedIndex = 0;
            } else {
                selectedIndex = Math.min(selectedIndex + gridCols, totalApps - 1);
            }
            renderApps();
            break;
        case 'ArrowUp':
            e.preventDefault();
            // Initialize selection on first arrow key press
            if (selectedIndex === null) {
                selectedIndex = 0;
            } else {
                selectedIndex = Math.max(selectedIndex - gridCols, 0);
            }
            renderApps();
            break;
        case 'Enter':
            e.preventDefault();
            // If no selection, select first app and launch it
            if (selectedIndex === null) {
                selectedIndex = 0;
                renderApps();
            }
            // Launch the selected app
            if (apps[selectedIndex]) {
                launchApp(apps[selectedIndex].id);
            }
            break;
        case 'Escape':
            e.preventDefault();
            hideWindow();
            break;
    }
}

// Launch app
async function launchApp(appId) {
    try {
        await invoke('launch', { appId });
    } catch (error) {
        console.error('Failed to launch app:', error);
    }
}

// Hide main window
async function hideWindow() {
    try {
        await invoke('hide_main_window');
    } catch (error) {
        console.error('Failed to hide window:', error);
    }
}

// Show context menu
function showContextMenu(e, app) {
    e.preventDefault();
    closeContextMenu();
    
    const menu = document.createElement('div');
    menu.className = 'context-menu';
    menu.id = 'context-menu';
    menu.style.left = `${e.pageX}px`;
    menu.style.top = `${e.pageY}px`;
    
    const editBtn = document.createElement('button');
    editBtn.textContent = 'Edit';
    editBtn.addEventListener('click', () => {
        showEditModal(app);
        closeContextMenu();
    });
    
    const deleteBtn = document.createElement('button');
    deleteBtn.className = 'danger';
    deleteBtn.textContent = 'Delete';
    deleteBtn.addEventListener('click', () => {
        deleteApp(app.id);
        closeContextMenu();
    });
    
    menu.appendChild(editBtn);
    menu.appendChild(deleteBtn);
    document.body.appendChild(menu);
}

// Close context menu
function closeContextMenu() {
    const menu = document.getElementById('context-menu');
    if (menu) {
        menu.remove();
    }
}

// Delete app
async function deleteApp(appId) {
    if (!confirm('Are you sure you want to delete this app?')) return;
    
    try {
        await invoke('delete_app', { appId });
        await loadApps();
    } catch (error) {
        console.error('Failed to delete app:', error);
        alert('Failed to delete app: ' + error);
    }
}

// Create drag ghost element that follows the cursor
function createDragGhost(item, startEvent) {
    // Clone the item for the ghost
    dragGhost = item.cloneNode(true);
    dragGhost.style.position = 'fixed';
    dragGhost.style.pointerEvents = 'none';
    dragGhost.style.opacity = '0.7';
    dragGhost.style.zIndex = '10000';
    dragGhost.style.transform = 'rotate(5deg)';
    dragGhost.style.transition = 'none';
    dragGhost.style.width = item.offsetWidth + 'px';
    dragGhost.style.height = item.offsetHeight + 'px';
    dragGhost.style.boxShadow = '0 8px 24px rgba(0, 0, 0, 0.3)';
    
    // Position initially at mouse position
    const rect = item.getBoundingClientRect();
    dragGhost.style.left = (startEvent.clientX - rect.width / 2) + 'px';
    dragGhost.style.top = (startEvent.clientY - rect.height / 2) + 'px';
    
    document.body.appendChild(dragGhost);
}

// Update drag ghost position
function updateDragGhostPosition(e) {
    if (dragGhost) {
        dragGhost.style.left = (e.clientX - dragGhost.offsetWidth / 2) + 'px';
        dragGhost.style.top = (e.clientY - dragGhost.offsetHeight / 2) + 'px';
    }
}

// Remove drag ghost
function removeDragGhost() {
    if (dragGhost) {
        if (document.body.contains(dragGhost)) {
            document.body.removeChild(dragGhost);
        }
        dragGhost = null;
    }
}

// Setup custom drag tracking using mouse events
function setupCustomDragTracking(startEvent) {
    const handleMouseMove = (e) => {
        if (!isDragging || !draggedElement) return;
        
        // Prevent default to avoid text selection
        e.preventDefault();
        
        // Update drag ghost position
        updateDragGhostPosition(e);
        
        // Temporarily hide drag ghost to detect elements below it
        const ghostWasVisible = dragGhost ? dragGhost.style.display !== 'none' : false;
        if (dragGhost) {
            dragGhost.style.display = 'none';
        }
        
        // Find which icon-item the mouse is over
        const elementBelow = document.elementFromPoint(e.clientX, e.clientY);
        const itemBelow = elementBelow ? elementBelow.closest('.icon-item') : null;
        
        // Restore drag ghost visibility
        if (dragGhost && ghostWasVisible) {
            dragGhost.style.display = '';
        }
        
        if (itemBelow && itemBelow !== draggedElement) {
            // Remove drag-over from all items
            document.querySelectorAll('.icon-item').forEach(el => {
                if (el !== draggedElement) {
                    el.classList.remove('drag-over');
                }
            });
            // Add drag-over to item below
            itemBelow.classList.add('drag-over');
            dragOverElement = itemBelow;
        } else if (!itemBelow || itemBelow === draggedElement) {
            // Remove drag-over if not over a valid item
            document.querySelectorAll('.icon-item').forEach(el => {
                if (el !== draggedElement) {
                    el.classList.remove('drag-over');
                }
            });
            dragOverElement = null;
        }
    };
    
    const handleMouseUp = (e) => {
        if (!isDragging) return;
        
        e.preventDefault();
        
        // Temporarily hide drag ghost to detect elements below it
        if (dragGhost) {
            dragGhost.style.display = 'none';
        }
        
        // Find which icon-item the mouse is over when released
        const elementBelow = document.elementFromPoint(e.clientX, e.clientY);
        const itemBelow = elementBelow ? elementBelow.closest('.icon-item') : null;
        
        // Restore drag ghost (will be removed in cleanup anyway)
        if (dragGhost) {
            dragGhost.style.display = '';
        }
        
        if (itemBelow && itemBelow !== draggedElement && draggedIndex !== null) {
            const targetIndex = parseInt(itemBelow.dataset.index);
            if (targetIndex !== draggedIndex) {
                handleDrop(null, targetIndex);
            }
        }
        
        cleanupCustomDragTracking(e);
    };
    
    document.addEventListener('mousemove', handleMouseMove, true);
    document.addEventListener('mouseup', handleMouseUp, true);
    
    // Store handlers for cleanup
    window._customDragHandlers = { handleMouseMove, handleMouseUp };
}

// Cleanup custom drag tracking
function cleanupCustomDragTracking(e) {
    if (window._customDragHandlers) {
        document.removeEventListener('mousemove', window._customDragHandlers.handleMouseMove, true);
        document.removeEventListener('mouseup', window._customDragHandlers.handleMouseUp, true);
        window._customDragHandlers = null;
    }
    
    // Remove drag ghost
    removeDragGhost();
    
    // Clean up visual state
    document.querySelectorAll('.icon-item').forEach(el => {
        el.classList.remove('dragging', 'drag-over');
    });
    
    // Reset state
    isDragging = false;
    draggedIndex = null;
    draggedElement = null;
    dragOverElement = null;
}

// Handle drag and drop
async function handleDrop(e, targetIndex) {
    if (e) {
        e.preventDefault();
        e.stopPropagation();
    }
    
    const sourceIndex = draggedIndex !== null ? draggedIndex : (e && e.dataTransfer ? parseInt(e.dataTransfer.getData('text/plain')) : null);
    
    if (sourceIndex === null || isNaN(sourceIndex) || sourceIndex === targetIndex || sourceIndex < 0 || targetIndex < 0) {
        return;
    }
    
    if (sourceIndex >= apps.length || targetIndex >= apps.length) {
        return;
    }
    
    // Reorder apps array locally first for immediate visual feedback
    const [movedApp] = apps.splice(sourceIndex, 1);
    apps.splice(targetIndex, 0, movedApp);
    
    // Update visual immediately
    renderApps();
    
    // Update positions in backend
    try {
        const appIds = apps.map(app => app.id);
        await invoke('reorder_apps', { appIds: appIds });
    } catch (error) {
        console.error('Failed to reorder apps:', error);
        // Reload on error to restore correct state
        await loadApps();
    }
}

// Show add modal
async function showAddModal() {
    console.log('showAddModal called');
    try {
        await invoke('open_add_app_window');
    } catch (error) {
        console.error('Error in showAddModal:', error);
    }
}

// Show edit modal
async function showEditModal(app) {
    try {
        await invoke('open_edit_app_window', { appId: app.id });
    } catch (error) {
        console.error('Error in showEditModal:', error);
    }
}

// Show settings modal
async function showSettingsModal() {
    console.log('showSettingsModal called');
    try {
        await invoke('open_settings_window');
    } catch (error) {
        console.error('Error in showSettingsModal:', error);
    }
}

// Create modal for add/edit
function createModal(title, onSave, app = null) {
    const isEdit = app !== null;
    const appType = app?.app_type || 'app';
    
    const overlay = document.createElement('div');
    overlay.className = 'modal-overlay';
    
    const content = document.createElement('div');
    content.className = 'modal-content';
    content.innerHTML = `
        <h2>${title}</h2>
        
        <div class="form-group">
            <label>Type</label>
            <select id="app-type" ${isEdit ? 'disabled' : ''}>
                <option value="app">Application</option>
                <option value="webapp">Web Application</option>
                <option value="tui">Terminal Application</option>
            </select>
        </div>
        
        <div class="form-group">
            <label>Name</label>
            <input type="text" id="app-name" value="${app?.name || ''}" placeholder="Application name">
        </div>
        
        <div class="form-group" id="url-group" style="display: none;">
            <label>URL</label>
            <input type="text" id="app-url" value="${app?.url || ''}" placeholder="https://example.com">
        </div>
        
        <div class="form-group" id="binary-group">
            <label>Binary Path</label>
            <div class="input-with-button">
                <input type="text" id="app-binary" value="${app?.binary_path || ''}" placeholder="/path/to/binary">
                <button class="btn btn-primary" id="browse-binary-btn">Browse</button>
            </div>
        </div>
        
        <div class="form-group" id="params-group">
            <label>Command Line Parameters</label>
            <input type="text" id="app-params" value="${app?.cli_params || ''}" placeholder="--flag value">
        </div>
        
        <div class="form-group">
            <label>Icon</label>
            <div class="input-with-button">
                <button class="btn btn-primary" id="browse-icon-btn">Choose Icon</button>
                <button class="btn btn-primary" id="paste-icon-btn">Paste Icon</button>
            </div>
            <div id="icon-preview" style="margin-top: 8px;"></div>
        </div>
        
        <div class="form-group">
            <label>Keyboard Shortcut</label>
            <div class="input-with-button">
                <input type="text" id="app-shortcut" value="${app?.shortcut || ''}" placeholder="CommandOrControl+1" readonly>
                <button type="button" class="btn-record" id="record-app-shortcut-btn">Record</button>
            </div>
        </div>
        
        <div class="modal-actions">
            <button class="btn btn-secondary" id="cancel-app-btn">Cancel</button>
            <button class="btn btn-success" id="save-app-btn">Save</button>
        </div>
    `;
    
    overlay.appendChild(content);
    
    // App type change handler
    const typeSelect = content.querySelector('#app-type');
    typeSelect.value = appType;
    
    function updateFieldsVisibility() {
        const type = typeSelect.value;
        content.querySelector('#url-group').style.display = type === 'webapp' ? 'block' : 'none';
        content.querySelector('#binary-group').style.display = type !== 'webapp' ? 'block' : 'none';
        content.querySelector('#params-group').style.display = type !== 'webapp' ? 'block' : 'none';
    }
    
    typeSelect.addEventListener('change', updateFieldsVisibility);
    updateFieldsVisibility();
    
    // Icon preview
    let iconPath = app?.icon_path || null;
    function updateIconPreview() {
        const preview = content.querySelector('#icon-preview');
        if (iconPath) {
            preview.innerHTML = `<img src="${toAssetUrl(iconPath)}" style="width: 48px; height: 48px; object-fit: contain;">`;
        }
    }
    updateIconPreview();
    
    // Browse binary
    content.querySelector('#browse-binary-btn').addEventListener('click', async () => {
        try {
            const selected = await openDialog({
                multiple: false,
                directory: false
            });
            if (selected) {
                content.querySelector('#app-binary').value = selected;
                
                // Try to extract icon
                try {
                    iconPath = await invoke('extract_icon_from_binary', { binaryPath: selected });
                    updateIconPreview();
                } catch (e) {
                    console.log('Could not extract icon:', e);
                }
            }
        } catch (error) {
            console.error('Failed to open file dialog:', error);
        }
    });
    
    // Browse icon
    content.querySelector('#browse-icon-btn').addEventListener('click', async () => {
        try {
            const selected = await openDialog({
                multiple: false,
                directory: false,
                filters: [{
                    name: 'Images',
                    extensions: ['png', 'jpg', 'jpeg', 'svg', 'ico', 'icns']
                }]
            });
            if (selected) {
                const appName = content.querySelector('#app-name').value || 'app';
                iconPath = await invoke('save_icon_from_file', {
                    sourcePath: selected,
                    appName: appName
                });
                updateIconPreview();
            }
        } catch (error) {
            console.error('Failed to save icon:', error);
        }
    });

    // Paste icon
    content.querySelector('#paste-icon-btn').addEventListener('click', async () => {
        try {
            const appName = content.querySelector('#app-name').value || 'app';
            iconPath = await invoke('paste_icon_from_clipboard', {
                appName: appName
            });
            updateIconPreview();
        } catch (error) {
            console.error('Failed to paste icon from clipboard:', error);
            alert('Failed to paste icon from clipboard. Make sure an image is in your clipboard.');
        }
    });

    // Cancel button
    content.querySelector('#cancel-app-btn').addEventListener('click', () => {
        stopRecording();
        overlay.remove();
    });
    
    // Record shortcut button
    const recordAppShortcutBtn = content.querySelector('#record-app-shortcut-btn');
    const appShortcutInput = content.querySelector('#app-shortcut');
    if (recordAppShortcutBtn && appShortcutInput) {
        recordAppShortcutBtn.addEventListener('click', () => {
            startRecording(appShortcutInput, recordAppShortcutBtn);
        });
    }
    
    // Save button
    content.querySelector('#save-app-btn').addEventListener('click', async () => {
        // Stop recording if active
        stopRecording();
        const formData = {
            app_type: typeSelect.value,
            name: content.querySelector('#app-name').value,
            icon_path: iconPath,
            shortcut: content.querySelector('#app-shortcut').value || null,
            binary_path: content.querySelector('#app-binary').value || null,
            cli_params: content.querySelector('#app-params').value || null,
            url: content.querySelector('#app-url').value || null
        };
        
        if (!formData.name) {
            alert('Please enter an application name');
            return;
        }
        
        try {
            await onSave(formData);
            await loadApps();
            overlay.remove();
        } catch (error) {
            console.error('Failed to save app:', error);
            alert('Failed to save app: ' + error);
        }
    });
    
    // Close on overlay click
    overlay.addEventListener('click', (e) => {
        if (e.target === overlay) {
            stopRecording();
            overlay.remove();
        }
    });
    
    return overlay;
}

// Create app
async function createApp(formData) {
    await invoke('create_app', { newApp: formData });
}

// Update app
async function updateApp(appId, formData) {
    const updatedApp = {
        id: appId,
        ...formData,
        position: apps.find(a => a.id === appId).position
    };
    await invoke('update_app', { app: updatedApp });
}

