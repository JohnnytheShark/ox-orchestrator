use serde::{Deserialize, Serialize};

/// The outcome of an interactive Human-in-the-Loop approval prompt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalAction {
    /// Proceed with execution.
    Approve,

    /// Deny execution and provide feedback to the model explaining why.
    Deny { reason: String },

    /// User modified the tool call arguments before execution.
    Modify { modified_json: String },

    /// Temporarily approve all remaining operations for the active session.
    ApproveSession,
}
