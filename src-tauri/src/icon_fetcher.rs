use anyhow::{anyhow, Result};
use scraper::{Html, Selector};
use std::path::Path;
use std::fs;
use url::Url;

/// Fetch and save a website's icon (favicon, apple-touch-icon, etc.)
pub fn fetch_web_icon(url_str: &str, icons_dir: &Path, app_name: &str) -> Result<String> {
    // Parse and validate the URL
    let base_url = Url::parse(url_str)
        .map_err(|e| anyhow!("Invalid URL: {}", e))?;
    
    // Ensure icons directory exists
    fs::create_dir_all(icons_dir)?;
    
    // Try to fetch the HTML page
    let html = fetch_html(&base_url)?;
    
    // Try different icon sources in order of preference
    let icon_url = find_apple_touch_icon(&html, &base_url)
        .or_else(|| find_high_res_favicon(&html, &base_url))
        .or_else(|| find_standard_favicon(&html, &base_url))
        .or_else(|| find_og_image(&html, &base_url))
        .or_else(|| Some(default_favicon_url(&base_url)))
        .ok_or_else(|| anyhow!("Could not find any icon for the website"))?;
    
    // Download the icon
    let icon_data = download_icon(&icon_url)?;
    
    // Save the icon to the icons directory
    let output_path = icons_dir.join(format!("{}.png", sanitize_filename(app_name)));
    save_icon_data(&icon_data, &output_path)?;
    
    Ok(output_path.to_string_lossy().to_string())
}

/// Fetch HTML content from a URL
fn fetch_html(url: &Url) -> Result<String> {
    let response = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()?
        .get(url.as_str())
        .send()?;
    
    if !response.status().is_success() {
        return Err(anyhow!("HTTP request failed with status: {}", response.status()));
    }
    
    Ok(response.text()?)
}

/// Find Apple Touch Icon (highest quality, preferred for web apps)
fn find_apple_touch_icon(html: &str, base_url: &Url) -> Option<String> {
    let document = Html::parse_document(html);
    
    // Try to find apple-touch-icon with sizes attribute (prefer larger sizes)
    let selector = Selector::parse("link[rel~='apple-touch-icon']").ok()?;
    let mut icons: Vec<(Option<u32>, String)> = Vec::new();
    
    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            let size = element.value().attr("sizes")
                .and_then(|s| s.split('x').next())
                .and_then(|s| s.parse::<u32>().ok());
            
            if let Ok(icon_url) = base_url.join(href) {
                icons.push((size, icon_url.to_string()));
            }
        }
    }
    
    // Sort by size (largest first) and return the largest
    icons.sort_by(|a, b| b.0.cmp(&a.0));
    icons.first().map(|(_, url)| url.clone())
}

/// Find high-resolution favicon
fn find_high_res_favicon(html: &str, base_url: &Url) -> Option<String> {
    let document = Html::parse_document(html);
    
    // Look for icon links with sizes attribute
    let selector = Selector::parse("link[rel~='icon'][sizes]").ok()?;
    let mut icons: Vec<(u32, String)> = Vec::new();
    
    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if let Some(sizes) = element.value().attr("sizes") {
                // Parse size like "192x192" or "any"
                if sizes == "any" {
                    // SVG icons - give them high priority
                    if let Ok(icon_url) = base_url.join(href) {
                        return Some(icon_url.to_string());
                    }
                } else if let Some(size_str) = sizes.split('x').next() {
                    if let Ok(size) = size_str.parse::<u32>() {
                        if let Ok(icon_url) = base_url.join(href) {
                            icons.push((size, icon_url.to_string()));
                        }
                    }
                }
            }
        }
    }
    
    // Sort by size (largest first) and return the largest
    icons.sort_by(|a, b| b.0.cmp(&a.0));
    icons.first().map(|(_, url)| url.clone())
}

/// Find standard favicon
fn find_standard_favicon(html: &str, base_url: &Url) -> Option<String> {
    let document = Html::parse_document(html);
    
    // Look for any icon link
    let selector = Selector::parse("link[rel~='icon']").ok()?;
    
    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if let Ok(icon_url) = base_url.join(href) {
                return Some(icon_url.to_string());
            }
        }
    }
    
    None
}

/// Find Open Graph image as fallback
fn find_og_image(html: &str, base_url: &Url) -> Option<String> {
    let document = Html::parse_document(html);
    
    let selector = Selector::parse("meta[property='og:image']").ok()?;
    
    for element in document.select(&selector) {
        if let Some(content) = element.value().attr("content") {
            if let Ok(icon_url) = base_url.join(content) {
                return Some(icon_url.to_string());
            }
        }
    }
    
    None
}

/// Get default favicon.ico URL
fn default_favicon_url(base_url: &Url) -> String {
    format!("{}://{}/favicon.ico", base_url.scheme(), base_url.host_str().unwrap_or(""))
}

/// Download icon from URL
fn download_icon(url: &str) -> Result<Vec<u8>> {
    let response = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()?
        .get(url)
        .send()?;
    
    if !response.status().is_success() {
        return Err(anyhow!("Failed to download icon: HTTP {}", response.status()));
    }
    
    Ok(response.bytes()?.to_vec())
}

/// Save icon data to file, converting to PNG if necessary
fn save_icon_data(data: &[u8], output_path: &Path) -> Result<()> {
    // Try to load the image and convert to PNG
    match image::load_from_memory(data) {
        Ok(img) => {
            // Convert to PNG and save
            img.save(output_path)?;
            Ok(())
        }
        Err(_) => {
            // If image loading fails, try to save as-is (might be SVG or other format)
            // For now, we'll just return an error since we want PNG output
            Err(anyhow!("Failed to load image data - unsupported format"))
        }
    }
}

/// Sanitize filename to remove invalid characters
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("My App"), "My App");
        assert_eq!(sanitize_filename("My/App"), "My_App");
        assert_eq!(sanitize_filename("My:App*"), "My_App_");
    }

    #[test]
    fn test_default_favicon_url() {
        let url = Url::parse("https://example.com/some/path").unwrap();
        assert_eq!(default_favicon_url(&url), "https://example.com/favicon.ico");
    }
}

