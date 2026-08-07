use crate::error::ToolError;
use crate::tool::{Tool, ToolContext};
use async_trait::async_trait;
use ox_core::types::{ToolCallId, ToolDefinition, ToolResult};
use serde::Deserialize;
use serde_json::Value;
use std::fs;

pub struct ReadFileTool;

#[derive(Deserialize)]
struct ReadFileArgs {
    path: String,
    start_line: Option<usize>,
    end_line: Option<usize>,
}

#[async_trait]
impl Tool for ReadFileTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "read_file",
            "Read the contents of a file within the workspace with optional line numbers.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative or absolute path to the file within the workspace."
                    },
                    "start_line": {
                        "type": "integer",
                        "description": "Optional 1-indexed start line."
                    },
                    "end_line": {
                        "type": "integer",
                        "description": "Optional 1-indexed end line (inclusive)."
                    }
                },
                "required": ["path"]
            }),
            false, // Read is safe / non-mutating
        )
    }

    async fn execute(
        &self,
        call_id: &ToolCallId,
        arguments: &Value,
        context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let args: ReadFileArgs =
            serde_json::from_value(arguments.clone()).map_err(|e| ToolError::InvalidArguments {
                tool: "read_file".to_string(),
                details: e.to_string(),
            })?;

        let safe_path = context.path_jail.resolve_and_verify(&args.path)?;
        if context.ignore_filter.is_ignored(&safe_path, false) {
            return Ok(ToolResult::error(
                call_id.clone(),
                "read_file",
                format!(
                    "File '{}' is ignored by .oxignore or .gitignore.",
                    args.path
                ),
            ));
        }

        if !safe_path.exists() || !safe_path.is_file() {
            return Ok(ToolResult::error(
                call_id.clone(),
                "read_file",
                format!("File '{}' does not exist or is not a file.", args.path),
            ));
        }

        let content = fs::read_to_string(&safe_path).map_err(|e| ToolError::ExecutionFailed {
            tool: "read_file".to_string(),
            message: format!("Failed to read file: {}", e),
        })?;

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let start = args.start_line.unwrap_or(1).saturating_sub(1);
        let end = args.end_line.unwrap_or(total_lines).min(total_lines);

        if start >= total_lines {
            return Ok(ToolResult::error(
                call_id.clone(),
                "read_file",
                format!(
                    "Start line {} exceeds total lines ({}).",
                    start + 1,
                    total_lines
                ),
            ));
        }

        let mut output = String::new();
        for (idx, line) in lines[start..end].iter().enumerate() {
            output.push_str(&format!("{:4} | {}\n", start + idx + 1, line));
        }

        Ok(ToolResult::success(call_id.clone(), "read_file", output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ox_security::{EnvScrubber, PathJail};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_read_file_tool() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("sample.txt");
        fs::write(&file_path, "line 1\nline 2\nline 3\n").unwrap();

        let jail = PathJail::new(dir.path()).unwrap();
        let ctx = ToolContext::new(jail, EnvScrubber::new());

        let tool = ReadFileTool;
        let res = tool
            .execute(
                &ToolCallId::new("1"),
                &serde_json::json!({ "path": "sample.txt", "start_line": 1, "end_line": 2 }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(!res.is_error);
        assert!(res.content.contains("line 1"));
        assert!(res.content.contains("line 2"));
        assert!(!res.content.contains("line 3"));
    }

    #[tokio::test]
    async fn test_read_file_masked_by_oxignore() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".oxignore"), "secret.env\n").unwrap();
        fs::write(dir.path().join("secret.env"), "API_KEY=12345\n").unwrap();

        let jail = PathJail::new(dir.path()).unwrap();
        let ctx = ToolContext::new(jail, EnvScrubber::new());

        let tool = ReadFileTool;
        let res = tool
            .execute(
                &ToolCallId::new("1"),
                &serde_json::json!({ "path": "secret.env" }),
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
