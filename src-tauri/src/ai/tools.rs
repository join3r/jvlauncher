use crate::database::DbPool;
use anyhow::{anyhow, Result};
use serde_json::Value;
use tauri::AppHandle;

/// Execute a tool call
pub fn execute_tool(
    pool: &DbPool,
    app_handle: &AppHandle,
    tool_name: &str,
    arguments: &Value,
) -> Result<String> {
    match tool_name {
        "send_notification" => execute_notification(pool, app_handle, arguments),
        "run_command" => execute_run_command(arguments),
        _ => Err(anyhow!("Unknown tool: {}", tool_name)),
    }
}

/// Execute notification tool
fn execute_notification(pool: &DbPool, app_handle: &AppHandle, arguments: &Value) -> Result<String> {
    let message = arguments
        .get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'message' argument"))?;

    crate::database::create_notification(pool, message)?;

    // Open the notifications window to show the notification
    if let Err(e) = crate::commands::open_notifications_window(app_handle.clone()) {
        eprintln!("Failed to open notifications window: {}", e);
    }

    Ok(format!("Notification sent: {}", message))
}

/// Execute run command tool (output action)
fn execute_run_command(arguments: &Value) -> Result<String> {
    use std::process::Command;

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
    if let Some(code) = output.status.code() {
        result.push_str(&format!("Exit code: {}\n", code));
    }

    Ok(result)
}

