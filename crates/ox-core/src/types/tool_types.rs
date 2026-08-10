use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Unique identifier for a tool invocation request.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolCallId(pub String);

impl ToolCallId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for ToolCallId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A request from the model to execute a specific tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: ToolCallId,
    pub name: String,
    pub arguments: Value,
    pub thought_signature: Option<String>,
}

impl ToolCall {
    pub fn new(id: impl Into<String>, name: impl Into<String>, arguments: Value) -> Self {
        Self {
            id: ToolCallId::new(id),
            name: name.into(),
            arguments,
            thought_signature: None,
        }
    }

    pub fn with_thought_signature(mut self, signature: impl Into<String>) -> Self {
        self.thought_signature = Some(signature.into());
        self
    }
}

/// The outcome of executing a tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: ToolCallId,
    pub tool_name: String,
    pub content: String,
    pub is_error: bool,
}

impl ToolResult {
    pub fn success(
        call_id: ToolCallId,
        tool_name: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            call_id,
            tool_name: tool_name.into(),
            content: content.into(),
            is_error: false,
        }
    }

    pub fn error(
        call_id: ToolCallId,
        tool_name: impl Into<String>,
        error_message: impl Into<String>,
    ) -> Self {
        Self {
            call_id,
            tool_name: tool_name.into(),
            content: error_message.into(),
            is_error: true,
        }
    }
}

/// Declarative schema of a tool available to the model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    /// Indicates whether this tool performs state mutations (requires HITL approval).
    pub is_mutating: bool,
}

impl ToolDefinition {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
        is_mutating: bool,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
            is_mutating,
        }
    }
}
