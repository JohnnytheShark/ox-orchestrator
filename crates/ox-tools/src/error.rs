use thiserror::Error;

/// Tool execution and MCP errors.
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool '{0}' not found")]
    NotFound(String),

    #[error("Invalid arguments for tool '{tool}': {details}")]
    InvalidArguments { tool: String, details: String },

    #[error("Execution failed for tool '{tool}': {message}")]
    ExecutionFailed { tool: String, message: String },

    #[error("MCP protocol error [{code}]: {message}")]
    McpProtocolError { code: i64, message: String },

    #[error("MCP server '{0}' communication timeout")]
    McpTimeout(String),

    #[error("I/O error during tool execution: {0}")]
    Io(#[from] std::io::Error),

    #[error("Security violation: {0}")]
    Security(#[from] ox_security::SecurityError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
