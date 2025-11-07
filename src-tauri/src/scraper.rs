use anyhow::{anyhow, Result};
use scraper::Html;
use html2text::from_read;

/// Scrape a website and extract text content
/// Returns text that can be used as LLM context
pub fn scrape_website(url: &str) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;
    
    let response = client.get(url).send()?;
    
    if !response.status().is_success() {
        return Err(anyhow!("Failed to fetch URL: {}", response.status()));
    }
    
    let html = response.text()?;
    let _document = Html::parse_document(&html);
    
    // Note: We parse the document but html2text will handle extracting text content
    
    // Extract text content
    let text = from_read(html.as_bytes(), 100000); // Limit to ~100k chars
    
    // Truncate if too long (rough estimate for token count)
    // Assume ~4 chars per token, so 8000 tokens = ~32000 chars
    const MAX_CHARS: usize = 32000;
    let truncated = if text.len() > MAX_CHARS {
        format!("{}...\n\n[Content truncated due to length]", &text[..MAX_CHARS])
    } else {
        text
    };
    
    Ok(truncated)
}

