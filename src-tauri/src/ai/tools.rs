use crate::database::DbPool;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::process::Command;

/// Execute a tool call
pub fn execute_tool(
    pool: &DbPool,
    tool_name: &str,
    arguments: &Value,
) -> Result<String> {
    match tool_name {
        "send_notification" => execute_notification(pool, arguments),
        "scrape_website" => execute_scrape_website(arguments),
        "run_command" => execute_run_command(arguments),
        _ => Err(anyhow!("Unknown tool: {}", tool_name)),
    }
}

/// Execute notification tool
fn execute_notification(pool: &DbPool, arguments: &Value) -> Result<String> {
    let message = arguments
        .get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'message' argument"))?;
    
    crate::database::create_notification(pool, message)?;
    
    Ok(format!("Notification sent: {}", message))
}

/// Execute website scrape tool
fn execute_scrape_website(arguments: &Value) -> Result<String> {
    let url = arguments
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'url' argument"))?;
    
    let content = crate::scraper::scrape_website(url)?;
    
    Ok(format!("Website content from {}:\n\n{}", url, content))
}

/// Execute run command tool
fn execute_run_command(arguments: &Value) -> Result<String> {
    let command_str = arguments
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'command' argument"))?;
    
    // Parse command (simple split for now, can be improved)
    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow!("Empty command"));
    }
    
    let program = parts[0];
    let args = &parts[1..];
    
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| anyhow!("Failed to execute command: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    let mut result = String::new();
    if !stdout.is_empty() {
        result.push_str(&format!("STDOUT:\n{}\n", stdout));
    }
    if !stderr.is_empty() {
        result.push_str(&format!("STDERR:\n{}\n", stderr));
    }
    if output.status.code().is_some() {
        result.push_str(&format!("Exit code: {}\n", output.status.code().unwrap()));
    }
    
    Ok(result)
}

