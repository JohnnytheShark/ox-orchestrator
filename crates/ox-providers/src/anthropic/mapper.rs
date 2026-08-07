use ox_core::types::{ContentBlock, Message, Role, ToolDefinition};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Anthropic Messages API request format.
#[derive(Debug, Serialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: usize,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<AnthropicTool>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: Vec<AnthropicContentBlock>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicContentBlock {
    Text {
        text: String,
    },
    Thinking {
        thinking: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        is_error: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnthropicTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub fn convert_tools(tools: &[ToolDefinition]) -> Vec<AnthropicTool> {
    tools
        .iter()
        .map(|t| AnthropicTool {
            name: t.name.clone(),
            description: t.description.clone(),
            input_schema: t.input_schema.clone(),
        })
        .collect()
}

pub fn convert_messages(messages: &[Message]) -> (Option<String>, Vec<AnthropicMessage>) {
    let mut system_prompt = None;
    let mut anthropic_msgs = Vec::new();

    for msg in messages {
        match msg.role {
            Role::System => {
                let text = msg.text_content();
                if !text.is_empty() {
                    system_prompt = Some(text);
                }
            }
            Role::User => {
                let blocks = msg
                    .content
                    .iter()
                    .map(|b| match b {
                        ContentBlock::Text { text } => {
                            AnthropicContentBlock::Text { text: text.clone() }
                        }
                        _ => AnthropicContentBlock::Text {
                            text: String::new(),
                        },
                    })
                    .collect();
                anthropic_msgs.push(AnthropicMessage {
                    role: "user".to_string(),
                    content: blocks,
                });
            }
            Role::Assistant => {
                let blocks = msg
                    .content
                    .iter()
                    .map(|b| match b {
                        ContentBlock::Text { text } => {
                            AnthropicContentBlock::Text { text: text.clone() }
                        }
                        ContentBlock::Thinking { thinking } => AnthropicContentBlock::Thinking {
                            thinking: thinking.clone(),
                        },
                        ContentBlock::ToolCall(call) => AnthropicContentBlock::ToolUse {
                            id: call.id.0.clone(),
                            name: call.name.clone(),
                            input: call.arguments.clone(),
                        },
                        ContentBlock::ToolResult(_) => AnthropicContentBlock::Text {
                            text: String::new(),
                        },
                    })
                    .collect();
                anthropic_msgs.push(AnthropicMessage {
                    role: "assistant".to_string(),
                    content: blocks,
                });
            }
            Role::Tool => {
                let blocks = msg
                    .content
                    .iter()
                    .filter_map(|b| match b {
                        ContentBlock::ToolResult(res) => Some(AnthropicContentBlock::ToolResult {
                            tool_use_id: res.call_id.0.clone(),
                            content: res.content.clone(),
                            is_error: res.is_error,
                        }),
                        _ => None,
                    })
                    .collect();
                anthropic_msgs.push(AnthropicMessage {
                    role: "user".to_string(), // Anthropic expects tool results within role "user"
                    content: blocks,
                });
            }
        }
    }

    (system_prompt, anthropic_msgs)
}
