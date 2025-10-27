use anyhow::{anyhow, Result};
use image::ImageFormat;
use std::path::Path;

/// Extract icon from a binary and save it to the icons directory
pub fn extract_icon_from_binary(binary_path: &str, icons_dir: &Path) -> Result<String> {
    let binary_path = Path::new(binary_path);
    
    if !binary_path.exists() {
        return Err(anyhow!("Binary path does not exist: {:?}", binary_path));
    }

    // Platform-specific icon extraction
    #[cfg(target_os = "macos")]
    {
        extract_icon_macos(binary_path, icons_dir)
    }

    #[cfg(target_os = "windows")]
    {
        extract_icon_windows(binary_path, icons_dir)
    }

    #[cfg(target_os = "linux")]
    {
        extract_icon_linux(binary_path, icons_dir)
    }
}

/// macOS: Extract icon from .app bundle or binary
#[cfg(target_os = "macos")]
fn extract_icon_macos(binary_path: &Path, icons_dir: &Path) -> Result<String> {
    use std::fs;
    use std::process::Command;

    // Check if it's an .app bundle
    let app_bundle = if binary_path.extension().and_then(|s| s.to_str()) == Some("app") {
        binary_path.to_path_buf()
    } else {
        // Try to find parent .app bundle
        let mut current = binary_path;
        loop {
            if let Some(parent) = current.parent() {
                if parent.extension().and_then(|s| s.to_str()) == Some("app") {
                    break parent.to_path_buf();
                }
                current = parent;
            } else {
                return Err(anyhow!("Could not find .app bundle for binary"));
            }
        }
    };

    // Look for .icns file in Resources
    let resources_dir = app_bundle.join("Contents").join("Resources");
    if resources_dir.exists() {
        // Find any .icns file
        if let Ok(entries) = fs::read_dir(&resources_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("icns") {
                    // Convert .icns to .png using sips command
                    let output_path = icons_dir.join(format!(
                        "{}.png",
                        app_bundle.file_stem().unwrap().to_string_lossy()
                    ));

                    let _ = Command::new("sips")
                        .args(&[
                            "-s", "format", "png",
                            path.to_str().unwrap(),
                            "--out", output_path.to_str().unwrap(),
                        ])
                        .output()?;

                    if output_path.exists() {
                        return Ok(output_path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    Err(anyhow!("Could not extract icon from macOS app"))
}

/// Windows: Extract icon from .exe file
#[cfg(target_os = "windows")]
fn extract_icon_windows(binary_path: &Path, icons_dir: &Path) -> Result<String> {
    use std::ptr;
    use winapi::um::shellapi::ExtractIconW;
    use winapi::um::winuser::{DestroyIcon, GetIconInfo, ICONINFO};
    use winapi::shared::windef::HICON;
    
    // Convert path to wide string for Windows API
    let wide_path: Vec<u16> = binary_path.to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        // Extract the first icon (index 0)
        let hicon: HICON = ExtractIconW(ptr::null_mut(), wide_path.as_ptr(), 0);
        
        if hicon.is_null() {
            return Err(anyhow!("Failed to extract icon from Windows executable"));
        }

        // Get icon info
        let mut icon_info: ICONINFO = std::mem::zeroed();
        if GetIconInfo(hicon, &mut icon_info) == 0 {
            DestroyIcon(hicon);
            return Err(anyhow!("Failed to get icon info"));
        }

        // For simplicity, we'll use a placeholder approach
        // In a full implementation, you'd convert the HBITMAP to an image
        DestroyIcon(hicon);
        
        // For now, return an error indicating manual icon selection is needed
        Err(anyhow!("Windows icon extraction requires manual implementation"))
    }
}

/// Linux: Extract icon from .desktop file or binary
#[cfg(target_os = "linux")]
fn extract_icon_linux(binary_path: &Path, icons_dir: &Path) -> Result<String> {
    use std::fs;

    // Try to find associated .desktop file
    let binary_name = binary_path.file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Invalid binary name"))?;

    // Common locations for .desktop files
    let desktop_locations = vec![
        format!("/usr/share/applications/{}.desktop", binary_name),
        format!("/usr/local/share/applications/{}.desktop", binary_name),
        format!("{}/.local/share/applications/{}.desktop", 
                std::env::var("HOME").unwrap_or_default(), binary_name),
    ];

    for desktop_path in desktop_locations {
        if let Ok(content) = fs::read_to_string(&desktop_path) {
            // Parse .desktop file for Icon= entry
            for line in content.lines() {
                if line.starts_with("Icon=") {
                    let icon_name = line.trim_start_matches("Icon=").trim();
                    
                    // Try to find the icon file
                    if let Some(icon_path) = find_icon_on_linux(icon_name) {
                        // Copy to icons directory
                        let output_path = icons_dir.join(format!("{}.png", binary_name));
                        fs::copy(&icon_path, &output_path)?;
                        return Ok(output_path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    Err(anyhow!("Could not find icon for Linux application"))
}

#[cfg(target_os = "linux")]
fn find_icon_on_linux(icon_name: &str) -> Option<PathBuf> {
    use std::fs;

    // If it's already a full path and exists, use it
    let icon_path = Path::new(icon_name);
    if icon_path.exists() {
        return Some(icon_path.to_path_buf());
    }

    // Search in common icon directories
    let icon_dirs = vec![
        "/usr/share/icons",
        "/usr/share/pixmaps",
        &format!("{}/.local/share/icons", std::env::var("HOME").unwrap_or_default()),
    ];

    let extensions = vec!["png", "svg", "xpm"];

    for base_dir in icon_dirs {
        for ext in &extensions {
            // Try direct path
            let direct_path = Path::new(base_dir).join(format!("{}.{}", icon_name, ext));
            if direct_path.exists() {
                return Some(direct_path);
            }

            // Try searching in subdirectories (hicolor theme structure)
            let hicolor_dir = Path::new(base_dir).join("hicolor");
            if hicolor_dir.exists() {
                if let Ok(entries) = fs::read_dir(&hicolor_dir) {
                    for entry in entries.flatten() {
                        let size_dir = entry.path();
                        if size_dir.is_dir() {
                            let icon_file = size_dir
                                .join("apps")
                                .join(format!("{}.{}", icon_name, ext));
                            if icon_file.exists() {
                                return Some(icon_file);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Save an icon from a user-provided image file
pub fn save_icon_from_file(source_path: &str, icons_dir: &Path, app_name: &str) -> Result<String> {

    let source = Path::new(source_path);
    if !source.exists() {
        return Err(anyhow!("Source icon file does not exist"));
    }

    // Load and convert to PNG
    let img = image::open(source)?;
    
    // Resize to 256x256 for consistency
    let resized = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
    
    // Save to icons directory
    let output_path = icons_dir.join(format!("{}.png", app_name));
    resized.save_with_format(&output_path, ImageFormat::Png)?;
    
    Ok(output_path.to_string_lossy().to_string())
}

/// Create icons directory if it doesn't exist
pub fn ensure_icons_dir(icons_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(icons_dir)?;
    Ok(())
}

