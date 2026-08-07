use thiserror::Error;

/// Core domain errors for ox-orchestrator.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Session node '{0}' not found in session tree")]
    NodeNotFound(String),

    #[error("Invalid session tree state: {0}")]
    InvalidSessionState(String),

    #[error("Context limit exceeded: total tokens {total} > limit {limit}")]
    ContextLimitExceeded { total: usize, limit: usize },

    #[error("Agent execution loop exceeded maximum iterations ({0})")]
    MaxTurnsExceeded(usize),

    #[error("Agent execution was cancelled by user")]
    Cancelled,

    #[error("I/O or storage failure: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization / Deserialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Security violation in core loop: {0}")]
    Security(#[from] ox_security::SecurityError),
}
