// Tauri API
const getTauriAPI = () => window.__TAURI__;

const invoke = async (cmd, args = {}) => {
    const tauri = getTauriAPI();
    if (!tauri) {
        throw new Error('Tauri API not available');
    }
    return await tauri.core.invoke(cmd, args);
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

// Escape HTML
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Parse messages from JSON
function parseMessages(messageJson) {
    try {
        const messages = JSON.parse(messageJson);
        if (!Array.isArray(messages)) {
            return null;
        }
        return messages;
    } catch (e) {
        console.error('Failed to parse messages:', e);
        return null;
    }
}

// Load and display queue item
async function loadQueueItem() {
    try {
        // Get queue item ID from URL
        const urlParams = new URLSearchParams(window.location.search);
        const queueId = urlParams.get('id');

        if (!queueId) {
            throw new Error('No queue item ID provided');
        }

        const item = await invoke('get_queue_item', { id: parseInt(queueId) });

        if (!item) {
            throw new Error('Queue item not found');
        }

        const statusClass = `status-${item.status}`;
        const agentName = item.agent_name || 'Unknown Agent';
        const messages = parseMessages(item.message);

        // Build HTML
        let html = `
            <div class="header">
                <div class="header-left">
                    <div class="title">${escapeHtml(agentName)}</div>
                    <div class="queue-number">#${item.id}</div>
                </div>
                <div class="header-right">
                    <button onclick="loadQueueItem()">Refresh</button>
                    <span class="status-badge ${statusClass}">${item.status}</span>
                </div>
            </div>
        `;

        // Info section
        html += `
            <div class="section">
                <div class="section-header">Information</div>
                <div class="section-content">
                    <div class="info-row">
                        <div class="info-label">Queue ID</div>
                        <div class="info-value">#${item.id}</div>
                    </div>
                    <div class="info-row">
                        <div class="info-label">Agent</div>
                        <div class="info-value">${escapeHtml(agentName)}</div>
                    </div>
                    <div class="info-row">
                        <div class="info-label">Status</div>
                        <div class="info-value">${item.status}</div>
                    </div>
                    <div class="info-row">
                        <div class="info-label">Created</div>
                        <div class="info-value">${formatTimestamp(item.created_at)}</div>
                    </div>
                    ${item.completed_at ? `
                    <div class="info-row">
                        <div class="info-label">Completed</div>
                        <div class="info-value">${formatTimestamp(item.completed_at)}</div>
                    </div>
                    ` : ''}
                </div>
            </div>
        `;

        // Request section
        html += `
            <div class="section">
                <div class="section-header">Request</div>
                <div class="section-content">
        `;

        if (messages && messages.length > 0) {
            html += '<div class="message-list">';
            messages.forEach(msg => {
                const roleClass = `role-${msg.role || 'unknown'}`;
                html += `
                    <div class="message-item">
                        <div class="message-role ${roleClass}">${escapeHtml(msg.role || 'unknown')}</div>
                        <div class="message-content">${escapeHtml(msg.content || '')}</div>
                    </div>
                `;
            });
            html += '</div>';
        } else {
            html += `<div class="message-content">${escapeHtml(item.message)}</div>`;
        }

        html += `
                </div>
            </div>
        `;

        // Response section
        html += `
            <div class="section">
                <div class="section-header">Response</div>
                <div class="section-content">
        `;

        if (item.response && item.response.trim() !== '') {
            html += `<div class="response-content">${escapeHtml(item.response)}</div>`;
        } else {
            html += `<div class="empty-response">No response available</div>`;
        }

        html += `
                </div>
            </div>
        `;

        document.getElementById('content').innerHTML = html;
    } catch (error) {
        console.error('Failed to load queue item:', error);
        document.getElementById('content').innerHTML = `
            <div class="error">
                <div class="error-icon">⚠️</div>
                <div>Failed to load queue item: ${error}</div>
            </div>
        `;
    }
}

// Initialize
async function init() {
    detectPlatform();
    await applyTheme();
    await loadQueueItem();
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}


