use thiserror::Error;

/// Provider and LLM gateway errors.
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Missing API key or credentials for provider '{0}'")]
    MissingApiKey(String),

    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Provider returned API error [status {status}]: {body}")]
    ApiError { status: u16, body: String },

    #[error("Failed to parse SSE streaming response: {0}")]
    StreamParseError(String),

    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Unsupported provider type: {0}")]
    UnsupportedProvider(String),
}
