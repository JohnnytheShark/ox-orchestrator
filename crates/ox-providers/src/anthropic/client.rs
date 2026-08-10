use crate::anthropic::mapper::{convert_messages, convert_tools, AnthropicRequest};
use crate::config::ProviderConfig;
use crate::error::ProviderError;
use crate::provider::{LlmProvider, ProviderStream};
use crate::stream::ChannelStream;
use async_trait::async_trait;
use futures_util::TryStreamExt;
use ox_core::agent::StreamEvent;
use ox_core::types::{Message, TokenUsage, ToolCall, ToolDefinition};
use reqwest::Client;
use serde_json::Value;
use tokio::sync::mpsc;

pub struct AnthropicProvider {
    config: ProviderConfig,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn stream_chat(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<ProviderStream, ProviderError> {
        let api_key = self
            .config
            .get_api_key()
            .ok_or_else(|| ProviderError::MissingApiKey("Anthropic".to_string()))?;

        let (system_prompt, anthropic_messages) = convert_messages(messages);
        let anthropic_tools = convert_tools(tools);

        let request_payload = AnthropicRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(4096),
            messages: anthropic_messages,
            system: system_prompt,
            tools: anthropic_tools,
            stream: true,
            temperature: self.config.temperature,
        };

        let url = format!("{}/messages", self.config.effective_base_url());
        let response = self
            .client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
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
            fn find_sse_boundary(buf: &[u8]) -> Option<(usize, usize)> {
                if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    return Some((pos, 4));
                }
                if let Some(pos) = buf.windows(2).position(|w| w == b"\n\n") {
                    return Some((pos, 2));
                }
                if let Some(pos) = buf.windows(2).position(|w| w == b"\r\r") {
                    return Some((pos, 2));
                }
                None
            }
            let mut buffer = Vec::new();
            let mut current_tool_id = String::new();
            let mut current_tool_name = String::new();
            let mut current_tool_input = String::new();
            let mut usage = TokenUsage::default();

            while let Ok(Some(chunk)) = byte_stream.try_next().await {
                buffer.extend_from_slice(&chunk);

                while let Some((pos, sep_len)) = find_sse_boundary(&buffer) {
                    let event_bytes = buffer[..pos].to_vec();
                    buffer.drain(..pos + sep_len);

                    let event_block = String::from_utf8_lossy(&event_bytes);

                    for line in event_block.lines() {
                        let line = line.trim_end_matches('\r');
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data.trim() == "[DONE]" {
                                break;
                            }

                            if let Ok(v) = serde_json::from_str::<Value>(data) {
                                let event_type =
                                    v.get("type").and_then(|t| t.as_str()).unwrap_or_default();

                                match event_type {
                                    "content_block_start" => {
                                        if let Some(content_block) = v.get("content_block") {
                                            let b_type = content_block
                                                .get("type")
                                                .and_then(|t| t.as_str())
                                                .unwrap_or_default();
                                            if b_type == "tool_use" {
                                                current_tool_id = content_block
                                                    .get("id")
                                                    .and_then(|s| s.as_str())
                                                    .unwrap_or_default()
                                                    .to_string();
                                                current_tool_name = content_block
                                                    .get("name")
                                                    .and_then(|s| s.as_str())
                                                    .unwrap_or_default()
                                                    .to_string();
                                                current_tool_input.clear();
                                            }
                                        }
                                    }
                                    "content_block_delta" => {
                                        if let Some(delta) = v.get("delta") {
                                            let delta_type = delta
                                                .get("type")
                                                .and_then(|t| t.as_str())
                                                .unwrap_or_default();
                                            if delta_type == "text_delta" {
                                                if let Some(text) =
                                                    delta.get("text").and_then(|t| t.as_str())
                                                {
                                                    let _ = tx
                                                        .send(Ok(StreamEvent::TextDelta {
                                                            text: text.to_string(),
                                                        }))
                                                        .await;
                                                }
                                            } else if delta_type == "thinking_delta" {
                                                if let Some(th) =
                                                    delta.get("thinking").and_then(|t| t.as_str())
                                                {
                                                    let _ = tx
                                                        .send(Ok(StreamEvent::ThinkingDelta {
                                                            thinking: th.to_string(),
                                                        }))
                                                        .await;
                                                }
                                            } else if delta_type == "input_json_delta" {
                                                if let Some(partial) = delta
                                                    .get("partial_json")
                                                    .and_then(|p| p.as_str())
                                                {
                                                    current_tool_input.push_str(partial);
                                                }
                                            }
                                        }
                                    }
                                    "content_block_stop" => {
                                        if !current_tool_name.is_empty() {
                                            let args =
                                                serde_json::from_str::<Value>(&current_tool_input)
                                                    .unwrap_or_else(|_| {
                                                        Value::Object(serde_json::Map::new())
                                                    });
                                            let tool_call = ToolCall::new(
                                                &current_tool_id,
                                                &current_tool_name,
                                                args,
                                            );
                                            let _ = tx
                                                .send(Ok(StreamEvent::ToolCallStarted {
                                                    call: tool_call,
                                                }))
                                                .await;
                                            current_tool_name.clear();
                                            current_tool_id.clear();
                                            current_tool_input.clear();
                                        }
                                    }
                                    "message_delta" => {
                                        if let Some(u) = v.get("usage") {
                                            if let Some(out) =
                                                u.get("output_tokens").and_then(|n| n.as_u64())
                                            {
                                                usage.output_tokens = out as usize;
                                            }
                                        }
                                    }
                                    "message_start" => {
                                        if let Some(msg) = v.get("message") {
                                            if let Some(u) = msg.get("usage") {
                                                if let Some(inp) =
                                                    u.get("input_tokens").and_then(|n| n.as_u64())
                                                {
                                                    usage.input_tokens = inp as usize;
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }

            let _ = tx
                .send(Ok(StreamEvent::TurnCompleted {
                    node_id: ox_core::session::NodeId::new(),
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
