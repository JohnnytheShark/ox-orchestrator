use crate::error::ToolError;
use crate::ignore::IgnoreFilter;
use async_trait::async_trait;
use ox_core::types::{ToolCallId, ToolDefinition, ToolResult};
use ox_security::{EnvScrubber, PathJail};
use serde_json::Value;
use std::sync::Arc;

/// Execution context provided to every tool invocation, ensuring sandboxing and security.
#[derive(Clone)]
pub struct ToolContext {
    pub path_jail: Arc<PathJail>,
    pub env_scrubber: Arc<EnvScrubber>,
    pub ignore_filter: Arc<IgnoreFilter>,
}

impl ToolContext {
    pub fn new(path_jail: PathJail, env_scrubber: EnvScrubber) -> Self {
        let filter = IgnoreFilter::new(path_jail.root());
        Self {
            path_jail: Arc::new(path_jail),
            env_scrubber: Arc::new(env_scrubber),
            ignore_filter: Arc::new(filter),
        }
    }

    pub fn with_ignore_filter(
        path_jail: PathJail,
        env_scrubber: EnvScrubber,
        ignore_filter: IgnoreFilter,
    ) -> Self {
        Self {
            path_jail: Arc::new(path_jail),
            env_scrubber: Arc::new(env_scrubber),
            ignore_filter: Arc::new(ignore_filter),
        }
    }
}

/// Unified trait implemented by all built-in and external tools.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Returns the declarative schema and metadata for this tool.
    fn definition(&self) -> ToolDefinition;

    /// Executes the tool with the given JSON arguments within the security context.
    async fn execute(
        &self,
        call_id: &ToolCallId,
        arguments: &Value,
        context: &ToolContext,
    ) -> Result<ToolResult, ToolError>;
}
