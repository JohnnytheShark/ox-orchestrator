use crate::config::{ProviderConfig, ProviderType};
use crate::error::ProviderError;
use crate::openai::mapper::{convert_messages, convert_tools, OpenAiChatRequest};
use crate::provider::{LlmProvider, ProviderStream};
use crate::stream::ChannelStream;
use async_trait::async_trait;
use futures_util::TryStreamExt;
use ox_core::agent::StreamEvent;
use ox_core::types::{Message, ToolCall, ToolDefinition};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::mpsc;

pub struct OpenAiProvider {
    config: ProviderConfig,
    client: Client,
}

impl OpenAiProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn stream_chat(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<ProviderStream, ProviderError> {
        let is_ollama = self.config.provider_type == ProviderType::Ollama;
        let api_key = self.config.get_api_key();

        if !is_ollama && api_key.is_none() {
            return Err(ProviderError::MissingApiKey(
                "OpenAI / Compatible".to_string(),
            ));
        }

        let openai_messages = convert_messages(messages);
        let openai_tools = convert_tools(tools);

        let request_payload = OpenAiChatRequest {
            model: self.config.model.clone(),
            messages: openai_messages,
            tools: openai_tools,
            stream: true,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
        };

        let url = format!("{}/chat/completions", self.config.effective_base_url());
        let mut req = self
            .client
            .post(&url)
            .header("content-type", "application/json");

        if let Some(key) = api_key {
            req = req.header("authorization", format!("Bearer {}", key));
        }

        let response = req.json(&request_payload).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError { status, body });
        }

        let (tx, rx) = mpsc::channel(100);
        let mut byte_stream = response.bytes_stream();

        tokio::spawn(async move {
            let mut buffer = Vec::new();
            // tool_call_index -> (id, name, accumulated_arguments)
            let mut active_tool_calls: HashMap<usize, (String, String, String)> = HashMap::new();

            while let Ok(Some(chunk)) = byte_stream.try_next().await {
                buffer.extend_from_slice(&chunk);

                while let Some(pos) = buffer.windows(2).position(|w| w == b"\n\n") {
                    let event_bytes = buffer[..pos].to_vec();
                    buffer.drain(..pos + 2);

                    let event_block = String::from_utf8_lossy(&event_bytes);

                    for line in event_block.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            let trimmed = data.trim();
                            if trimmed == "[DONE]" {
                                // Flush all accumulated tool calls
                                for (_, (id, name, args_str)) in active_tool_calls.drain() {
                                    if !name.is_empty() {
                                        let args = serde_json::from_str::<Value>(&args_str)
                                            .unwrap_or_else(|_| {
                                                Value::Object(serde_json::Map::new())
                                            });
                                        let call = ToolCall::new(id, name, args);
                                        let _ = tx
                                            .send(Ok(StreamEvent::ToolCallStarted { call }))
                                            .await;
                                    }
                                }
                                break;
                            }

                            if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
                                if let Some(choices) = v.get("choices").and_then(|c| c.as_array()) {
                                    for choice in choices {
                                        if let Some(delta) = choice.get("delta") {
                                            // Handle text content delta
                                            if let Some(content) =
                                                delta.get("content").and_then(|c| c.as_str())
                                            {
                                                let _ = tx
                                                    .send(Ok(StreamEvent::TextDelta {
                                                        text: content.to_string(),
                                                    }))
                                                    .await;
                                            }

                                            // Handle reasoning content delta (DeepSeek / OpenAI reasoning)
                                            if let Some(reasoning) = delta
                                                .get("reasoning_content")
                                                .and_then(|r| r.as_str())
                                            {
                                                let _ = tx
                                                    .send(Ok(StreamEvent::ThinkingDelta {
                                                        thinking: reasoning.to_string(),
                                                    }))
                                                    .await;
                                            }

                                            // Handle tool call deltas
                                            if let Some(tool_calls_arr) =
                                                delta.get("tool_calls").and_then(|t| t.as_array())
                                            {
                                                for tc in tool_calls_arr {
                                                    let index = tc
                                                        .get("index")
                                                        .and_then(|i| i.as_u64())
                                                        .unwrap_or(0)
                                                        as usize;
                                                    let entry = active_tool_calls
                                                        .entry(index)
                                                        .or_insert_with(|| {
                                                            (
                                                                String::new(),
                                                                String::new(),
                                                                String::new(),
                                                            )
                                                        });

                                                    if let Some(id) =
                                                        tc.get("id").and_then(|s| s.as_str())
                                                    {
                                                        entry.0 = id.to_string();
                                                    }

                                                    if let Some(func) = tc.get("function") {
                                                        if let Some(name) = func
                                                            .get("name")
                                                            .and_then(|s| s.as_str())
                                                        {
                                                            entry.1.push_str(name);
                                                        }
                                                        if let Some(args_chunk) = func
                                                            .get("arguments")
                                                            .and_then(|s| s.as_str())
                                                        {
                                                            entry.2.push_str(args_chunk);
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

            // Flush any remaining tool calls if [DONE] was missing
            for (_, (id, name, args_str)) in active_tool_calls.drain() {
                if !name.is_empty() {
                    let args = serde_json::from_str::<Value>(&args_str)
                        .unwrap_or_else(|_| Value::Object(serde_json::Map::new()));
                    let call = ToolCall::new(id, name, args);
                    let _ = tx.send(Ok(StreamEvent::ToolCallStarted { call })).await;
                }
            }
        });

        Ok(Box::pin(ChannelStream::new(rx)))
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }
}
