use std::path::PathBuf;
use thiserror::Error;

/// Errors originating in the security and capability layer.
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Path escape detected: target '{target}' resolves outside workspace root '{root}'")]
    PathEscape { target: PathBuf, root: PathBuf },

    #[error("Target path does not exist: {0}")]
    PathNotFound(PathBuf),

    #[error("Symlink resolution failed for '{path}': {source}")]
    SymlinkError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Action was explicitly denied by human operator: {reason}")]
    ActionDenied { reason: String },

    #[error("Action violates active security policy '{policy}': {operation}")]
    PolicyViolation { policy: String, operation: String },

    #[error("I/O error during security check: {0}")]
    Io(#[from] std::io::Error),
}
