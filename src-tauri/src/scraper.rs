use anyhow::{anyhow, Result};
use scraper::{Html, Selector};
use html2text::from_read;

/// Scrape a website and extract text content with smart chunking
/// Returns text that can be used as LLM context
pub fn scrape_website(url: &str) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client.get(url).send()?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to fetch URL: {}", response.status()));
    }

    let html = response.text()?;
    let document = Html::parse_document(&html);

    // Try to extract main content using smart content extraction
    let main_content = extract_main_content(&document, &html);

    // Apply semantic chunking to preserve context
    let chunked = apply_semantic_chunking(&main_content);

    Ok(chunked)
}

/// Extract main content from HTML, removing boilerplate
fn extract_main_content(document: &Html, html: &str) -> String {
    // Try to find main content using common selectors
    let main_selectors = vec![
        "main",
        "article",
        "[role='main']",
        ".main-content",
        ".content",
        "#content",
        "#main",
        ".post-content",
        ".article-content",
    ];

    for selector_str in main_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                // Found main content, extract text from it
                let html_fragment = element.html();
                let text = from_read(html_fragment.as_bytes(), 100000);
                if !text.trim().is_empty() && text.len() > 200 {
                    return text;
                }
            }
        }
    }

    // If no main content found, try to remove common boilerplate elements
    // and extract from body
    let mut cleaned_html = html.to_string();

    // Remove script and style tags
    let remove_selectors = vec![
        "script", "style", "nav", "header", "footer",
        "aside", ".sidebar", "#sidebar", ".navigation",
        ".menu", ".ad", ".advertisement", ".social-share",
        ".comments", "#comments", ".cookie-notice"
    ];

    for selector_str in remove_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let element_html = element.html();
                cleaned_html = cleaned_html.replace(&element_html, "");
            }
        }
    }

    // Extract text from cleaned HTML
    let text = from_read(cleaned_html.as_bytes(), 100000);
    text
}

/// Apply semantic chunking to preserve context and fit within token limits
fn apply_semantic_chunking(text: &str) -> String {
    // Target: ~8000 tokens = ~32000 chars (4 chars per token estimate)
    const MAX_CHARS: usize = 32000;

    if text.len() <= MAX_CHARS {
        return text.to_string();
    }

    // Split into paragraphs
    let paragraphs: Vec<&str> = text.split("\n\n").collect();

    let mut result = String::new();
    let mut current_length = 0;
    let mut chunks = Vec::new();

    // Group paragraphs into semantic chunks
    for para in paragraphs {
        let para_len = para.len();

        if current_length + para_len > MAX_CHARS * 3 / 4 {
            // We've collected enough content, stop here
            break;
        }

        result.push_str(para);
        result.push_str("\n\n");
        current_length += para_len + 2;

        // Track chunks by headers (lines starting with # or all caps)
        if para.starts_with('#') || (para.len() < 100 && para.chars().all(|c| c.is_uppercase() || c.is_whitespace() || c.is_ascii_punctuation())) {
            chunks.push(current_length);
        }
    }

    // Add metadata about chunking
    if text.len() > current_length {
        let percentage = (current_length * 100) / text.len();
        result.push_str(&format!("\n\n[Content extracted: ~{}% of original page. Focused on main content and removed boilerplate.]", percentage));
    }

    result
}

