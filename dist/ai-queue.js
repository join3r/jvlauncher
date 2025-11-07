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

// Detect platform and apply platform-specific class
function detectPlatform() {
    try {
        const tauri = getTauriAPI();
        const root = document.documentElement;

        if (tauri && tauri.os && tauri.os.platform) {
            const platform = tauri.os.platform();
            if (platform === 'macos') {
                root.classList.add('platform-macos');
            } else {
                root.classList.add('platform-other');
            }
        } else {
            // Fallback: detect from user agent
            const userAgent = navigator.userAgent.toLowerCase();
            if (userAgent.includes('mac')) {
                root.classList.add('platform-macos');
            } else {
                root.classList.add('platform-other');
            }
        }
    } catch (error) {
        console.error('Failed to detect platform:', error);
        // Fallback
        const userAgent = navigator.userAgent.toLowerCase();
        if (userAgent.includes('mac')) {
            document.documentElement.classList.add('platform-macos');
        } else {
            document.documentElement.classList.add('platform-other');
        }
    }
}

// Apply theme
async function applyTheme() {
    try {
        const settings = await invoke('get_settings');
        const theme = settings.theme || 'system';
        const root = document.documentElement;

        if (theme === 'light') {
            root.setAttribute('data-theme', 'light');
        } else if (theme === 'dark') {
            root.setAttribute('data-theme', 'dark');
        } else {
            root.removeAttribute('data-theme');
        }
    } catch (error) {
        console.error('Failed to apply theme:', error);
    }
}

// Format timestamp
function formatTimestamp(timestamp) {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = now - date;

    // Less than 1 minute
    if (diff < 60000) {
        return 'Just now';
    }
    // Less than 1 hour
    if (diff < 3600000) {
        const mins = Math.floor(diff / 60000);
        return `${mins} min${mins > 1 ? 's' : ''} ago`;
    }
    // Less than 24 hours
    if (diff < 86400000) {
        const hours = Math.floor(diff / 3600000);
        return `${hours} hour${hours > 1 ? 's' : ''} ago`;
    }
    // Otherwise show date
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

// Load queue items
async function loadQueue() {
    try {
        const items = await invoke('get_ai_queue');
        const queueList = document.getElementById('queue-list');

        if (items.length === 0) {
            queueList.innerHTML = `
                <div class="empty-state">
                    <div class="empty-state-icon">📭</div>
                    <div class="empty-state-text">No queue items</div>
                </div>
            `;
            return;
        }

        queueList.innerHTML = items.map(item => {
            const statusClass = `status-${item.status}`;
            const agentName = item.agent_name || 'Unknown Agent';

            return `
                <div class="queue-item" data-id="${item.id}">
                    <div class="queue-item-number">#${item.id}</div>
                    <div class="queue-item-content">
                        <div class="queue-item-agent">${escapeHtml(agentName)}</div>
                        <div class="queue-item-time">${formatTimestamp(item.created_at)}</div>
                    </div>
                    <span class="status-badge ${statusClass}">${item.status}</span>
                </div>
            `;
        }).join('');

        // Add click handlers to open detail window
        document.querySelectorAll('.queue-item').forEach(item => {
            item.addEventListener('click', async () => {
                const id = parseInt(item.dataset.id);
                try {
                    await invoke('open_queue_detail_window', { queueId: id });
                } catch (error) {
                    console.error('Failed to open queue detail window:', error);
                    alert('Failed to open detail window: ' + error);
                }
            });
        });
    } catch (error) {
        console.error('Failed to load queue:', error);
        document.getElementById('queue-list').innerHTML = `
            <div class="empty-state">
                <div class="empty-state-icon">⚠️</div>
                <div class="empty-state-text">Failed to load queue: ${error}</div>
            </div>
        `;
    }
}

// Escape HTML
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Clear finished queue items
async function clearFinished() {
    try {
        await invoke('clear_finished_queue_items');
        await loadQueue();
    } catch (error) {
        console.error('Failed to clear finished items:', error);
        alert('Failed to clear finished items: ' + error);
    }
}

// Initialize
async function init() {
    detectPlatform();
    await applyTheme();
    await loadQueue();

    document.getElementById('refresh-btn').addEventListener('click', loadQueue);
    document.getElementById('clear-finished-btn').addEventListener('click', clearFinished);

    // Auto-refresh every 5 seconds
    setInterval(loadQueue, 5000);
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}

