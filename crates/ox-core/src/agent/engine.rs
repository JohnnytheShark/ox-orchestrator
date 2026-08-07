use crate::context::{ContextCompactor, SystemPromptBuilder, TokenBudgeter};
use crate::session::SessionTree;
use crate::types::{ContentBlock, Message, Role, TokenUsage, ToolResult};
use std::path::{Path, PathBuf};

/// Configuration options for agent execution.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub workspace_root: PathBuf,
    pub max_turns_per_step: usize,
    pub max_context_tokens: usize,
    pub max_output_tokens: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            max_turns_per_step: 30,
            max_context_tokens: 128_000,
            max_output_tokens: 4_096,
        }
    }
}

/// The core orchestrator engine managing the agent reasoning & tool execution loop.
pub struct AgentEngine {
    pub config: AgentConfig,
    pub session: SessionTree,
    compactor: ContextCompactor,
}

impl AgentEngine {
    pub fn new(config: AgentConfig, session: SessionTree) -> Self {
        let budgeter = TokenBudgeter::new(config.max_context_tokens, config.max_output_tokens);
        let compactor = ContextCompactor::new(budgeter);

        Self {
            config,
            session,
            compactor,
        }
    }

    /// Returns the workspace root path.
    pub fn workspace_root(&self) -> &Path {
        &self.config.workspace_root
    }

    /// Prepares the full context message history ready to send to an LLM provider,
    /// ensuring system instructions are injected at index 0 and history is safely compacted.
    pub fn prepare_context(&self) -> Vec<Message> {
        let system_text = SystemPromptBuilder::build(&self.config.workspace_root);
        let system_msg = Message::system(system_text);

        let mut history = vec![system_msg];
        let session_history = self.session.linear_history();
        history.extend(session_history);

        self.compactor.compact(&history)
    }

    /// Records a user message into the session tree.
    pub fn submit_user_message(&mut self, text: impl Into<String>) -> crate::session::NodeId {
        let msg = Message::user(text);
        self.session.append(msg, None)
    }

    /// Records an assistant turn (with text, thinking, and/or tool calls) into the session tree.
    pub fn record_assistant_turn(
        &mut self,
        blocks: Vec<ContentBlock>,
        usage: Option<TokenUsage>,
    ) -> crate::session::NodeId {
        let msg = Message::new(Role::Assistant, blocks);
        self.session.append(msg, usage)
    }

    /// Records a batch of tool execution results into the session tree.
    pub fn record_tool_results(&mut self, results: Vec<ToolResult>) -> crate::session::NodeId {
        let blocks = results.into_iter().map(ContentBlock::ToolResult).collect();
        let msg = Message::new(Role::Tool, blocks);
        self.session.append(msg, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Role, TokenUsage, ToolCall, ToolResult};

    #[test]
    fn test_agent_engine_flow() {
        let config = AgentConfig::default();
        let session = SessionTree::new("test", "Test Agent");
        let mut engine = AgentEngine::new(config, session);

        engine.submit_user_message("Create a new hello world file");
        let context = engine.prepare_context();

        assert_eq!(context.len(), 2);
        assert_eq!(context[0].role, Role::System);
        assert_eq!(context[1].role, Role::User);

        let assistant_turn = vec![
            ContentBlock::text("I will create the file."),
            ContentBlock::ToolCall(ToolCall::new(
                "call_1",
                "write_file",
                serde_json::json!({
                    "path": "hello.rs",
                    "content": "fn main() {}"
                }),
            )),
        ];

        engine.record_assistant_turn(assistant_turn, Some(TokenUsage::new(100, 50)));

        let results = vec![ToolResult::success(
            crate::types::ToolCallId::new("call_1"),
            "write_file",
            "File written successfully",
        )];
        engine.record_tool_results(results);

        let history = engine.session.linear_history();
        assert_eq!(history.len(), 3); // User -> Assistant -> ToolResult
    }
}
