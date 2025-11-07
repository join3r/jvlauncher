use crate::database::{AgentApp, DbPool};
use crate::ai::{llm_client, queue, tools};
use anyhow::{anyhow, Result};

/// Execute an agent
pub fn execute_agent(pool: &DbPool, agent: &AgentApp, agent_name: Option<&str>) -> Result<String> {
    // Get AI settings
    let ai_settings = crate::database::get_ai_settings(pool)?;
    
    if !ai_settings.enabled {
        return Err(anyhow!("AI features are not enabled"));
    }
    
    // Determine model to use
    let model = agent
        .model
        .as_ref()
        .or(ai_settings.default_model.as_ref())
        .ok_or_else(|| anyhow!("No model specified and no default model set"))?;
    
    // Build system prompt with tool descriptions
    let mut system_prompt = agent.prompt.clone();
    
    // Add tool descriptions
    let mut tool_definitions = Vec::new();
    let mut tool_descriptions = Vec::new();
    
    if agent.tool_notification {
        tool_definitions.push(llm_client::ToolDefinition::notification());
        tool_descriptions.push("send_notification: Send a notification to the user. Use this when you need to inform the user about something important.");
    }
    
    if agent.tool_website_scrape {
        tool_definitions.push(llm_client::ToolDefinition::website_scrape());
        tool_descriptions.push("scrape_website: Scrape a website and extract its text content. The content will be provided to you as context.");
        
        // Add website URL to context if provided
        if let Some(url) = &agent.website_url {
            system_prompt.push_str(&format!("\n\nWebsite URL available for scraping: {}", url));
        }
    }
    
    if agent.tool_run_command {
        tool_definitions.push(llm_client::ToolDefinition::run_command());
        tool_descriptions.push("run_command: Run a system command. You can only run the exact command provided - you cannot modify or alter it.");
        
        // Add command to context if provided
        if let Some(cmd) = &agent.command {
            system_prompt.push_str(&format!("\n\nCommand available to run: {}", cmd));
        }
    }
    
    if !tool_descriptions.is_empty() {
        system_prompt.push_str("\n\nAvailable tools:\n");
        for desc in tool_descriptions {
            system_prompt.push_str(&format!("- {}\n", desc));
        }
    }
    
    // Build messages
    let mut messages = vec![llm_client::ChatMessage {
        role: "system".to_string(),
        content: system_prompt,
    }];
    
    // Add user message if website URL is provided and scrape is enabled
    if agent.tool_website_scrape {
        if let Some(url) = &agent.website_url {
            match crate::scraper::scrape_website(url) {
                Ok(content) => {
                    messages.push(llm_client::ChatMessage {
                        role: "user".to_string(),
                        content: format!("Please analyze the following website content from {}:\n\n{}", url, content),
                    });
                }
                Err(e) => {
                    // Continue even if scraping fails
                    eprintln!("Failed to scrape website {}: {}", url, e);
                }
            }
        }
    }
    
    // Enqueue request
    let queue_manager = queue::get_queue_manager()?;
    let message_text = serde_json::to_string(&messages).unwrap_or_default();
    let queue_id = queue_manager.enqueue(&message_text, agent_name)?;
    
    // Wait for processing slot
    while !queue_manager.can_process() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    // Start processing
    queue_manager.start_processing(queue_id)?;
    
    // Convert tool definitions
    let api_tools = if tool_definitions.is_empty() {
        None
    } else {
        Some(tool_definitions)
    };
    
    // Clone messages before using them (needed for potential second request)
    let messages_clone = messages.clone();
    
    // Make LLM request
    let response = match llm_client::chat_completion(pool, model, messages, api_tools) {
        Ok(resp) => resp,
        Err(e) => {
            queue_manager.fail(queue_id, &format!("LLM request failed: {}", e))?;
            return Err(e);
        }
    };
    
    // Process response
    if let Some(choice) = response.choices.first() {
        // Check for tool calls
        if let Some(ref tool_calls) = choice.message.tool_calls {
            // Execute tools and continue conversation
            let mut tool_results = Vec::new();
            
            for tool_call in tool_calls {
                let function_name = &tool_call.function.name;
                let arguments: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
                    .unwrap_or_else(|_| serde_json::json!({}));
                
                match tools::execute_tool(pool, function_name, &arguments) {
                    Ok(result) => {
                        // Format tool result message (OpenAI format: role="tool", content=result)
                        tool_results.push(llm_client::ChatMessage {
                            role: "tool".to_string(),
                            content: result,
                        });
                    }
                    Err(e) => {
                        // Format tool error message
                        tool_results.push(llm_client::ChatMessage {
                            role: "tool".to_string(),
                            content: format!("Error: {}", e),
                        });
                    }
                }
            }
            
            // Add tool results to messages and make another request
            let mut final_messages = messages_clone;
            final_messages.push(llm_client::ChatMessage {
                role: "assistant".to_string(),
                content: choice.message.content.clone().unwrap_or_default(),
            });
            final_messages.extend(tool_results);
            
            // Make final request
            let final_response = llm_client::chat_completion(pool, model, final_messages, None)?;
            
            if let Some(final_choice) = final_response.choices.first() {
                let final_content = final_choice.message.content.clone().unwrap_or_default();
                queue_manager.complete(queue_id, &final_content)?;
                Ok(final_content)
            } else {
                queue_manager.fail(queue_id, "No response from LLM")?;
                Err(anyhow!("No response from LLM"))
            }
        } else {
            // No tool calls, return content directly
            let content = choice.message.content.clone().unwrap_or_default();
            queue_manager.complete(queue_id, &content)?;
            Ok(content)
        }
    } else {
        queue_manager.fail(queue_id, "No choices in response")?;
        Err(anyhow!("No choices in response"))
    }
}

