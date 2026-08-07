use crate::error::ToolError;
use crate::tool::{Tool, ToolContext};
use async_trait::async_trait;
use ox_core::types::{ToolCallId, ToolDefinition, ToolResult};
use serde::Deserialize;
use serde_json::Value;
use std::fs;

pub struct EditFileTool;

#[derive(Deserialize)]
struct EditFileArgs {
    path: String,
    target: String,
    replacement: String,
}

#[async_trait]
impl Tool for EditFileTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "edit_file",
            "Perform a precise search-and-replace edit on an existing file within the workspace.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to edit."
                    },
                    "target": {
                        "type": "string",
                        "description": "Exact text block to find and replace in the file."
                    },
                    "replacement": {
                        "type": "string",
                        "description": "New text to replace the target text with."
                    }
                },
                "required": ["path", "target", "replacement"]
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
        let args: EditFileArgs =
            serde_json::from_value(arguments.clone()).map_err(|e| ToolError::InvalidArguments {
                tool: "edit_file".to_string(),
                details: e.to_string(),
            })?;

        let safe_path = context.path_jail.resolve_and_verify(&args.path)?;
        if context.ignore_filter.is_ignored(&safe_path, false) {
            return Ok(ToolResult::error(
                call_id.clone(),
                "edit_file",
                format!(
                    "File '{}' is ignored by .oxignore or .gitignore.",
                    args.path
                ),
            ));
        }

        if !safe_path.exists() || !safe_path.is_file() {
            return Ok(ToolResult::error(
                call_id.clone(),
                "edit_file",
                format!("Target file '{}' does not exist.", args.path),
            ));
        }

        let content = fs::read_to_string(&safe_path)?;

        let occurrences = content.matches(&args.target).count();
        if occurrences == 0 {
            return Ok(ToolResult::error(
                call_id.clone(),
                "edit_file",
                format!(
                    "Target string not found in '{}'. Ensure exact whitespace and newline match.",
                    args.path
                ),
            ));
        }

        if occurrences > 1 {
            return Ok(ToolResult::error(
                call_id.clone(),
                "edit_file",
                format!(
                    "Target string occurs {} times in '{}'. Provide more surrounding context to disambiguate.",
                    occurrences, args.path
                ),
            ));
        }

        let new_content = content.replacen(&args.target, &args.replacement, 1);

        // Atomic write via temporary file
        let temp_path = safe_path.with_extension(format!("tmp.{}", uuid::Uuid::new_v4()));
        fs::write(&temp_path, &new_content)?;
        fs::rename(&temp_path, &safe_path)?;

        Ok(ToolResult::success(
            call_id.clone(),
            "edit_file",
            format!("Successfully applied edit to '{}'.", args.path),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ox_security::{EnvScrubber, PathJail};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_edit_file_tool() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("code.rs");
        fs::write(
            &file_path,
            "fn add(a: i32, b: i32) -> i32 {\n    a - b\n}\n",
        )
        .unwrap();

        let jail = PathJail::new(dir.path()).unwrap();
        let ctx = ToolContext::new(jail, EnvScrubber::new());

        let tool = EditFileTool;
        let res = tool
            .execute(
                &ToolCallId::new("1"),
                &serde_json::json!({
                    "path": "code.rs",
                    "target": "    a - b",
                    "replacement": "    a + b"
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(!res.is_error);
        let updated = fs::read_to_string(&file_path).unwrap();
        assert!(updated.contains("a + b"));
    }

    #[tokio::test]
    async fn test_edit_file_masked_by_oxignore() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".oxignore"), "keys.json\n").unwrap();
        fs::write(dir.path().join("keys.json"), "{\"token\": \"old\"}\n").unwrap();

        let jail = PathJail::new(dir.path()).unwrap();
        let ctx = ToolContext::new(jail, EnvScrubber::new());

        let tool = EditFileTool;
        let res = tool
            .execute(
                &ToolCallId::new("1"),
                &serde_json::json!({
                    "path": "keys.json",
                    "target": "old",
                    "replacement": "new"
                }),
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
