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
    return date.toLocaleString();
}

// Load notifications
async function loadNotifications() {
    try {
        const notifications = await invoke('get_notifications', { includeDismissed: false });
        const list = document.getElementById('notifications-list');
        
        if (notifications.length === 0) {
            list.innerHTML = '<div class="empty-state">No notifications</div>';
            return;
        }
        
        list.innerHTML = notifications.map(notif => {
            const preview = notif.text.length > 100 
                ? notif.text.substring(0, 100) + '...' 
                : notif.text;
            
            return `
                <div class="notification-item" data-id="${notif.id}">
                    <div class="notification-preview">${escapeHtml(preview)}</div>
                    <div class="notification-full">${escapeHtml(notif.text)}</div>
                    <div class="notification-time">${formatTimestamp(notif.created_at)}</div>
                    <button class="btn btn-secondary" style="margin-top: 8px; width: 100%;" onclick="dismissNotification(${notif.id})">Dismiss</button>
                </div>
            `;
        }).join('');
        
        // Add click handlers to expand/collapse
        document.querySelectorAll('.notification-item').forEach(item => {
            const dismissBtn = item.querySelector('button');
            item.addEventListener('click', (e) => {
                if (e.target !== dismissBtn && !dismissBtn.contains(e.target)) {
                    item.classList.toggle('expanded');
                }
            });
        });
    } catch (error) {
        console.error('Failed to load notifications:', error);
        document.getElementById('notifications-list').innerHTML = 
            '<div class="empty-state" style="color: #dc3545;">Failed to load notifications: ' + error + '</div>';
    }
}

// Dismiss notification
async function dismissNotification(id) {
    try {
        await invoke('dismiss_notification', { id: id });
        await loadNotifications();
    } catch (error) {
        console.error('Failed to dismiss notification:', error);
        alert('Failed to dismiss notification: ' + error);
    }
}

// Dismiss all notifications
async function dismissAll() {
    if (!confirm('Dismiss all notifications?')) {
        return;
    }
    
    try {
        await invoke('dismiss_all_notifications');
        await loadNotifications();
    } catch (error) {
        console.error('Failed to dismiss all notifications:', error);
        alert('Failed to dismiss all notifications: ' + error);
    }
}

// Escape HTML
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Make dismissNotification available globally
window.dismissNotification = dismissNotification;

// Initialize
async function init() {
    detectPlatform();
    await applyTheme();
    await loadNotifications();

    document.getElementById('dismiss-all-btn').addEventListener('click', dismissAll);

    // Auto-refresh every 2 seconds
    setInterval(loadNotifications, 2000);
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}

