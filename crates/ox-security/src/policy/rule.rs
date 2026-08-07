use serde::{Deserialize, Serialize};

/// Defines the security and Human-in-the-Loop policy for tool executions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SecurityPolicy {
    /// Every single tool call requires interactive human approval.
    AlwaysAsk,

    /// Safe read-only tools (read_file, grep_search, find_files) run automatically.
    /// Mutating tools (write_file, edit_file, exec_command, and external MCP tools) require human approval.
    #[default]
    AskOnMutate,

    /// All tools run without prompting (autonomous mode - for trusted non-interactive CI runs).
    AutoApprove,

    /// Blocks all tool execution.
    Deny,
}

impl SecurityPolicy {
    /// Determines whether a given tool operation requires human confirmation.
    pub fn requires_approval(&self, is_mutating: bool) -> bool {
        match self {
            SecurityPolicy::AlwaysAsk => true,
            SecurityPolicy::AskOnMutate => is_mutating,
            SecurityPolicy::AutoApprove => false,
            SecurityPolicy::Deny => true,
        }
    }
}
