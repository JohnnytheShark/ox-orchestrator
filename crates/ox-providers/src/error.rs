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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_rate_limit() {
        let err_429 = ProviderError::ApiError {
            status: 429,
            body: "Too Many Requests".to_string(),
        };
        assert!(err_429.is_rate_limit());

        let err_500 = ProviderError::ApiError {
            status: 500,
            body: "Internal Server Error".to_string(),
        };
        assert!(!err_500.is_rate_limit());

        let err_other = ProviderError::MissingApiKey("test".to_string());
        assert!(!err_other.is_rate_limit());
    }

    #[test]
    fn test_extract_clean_message_with_valid_json() {
        let json_body = r#"{
            "error": {
                "message": "You exceeded your current quota, please check your plan and billing details.",
                "type": "insufficient_quota",
                "param": null,
                "code": "insufficient_quota"
            }
        }"#;
        
        let err = ProviderError::ApiError {
            status: 429,
            body: json_body.to_string(),
        };

        assert_eq!(
            err.extract_clean_message(),
            "API Error [429]: You exceeded your current quota, please check your plan and billing details."
        );
    }

    #[test]
    fn test_extract_clean_message_with_invalid_json() {
        let body = "Not a JSON body";
        let err = ProviderError::ApiError {
            status: 502,
            body: body.to_string(),
        };

        assert_eq!(
            err.extract_clean_message(),
            "API Error [502]: Not a JSON body"
        );
    }

    #[test]
    fn test_extract_clean_message_with_json_missing_message() {
        let json_body = r#"{ "error": { "code": 123 } }"#;
        let err = ProviderError::ApiError {
            status: 400,
            body: json_body.to_string(),
        };

        assert_eq!(
            err.extract_clean_message(),
            format!("API Error [400]: {}", json_body)
        );
    }
}
