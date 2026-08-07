use crate::builtin::{
    EditFileTool, ExecCommandTool, FindFilesTool, GrepSearchTool, ReadFileTool, WriteFileTool,
};
use crate::error::ToolError;
use crate::tool::{Tool, ToolContext};
use ox_core::types::{ToolCall, ToolDefinition, ToolResult};
use std::collections::HashMap;
use std::sync::Arc;

/// Central registry and routing engine for all built-in and MCP tools.
pub struct ToolDispatcher {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl Default for ToolDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolDispatcher {
    /// Creates a new empty `ToolDispatcher`.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Creates a `ToolDispatcher` initialized with all standard built-in tools.
    pub fn with_defaults() -> Self {
        let mut dispatcher = Self::new();
        dispatcher.register(Arc::new(ReadFileTool));
        dispatcher.register(Arc::new(WriteFileTool));
        dispatcher.register(Arc::new(EditFileTool));
        dispatcher.register(Arc::new(ExecCommandTool));
        dispatcher.register(Arc::new(GrepSearchTool));
        dispatcher.register(Arc::new(FindFilesTool));
        dispatcher
    }

    /// Registers a tool into the dispatcher.
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.definition().name;
        self.tools.insert(name, tool);
    }

    /// Returns list of all available tool definitions for LLM context injection.
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        let mut defs: Vec<ToolDefinition> = self.tools.values().map(|t| t.definition()).collect();
        defs.sort_by(|a, b| a.name.cmp(&b.name));
        defs
    }

    /// Looks up a tool by name.
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Dispatches a tool call for execution.
    pub async fn execute(
        &self,
        call: &ToolCall,
        context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let tool = self
            .tools
            .get(&call.name)
            .ok_or_else(|| ToolError::NotFound(call.name.clone()))?;

        tool.execute(&call.id, &call.arguments, context).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ox_security::{EnvScrubber, PathJail};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_dispatcher_defaults() {
        let dispatcher = ToolDispatcher::with_defaults();
        let defs = dispatcher.definitions();
        assert_eq!(defs.len(), 6);

        let dir = tempdir().unwrap();
        let jail = PathJail::new(dir.path()).unwrap();
        let ctx = ToolContext::new(jail, EnvScrubber::new());

        let call = ToolCall::new("1", "find_files", serde_json::json!({}));
        let res = dispatcher.execute(&call, &ctx).await.unwrap();
        assert!(!res.is_error);
    }
}
