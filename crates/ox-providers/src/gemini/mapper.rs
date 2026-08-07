use ox_core::types::{ContentBlock, Message, Role, ToolDefinition};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct GeminiRequest {
    pub contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<GeminiSystemInstruction>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<GeminiTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GeminiGenConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiSystemInstruction {
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiContent {
    pub role: String,
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GeminiPart {
    Text {
        text: String,
    },
    FunctionCall {
        function_call: GeminiFunctionCall,
    },
    FunctionResponse {
        function_response: GeminiFunctionResponse,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiFunctionCall {
    pub name: String,
    pub args: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiFunctionResponse {
    pub name: String,
    pub response: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiTool {
    pub function_declarations: Vec<GeminiFunctionDecl>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiFunctionDecl {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Serialize)]
pub struct GeminiGenConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<usize>,
}

pub fn convert_tools(tools: &[ToolDefinition]) -> Vec<GeminiTool> {
    if tools.is_empty() {
        return Vec::new();
    }

    let decls = tools
        .iter()
        .map(|t| GeminiFunctionDecl {
            name: t.name.clone(),
            description: t.description.clone(),
            parameters: t.input_schema.clone(),
        })
        .collect();

    vec![GeminiTool {
        function_declarations: decls,
    }]
}

pub fn convert_messages(
    messages: &[Message],
) -> (Option<GeminiSystemInstruction>, Vec<GeminiContent>) {
    let mut system_instruction = None;
    let mut contents = Vec::new();

    for msg in messages {
        match msg.role {
            Role::System => {
                let text = msg.text_content();
                if !text.is_empty() {
                    system_instruction = Some(GeminiSystemInstruction {
                        parts: vec![GeminiPart::Text { text }],
                    });
                }
            }
            Role::User => {
                contents.push(GeminiContent {
                    role: "user".to_string(),
                    parts: vec![GeminiPart::Text {
                        text: msg.text_content(),
                    }],
                });
            }
            Role::Assistant => {
                let mut parts = Vec::new();
                for block in &msg.content {
                    match block {
                        ContentBlock::Text { text } => {
                            parts.push(GeminiPart::Text { text: text.clone() })
                        }
                        ContentBlock::Thinking { .. } => {}
                        ContentBlock::ToolCall(call) => parts.push(GeminiPart::FunctionCall {
                            function_call: GeminiFunctionCall {
                                name: call.name.clone(),
                                args: call.arguments.clone(),
                            },
                        }),
                        ContentBlock::ToolResult(_) => {}
                    }
                }
                contents.push(GeminiContent {
                    role: "model".to_string(),
                    parts,
                });
            }
            Role::Tool => {
                let mut parts = Vec::new();
                for block in &msg.content {
                    if let ContentBlock::ToolResult(res) = block {
                        parts.push(GeminiPart::FunctionResponse {
                            function_response: GeminiFunctionResponse {
                                name: res.tool_name.clone(),
                                response: serde_json::json!({ "result": res.content }),
                            },
                        });
                    }
                }
                contents.push(GeminiContent {
                    role: "user".to_string(),
                    parts,
                });
            }
        }
    }

    (system_instruction, contents)
}
