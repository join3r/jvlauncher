use crate::database::{AIModel, DbPool};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// OpenAI-compatible models response
#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelData>,
}

#[derive(Debug, Deserialize)]
struct ModelData {
    id: String,
    created: Option<i64>,
}

/// Chat completion request
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct Tool {
    #[serde(rename = "type")]
    tool_type: String,
    function: ToolFunction,
}

#[derive(Debug, Serialize)]
struct ToolFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

/// Chat completion response
#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolCallFunction,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

/// Fetch available models from the endpoint
pub fn fetch_models(pool: &DbPool) -> Result<Vec<AIModel>> {
    let settings = crate::database::get_ai_settings(pool)?;
    
    if !settings.enabled {
        return Err(anyhow!("AI features are not enabled"));
    }

    fetch_models_from_endpoint(pool, &settings.endpoint_url, &settings.api_key)
}

/// Fetch models from a specific endpoint (allows fetching even if AI not enabled in settings)
pub fn fetch_models_from_endpoint(pool: &DbPool, endpoint_url: &str, api_key: &str) -> Result<Vec<AIModel>> {
    let url = format!("{}/v1/models", endpoint_url.trim_end_matches('/'));
    
    let client = reqwest::blocking::Client::new();
    let mut request = client.get(&url);
    
    if !api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }
    
    let response = request.send()?;
    
    let status_code = response.status();
    if !status_code.is_success() {
        return Err(anyhow!("Failed to fetch models: {}", status_code));
    }
    
    let models_response: ModelsResponse = response.json()?;
    
    let models: Vec<AIModel> = models_response
        .data
        .into_iter()
        .map(|m| AIModel {
            id: m.id,
            created: m.created,
        })
        .collect();
    
    // Save models to database
    crate::database::save_models(pool, models.clone())?;
    
    Ok(models)
}

/// Send a chat completion request
pub fn chat_completion(
    pool: &DbPool,
    model: &str,
    messages: Vec<ChatMessage>,
    tools: Option<Vec<ToolDefinition>>,
) -> Result<ChatCompletionResponse> {
    let settings = crate::database::get_ai_settings(pool)?;
    
    if !settings.enabled {
        return Err(anyhow!("AI features are not enabled"));
    }

    let url = format!("{}/v1/chat/completions", settings.endpoint_url.trim_end_matches('/'));
    
    let client = reqwest::blocking::Client::new();
    let mut request_builder = client.post(&url);
    
    if !settings.api_key.is_empty() {
        request_builder = request_builder.header("Authorization", format!("Bearer {}", settings.api_key));
    }
    
    // Convert tool definitions to API format
    let api_tools = tools.map(|defs| {
        defs.into_iter()
            .map(|def| Tool {
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: def.name,
                    description: def.description,
                    parameters: def.parameters,
                },
            })
            .collect()
    });
    
    let request_body = ChatCompletionRequest {
        model: model.to_string(),
        messages,
        tools: api_tools,
    };
    
    let response = request_builder
        .json(&request_body)
        .send()?;
    
    let status_code = response.status();
    if !status_code.is_success() {
        // Consume response to get error text
        let error_text = response.text().unwrap_or_default();
        return Err(anyhow!("Failed to get chat completion: {} - {}", status_code, error_text));
    }
    
    // Parse JSON response (only reached if status is success)
    let completion: ChatCompletionResponse = response.json()?;
    
    Ok(completion)
}

/// Tool definition for LLM
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

impl ToolDefinition {
    /// Create notification tool
    pub fn notification() -> Self {
        Self {
            name: "send_notification".to_string(),
            description: "Send a notification to the user. Use this when you need to inform the user about something important.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The notification message to display to the user"
                    }
                },
                "required": ["message"]
            }),
        }
    }
    
    /// Website scrape tool
    pub fn website_scrape() -> Self {
        Self {
            name: "scrape_website".to_string(),
            description: "Scrape a website and extract its text content. The content will be provided to you as context. Use this when you need information from a specific URL.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL of the website to scrape"
                    }
                },
                "required": ["url"]
            }),
        }
    }
    
    /// Run command tool
    pub fn run_command() -> Self {
        Self {
            name: "run_command".to_string(),
            description: "Run a system command. You can only run the exact command provided - you cannot modify or alter it. Use this when you need to execute a specific command.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The exact command to run (cannot be modified)"
                    }
                },
                "required": ["command"]
            }),
        }
    }
}

