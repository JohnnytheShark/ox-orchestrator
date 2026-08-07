use crate::session::NodeId;
use crate::types::{TokenUsage, ToolCall, ToolCallId, ToolResult};
use serde::{Deserialize, Serialize};

/// Real-time streaming events emitted by the agent engine to the UI/renderer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// Incremental chunk of generated assistant text.
    TextDelta { text: String },

    /// Incremental chunk of model internal chain-of-thought / reasoning.
    ThinkingDelta { thinking: String },

    /// Model initiated a tool call request.
    ToolCallStarted { call: ToolCall },

    /// Tool call was reviewed and approved for execution.
    ToolCallApproved { id: ToolCallId },

    /// Tool call was denied by human or security policy.
    ToolCallDenied { id: ToolCallId, reason: String },

    /// Execution output of a tool call.
    ToolResultEmitted { result: ToolResult },

    /// Current conversational turn finished with metadata.
    TurnCompleted { node_id: NodeId, usage: TokenUsage },

    /// An error occurred during the turn.
    Error { message: String },
}
