use crate::config::ProviderConfig;
use crate::error::ProviderError;
use crate::gemini::mapper::{convert_messages, convert_tools, GeminiGenConfig, GeminiRequest};
use crate::provider::{LlmProvider, ProviderStream};
use crate::stream::ChannelStream;
use async_trait::async_trait;
use futures_util::TryStreamExt;
use ox_core::agent::StreamEvent;
use ox_core::types::{Message, ToolCall, ToolDefinition};
use reqwest::Client;
use serde_json::Value;
use tokio::sync::mpsc;

pub struct GeminiProvider {
    config: ProviderConfig,
    client: Client,
}

impl GeminiProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

/// Finds the position of a `\n\n` or `\r\n\r\n` SSE event boundary in `buf`.
/// Returns `(position_of_separator_start, length_of_separator)`.
fn find_sse_boundary(buf: &[u8]) -> Option<(usize, usize)> {
    // Prefer \r\n\r\n first (4-byte boundary used by Gemini)
    if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
        return Some((pos, 4));
    }
    // Fall back to \n\n (2-byte standard SSE boundary)
    if let Some(pos) = buf.windows(2).position(|w| w == b"\n\n") {
        return Some((pos, 2));
    }
    // Fall back to \r\r
    if let Some(pos) = buf.windows(2).position(|w| w == b"\r\r") {
        return Some((pos, 2));
    }
    None
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn stream_chat(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<ProviderStream, ProviderError> {
        let api_key = self
            .config
            .get_api_key()
            .ok_or_else(|| ProviderError::MissingApiKey("Gemini".to_string()))?;

        let (system_instruction, contents) = convert_messages(messages);
        let gemini_tools = convert_tools(tools);

        let request_payload = GeminiRequest {
            contents,
            system_instruction,
            tools: gemini_tools,
            generation_config: Some(GeminiGenConfig {
                temperature: self.config.temperature,
                max_output_tokens: self.config.max_tokens,
            }),
        };

        let url = format!(
            "{}/models/{}:streamGenerateContent?alt=sse&key={}",
            self.config.effective_base_url(),
            self.config.model,
            api_key
        );

        let response = self
            .client
            .post(&url)
            .header("content-type", "application/json")
            .json(&request_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError { status, body });
        }

        let (tx, rx) = mpsc::channel(100);
        let mut byte_stream = response.bytes_stream();

        tokio::spawn(async move {
            let mut buffer = Vec::new();
            let mut usage = ox_core::types::TokenUsage::default();

            while let Ok(Some(chunk)) = byte_stream.try_next().await {
                buffer.extend_from_slice(&chunk);

                while let Some((pos, sep_len)) = find_sse_boundary(&buffer) {
                    let event_bytes = buffer[..pos].to_vec();
                    buffer.drain(..pos + sep_len);

                    let event_block = String::from_utf8_lossy(&event_bytes);
                    for line in event_block.lines() {
                        // Strip any trailing \r left from \r\n line endings
                        let line = line.trim_end_matches('\r');
                        if let Some(data) = line.strip_prefix("data: ") {
                            if let Ok(v) = serde_json::from_str::<Value>(data.trim()) {
                                // Parse token usage metadata
                                if let Some(usage_metadata) = v.get("usageMetadata") {
                                    if let Some(pt) = usage_metadata
                                        .get("promptTokenCount")
                                        .and_then(|t| t.as_u64())
                                    {
                                        usage.input_tokens = pt as usize;
                                    }
                                    if let Some(ct) = usage_metadata
                                        .get("candidatesTokenCount")
                                        .and_then(|t| t.as_u64())
                                    {
                                        usage.output_tokens = ct as usize;
                                    }
                                }

                                if let Some(candidates) =
                                    v.get("candidates").and_then(|c| c.as_array())
                                {
                                    for candidate in candidates {
                                        if let Some(content) = candidate.get("content") {
                                            if let Some(parts) =
                                                content.get("parts").and_then(|p| p.as_array())
                                            {
                                                for part in parts {
                                                    // Text response — skip empty strings
                                                    if let Some(text) = part
                                                        .get("text")
                                                        .and_then(|t| t.as_str())
                                                        .filter(|t| !t.is_empty())
                                                    {
                                                        let _ = tx
                                                            .send(Ok(StreamEvent::TextDelta {
                                                                text: text.to_string(),
                                                            }))
                                                            .await;
                                                    }

                                                    // Function call
                                                    if let Some(fc) = part.get("functionCall") {
                                                        if let Some(name) =
                                                            fc.get("name").and_then(|n| n.as_str())
                                                        {
                                                            let args = fc
                                                                .get("args")
                                                                .cloned()
                                                                .unwrap_or_else(|| {
                                                                    Value::Object(
                                                                        serde_json::Map::new(),
                                                                    )
                                                                });
                                                            let mut call = ToolCall::new(
                                                                uuid::Uuid::new_v4().to_string(),
                                                                name,
                                                                args,
                                                            );
                                                            if let Some(sig) = part
                                                                .get("thoughtSignature")
                                                                .and_then(|s| s.as_str())
                                                            {
                                                                call = call
                                                                    .with_thought_signature(sig);
                                                            }
                                                            let _ = tx
                                                                .send(Ok(
                                                                    StreamEvent::ToolCallStarted {
                                                                        call,
                                                                    },
                                                                ))
                                                                .await;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let _ = tx
                .send(Ok(StreamEvent::TurnCompleted {
                    node_id: ox_core::NodeId::new(),
                    usage,
                }))
                .await;
        });

        Ok(Box::pin(ChannelStream::new(rx)))
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }
}
