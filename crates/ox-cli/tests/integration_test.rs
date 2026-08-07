use async_trait::async_trait;
use futures_util::Stream;
use ox_core::agent::{AgentConfig, AgentEngine, StreamEvent};
use ox_core::session::SessionTree;
use ox_core::types::{ContentBlock, Message, ToolCall, ToolDefinition};
use ox_providers::{LlmProvider, ProviderConfig, ProviderError, ProviderType};
use ox_security::{EnvScrubber, PathJail};
use ox_tools::{ToolContext, ToolDispatcher};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

/// Mock provider simulating an agent creating a file and then answering.
struct MockAgentProvider {
    config: ProviderConfig,
    turn: AtomicUsize,
}

impl MockAgentProvider {
    fn new() -> Self {
        Self {
            config: ProviderConfig::new(ProviderType::Custom, "mock-model"),
            turn: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl LlmProvider for MockAgentProvider {
    async fn stream_chat(
        &self,
        _messages: &[Message],
        _tools: &[ToolDefinition],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, ProviderError>> + Send>>, ProviderError>
    {
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        let turn = self.turn.fetch_add(1, Ordering::SeqCst);

        tokio::spawn(async move {
            if turn == 0 {
                // First turn: model requests tool call
                let call = ToolCall::new(
                    "call_mock_1",
                    "write_file",
                    serde_json::json!({
                        "path": "test_output.txt",
                        "content": "Hello from Ox secure harness!"
                    }),
                );
                let _ = tx
                    .send(Ok(StreamEvent::TextDelta {
                        text: "I will create test_output.txt\n".to_string(),
                    }))
                    .await;
                let _ = tx.send(Ok(StreamEvent::ToolCallStarted { call })).await;
                let _ = tx
                    .send(Ok(StreamEvent::TurnCompleted {
                        node_id: ox_core::session::NodeId::new(),
                        usage: ox_core::types::TokenUsage::new(50, 25),
                    }))
                    .await;
            } else {
                // Second turn: model concludes after tool execution
                let _ = tx
                    .send(Ok(StreamEvent::TextDelta {
                        text: "File test_output.txt was created successfully.".to_string(),
                    }))
                    .await;
                let _ = tx
                    .send(Ok(StreamEvent::TurnCompleted {
                        node_id: ox_core::session::NodeId::new(),
                        usage: ox_core::types::TokenUsage::new(100, 30),
                    }))
                    .await;
            }
        });

        // Use custom stream wrapper around tokio mpsc channel
        struct SimpleChannelStream {
            rx: tokio::sync::mpsc::Receiver<Result<StreamEvent, ProviderError>>,
        }

        impl Stream for SimpleChannelStream {
            type Item = Result<StreamEvent, ProviderError>;

            fn poll_next(
                mut self: Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Option<Self::Item>> {
                self.rx.poll_recv(cx)
            }
        }

        Ok(Box::pin(SimpleChannelStream { rx }))
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }
}

#[tokio::test]
async fn test_end_to_end_agent_execution_loop() {
    let dir = tempdir().unwrap();
    let workspace = dir.path();

    let jail = PathJail::new(workspace).unwrap();
    let scrubber = EnvScrubber::new();
    let tool_context = ToolContext::new(jail, scrubber);
    let dispatcher = ToolDispatcher::with_defaults();

    let session = SessionTree::new("test_e2e", "E2E Test Session");
    let agent_cfg = AgentConfig {
        workspace_root: workspace.to_path_buf(),
        max_turns_per_step: 5,
        max_context_tokens: 8192,
        max_output_tokens: 1024,
    };

    let mut engine = AgentEngine::new(agent_cfg, session);
    let provider = Arc::new(MockAgentProvider::new());

    // 1. Submit user prompt
    engine.submit_user_message("Please write test_output.txt");

    // Turn 1: Run LLM turn
    let ctx = engine.prepare_context();
    let mut stream = provider
        .stream_chat(&ctx, &dispatcher.definitions())
        .await
        .unwrap();

    let mut tool_calls = Vec::new();
    use futures_util::StreamExt;

    while let Some(Ok(event)) = stream.next().await {
        if let StreamEvent::ToolCallStarted { call } = event {
            tool_calls.push(call);
        }
    }

    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0].name, "write_file");

    // Record turn
    let blocks = vec![
        ContentBlock::text("I will create test_output.txt\n"),
        ContentBlock::ToolCall(tool_calls[0].clone()),
    ];
    engine.record_assistant_turn(blocks, None);

    // Execute tool
    let res = dispatcher
        .execute(&tool_calls[0], &tool_context)
        .await
        .unwrap();
    assert!(!res.is_error);

    // Verify file actually exists in workspace
    let written_file = workspace.join("test_output.txt");
    assert!(written_file.exists());
    let content = std::fs::read_to_string(&written_file).unwrap();
    assert_eq!(content, "Hello from Ox secure harness!");

    // Record tool result
    engine.record_tool_results(vec![res]);

    // Turn 2: LLM receives tool result
    let ctx2 = engine.prepare_context();
    let mut stream2 = provider
        .stream_chat(&ctx2, &dispatcher.definitions())
        .await
        .unwrap();
    let mut final_text = String::new();

    while let Some(Ok(event)) = stream2.next().await {
        if let StreamEvent::TextDelta { text } = event {
            final_text.push_str(&text);
        }
    }

    assert_eq!(final_text, "File test_output.txt was created successfully.");

    // Record turn 2
    engine.record_assistant_turn(vec![ContentBlock::text(final_text)], None);

    // Verify history DAG structure
    let history = engine.session.linear_history();
    assert_eq!(history.len(), 4); // User -> Assistant(ToolCall) -> ToolResult -> Assistant(Final)
}
