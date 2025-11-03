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

// Platform detection
let isMacOS = false;

// Detect platform and apply platform-specific class
async function detectPlatform() {
    console.log('[Settings] Starting platform detection...');
    try {
        const tauri = getTauriAPI();
        console.log('[Settings] Tauri API available:', !!tauri);
        console.log('[Settings] Available Tauri modules:', tauri ? Object.keys(tauri) : 'none');

        if (tauri && tauri.os && tauri.os.platform) {
            const platform = tauri.os.platform();
            isMacOS = platform === 'macos';
            console.log('[Settings] Platform detected via Tauri:', platform, '(isMacOS:', isMacOS, ')');

            // Apply platform-specific class to root element
            if (isMacOS) {
                document.documentElement.classList.add('platform-macos');
                console.log('[Settings] Applied platform-macos class');
            } else {
                document.documentElement.classList.add('platform-other');
                console.log('[Settings] Applied platform-other class');
            }
        } else {
            console.warn('[Settings] Tauri OS plugin not available, using fallback');
            // Fallback: try to detect from user agent
            const userAgent = navigator.userAgent.toLowerCase();
            if (userAgent.includes('mac')) {
                isMacOS = true;
                document.documentElement.classList.add('platform-macos');
                console.log('[Settings] Applied platform-macos class (fallback)');
            } else {
                document.documentElement.classList.add('platform-other');
                console.log('[Settings] Applied platform-other class (fallback)');
            }
        }

        console.log('[Settings] Document classes after platform detection:', document.documentElement.className);
    } catch (error) {
        console.error('[Settings] Failed to detect platform:', error);
        // Fallback
        const userAgent = navigator.userAgent.toLowerCase();
        if (userAgent.includes('mac')) {
            isMacOS = true;
            document.documentElement.classList.add('platform-macos');
            console.log('[Settings] Applied platform-macos class (error fallback)');
        } else {
            document.documentElement.classList.add('platform-other');
            console.log('[Settings] Applied platform-other class (error fallback)');
        }
    }
}

// Apply theme based on settings
function applyTheme(theme) {
    console.log('[Settings] Applying theme:', theme);
    const root = document.documentElement;

    if (theme === 'light') {
        root.setAttribute('data-theme', 'light');
        console.log('[Settings] Set data-theme="light"');
    } else if (theme === 'dark') {
        root.setAttribute('data-theme', 'dark');
        console.log('[Settings] Set data-theme="dark"');
    } else {
        // System theme - remove attribute to use CSS media queries
        root.removeAttribute('data-theme');
        console.log('[Settings] Removed data-theme attribute (system theme)');
    }

    console.log('[Settings] Document attributes after theme:', {
        'data-theme': root.getAttribute('data-theme'),
        'class': root.className
    });

    // Log computed background color to verify CSS is working
    const bgColor = window.getComputedStyle(document.body).backgroundColor;
    console.log('[Settings] Computed background color:', bgColor);
}

// Shortcut recording state
let isRecording = false;
let currentRecordingInput = null;
let currentRecordingButton = null;

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

function setThemeSegment(value) {
    const container = document.getElementById('theme-segment');
    if (!container) return;
    const buttons = Array.from(container.querySelectorAll('button'));
    buttons.forEach(btn => btn.classList.toggle('is-active', btn.dataset.value === value));
}

function getThemeFromSegment() {
    const container = document.getElementById('theme-segment');
    if (!container) return 'system';
    const active = container.querySelector('button.is-active');
    return active ? active.dataset.value : 'system';
}

// Load settings
async function loadSettings() {
    console.log('[Settings] Loading settings...');
    try {
        const settings = await invoke('get_settings');
        console.log('[Settings] Settings loaded:', settings);

        setThemeSegment(settings.theme || 'system');
        document.getElementById('settings-grid-cols').value = settings.grid_cols || 4;
        document.getElementById('settings-grid-rows').value = settings.grid_rows || 3;
        document.getElementById('settings-shortcut').value = settings.global_shortcut || 'CommandOrControl+Shift+Space';
        document.getElementById('settings-start-login').checked = settings.start_at_login || false;

        // Apply the theme to this window
        applyTheme(settings.theme || 'system');
    } catch (error) {
        console.error('[Settings] Failed to load settings:', error);
    }
}

// Update functionality
async function checkForUpdates() {
    const checkBtn = document.getElementById('check-updates-btn');
    const statusEl = document.getElementById('update-status');
    const updateInfoEl = document.getElementById('update-info');

    try {
        checkBtn.disabled = true;
        checkBtn.textContent = 'Checking...';
        statusEl.textContent = 'Checking for updates...';
        updateInfoEl.style.display = 'none';

        const updateInfo = await invoke('check_for_updates');

        if (updateInfo.available) {
            statusEl.textContent = 'Update available!';
            statusEl.style.color = 'var(--accent)';

            // Show update info
            document.getElementById('current-version').textContent = updateInfo.current_version;
            document.getElementById('latest-version').textContent = updateInfo.latest_version;

            const bodyEl = document.getElementById('update-body');
            if (updateInfo.body) {
                bodyEl.textContent = updateInfo.body;
            } else {
                bodyEl.textContent = 'No release notes available.';
            }

            updateInfoEl.style.display = 'block';
        } else {
            statusEl.textContent = `You're up to date! (v${updateInfo.current_version})`;
            statusEl.style.color = 'var(--text-secondary)';
            updateInfoEl.style.display = 'none';
        }
    } catch (error) {
        console.error('Failed to check for updates:', error);
        statusEl.textContent = 'Failed to check for updates';
        statusEl.style.color = '#dc3545';
        updateInfoEl.style.display = 'none';
    } finally {
        checkBtn.disabled = false;
        checkBtn.textContent = 'Check for Updates';
    }
}

async function installUpdate() {
    const installBtn = document.getElementById('install-update-btn');
    const statusEl = document.getElementById('update-status');

    try {
        installBtn.disabled = true;
        installBtn.textContent = 'Downloading...';
        statusEl.textContent = 'Downloading and installing update...';

        await invoke('download_and_install_update');

        statusEl.textContent = 'Update installed! Restart the app to apply.';
        statusEl.style.color = '#28a745';
        installBtn.textContent = 'Installed';

        // Show restart prompt
        setTimeout(() => {
            const restart = confirm('Update installed successfully! Restart the application now?');
            if (restart) {
                invoke('quit_app');
            }
        }, 1000);
    } catch (error) {
        console.error('Failed to install update:', error);
        statusEl.textContent = 'Failed to install update: ' + error;
        statusEl.style.color = '#dc3545';
        installBtn.disabled = false;
        installBtn.textContent = 'Download & Install';
    }
}

// Listen for update notifications from backend
async function setupUpdateListener() {
    const tauri = getTauriAPI();
    if (tauri && tauri.event) {
        await tauri.event.listen('update-available', (event) => {
            console.log('Update available notification:', event.payload);
            const statusEl = document.getElementById('update-status');
            if (statusEl) {
                statusEl.textContent = 'New update available!';
                statusEl.style.color = 'var(--accent)';
            }
        });
    }
}

// Save settings
async function saveSettings() {
    stopRecording();

    const newSettings = {
        theme: getThemeFromSegment(),
        grid_cols: parseInt(document.getElementById('settings-grid-cols').value),
        grid_rows: parseInt(document.getElementById('settings-grid-rows').value),
        global_shortcut: document.getElementById('settings-shortcut').value,
        start_at_login: document.getElementById('settings-start-login').checked
    };

    try {
        await invoke('update_setting', { key: 'theme', value: newSettings.theme });
        await invoke('update_setting', { key: 'grid_cols', value: newSettings.grid_cols.toString() });
        await invoke('update_setting', { key: 'grid_rows', value: newSettings.grid_rows.toString() });
        await invoke('update_setting', { key: 'global_shortcut', value: newSettings.global_shortcut });
        await invoke('update_setting', { key: 'start_at_login', value: newSettings.start_at_login ? 'true' : 'false' });

        // Update the global shortcut registration in the backend
        await invoke('update_global_shortcut', { shortcut: newSettings.global_shortcut });

        // Resize the main window based on new grid dimensions
        await invoke('resize_main_window', {
            gridCols: newSettings.grid_cols,
            gridRows: newSettings.grid_rows
        });

        // Close the window
        const window = getCurrentWindow();
        await window.close();
    } catch (error) {
        console.error('Failed to save settings:', error);
        alert('Failed to save settings: ' + error);
    }
}

// Initialize
async function init() {
    console.log('[Settings] Initializing...');

    // Check if Tauri API is available
    if (!window.__TAURI__) {
        console.error('[Settings] Tauri API not available! window.__TAURI__ is undefined');
        console.log('[Settings] Available window properties:', Object.keys(window));
        alert('Error: Tauri API not loaded. Please restart the application.');
        return;
    }

    console.log('[Settings] Tauri API ready');
    console.log('[Settings] Available Tauri modules:', Object.keys(window.__TAURI__));

    try {
        // Detect platform first
        await detectPlatform();

        // Load settings and apply theme
        await loadSettings();

        // Theme segmented control
        const themeSegment = document.getElementById('theme-segment');
        if (themeSegment) {
            themeSegment.addEventListener('click', (e) => {
                const target = e.target;
                if (target && target.tagName === 'BUTTON' && target.dataset.value) {
                    setThemeSegment(target.dataset.value);
                    applyTheme(target.dataset.value);
                }
            });
        }

        // Record button
        const recordBtn = document.getElementById('record-shortcut-btn');
        const shortcutInput = document.getElementById('settings-shortcut');
        if (recordBtn && shortcutInput) {
            recordBtn.addEventListener('click', () => {
                startRecording(shortcutInput, recordBtn);
            });
        }

        // Save button
        document.getElementById('save-settings-btn').addEventListener('click', saveSettings);

        // Cancel button
        document.getElementById('cancel-settings-btn').addEventListener('click', async () => {
            stopRecording();
            const window = getCurrentWindow();
            await window.close();
        });

        // Quit button
        document.getElementById('quit-app-btn').addEventListener('click', async () => {
            const confirmed = confirm('Are you sure you want to quit the application?');
            if (confirmed) {
                try {
                    await invoke('quit_app');
                } catch (error) {
                    console.error('[Settings] Failed to quit app:', error);
                }
            }
        });

        // Update buttons
        const checkUpdatesBtn = document.getElementById('check-updates-btn');
        if (checkUpdatesBtn) {
            checkUpdatesBtn.addEventListener('click', checkForUpdates);
        }

        const installUpdateBtn = document.getElementById('install-update-btn');
        if (installUpdateBtn) {
            installUpdateBtn.addEventListener('click', installUpdate);
        }

        // Setup update listener
        await setupUpdateListener();

        // Handle Escape key
        document.addEventListener('keydown', async (e) => {
            if (e.key === 'Escape' && !isRecording) {
                const window = getCurrentWindow();
                await window.close();
            }
        });

        console.log('[Settings] Initialization complete');
    } catch (error) {
        console.error('[Settings] Error during initialization:', error);
        alert('Failed to initialize settings: ' + error.message);
    }
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}

