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

impl ProviderError {
    /// Returns true if the error is an HTTP 429 rate limit error.
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, ProviderError::ApiError { status: 429, .. })
    }

    /// Attempts to parse a clean, human-readable message from the provider's API error body.
    /// Falls back to formatting the error normally if parsing fails or if it's not an ApiError.
    pub fn extract_clean_message(&self) -> String {
        if let ProviderError::ApiError { status, body } = self {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
                // OpenAI, Gemini, Anthropic typically nest the human message inside `error.message`
                if let Some(msg) = json.pointer("/error/message").and_then(|m| m.as_str()) {
                    return format!("API Error [{}]: {}", status, msg);
                }
            }
            // Fallback for ApiError if JSON parsing fails or `error.message` is missing
            format!("API Error [{}]: {}", status, body)
        } else {
            // Fallback for non-ApiError variants
            self.to_string()
        }
    }
}
