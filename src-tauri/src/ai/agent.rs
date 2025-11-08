use crate::database::{AgentApp, DbPool};
use crate::ai::{llm_client, queue, tools};
use anyhow::{anyhow, Result};
use tauri::AppHandle;

/// Execute an agent
pub fn execute_agent(pool: &DbPool, agent: &AgentApp, agent_name: Option<&str>, app_handle: &AppHandle) -> Result<String> {
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
        tool_descriptions.push("• send_notification(message: string) - Send a notification to the user with your findings or results");
    }

    if agent.tool_website_scrape {
        tool_definitions.push(llm_client::ToolDefinition::website_scrape());
        tool_descriptions.push("• scrape_website(url: string) - Scrape a website and get its text content for analysis");

        // Add website URL to context if provided
        if let Some(url) = &agent.website_url {
            system_prompt.push_str(&format!("\n\nPre-configured website URL: {}", url));
        }
    }

    if agent.tool_run_command {
        tool_definitions.push(llm_client::ToolDefinition::run_command());
        tool_descriptions.push("• run_command(command: string) - Execute the pre-configured system command and get its output");

        // Add command to context if provided
        if let Some(cmd) = &agent.command {
            system_prompt.push_str(&format!("\n\nPre-configured command: {}", cmd));
        }
    }

    if !tool_descriptions.is_empty() {
        system_prompt.push_str("\n\n=== AVAILABLE TOOLS ===\n");
        for desc in &tool_descriptions {
            system_prompt.push_str(&format!("{}\n", desc));
        }

        // Add usage instructions
        system_prompt.push_str("\n=== TOOL USAGE INSTRUCTIONS ===\n");

        if agent.tool_notification {
            system_prompt.push_str("NOTIFICATION: You MUST use send_notification to report your findings to the user. This is how you communicate results.\n");
            system_prompt.push_str("Examples:\n");
            system_prompt.push_str("  - send_notification({\"message\": \"Website check complete: Found 3 new articles about AI\"})\n");
            system_prompt.push_str("  - send_notification({\"message\": \"Error: Unable to access the website - connection timeout\"})\n");
            system_prompt.push_str("  - send_notification({\"message\": \"Command executed successfully. Exit code: 0\"})\n");
            system_prompt.push_str("  - send_notification({\"message\": \"No changes detected since last check\"})\n\n");
        }

        if agent.tool_run_command {
            if let Some(cmd) = &agent.command {
                system_prompt.push_str(&format!("RUN COMMAND: Execute '{}' to gather information, then analyze the output and send a notification with your findings.\n", cmd));
                system_prompt.push_str("Workflow:\n");
                system_prompt.push_str(&format!("  1. Call run_command({{\"command\": \"{}\"}})\n", cmd));
                system_prompt.push_str("  2. Analyze the stdout, stderr, and exit code\n");
                system_prompt.push_str("  3. Call send_notification with a summary of the results\n\n");
            }
        }

        if agent.tool_website_scrape {
            if let Some(url) = &agent.website_url {
                system_prompt.push_str(&format!("WEBSITE SCRAPE: Scrape '{}' to get current content, analyze it, then send a notification with your findings.\n", url));
                system_prompt.push_str("Workflow:\n");
                system_prompt.push_str(&format!("  1. Call scrape_website({{\"url\": \"{}\"}})\n", url));
                system_prompt.push_str("  2. Analyze the content for relevant information\n");
                system_prompt.push_str("  3. Call send_notification with a summary of what you found\n\n");
            }
        }

        system_prompt.push_str("IMPORTANT: Always end your workflow by calling send_notification to inform the user of your results!\n");
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
        Some(tool_definitions.clone())
    };

    // Debug: Log tool definitions
    if let Some(ref tools) = api_tools {
        println!("[Agent] Sending {} tools to LLM:", tools.len());
        for tool in tools {
            println!("[Agent]   - {}: {}", tool.name, tool.description);
        }
    } else {
        println!("[Agent] No tools configured");
    }

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
        // Debug: Log the response
        println!("[Agent] LLM Response - Content: {:?}", choice.message.content);
        println!("[Agent] LLM Response - Tool Calls: {:?}", choice.message.tool_calls);

        // Check for tool calls
        if let Some(ref tool_calls) = choice.message.tool_calls {
            println!("[Agent] Processing {} tool calls", tool_calls.len());

            // Execute tools and continue conversation
            let mut tool_results = Vec::new();

            for tool_call in tool_calls {
                let function_name = &tool_call.function.name;
                let arguments: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
                    .unwrap_or_else(|_| serde_json::json!({}));

                println!("[Agent] Executing tool: {} with args: {}", function_name, arguments);

                match tools::execute_tool(pool, app_handle, function_name, &arguments) {
                    Ok(result) => {
                        println!("[Agent] Tool execution success: {}", result);
                        // Format tool result message (OpenAI format: role="tool", content=result)
                        tool_results.push(llm_client::ChatMessage {
                            role: "tool".to_string(),
                            content: result,
                        });
                    }
                    Err(e) => {
                        println!("[Agent] Tool execution error: {}", e);
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
                println!("[Agent] Final response after tool execution: {}", final_content);
                queue_manager.complete(queue_id, &final_content)?;
                Ok(final_content)
            } else {
                queue_manager.fail(queue_id, "No response from LLM")?;
                Err(anyhow!("No response from LLM"))
            }
        } else {
            // No tool calls, return content directly
            println!("[Agent] No tool calls detected, returning content directly");
            let content = choice.message.content.clone().unwrap_or_default();
            queue_manager.complete(queue_id, &content)?;
            Ok(content)
        }
    } else {
        queue_manager.fail(queue_id, "No choices in response")?;
        Err(anyhow!("No choices in response"))
    }
}

