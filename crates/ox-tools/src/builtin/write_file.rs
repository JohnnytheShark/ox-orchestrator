use crate::error::ToolError;
use crate::tool::{Tool, ToolContext};
use async_trait::async_trait;
use ox_core::types::{ToolCallId, ToolDefinition, ToolResult};
use serde::Deserialize;
use serde_json::Value;
use std::fs;

pub struct WriteFileTool;

#[derive(Deserialize)]
struct WriteFileArgs {
    path: String,
    content: String,
}

#[async_trait]
impl Tool for WriteFileTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "write_file",
            "Write full content to a file atomically within the workspace. Creates parent directories if needed.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative or absolute file path to write."
                    },
                    "content": {
                        "type": "string",
                        "description": "Complete text content to write to the file."
                    }
                },
                "required": ["path", "content"]
            }),
            true, // Mutating tool
        )
    }

    async fn execute(
        &self,
        call_id: &ToolCallId,
        arguments: &Value,
        context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let args: WriteFileArgs =
            serde_json::from_value(arguments.clone()).map_err(|e| ToolError::InvalidArguments {
                tool: "write_file".to_string(),
                details: e.to_string(),
            })?;

        let safe_path = context.path_jail.resolve_and_verify(&args.path)?;
        if context.ignore_filter.is_ignored(&safe_path, false) {
            return Ok(ToolResult::error(
                call_id.clone(),
                "write_file",
                format!(
                    "File '{}' is ignored by .oxignore or .gitignore.",
                    args.path
                ),
            ));
        }

        if let Some(parent) = safe_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Atomic write via tempfile in the same directory
        let temp_path = safe_path.with_extension(format!("tmp.{}", uuid::Uuid::new_v4()));
        fs::write(&temp_path, &args.content)?;
        fs::rename(&temp_path, &safe_path)?;

        Ok(ToolResult::success(
            call_id.clone(),
            "write_file",
            format!(
                "Successfully wrote {} bytes to '{}'.",
                args.content.len(),
                args.path
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ox_security::{EnvScrubber, PathJail};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_write_file_tool() {
        let dir = tempdir().unwrap();
        let jail = PathJail::new(dir.path()).unwrap();
        let ctx = ToolContext::new(jail, EnvScrubber::new());

        let tool = WriteFileTool;
        let res = tool
            .execute(
                &ToolCallId::new("1"),
                &serde_json::json!({ "path": "nested/hello.txt", "content": "Hello Rust!" }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(!res.is_error);
        let written = fs::read_to_string(dir.path().join("nested/hello.txt")).unwrap();
        assert_eq!(written, "Hello Rust!");
    }

    #[tokio::test]
    async fn test_write_file_masked_by_oxignore() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".oxignore"), "creds.json\n").unwrap();

        let jail = PathJail::new(dir.path()).unwrap();
        let ctx = ToolContext::new(jail, EnvScrubber::new());

        let tool = WriteFileTool;
        let res = tool
            .execute(
                &ToolCallId::new("1"),
                &serde_json::json!({ "path": "creds.json", "content": "{}" }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(res.is_error);
        assert!(res
            .content
            .contains("is ignored by .oxignore or .gitignore"));
    }
}
