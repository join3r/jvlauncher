use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub body: Option<String>,
    pub date: Option<String>,
}

/// Check for available updates
#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<UpdateInfo, String> {
    log::info!("Checking for updates...");
    
    let current_version = app.package_info().version.to_string();
    
    match app.updater() {
        Ok(updater) => {
            match updater.check().await {
                Ok(update_response) => {
                    if let Some(update) = update_response {
                        log::info!("Update available: {}", update.version);
                        
                        Ok(UpdateInfo {
                            available: true,
                            current_version,
                            latest_version: Some(update.version.clone()),
                            body: update.body.clone(),
                            date: update.date.map(|d| d.to_string()),
                        })
                    } else {
                        log::info!("No updates available");
                        Ok(UpdateInfo {
                            available: false,
                            current_version,
                            latest_version: None,
                            body: None,
                            date: None,
                        })
                    }
                }
                Err(e) => {
                    log::error!("Failed to check for updates: {}", e);
                    Err(format!("Failed to check for updates: {}", e))
                }
            }
        }
        Err(e) => {
            log::error!("Failed to get updater: {}", e);
            Err(format!("Failed to get updater: {}", e))
        }
    }
}

/// Download and install update
#[tauri::command]
pub async fn download_and_install_update(app: AppHandle) -> Result<(), String> {
    log::info!("Starting update download and installation...");
    
    match app.updater() {
        Ok(updater) => {
            match updater.check().await {
                Ok(update_response) => {
                    if let Some(update) = update_response {
                        log::info!("Downloading update version: {}", update.version);
                        
                        // Download and install the update
                        match update.download_and_install(
                            |chunk_length, content_length| {
                                if let Some(total) = content_length {
                                    let progress = (chunk_length as f64 / total as f64) * 100.0;
                                    log::debug!("Download progress: {:.2}%", progress);
                                }
                            },
                            || {
                                log::info!("Download complete, installing...");
                            }
                        ).await {
                            Ok(_) => {
                                log::info!("Update installed successfully");
                                Ok(())
                            }
                            Err(e) => {
                                log::error!("Failed to download/install update: {}", e);
                                Err(format!("Failed to download/install update: {}", e))
                            }
                        }
                    } else {
                        Err("No update available".to_string())
                    }
                }
                Err(e) => {
                    log::error!("Failed to check for updates: {}", e);
                    Err(format!("Failed to check for updates: {}", e))
                }
            }
        }
        Err(e) => {
            log::error!("Failed to get updater: {}", e);
            Err(format!("Failed to get updater: {}", e))
        }
    }
}

/// Check for updates silently on startup
pub async fn check_updates_on_startup(app: AppHandle) {
    log::info!("Performing startup update check...");
    
    // Wait a bit before checking to not slow down startup
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    match check_for_updates(app.clone()).await {
        Ok(update_info) => {
            if update_info.available {
                log::info!("Update available on startup: {:?}", update_info.latest_version);
                
                // Emit event to frontend to show update notification
                if let Err(e) = app.emit("update-available", &update_info) {
                    log::error!("Failed to emit update-available event: {}", e);
                }
            } else {
                log::info!("No updates available on startup");
            }
        }
        Err(e) => {
            log::warn!("Startup update check failed: {}", e);
        }
    }
}

