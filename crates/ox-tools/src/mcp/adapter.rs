use crate::error::ToolError;
use crate::mcp::client::McpClient;
use crate::mcp::protocol::{McpContentItem, McpToolInfo};
use crate::tool::{Tool, ToolContext};
use async_trait::async_trait;
use ox_core::types::{ToolCallId, ToolDefinition, ToolResult};
use serde_json::Value;
use std::sync::Arc;

/// Bridges an MCP server-hosted tool into the native ox `Tool` interface.
pub struct McpToolAdapter {
    qualified_name: String,
    raw_name: String,
    description: String,
    input_schema: Value,
    client: Arc<McpClient>,
}

impl McpToolAdapter {
    pub fn new(server_name: &str, info: McpToolInfo, client: Arc<McpClient>) -> Self {
        let qualified_name = format!("{}__{}", server_name, info.name);
        Self {
            qualified_name,
            raw_name: info.name,
            description: info
                .description
                .unwrap_or_else(|| "MCP dynamic tool".to_string()),
            input_schema: info.input_schema,
            client,
        }
    }
}

#[async_trait]
impl Tool for McpToolAdapter {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            &self.qualified_name,
            &self.description,
            self.input_schema.clone(),
            true, // Default to requiring approval for external MCP operations
        )
    }

    async fn execute(
        &self,
        call_id: &ToolCallId,
        arguments: &Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let mcp_res = self.client.call_tool(&self.raw_name, arguments).await?;

        let mut output = String::new();
        for item in mcp_res.content {
            match item {
                McpContentItem::Text { text } => {
                    output.push_str(&text);
                }
                McpContentItem::Image { mime_type, .. } => {
                    output.push_str(&format!("[Binary Image: {}]", mime_type));
                }
                McpContentItem::Resource { resource } => {
                    output.push_str(&format!("[Resource: {}]", resource));
                }
            }
        }

        let is_err = mcp_res.is_error.unwrap_or(false);
        if is_err {
            Ok(ToolResult::error(
                call_id.clone(),
                &self.qualified_name,
                output,
            ))
        } else {
            Ok(ToolResult::success(
                call_id.clone(),
                &self.qualified_name,
                output,
            ))
        }
    }
}
