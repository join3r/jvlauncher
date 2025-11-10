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

// Auto-resize window to fit content
async function autoResizeWindow() {
    console.log('[Settings] autoResizeWindow called');
    try {
        const window = getCurrentWindow();
        const windowLabel = window.label;
        console.log('[Settings] Window label:', windowLabel);

        // Measure the actual content container, not the body/html which have height: 100%
        const container = document.querySelector('.settings-container');
        if (!container) {
            console.warn('[Settings] Settings container not found, skipping auto-resize');
            return;
        }

        console.log('[Settings] Container found:', container);
        console.log('[Settings] Container scrollHeight:', container.scrollHeight);
        console.log('[Settings] Container offsetWidth:', container.offsetWidth);

        // Get the actual content dimensions from the container
        const contentHeight = container.scrollHeight + 36; // Add some padding for window chrome
        const contentWidth = container.offsetWidth + 40; // Add horizontal padding

        console.log('[Settings] Auto-resize - Content dimensions:', contentWidth, 'x', contentHeight);

        // Call the backend to resize the window
        await invoke('auto_resize_window', {
            windowLabel: windowLabel,
            contentWidth: contentWidth,
            contentHeight: contentHeight
        });

        console.log('[Settings] Window resized successfully');
    } catch (error) {
        console.error('[Settings] Failed to auto-resize window:', error);
    }
}

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

// Format shortcut for display with platform-specific symbols
function formatShortcutDisplay(shortcut) {
    if (!shortcut) return '';

    // On macOS, use symbols; on other platforms, use text
    if (isMacOS) {
        // Check if this is a Hyper key combination (Command+Control+Alt+Shift)
        const hasCommand = shortcut.includes('Command');
        const hasControl = shortcut.includes('Control');
        const hasAlt = shortcut.includes('Alt');
        const hasShift = shortcut.includes('Shift');

        // If all four modifiers are present, use the Hyper key symbol
        if (hasCommand && hasControl && hasAlt && hasShift) {
            // Replace all four modifiers with the Hyper symbol
            let result = shortcut
                .replace(/Command\+/g, '')
                .replace(/Control\+/g, '')
                .replace(/Alt\+/g, '')
                .replace(/Shift\+/g, '');
            return '✦' + result;
        }

        // Otherwise, replace individual modifiers with their symbols
        return shortcut
            .replace(/CommandOrControl/g, '⌘')
            .replace(/Command/g, '⌘')
            .replace(/Control/g, '⌃')
            .replace(/Alt/g, '⌥')
            .replace(/Shift/g, '⇧');
    } else {
        // On Windows/Linux, convert to more readable text
        return shortcut
            .replace(/CommandOrControl/g, 'Ctrl')
            .replace(/Command/g, 'Cmd')
            .replace(/Control/g, 'Ctrl')
            .replace(/Alt/g, 'Alt')
            .replace(/Shift/g, 'Shift');
    }
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
            // Store raw value and display formatted version
            currentRecordingInput.dataset.rawValue = shortcut;
            currentRecordingInput.value = formatShortcutDisplay(shortcut);
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

// Tab switching
function switchTab(tabName) {
    // Update tab buttons
    document.querySelectorAll('.tab').forEach(tab => {
        tab.classList.toggle('active', tab.dataset.tab === tabName);
    });
    
    // Update tab content
    document.querySelectorAll('.tab-content').forEach(content => {
        content.classList.toggle('active', content.id === `${tabName}-tab`);
    });
    
    // Auto-resize window when switching tabs
    setTimeout(() => {
        autoResizeWindow();
    }, 50);
}

// Update AI settings disabled state
function updateAISettingsDisabled() {
    const enabled = document.getElementById('ai-enabled').checked;
    const aiSettingsRows = document.getElementById('ai-settings-rows');
    const aiWarning = document.getElementById('ai-warning');

    if (enabled) {
        aiSettingsRows.classList.remove('ai-settings-disabled');
        aiWarning.classList.add('visible');
    } else {
        aiSettingsRows.classList.add('ai-settings-disabled');
        aiWarning.classList.remove('visible');
    }
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

        // Store raw shortcut value and display formatted version
        const shortcutInput = document.getElementById('settings-shortcut');
        const rawShortcut = settings.global_shortcut || 'CommandOrControl+Shift+Space';
        shortcutInput.dataset.rawValue = rawShortcut;
        shortcutInput.value = formatShortcutDisplay(rawShortcut);

        document.getElementById('settings-start-login').checked = settings.start_at_login || false;
        document.getElementById('settings-hide-app-names').checked = settings.hide_app_names || false;
        document.getElementById('settings-separate-agent-apps').checked = settings.separate_agent_apps || false;

        // Load AI settings
        try {
            const aiSettings = await invoke('get_ai_settings');
            document.getElementById('ai-enabled').checked = aiSettings.enabled || false;
            document.getElementById('ai-endpoint-url').value = aiSettings.endpoint_url || 'http://192.168.1.113:1234';
            document.getElementById('ai-api-key').value = aiSettings.api_key || '';
            document.getElementById('ai-max-concurrent').value = aiSettings.max_concurrent_agents || 1;
            
            // Load models
            await loadModels();
            
            // Set default model
            if (aiSettings.default_model) {
                document.getElementById('ai-default-model').value = aiSettings.default_model;
            }
            
            updateAISettingsDisabled();
        } catch (error) {
            console.error('[Settings] Failed to load AI settings:', error);
            // Set defaults if AI settings don't exist yet
            document.getElementById('ai-enabled').checked = false;
            document.getElementById('ai-endpoint-url').value = 'http://192.168.1.113:1234';
            document.getElementById('ai-max-concurrent').value = 1;
            updateAISettingsDisabled();
        }

        // Apply the theme to this window
        applyTheme(settings.theme || 'system');
    } catch (error) {
        console.error('[Settings] Failed to load settings:', error);
    }
}

// Load models list
async function loadModels() {
    try {
        const models = await invoke('get_models');
        const defaultModel = document.getElementById('ai-default-model');

        // Clear existing options
        defaultModel.innerHTML = '<option value="">Select default model</option>';

        if (models && models.length > 0) {
            models.forEach(model => {
                const option = document.createElement('option');
                option.value = model.id;
                option.textContent = model.id;
                defaultModel.appendChild(option);
            });
        }
    } catch (error) {
        console.error('[Settings] Failed to load models:', error);
    }
}

// Update models from endpoint
async function updateModels() {
    const btn = document.getElementById('update-models-btn');
    const originalText = btn.textContent;
    
    // Check if AI is enabled in UI (even if not saved yet)
    const aiEnabled = document.getElementById('ai-enabled').checked;
    if (!aiEnabled) {
        alert('Please enable AI features first');
        return;
    }
    
    // Get endpoint URL and API key from UI
    const endpointUrl = document.getElementById('ai-endpoint-url').value.trim();
    if (!endpointUrl) {
        alert('Please enter an endpoint URL first');
        return;
    }
    
    const apiKey = document.getElementById('ai-api-key').value.trim();
    
    try {
        btn.disabled = true;
        btn.textContent = 'Updating...';
        
        // Use a special fetch_models command that accepts endpoint URL and API key directly
        // This allows fetching models even if AI is not saved as enabled yet
        await invoke('fetch_models_with_endpoint', {
            endpointUrl: endpointUrl,
            apiKey: apiKey
        });
        await loadModels();
        
        btn.textContent = 'Updated!';
        setTimeout(() => {
            btn.textContent = originalText;
        }, 2000);
    } catch (error) {
        console.error('[Settings] Failed to update models:', error);
        alert('Failed to update models: ' + error);
        btn.textContent = originalText;
    } finally {
        btn.disabled = false;
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

            // Auto-resize window to fit the expanded update section
            requestAnimationFrame(() => {
                setTimeout(() => {
                    autoResizeWindow();
                }, 50);
            });
        } else {
            statusEl.textContent = `You're up to date! (v${updateInfo.current_version})`;
            statusEl.style.color = 'var(--text-secondary)';
            updateInfoEl.style.display = 'none';

            // Auto-resize window after hiding update section
            requestAnimationFrame(() => {
                setTimeout(() => {
                    autoResizeWindow();
                }, 50);
            });
        }
    } catch (error) {
        console.error('Failed to check for updates:', error);
        statusEl.textContent = 'Failed to check for updates';
        statusEl.style.color = '#dc3545';
        updateInfoEl.style.display = 'none';

        // Auto-resize window after hiding update section
        requestAnimationFrame(() => {
            setTimeout(() => {
                autoResizeWindow();
            }, 50);
        });
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

    const shortcutInput = document.getElementById('settings-shortcut');
    const newSettings = {
        theme: getThemeFromSegment(),
        grid_cols: parseInt(document.getElementById('settings-grid-cols').value),
        grid_rows: parseInt(document.getElementById('settings-grid-rows').value),
        global_shortcut: shortcutInput.dataset.rawValue || shortcutInput.value,
        start_at_login: document.getElementById('settings-start-login').checked,
        hide_app_names: document.getElementById('settings-hide-app-names').checked,
        separate_agent_apps: document.getElementById('settings-separate-agent-apps').checked
    };

    // Save AI settings
    try {
        await invoke('update_ai_setting', { key: 'enabled', value: document.getElementById('ai-enabled').checked ? 'true' : 'false' });
        await invoke('update_ai_setting', { key: 'endpoint_url', value: document.getElementById('ai-endpoint-url').value });
        await invoke('update_ai_setting', { key: 'api_key', value: document.getElementById('ai-api-key').value });
        await invoke('update_ai_setting', { key: 'max_concurrent_agents', value: document.getElementById('ai-max-concurrent').value });
        
        const defaultModel = document.getElementById('ai-default-model').value;
        if (defaultModel) {
            await invoke('set_default_model', { modelId: defaultModel });
        }
    } catch (error) {
        console.error('[Settings] Failed to save AI settings:', error);
        // Don't block save if AI settings fail
    }

    // Check for keyboard shortcut conflicts with app shortcuts
    if (newSettings.global_shortcut) {
        try {
            const conflict = await invoke('check_shortcut_conflict', {
                shortcut: newSettings.global_shortcut,
                excludeAppId: null
            });

            if (conflict && conflict.conflict_type === 'app') {
                alert(`Keyboard Shortcut Conflict\n\nThis keyboard shortcut is already assigned to "${conflict.app_name}".\n\nPlease choose a different shortcut or remove the shortcut from the application first.`);
                return;
            }
        } catch (error) {
            console.error('Failed to check shortcut conflict:', error);
            alert('Failed to validate keyboard shortcut: ' + error);
            return;
        }
    }

    try {
        await invoke('update_setting', { key: 'theme', value: newSettings.theme });
        await invoke('update_setting', { key: 'grid_cols', value: newSettings.grid_cols.toString() });
        await invoke('update_setting', { key: 'grid_rows', value: newSettings.grid_rows.toString() });
        await invoke('update_setting', { key: 'global_shortcut', value: newSettings.global_shortcut });
        await invoke('update_setting', { key: 'start_at_login', value: newSettings.start_at_login ? 'true' : 'false' });
        await invoke('update_setting', { key: 'hide_app_names', value: newSettings.hide_app_names ? 'true' : 'false' });
        await invoke('update_setting', { key: 'separate_agent_apps', value: newSettings.separate_agent_apps ? 'true' : 'false' });

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

        // Tab switching
        document.querySelectorAll('.tab').forEach(tab => {
            tab.addEventListener('click', () => {
                switchTab(tab.dataset.tab);
            });
        });

        // AI enabled checkbox
        document.getElementById('ai-enabled').addEventListener('change', updateAISettingsDisabled);

        // Update models button
        document.getElementById('update-models-btn').addEventListener('click', updateModels);

        // View AI queue button
        document.getElementById('view-ai-queue-btn').addEventListener('click', async () => {
            try {
                await invoke('open_ai_queue_window');
            } catch (error) {
                console.error('[Settings] Failed to open AI queue window:', error);
                alert('Failed to open AI queue window: ' + error);
            }
        });

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

        // Auto-resize window to fit content after everything is loaded
        // Use a small delay to ensure all content is rendered
        setTimeout(() => {
            autoResizeWindow();
        }, 100);
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

