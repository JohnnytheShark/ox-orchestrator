use ox_security::SecretString;
use serde::{Deserialize, Serialize};
use std::env;

/// The family / protocol dialect of the LLM provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    Anthropic,
    OpenAi,
    Gemini,
    Ollama,
    Custom,
}

impl ProviderType {
    /// Infers provider type from model name string (e.g. `claude-3-7-sonnet` -> Anthropic, `gpt-4o` -> OpenAi).
    pub fn infer_from_model_name(model: &str) -> Self {
        let lower = model.to_lowercase();
        if lower.starts_with("claude") {
            ProviderType::Anthropic
        } else if lower.starts_with("gpt")
            || lower.starts_with("o1")
            || lower.starts_with("o3")
            || lower.starts_with("deepseek")
        {
            ProviderType::OpenAi
        } else if lower.starts_with("gemini") {
            ProviderType::Gemini
        } else if lower.starts_with("llama")
            || lower.starts_with("mistral")
            || lower.starts_with("qwen")
        {
            ProviderType::Ollama
        } else {
            ProviderType::OpenAi
        }
    }
}

/// Configuration settings for connecting to an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: ProviderType,
    pub model: String,
    pub api_key: Option<SecretString>,
    pub base_url: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
}

impl ProviderConfig {
    pub fn new(provider_type: ProviderType, model: impl Into<String>) -> Self {
        let model_str = model.into();
        let mut config = Self {
            provider_type,
            model: model_str,
            api_key: None,
            base_url: None,
            temperature: Some(0.2),
            max_tokens: Some(4096),
        };
        config.load_env_credentials();
        config
    }

    /// Creates a configuration with an explicit API key.
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(SecretString::new(key));
        self
    }

    /// Sets custom base URL (e.g., `http://localhost:11434/v1` or OpenRouter endpoint).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Reads credentials from standard environment variables if not explicitly set.
    pub fn load_env_credentials(&mut self) {
        if self.api_key.is_some() {
            return;
        }

        let key_str = match self.provider_type {
            ProviderType::Anthropic => env::var("ANTHROPIC_API_KEY").ok(),
            ProviderType::OpenAi | ProviderType::Custom => env::var("OPENAI_API_KEY")
                .ok()
                .or_else(|| env::var("DEEPSEEK_API_KEY").ok()),
            ProviderType::Gemini => env::var("GEMINI_API_KEY")
                .ok()
                .or_else(|| env::var("GOOGLE_API_KEY").ok()),
            ProviderType::Ollama => None, // Local Ollama does not require an API key by default
        };

        if let Some(k) = key_str {
            self.api_key = Some(SecretString::new(k));
        }
    }

    /// Returns the effective API key string slice if present.
    pub fn get_api_key(&self) -> Option<&str> {
        self.api_key.as_ref().map(|k| k.expose_secret())
    }

    /// Returns the effective API base URL.
    pub fn effective_base_url(&self) -> &str {
        if let Some(url) = &self.base_url {
            return url.as_str();
        }

        match self.provider_type {
            ProviderType::Anthropic => "https://api.anthropic.com/v1",
            ProviderType::OpenAi => "https://api.openai.com/v1",
            ProviderType::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            ProviderType::Ollama => "http://localhost:11434/v1",
            ProviderType::Custom => "https://api.openai.com/v1",
        }
    }
}
