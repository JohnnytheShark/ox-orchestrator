use crate::config::ProviderConfig;
use crate::error::ProviderError;
use futures_util::Stream;
use ox_core::agent::StreamEvent;
use ox_core::types::{Message, ToolDefinition};
use std::pin::Pin;

/// Pinned asynchronous stream of agent events emitted during generation.
pub type ProviderStream = Pin<Box<dyn Stream<Item = Result<StreamEvent, ProviderError>> + Send>>;

/// Trait implemented by all LLM client adapters.
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    /// Dispatches a chat completion request with streaming events.
    async fn stream_chat(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<ProviderStream, ProviderError>;

    /// Returns the provider configuration.
    fn config(&self) -> &ProviderConfig;
}
