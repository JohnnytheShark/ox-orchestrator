use std::collections::HashMap;
use std::env;

/// Sanitizes environment variables to prevent leaking credentials and API keys
/// to child subprocesses or untrusted MCP servers.
#[derive(Debug, Clone, Default)]
pub struct EnvScrubber {
    additional_blocked_keys: Vec<String>,
}

impl EnvScrubber {
    /// Creates a new `EnvScrubber`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a key pattern to the blacklist.
    pub fn block_key(&mut self, key: impl Into<String>) {
        self.additional_blocked_keys.push(key.into().to_uppercase());
    }

    /// Checks if a given environment variable key is considered sensitive.
    pub fn is_sensitive_key(&self, key: &str) -> bool {
        let upper = key.to_uppercase();

        // 1. Explicit API keys and vendor tokens
        let sensitive_prefixes = [
            "OPENAI_",
            "ANTHROPIC_",
            "GEMINI_",
            "DEEPSEEK_",
            "GROQ_",
            "MISTRAL_",
            "COHERE_",
            "AWS_",
            "AZURE_",
            "GCP_",
            "GITHUB_TOKEN",
            "GH_TOKEN",
            "GITLAB_TOKEN",
            "SLACK_",
            "DISCORD_",
        ];

        for prefix in sensitive_prefixes {
            if upper.starts_with(prefix) {
                return true;
            }
        }

        // 2. Generic secret substrings
        let sensitive_substrings = [
            "API_KEY",
            "APIKEY",
            "SECRET",
            "TOKEN",
            "PASSWORD",
            "PASSWD",
            "AUTH",
            "CREDENTIAL",
            "PRIVATE_KEY",
        ];

        for sub in sensitive_substrings {
            if upper.contains(sub) {
                return true;
            }
        }

        // 3. User-defined blocked keys
        for blocked in &self.additional_blocked_keys {
            if upper.contains(blocked) {
                return true;
            }
        }

        false
    }

    /// Returns a clean map of environment variables safe for subprocess execution.
    pub fn sanitize_current_env(&self) -> HashMap<String, String> {
        let mut clean_env = HashMap::new();

        for (k, v) in env::vars() {
            if !self.is_sensitive_key(&k) {
                clean_env.insert(k, v);
            }
        }

        clean_env
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifies_sensitive_keys() {
        let scrubber = EnvScrubber::new();
        assert!(scrubber.is_sensitive_key("OPENAI_API_KEY"));
        assert!(scrubber.is_sensitive_key("ANTHROPIC_API_KEY"));
        assert!(scrubber.is_sensitive_key("GEMINI_API_KEY"));
        assert!(scrubber.is_sensitive_key("AWS_SECRET_ACCESS_KEY"));
        assert!(scrubber.is_sensitive_key("GITHUB_TOKEN"));
        assert!(scrubber.is_sensitive_key("MY_APP_PASSWORD"));
        assert!(scrubber.is_sensitive_key("AUTH_BEARER"));

        assert!(!scrubber.is_sensitive_key("PATH"));
        assert!(!scrubber.is_sensitive_key("HOME"));
        assert!(!scrubber.is_sensitive_key("USER"));
        assert!(!scrubber.is_sensitive_key("LANG"));
    }
}
