use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

/// Tracks the last activity time for each webapp window
#[derive(Clone)]
pub struct WebappActivityTracker {
    /// Map of window label to (last_focus_time, timeout_minutes)
    activities: Arc<Mutex<HashMap<String, (Instant, Option<i32>)>>>,
    app_handle: AppHandle,
}

impl WebappActivityTracker {
    /// Create a new activity tracker
    pub fn new(app_handle: AppHandle) -> Self {
        let tracker = Self {
            activities: Arc::new(Mutex::new(HashMap::new())),
            app_handle: app_handle.clone(),
        };

        // Start the background timer that checks for inactive windows
        tracker.start_timer();

        tracker
    }

    /// Register a webapp window with its auto-close timeout
    /// timeout_minutes: None = disabled, Some(0) = close immediately on blur, Some(n) = close after n minutes
    pub fn register_window(&self, window_label: String, timeout_minutes: Option<i32>) {
        if let Ok(mut activities) = self.activities.lock() {
            activities.insert(window_label, (Instant::now(), timeout_minutes));
        }
    }

    /// Update the last activity time for a window (called on focus)
    pub fn update_activity(&self, window_label: &str) {
        if let Ok(mut activities) = self.activities.lock() {
            if let Some((last_time, _timeout)) = activities.get_mut(window_label) {
                *last_time = Instant::now();
            }
        }
    }

    /// Remove a window from tracking (called when window is closed)
    pub fn unregister_window(&self, window_label: &str) {
        if let Ok(mut activities) = self.activities.lock() {
            activities.remove(window_label);
        }
    }

    /// Start the background timer that periodically checks for inactive windows
    fn start_timer(&self) {
        let activities = Arc::clone(&self.activities);
        let app_handle = self.app_handle.clone();

        std::thread::spawn(move || {
            loop {
                // Check every 10 seconds
                std::thread::sleep(Duration::from_secs(10));

                if let Ok(activities_map) = activities.lock() {
                    let now = Instant::now();
                    let mut windows_to_close = Vec::new();

                    for (window_label, (last_activity, timeout_minutes)) in activities_map.iter() {
                        if let Some(timeout) = timeout_minutes {
                            // Check if the window is currently focused
                            if let Some(window) = app_handle.get_webview_window(window_label) {
                                // Only check timeout if window is not focused
                                if let Ok(is_focused) = window.is_focused() {
                                    if !is_focused {
                                        let elapsed = now.duration_since(*last_activity);
                                        
                                        // If timeout is 0, close immediately (we check every 10s, so this is close enough)
                                        // Otherwise, check if elapsed time exceeds timeout
                                        if *timeout == 0 || elapsed >= Duration::from_secs((*timeout as u64) * 60) {
                                            windows_to_close.push(window_label.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Close windows that have exceeded their timeout
                    // We do this outside the iteration to avoid holding the lock while closing
                    drop(activities_map);

                    for window_label in windows_to_close {
                        if let Some(window) = app_handle.get_webview_window(&window_label) {
                            let _ = window.close();
                        }
                    }
                }
            }
        });
    }
}

