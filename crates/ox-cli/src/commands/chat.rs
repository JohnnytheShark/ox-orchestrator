use crate::config::{ConfigResolver, OxConfigFile};
use crate::tui::{ApprovalDecision, HitlPrompter, TerminalRenderer};
use crossterm::style::Stylize;
use futures_util::StreamExt;
use ox_core::agent::{AgentConfig, AgentEngine, StreamEvent};
use ox_core::session::{NodeId, SessionStorage, SessionTree};
use ox_core::types::{ContentBlock, TokenUsage, ToolCall, ToolResult};
use ox_providers::{create_provider, ProviderConfig};
use ox_security::{EnvScrubber, PathJail};
use ox_tools::{ToolContext, ToolDispatcher};
use std::io;
use std::path::PathBuf;

pub async fn run_chat(
    provider_config: ProviderConfig,
    workspace_root: PathBuf,
    session_id: Option<String>,
    initial_prompt: Option<String>,
    mut auto_approve: bool,
    config_file: OxConfigFile,
) -> Result<(), Box<dyn std::error::Error>> {
    let jail = PathJail::new(&workspace_root)?;
    let scrubber = EnvScrubber::new();
    let tool_context = ToolContext::new(jail, scrubber);

    let mut dispatcher = ToolDispatcher::with_defaults();

    // Register external MCP servers if configured
    let mcp_servers = config_file.all_mcp_servers();
    if !mcp_servers.is_empty() {
        ConfigResolver::register_mcp_servers(&mut dispatcher, &mcp_servers).await;
    }

    let sessions_dir = workspace_root.join(".ox").join("sessions");
    let storage = SessionStorage::new(&sessions_dir)?;

    let session_name =
        session_id.unwrap_or_else(|| format!("session-{}", &uuid::Uuid::new_v4().to_string()[..8]));
    let session_tree = storage
        .load(&session_name)
        .unwrap_or_else(|_| SessionTree::new(&session_name, "Interactive Session"));

    let agent_cfg = AgentConfig {
        workspace_root: workspace_root.clone(),
        max_turns_per_step: 30,
        max_context_tokens: 128_000,
        max_output_tokens: 4_096,
    };

    let mut engine = AgentEngine::new(agent_cfg, session_tree);

    let model_name = provider_config.model.clone();
    let provider_name = format!("{:?}", provider_config.provider_type);

    TerminalRenderer::render_banner(
        &model_name,
        &provider_name,
        &engine.session.id,
        &workspace_root,
    );

    let mut provider = create_provider(provider_config)?;

    // Handle optional initial prompt
    if let Some(prompt) = initial_prompt {
        process_prompt(
            Some(&prompt),
            &mut engine,
            &*provider,
            &dispatcher,
            &tool_context,
            &storage,
            &mut auto_approve,
        )
        .await?;
    }

    // Interactive REPL loop
    loop {
        TerminalRenderer::print_user_prompt();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Handle slash commands
        if trimmed.starts_with('/') {
            let mut parts = trimmed.split_whitespace();
            let cmd = parts.next().unwrap_or_default();

            match cmd {
                "/exit" | "/quit" => {
                    storage.save(&engine.session)?;
                    println!("Session saved. Goodbye!");
                    break;
                }
                "/help" => {
                    println!("\nAvailable Slash Commands:");
                    println!("  /cost             - Display cumulative session tokens and estimated cost");
                    println!(
                        "  /diff             - Inspect colorized git diff of workspace changes"
                    );
                    println!("  /undo             - Rewind history by one step");
                    println!("  /tree             - Display ASCII DAG of branches and turns");
                    println!("  /checkout <id>    - Switch active branch to historical node ID");
                    println!("  /save             - Manually save session snapshot to disk");
                    println!("  /history          - Print linear conversation history");
                    println!(
                        "  /sidequest <msg>  - Ask a question without interrupting current flow"
                    );
                    println!("  /auto             - Toggle auto-approve for mutating tools");
                    println!("  /exit, /quit      - Save session and exit\n");
                    continue;
                }
                "/cost" => {
                    let usage = engine.session.active_token_usage();
                    let turns = engine.session.active_path().len();
                    let cost_usd =
                        ox_core::types::ModelPricing::calculate_cost(&model_name, &usage);
                    TerminalRenderer::print_cost_summary(&model_name, turns, &usage, cost_usd);
                    continue;
                }
                "/diff" => {
                    if !workspace_root.join(".git").exists() {
                        println!(
                            "Not a git repository (no .git directory found in workspace root)."
                        );
                        continue;
                    }

                    match std::process::Command::new("git")
                        .arg("diff")
                        .current_dir(&workspace_root)
                        .output()
                    {
                        Ok(output) => {
                            let diff_str = String::from_utf8_lossy(&output.stdout);
                            if diff_str.trim().is_empty() {
                                println!("No changes detected in working tree (clean git status).");
                            } else {
                                TerminalRenderer::print_diff(&diff_str);
                            }
                        }
                        Err(e) => {
                            println!("Failed to run git diff: {}", e);
                        }
                    }
                    continue;
                }
                "/undo" => {
                    if let Ok(Some(prev)) = engine.session.undo() {
                        storage.save(&engine.session)?;
                        println!("Rewound to turn: {}", prev.short());
                    } else {
                        println!("Already at root node.");
                    }
                    continue;
                }
                "/tree" => {
                    TerminalRenderer::print_ascii_dag(&engine.session);
                    continue;
                }
                "/checkout" => {
                    if let Some(id_str) = parts.next() {
                        let target = NodeId::from(id_str);
                        if engine.session.checkout(&target).is_ok() {
                            storage.save(&engine.session)?;
                            println!("Checked out node {}", target.short());
                        } else {
                            println!("Node '{}' not found.", id_str);
                        }
                    } else {
                        println!("Usage: /checkout <node_id>");
                    }
                    continue;
                }
                "/history" => {
                    println!("\n--- Linear History ---");
                    for msg in engine.session.linear_history() {
                        println!("[{:?}] {}", msg.role, msg.text_content());
                    }
                    println!("-----------------------\n");
                    continue;
                }
                "/auto" => {
                    auto_approve = !auto_approve;
                    println!("Auto-approve mutating tools: {}", auto_approve);
                    continue;
                }
                "/save" => {
                    let path = storage.save(&engine.session)?;
                    println!("Saved to {}", path.display());
                    continue;
                }
                "/sidequest" => {
                    let side_prompt = parts.collect::<Vec<_>>().join(" ");
                    if side_prompt.is_empty() {
                        println!(
                            "{}",
                            "[sidequest] Usage: /sidequest <your question>".magenta()
                        );
                        continue;
                    }

                    let original_leaf = engine.session.current_leaf_id.clone();

                    println!("{}", "[sidequest] --- Starting Sidequest ---".magenta());
                    let res = process_prompt(
                        Some(&side_prompt),
                        &mut engine,
                        &*provider,
                        &dispatcher,
                        &tool_context,
                        &storage,
                        &mut auto_approve,
                    )
                    .await;
                    println!("{}", "[sidequest] --- Sidequest Completed ---".magenta());

                    if let Some(leaf) = original_leaf {
                        let _ = engine.session.checkout(&leaf);
                    } else {
                        engine.session.current_leaf_id = None;
                    }
                    let _ = storage.save(&engine.session);
                    println!(
                        "{}",
                        "[sidequest] Returned to original session flow.".magenta()
                    );

                    res?;
                    continue;
                }
                "/retry" => {
                    process_prompt(
                        None,
                        &mut engine,
                        &*provider,
                        &dispatcher,
                        &tool_context,
                        &storage,
                        &mut auto_approve,
                    )
                    .await?;
                    continue;
                }
                "/model" => {
                    let new_model = parts.next().unwrap_or_default();
                    if new_model.is_empty() {
                        println!("Usage: /model <model_name>");
                        continue;
                    }
                    let provider_type =
                        ox_providers::ProviderType::infer_from_model_name(new_model);
                    let new_config = ox_providers::ProviderConfig::new(provider_type, new_model);

                    if provider_type != ox_providers::ProviderType::Ollama
                        && new_config.get_api_key().is_none()
                    {
                        TerminalRenderer::print_error(&format!("Cannot switch to model: No API key found for provider '{:?}'. Please configure it in your environment.", provider_type));
                        continue;
                    }

                    match ox_providers::create_provider(new_config) {
                        Ok(new_p) => {
                            provider = new_p;
                            println!("Switched model to {}", new_model);
                        }
                        Err(e) => {
                            TerminalRenderer::print_error(&format!(
                                "Failed to switch model: {}",
                                e
                            ));
                        }
                    }
                    continue;
                }
                _ => {
                    println!("Unknown command '{}'. Type /help for assistance.", cmd);
                    continue;
                }
            }
        }

        process_prompt(
            Some(trimmed),
            &mut engine,
            &*provider,
            &dispatcher,
            &tool_context,
            &storage,
            &mut auto_approve,
        )
        .await?;
    }

    Ok(())
}

async fn process_prompt(
    user_prompt: Option<&str>,
    engine: &mut AgentEngine,
    provider: &dyn ox_providers::LlmProvider,
    dispatcher: &ToolDispatcher,
    tool_context: &ToolContext,
    storage: &SessionStorage,
    auto_approve: &mut bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(prompt) = user_prompt {
        engine.submit_user_message(prompt);
    }

    let mut turn_count = 0;
    let max_turns = engine.config.max_turns_per_step;

    while turn_count < max_turns {
        turn_count += 1;
        let context_messages = engine.prepare_context();
        let tool_definitions = dispatcher.definitions();

        TerminalRenderer::print_assistant_prefix();

        let mut stream = match provider
            .stream_chat(&context_messages, &tool_definitions)
            .await
        {
            Ok(s) => s,
            Err(e) => {
                TerminalRenderer::print_error(&e.extract_clean_message());
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
                    TerminalRenderer::print_text_delta(&text);
                }
                Ok(StreamEvent::ThinkingDelta { thinking }) => {
                    accumulated_thinking.push_str(&thinking);
                    TerminalRenderer::print_thinking_delta(&thinking);
                }
                Ok(StreamEvent::ToolCallStarted { call }) => {
                    pending_tool_calls.push(call);
                }
                Ok(StreamEvent::TurnCompleted { usage: u, .. }) => {
                    usage = u;
                }
                Ok(_) => {}
                Err(e) => {
                    TerminalRenderer::print_error(&e.extract_clean_message());
                }
            }
        }

        // Build assistant content blocks
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

        let assistant_node_id = engine.record_assistant_turn(blocks, Some(usage));
        TerminalRenderer::print_turn_summary(&assistant_node_id, Some(&usage));
        storage.save(&engine.session)?;

        // If no tool calls were requested, the model has completed its response
        if pending_tool_calls.is_empty() {
            break;
        }

        // Execute tool calls
        let mut tool_results = Vec::new();

        for call in pending_tool_calls {
            TerminalRenderer::print_tool_start(&call);

            let tool_opt = dispatcher.get_tool(&call.name);
            let is_mutating = tool_opt
                .as_ref()
                .map(|t| t.definition().is_mutating)
                .unwrap_or(true);

            let approved = if is_mutating {
                match HitlPrompter::prompt_for_approval(&call, *auto_approve) {
                    ApprovalDecision::Approved => true,
                    ApprovalDecision::AlwaysApprove => {
                        *auto_approve = true;
                        true
                    }
                    ApprovalDecision::Denied => false,
                }
            } else {
                true // Safe read tools execute automatically
            };

            if approved {
                match dispatcher.execute(&call, tool_context).await {
                    Ok(res) => {
                        TerminalRenderer::print_tool_result(&res);
                        tool_results.push(res);
                    }
                    Err(e) => {
                        let err_res = ToolResult::error(
                            call.id.clone(),
                            &call.name,
                            format!("Execution failed: {}", e),
                        );
                        TerminalRenderer::print_tool_result(&err_res);
                        tool_results.push(err_res);
                    }
                }
            } else {
                let denied_res = ToolResult::error(
                    call.id.clone(),
                    &call.name,
                    "Tool execution was denied by human operator.",
                );
                TerminalRenderer::print_tool_result(&denied_res);
                tool_results.push(denied_res);
            }
        }

        engine.record_tool_results(tool_results);
        storage.save(&engine.session)?;
    }

    Ok(())
}
