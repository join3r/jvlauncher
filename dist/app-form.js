// Tauri API
const getTauriAPI = () => window.__TAURI__;

const invoke = async (cmd, args = {}) => {
    const tauri = getTauriAPI();
    if (!tauri) {
        throw new Error('Tauri API not available');
    }
    return await tauri.core.invoke(cmd, args);
};

const getCurrentWindow = () => {
    const tauri = getTauriAPI();
    if (!tauri) {
        throw new Error('Tauri API not available');
    }
    return tauri.window.getCurrentWindow();
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

const openUrl = async (url) => {
    const tauri = getTauriAPI();
    if (!tauri || !tauri.shell) {
        console.error('Tauri shell API not available');
        return;
    }
    await tauri.shell.open(url);
};

// Convert file path to Tauri-compatible URL
function toAssetUrl(filePath) {
    if (!filePath) return '';
    return convertFileSrc(filePath);
}

// Auto-resize window to fit content
async function autoResizeWindow() {
    console.log('[AppForm] autoResizeWindow called');
    try {
        const window = getCurrentWindow();
        const windowLabel = window.label;
        console.log('[AppForm] Window label:', windowLabel);

        // Measure the actual content container, not the body/html which have height: 100%
        const container = document.querySelector('.app-form-container');
        if (!container) {
            console.warn('[AppForm] App form container not found, skipping auto-resize');
            return;
        }

        console.log('[AppForm] Container found:', container);
        console.log('[AppForm] Container scrollHeight:', container.scrollHeight);
        console.log('[AppForm] Container offsetWidth:', container.offsetWidth);

        // Get the actual content dimensions from the container
        const contentHeight = container.scrollHeight + 36; // Add some padding for window chrome
        const contentWidth = container.offsetWidth + 40; // Add horizontal padding

        console.log('[AppForm] Auto-resize - Content dimensions:', contentWidth, 'x', contentHeight);

        // Call the backend to resize the window
        await invoke('auto_resize_window', {
            windowLabel: windowLabel,
            contentWidth: contentWidth,
            contentHeight: contentHeight
        });

        console.log('[AppForm] Window resized successfully');
    } catch (error) {
        console.error('[AppForm] Failed to auto-resize window:', error);
    }
}

// Platform detection
let isMacOS = false;

// Detect platform and apply platform-specific class
async function detectPlatform() {
    console.log('[AppForm] Starting platform detection...');
    try {
        const tauri = getTauriAPI();
        console.log('[AppForm] Tauri API available:', !!tauri);
        console.log('[AppForm] Available Tauri modules:', tauri ? Object.keys(tauri) : 'none');

        if (tauri && tauri.os && tauri.os.platform) {
            const platform = tauri.os.platform();
            isMacOS = platform === 'macos';
            console.log('[AppForm] Platform detected via Tauri:', platform, '(isMacOS:', isMacOS, ')');

            // Apply platform-specific class to root element
            if (isMacOS) {
                document.documentElement.classList.add('platform-macos');
                console.log('[AppForm] Applied platform-macos class');
            } else {
                document.documentElement.classList.add('platform-other');
                console.log('[AppForm] Applied platform-other class');
            }
        } else {
            console.warn('[AppForm] Tauri OS plugin not available, using fallback');
            // Fallback: try to detect from user agent
            const userAgent = navigator.userAgent.toLowerCase();
            if (userAgent.includes('mac')) {
                isMacOS = true;
                document.documentElement.classList.add('platform-macos');
                console.log('[AppForm] Applied platform-macos class (fallback)');
            } else {
                document.documentElement.classList.add('platform-other');
                console.log('[AppForm] Applied platform-other class (fallback)');
            }
        }

        console.log('[AppForm] Document classes after platform detection:', document.documentElement.className);
    } catch (error) {
        console.error('[AppForm] Failed to detect platform:', error);
        // Fallback
        const userAgent = navigator.userAgent.toLowerCase();
        if (userAgent.includes('mac')) {
            isMacOS = true;
            document.documentElement.classList.add('platform-macos');
            console.log('[AppForm] Applied platform-macos class (error fallback)');
        } else {
            document.documentElement.classList.add('platform-other');
            console.log('[AppForm] Applied platform-other class (error fallback)');
        }
    }
}

// Apply theme based on settings
async function applyTheme() {
    console.log('[AppForm] Loading settings for theme...');
    try {
        const settings = await invoke('get_settings');
        console.log('[AppForm] Settings loaded:', settings);
        const theme = settings.theme || 'system';
        const root = document.documentElement;

        console.log('[AppForm] Applying theme:', theme);
        if (theme === 'light') {
            root.setAttribute('data-theme', 'light');
            console.log('[AppForm] Set data-theme="light"');
        } else if (theme === 'dark') {
            root.setAttribute('data-theme', 'dark');
            console.log('[AppForm] Set data-theme="dark"');
        } else {
            // System theme - remove attribute to use CSS media queries
            root.removeAttribute('data-theme');
            console.log('[AppForm] Removed data-theme attribute (system theme)');
        }

        console.log('[AppForm] Document attributes after theme:', {
            'data-theme': root.getAttribute('data-theme'),
            'class': root.className
        });

        // Log computed background color to verify CSS is working
        const bgColor = window.getComputedStyle(document.body).backgroundColor;
        console.log('[AppForm] Computed background color:', bgColor);
    } catch (error) {
        console.error('[AppForm] Failed to apply theme:', error);
    }
}

function setTypeSegment(value) {
    const container = document.getElementById('type-segment');
    if (!container) return;
    const buttons = Array.from(container.querySelectorAll('button'));
    buttons.forEach(btn => btn.classList.toggle('is-active', btn.dataset.value === value));
}

function getTypeFromSegment() {
    const container = document.getElementById('type-segment');
    if (!container) return 'app';
    const active = container.querySelector('button.is-active');
    return active ? active.dataset.value : 'app';
}

// Shortcut recording state
let isRecording = false;
let currentRecordingInput = null;
let currentRecordingButton = null;

// Icon state
let iconPath = null;
let tempIconPath = null; // Track temporary icon path before save
let userSelectedIcon = false; // Track if user manually selected an icon

// App data (for edit mode)
let appData = null;
let isEditMode = false;

// Debounce timer for URL input
let urlDebounceTimer = null;

// Format keyboard shortcut from event
function formatShortcut(event) {
    const parts = [];

    // Add modifiers - handle the case where both Ctrl and Meta (Command) are pressed
    // This is important for "hyperkey" combinations (Cmd+Ctrl+Alt+Shift)
    if (event.ctrlKey && event.metaKey) {
        // Both Control and Command are pressed (hyperkey scenario on macOS)
        parts.push('Command');
        parts.push('Control');
    } else if (event.ctrlKey || event.metaKey) {
        // Only one of them is pressed - use cross-platform abstraction
        parts.push('CommandOrControl');
    }

    if (event.altKey) parts.push('Alt');
    if (event.shiftKey) parts.push('Shift');

    const key = event.key;
    const code = event.code;

    if (key && key !== 'Control' && key !== 'Meta' && key !== 'Alt' && key !== 'Shift') {
        // Use event.code for letter keys to get the physical key, not the shifted character
        let formattedKey;
        if (code && code.startsWith('Key')) {
            // KeyA -> A, KeyR -> R, etc.
            formattedKey = code.substring(3).toUpperCase();
        } else if (code && code.startsWith('Digit')) {
            // Digit0 -> 0, Digit1 -> 1, etc.
            formattedKey = code.substring(5);
        } else {
            formattedKey = key.toUpperCase();
        }
        parts.push(formattedKey);
    }

    return parts.join('+');
}

// Start recording shortcut
function startRecording(input, button) {
    if (isRecording) {
        stopRecording();
        return;
    }
    
    isRecording = true;
    currentRecordingInput = input;
    currentRecordingButton = button;
    
    button.textContent = 'Recording...';
    button.classList.add('recording');
    input.value = 'Press keys...';
    
    document.addEventListener('keydown', handleRecordingKeyDown);
}

// Stop recording shortcut
function stopRecording() {
    if (!isRecording) return;
    
    isRecording = false;
    document.removeEventListener('keydown', handleRecordingKeyDown);
    
    if (currentRecordingButton) {
        currentRecordingButton.textContent = 'Record';
        currentRecordingButton.classList.remove('recording');
    }
    
    currentRecordingInput = null;
    currentRecordingButton = null;
}

// Handle keydown during recording
function handleRecordingKeyDown(event) {
    if (!isRecording || !currentRecordingInput) return;

    event.preventDefault();
    event.stopPropagation();

    // Only stop recording if a non-modifier key is pressed
    const key = event.key;
    if (key && key !== 'Control' && key !== 'Meta' && key !== 'Alt' && key !== 'Shift') {
        const shortcut = formatShortcut(event);
        if (shortcut) {
            currentRecordingInput.value = shortcut;
            stopRecording();
        }
    }
}

// Update icon preview
function updateIconPreview() {
    const preview = document.getElementById('icon-preview');
    // Use temp icon path if available, otherwise use permanent icon path
    const displayPath = tempIconPath || iconPath;
    if (displayPath) {
        // Add cache-busting timestamp to force browser to reload the image
        const cacheBuster = `?t=${Date.now()}`;
        preview.innerHTML = `<img src="${toAssetUrl(displayPath)}${cacheBuster}">`;
    } else {
        preview.innerHTML = '';
    }

    // Auto-resize window when icon preview changes
    // Use requestAnimationFrame to ensure DOM has updated
    requestAnimationFrame(() => {
        setTimeout(() => {
            autoResizeWindow();
        }, 50);
    });
}

// Fetch web icon from URL
async function fetchWebIcon(url) {
    // Only fetch if user hasn't manually selected an icon
    if (userSelectedIcon) {
        console.log('[AppForm] User has selected a custom icon, skipping auto-fetch');
        return;
    }

    // Validate URL format
    if (!url || !url.trim()) {
        return;
    }

    // Basic URL validation
    try {
        new URL(url);
    } catch (e) {
        console.log('[AppForm] Invalid URL format, skipping icon fetch:', url);
        return;
    }

    console.log('[AppForm] Fetching web icon for URL:', url);

    try {
        const appName = document.getElementById('app-name').value.trim() || 'webapp';
        const fetchedIconPath = await invoke('fetch_web_icon', {
            url: url,
            appName: appName
        });

        if (fetchedIconPath) {
            console.log('[AppForm] Successfully fetched web icon:', fetchedIconPath);
            iconPath = fetchedIconPath;
            updateIconPreview();
        }
    } catch (error) {
        console.log('[AppForm] Failed to fetch web icon:', error);
        // Don't show error to user - it's an automatic feature
    }
}

// Update field visibility based on app type
function updateFieldsVisibility() {
    const type = getTypeFromSegment();
    // Show/hide both label and control for each field
    const urlLabel = document.getElementById('url-label');
    const urlGroup = document.getElementById('url-group');
    const binaryLabel = document.getElementById('binary-label');
    const binaryGroup = document.getElementById('binary-group');
    const paramsLabel = document.getElementById('params-label');
    const paramsGroup = document.getElementById('params-group');

    if (type === 'webapp') {
        urlLabel.style.display = 'block';
        urlGroup.style.display = 'flex';
        binaryLabel.style.display = 'none';
        binaryGroup.style.display = 'none';
        paramsLabel.style.display = 'none';
        paramsGroup.style.display = 'none';
    } else {
        urlLabel.style.display = 'none';
        urlGroup.style.display = 'none';
        binaryLabel.style.display = 'block';
        binaryGroup.style.display = 'flex';
        paramsLabel.style.display = 'block';
        paramsGroup.style.display = 'flex';
    }

    // Auto-resize window when field visibility changes
    // Use requestAnimationFrame to ensure DOM has updated
    requestAnimationFrame(() => {
        setTimeout(() => {
            autoResizeWindow();
        }, 50);
    });
}

// Load app data (for edit mode)
async function loadAppData() {
    try {
        // Get window label to determine if we're in edit mode
        const window = getCurrentWindow();
        const label = window.label;
        
        if (label.startsWith('edit-app-')) {
            isEditMode = true;
            const appId = parseInt(label.replace('edit-app-', ''));
            
            // Get app data from backend
            const apps = await invoke('get_all_apps');
            appData = apps.find(app => app.id === appId);
            
            if (appData) {
                document.getElementById('form-title').textContent = 'Edit Application';
                setTypeSegment(appData.app_type || 'app');
                const seg = document.getElementById('type-segment');
                if (seg) Array.from(seg.querySelectorAll('button')).forEach(b => b.disabled = true);
                document.getElementById('app-name').value = appData.name || '';
                document.getElementById('app-url').value = appData.url || '';
                document.getElementById('app-binary').value = appData.binary_path || '';
                document.getElementById('app-params').value = appData.cli_params || '';
                document.getElementById('app-shortcut').value = appData.shortcut || '';
                iconPath = appData.icon_path || null;
                // If app already has an icon, treat it as user-selected to prevent overwriting
                if (iconPath) {
                    userSelectedIcon = true;
                }
                updateIconPreview();
                updateFieldsVisibility();
            }
        }
    } catch (error) {
        console.error('Failed to load app data:', error);
    }
}

// Save app
async function saveApp() {
    stopRecording();

    const appType = getTypeFromSegment();
    const name = document.getElementById('app-name').value.trim();
    const url = document.getElementById('app-url').value.trim();
    const binaryPath = document.getElementById('app-binary').value.trim();
    const cliParams = document.getElementById('app-params').value.trim();
    const shortcut = document.getElementById('app-shortcut').value.trim();

    if (!name) {
        alert('Please enter an application name');
        return;
    }

    if (appType === 'webapp' && !url) {
        alert('Please enter a URL for the web application');
        return;
    }

    if (appType !== 'webapp' && !binaryPath) {
        alert('Please enter a binary path');
        return;
    }

    // Check for keyboard shortcut conflicts
    if (shortcut) {
        try {
            const conflict = await invoke('check_shortcut_conflict', {
                shortcut: shortcut,
                excludeAppId: isEditMode && appData ? appData.id : null
            });

            if (conflict) {
                const conflictMessage = conflict.conflict_type === 'global'
                    ? `This keyboard shortcut is already assigned to the ${conflict.app_name}.`
                    : `This keyboard shortcut is already assigned to "${conflict.app_name}".`;

                alert(`Keyboard Shortcut Conflict\n\n${conflictMessage}\n\nPlease choose a different shortcut or remove the shortcut from the other application first.`);
                return;
            }
        } catch (error) {
            console.error('Failed to check shortcut conflict:', error);
            alert('Failed to validate keyboard shortcut: ' + error);
            return;
        }
    }

    try {
        // Finalize temp icon if one was selected
        let finalIconPath = iconPath;
        if (tempIconPath) {
            try {
                finalIconPath = await invoke('finalize_temp_icon', {
                    tempIconPath: tempIconPath,
                    appName: name
                });
                console.log('[AppForm] Finalized temp icon:', finalIconPath);
            } catch (e) {
                console.error('[AppForm] Failed to finalize temp icon:', e);
                alert('Failed to save icon: ' + e);
                return;
            }
        }

        if (isEditMode && appData) {
            // Update existing app
            await invoke('update_app', {
                app: {
                    id: appData.id,
                    app_type: appData.app_type,
                    name: name,
                    icon_path: finalIconPath,
                    position: appData.position,
                    shortcut: shortcut || null,
                    binary_path: binaryPath || null,
                    cli_params: cliParams || null,
                    url: url || null,
                    session_data_path: appData.session_data_path
                }
            });
        } else {
            // Create new app
            await invoke('create_app', {
                newApp: {
                    app_type: appType,
                    name: name,
                    icon_path: finalIconPath,
                    shortcut: shortcut || null,
                    binary_path: binaryPath || null,
                    cli_params: cliParams || null,
                    url: url || null
                }
            });
        }

        // Close the window
        const window = getCurrentWindow();
        await window.close();
    } catch (error) {
        console.error('Failed to save app:', error);
        alert('Failed to save application: ' + error);
    }
}

// Initialize
async function init() {
    console.log('[AppForm] Initializing...');

    // Check if Tauri API is available
    if (!window.__TAURI__) {
        console.error('[AppForm] Tauri API not available! window.__TAURI__ is undefined');
        console.log('[AppForm] Available window properties:', Object.keys(window));
        alert('Error: Tauri API not loaded. Please restart the application.');
        return;
    }

    console.log('[AppForm] Tauri API ready');
    console.log('[AppForm] Available Tauri modules:', Object.keys(window.__TAURI__));

    try {
        // Detect platform first
        await detectPlatform();

        // Apply theme
        await applyTheme();

        // Load app data
        await loadAppData();

        // App type change handler
        // Type segmented control
        const typeSegment = document.getElementById('type-segment');
        if (typeSegment) {
            // Default selection (only if not in edit mode)
            if (!isEditMode) {
                setTypeSegment('app');
            }
            typeSegment.addEventListener('click', (e) => {
                const target = e.target;
                if (target && target.tagName === 'BUTTON' && target.dataset.value && !target.disabled) {
                    setTypeSegment(target.dataset.value);
                    updateFieldsVisibility();
                }
            });
        }
        updateFieldsVisibility();

        // Browse binary button
        document.getElementById('browse-binary-btn').addEventListener('click', async () => {
            try {
                const selected = await openDialog({
                    multiple: false,
                    directory: false
                });
                if (selected) {
                    document.getElementById('app-binary').value = selected;

                    // Try to extract icon
                    try {
                        iconPath = await invoke('extract_icon_from_binary', { binaryPath: selected });
                        updateIconPreview();
                    } catch (e) {
                        console.log('[AppForm] Could not extract icon:', e);
                    }
                }
            } catch (error) {
                console.error('[AppForm] Failed to open file dialog:', error);
            }
        });

        // URL input change handler - auto-fetch icon for webapps
        const urlInput = document.getElementById('app-url');
        if (urlInput) {
            urlInput.addEventListener('input', (e) => {
                const url = e.target.value.trim();

                // Clear existing debounce timer
                if (urlDebounceTimer) {
                    clearTimeout(urlDebounceTimer);
                }

                // Only fetch if we're in webapp mode
                const appType = getTypeFromSegment();
                if (appType !== 'webapp') {
                    return;
                }

                // Debounce the fetch to avoid excessive requests
                urlDebounceTimer = setTimeout(() => {
                    fetchWebIcon(url);
                }, 1000); // Wait 1 second after user stops typing
            });
        }

        // Browse icon button
        document.getElementById('browse-icon-btn').addEventListener('click', async () => {
            try {
                const selected = await openDialog({
                    multiple: false,
                    directory: false,
                    filters: [{
                        name: 'Images',
                        extensions: ['png', 'jpg', 'jpeg', 'icns', 'ico']
                    }]
                });
                if (selected) {
                    // Clean up any previous temp icon
                    if (tempIconPath) {
                        try {
                            await invoke('cleanup_temp_icon', { tempIconPath: tempIconPath });
                        } catch (e) {
                            console.log('[AppForm] Failed to cleanup previous temp icon:', e);
                        }
                    }

                    // Save to temporary location
                    tempIconPath = await invoke('save_icon_from_file_temp', {
                        sourcePath: selected
                    });
                    userSelectedIcon = true; // Mark that user manually selected an icon
                    updateIconPreview();
                }
            } catch (error) {
                console.error('[AppForm] Failed to open file dialog:', error);
            }
        });

        // Paste icon button
        document.getElementById('paste-icon-btn').addEventListener('click', async () => {
            try {
                // Clean up any previous temp icon
                if (tempIconPath) {
                    try {
                        await invoke('cleanup_temp_icon', { tempIconPath: tempIconPath });
                    } catch (e) {
                        console.log('[AppForm] Failed to cleanup previous temp icon:', e);
                    }
                }

                // Save to temporary location
                tempIconPath = await invoke('paste_icon_from_clipboard_temp');
                userSelectedIcon = true; // Mark that user manually selected an icon
                updateIconPreview();
            } catch (error) {
                console.error('[AppForm] Failed to paste icon from clipboard:', error);
                alert('Failed to paste icon from clipboard. Make sure an image is in your clipboard.');
            }
        });

        // Dashboard icons link
        const dashboardIconsLink = document.getElementById('dashboardicons-link');
        if (dashboardIconsLink) {
            dashboardIconsLink.addEventListener('click', async (e) => {
                e.preventDefault();
                try {
                    await openUrl('https://dashboardicons.com');
                } catch (error) {
                    console.error('[AppForm] Failed to open dashboardicons.com:', error);
                }
            });
        }

        // Record shortcut button
        const recordBtn = document.getElementById('record-app-shortcut-btn');
        const shortcutInput = document.getElementById('app-shortcut');
        if (recordBtn && shortcutInput) {
            recordBtn.addEventListener('click', () => {
                startRecording(shortcutInput, recordBtn);
            });
        }

        // Save button
        document.getElementById('save-app-btn').addEventListener('click', saveApp);

        // Cancel button
        document.getElementById('cancel-app-btn').addEventListener('click', async () => {
            stopRecording();

            // Clean up temp icon if exists
            if (tempIconPath) {
                try {
                    await invoke('cleanup_temp_icon', { tempIconPath: tempIconPath });
                } catch (e) {
                    console.log('[AppForm] Failed to cleanup temp icon on cancel:', e);
                }
            }

            const window = getCurrentWindow();
            await window.close();
        });

        // Handle Escape key
        document.addEventListener('keydown', async (e) => {
            if (e.key === 'Escape' && !isRecording) {
                // Clean up temp icon if exists
                if (tempIconPath) {
                    try {
                        await invoke('cleanup_temp_icon', { tempIconPath: tempIconPath });
                    } catch (e) {
                        console.log('[AppForm] Failed to cleanup temp icon on escape:', e);
                    }
                }

                const window = getCurrentWindow();
                await window.close();
            }
        });

        console.log('[AppForm] Initialization complete');

        // Auto-resize window to fit content after everything is loaded
        // Use a small delay to ensure all content is rendered
        setTimeout(() => {
            autoResizeWindow();
        }, 100);
    } catch (error) {
        console.error('[AppForm] Error during initialization:', error);
        alert('Failed to initialize app form: ' + error.message);
    }
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}

