use ox_core::types::{ContentBlock, Message, Role, ToolDefinition};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct OpenAiChatRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<OpenAiTool>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAiToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: OpenAiFunctionCall,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: OpenAiFunctionDef,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiFunctionDef {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

pub fn convert_tools(tools: &[ToolDefinition]) -> Vec<OpenAiTool> {
    tools
        .iter()
        .map(|t| OpenAiTool {
            tool_type: "function".to_string(),
            function: OpenAiFunctionDef {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.input_schema.clone(),
            },
        })
        .collect()
}

pub fn convert_messages(messages: &[Message]) -> Vec<OpenAiMessage> {
    let mut openai_msgs = Vec::new();

    for msg in messages {
        match msg.role {
            Role::System => {
                openai_msgs.push(OpenAiMessage {
                    role: "system".to_string(),
                    content: Some(msg.text_content()),
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
            Role::User => {
                openai_msgs.push(OpenAiMessage {
                    role: "user".to_string(),
                    content: Some(msg.text_content()),
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
            Role::Assistant => {
                let mut text_parts = Vec::new();
                let mut tool_calls = Vec::new();

                for block in &msg.content {
                    match block {
                        ContentBlock::Text { text } => text_parts.push(text.clone()),
                        ContentBlock::Thinking { .. } => {}
                        ContentBlock::ToolCall(call) => {
                            tool_calls.push(OpenAiToolCall {
                                id: call.id.0.clone(),
                                call_type: "function".to_string(),
                                function: OpenAiFunctionCall {
                                    name: call.name.clone(),
                                    arguments: call.arguments.to_string(),
                                },
                            });
                        }
                        ContentBlock::ToolResult(_) => {}
                    }
                }

                let content_str = if text_parts.is_empty() {
                    None
                } else {
                    Some(text_parts.join("\n"))
                };

                let calls_opt = if tool_calls.is_empty() {
                    None
                } else {
                    Some(tool_calls)
                };

                openai_msgs.push(OpenAiMessage {
                    role: "assistant".to_string(),
                    content: content_str,
                    tool_calls: calls_opt,
                    tool_call_id: None,
                });
            }
            Role::Tool => {
                for block in &msg.content {
                    if let ContentBlock::ToolResult(res) = block {
                        openai_msgs.push(OpenAiMessage {
                            role: "tool".to_string(),
                            content: Some(res.content.clone()),
                            tool_calls: None,
                            tool_call_id: Some(res.call_id.0.clone()),
                        });
                    }
                }
            }
        }
    }

    openai_msgs
}
