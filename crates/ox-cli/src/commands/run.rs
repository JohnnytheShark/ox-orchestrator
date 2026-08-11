use crate::config::{ConfigResolver, OxConfigFile};
use futures_util::StreamExt;
use ox_core::agent::{AgentConfig, AgentEngine, StreamEvent};
use ox_core::session::{SessionStorage, SessionTree};
use ox_core::types::{ContentBlock, TokenUsage, ToolCall, ToolResult};
use ox_providers::{create_provider, ProviderConfig};
use ox_security::{EnvScrubber, PathJail};
use ox_tools::{ToolContext, ToolDispatcher};
use std::io::{self, Write};
use std::path::PathBuf;

pub async fn run_prompt(
    provider_config: ProviderConfig,
    workspace_root: PathBuf,
    prompt: String,
    max_turns: usize,
    auto_approve: bool,
    config_file: OxConfigFile,
) -> Result<(), Box<dyn std::error::Error>> {
    let jail = PathJail::new(&workspace_root)?;
    let scrubber = EnvScrubber::new();
    let tool_context = ToolContext::new(jail, scrubber);
    let mut dispatcher = ToolDispatcher::with_defaults();

    let mcp_servers = config_file.all_mcp_servers();
    if !mcp_servers.is_empty() {
        ConfigResolver::register_mcp_servers(&mut dispatcher, &mcp_servers).await;
    }

    let session_id = format!("run-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let sessions_dir = workspace_root.join(".ox").join("sessions");
    let storage = SessionStorage::new(&sessions_dir)?;
    let session_tree = SessionTree::new(&session_id, "Non-interactive Run");

    let agent_cfg = AgentConfig {
        workspace_root: workspace_root.clone(),
        max_turns_per_step: max_turns,
        max_context_tokens: 128_000,
        max_output_tokens: 4_096,
    };

    let mut engine = AgentEngine::new(agent_cfg, session_tree);
    let provider = create_provider(provider_config)?;

    engine.submit_user_message(prompt);

    let mut turn_count = 0;

    while turn_count < max_turns {
        turn_count += 1;
        let context_messages = engine.prepare_context();
        let tool_definitions = dispatcher.definitions();

        let mut stream = match provider
            .stream_chat(&context_messages, &tool_definitions)
            .await
        {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[ERROR] {}", e.extract_clean_message());
                break;
            }
        };

        let mut accumulated_text = String::new();
        let mut accumulated_thinking = String::new();
        let mut pending_tool_calls: Vec<ToolCall> = Vec::new();
        let mut usage = TokenUsage::default();

        while let Some(event_res) = stream.next().await {
            match event_res {
                Ok(StreamEvent::TextDelta { text }) => {
                    accumulated_text.push_str(&text);
                    print!("{}", text);
                    let _ = io::stdout().flush();
                }
                Ok(StreamEvent::ThinkingDelta { thinking }) => {
                    accumulated_thinking.push_str(&thinking);
                }
                Ok(StreamEvent::ToolCallStarted { call }) => {
                    pending_tool_calls.push(call);
                }
                Ok(StreamEvent::TurnCompleted { usage: u, .. }) => {
                    usage = u;
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("\n[ERROR] {}", e.extract_clean_message());
                }
            }
        }

        let mut blocks = Vec::new();
        if !accumulated_thinking.is_empty() {
            blocks.push(ContentBlock::thinking(accumulated_thinking));
        }
        if !accumulated_text.is_empty() {
            blocks.push(ContentBlock::text(accumulated_text));
        }
        for call in &pending_tool_calls {
            blocks.push(ContentBlock::ToolCall(call.clone()));
        }

        engine.record_assistant_turn(blocks, Some(usage));
        storage.save(&engine.session)?;

        if pending_tool_calls.is_empty() {
            break;
        }

        let mut tool_results = Vec::new();
        for call in pending_tool_calls {
            let tool_opt = dispatcher.get_tool(&call.name);
            let is_mutating = tool_opt
                .as_ref()
                .map(|t| t.definition().is_mutating)
                .unwrap_or(true);

            if !is_mutating || auto_approve {
                match dispatcher.execute(&call, &tool_context).await {
                    Ok(res) => tool_results.push(res),
                    Err(e) => tool_results.push(ToolResult::error(
                        call.id.clone(),
                        &call.name,
                        format!("Failed: {}", e),
                    )),
                }
            } else {
                tool_results.push(ToolResult::error(
                    call.id.clone(),
                    &call.name,
                    "Mutating tool denied in non-interactive mode without --auto-approve (-y)",
                ));
            }
        }

        engine.record_tool_results(tool_results);
        storage.save(&engine.session)?;
    }

    println!();
    Ok(())
}
