use crate::anthropic::AnthropicProvider;
use crate::config::{ProviderConfig, ProviderType};
use crate::error::ProviderError;
use crate::gemini::GeminiProvider;
use crate::openai::OpenAiProvider;
use crate::provider::LlmProvider;

/// Instantiates an LLM provider adapter matching the specified configuration.
pub fn create_provider(config: ProviderConfig) -> Result<Box<dyn LlmProvider>, ProviderError> {
    match config.provider_type {
        ProviderType::Anthropic => Ok(Box::new(AnthropicProvider::new(config))),
        ProviderType::OpenAi | ProviderType::Ollama | ProviderType::Custom => {
            Ok(Box::new(OpenAiProvider::new(config)))
        }
        ProviderType::Gemini => Ok(Box::new(GeminiProvider::new(config))),
    }
}
