use crate::error::ToolError;
use crate::tool::{Tool, ToolContext};
use async_trait::async_trait;
use ox_core::types::{ToolCallId, ToolDefinition, ToolResult};
use serde::Deserialize;
use serde_json::Value;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

pub const MAX_OUTPUT_LINES: usize = 300;
pub const MAX_OUTPUT_BYTES: usize = 12 * 1024; // 12 KB
pub const HEAD_LINES: usize = 80;
pub const TAIL_LINES: usize = 150;

/// Truncates output exceeding line or byte limits, keeping head and tail lines with a contextual indicator.
pub fn truncate_output(text: &str) -> String {
    let line_count = text.lines().count();
    let byte_count = text.len();

    if line_count <= MAX_OUTPUT_LINES && byte_count <= MAX_OUTPUT_BYTES {
        return text.to_string();
    }

    let lines: Vec<&str> = text.lines().collect();
    if lines.len() > HEAD_LINES + TAIL_LINES {
        let omitted = lines.len() - (HEAD_LINES + TAIL_LINES);
        let head = &lines[..HEAD_LINES];
        let tail = &lines[lines.len() - TAIL_LINES..];
        format!(
            "{}\n\n... [Output truncated: {} lines omitted to preserve context] ...\n\n{}",
            head.join("\n"),
            omitted,
            tail.join("\n")
        )
    } else if byte_count > MAX_OUTPUT_BYTES {
        let half_bytes = (MAX_OUTPUT_BYTES.saturating_sub(200)) / 2;
        let head_idx = text.floor_char_boundary(half_bytes);
        let tail_idx = text.ceil_char_boundary(text.len().saturating_sub(half_bytes));
        let head_str = &text[..head_idx];
        let tail_str = &text[tail_idx..];
        let omitted_bytes = byte_count.saturating_sub(head_str.len() + tail_str.len());
        format!(
            "{}\n\n... [Output truncated: {} bytes omitted to preserve context] ...\n\n{}",
            head_str, omitted_bytes, tail_str
        )
    } else {
        text.to_string()
    }
}

pub struct ExecCommandTool;

#[derive(Deserialize)]
struct ExecCommandArgs {
    command: String,
    cwd: Option<String>,
    timeout_secs: Option<u64>,
}

#[async_trait]
impl Tool for ExecCommandTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "exec_command",
            "Execute a shell command inside the workspace sandbox with scrubbed environment variables.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Shell command line string to run."
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Optional subdirectory relative to workspace root to run command in."
                    },
                    "timeout_secs": {
                        "type": "integer",
                        "description": "Maximum execution time in seconds (default 30)."
                    }
                },
                "required": ["command"]
            }),
            true, // Mutating / dangerous tool requiring HITL approval by default
        )
    }

    async fn execute(
        &self,
        call_id: &ToolCallId,
        arguments: &Value,
        context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let args: ExecCommandArgs =
            serde_json::from_value(arguments.clone()).map_err(|e| ToolError::InvalidArguments {
                tool: "exec_command".to_string(),
                details: e.to_string(),
            })?;

        let work_dir = match args.cwd {
            Some(ref sub) => context.path_jail.resolve_and_verify(sub)?,
            None => context.path_jail.root().to_path_buf(),
        };

        let scrubbed_env = context.env_scrubber.sanitize_current_env();
        let timeout_dur = Duration::from_secs(args.timeout_secs.unwrap_or(30));

        #[cfg(target_os = "windows")]
        let mut cmd = Command::new("powershell.exe");
        #[cfg(target_os = "windows")]
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", &args.command]);

        #[cfg(not(target_os = "windows"))]
        let mut cmd = Command::new("sh");
        #[cfg(not(target_os = "windows"))]
        cmd.args(["-c", &args.command]);

        cmd.current_dir(work_dir)
            .env_clear()
            .envs(scrubbed_env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let execution = async {
            let child = cmd.spawn()?;
            let output = child.wait_with_output().await?;
            Ok::<_, std::io::Error>(output)
        };

        match timeout(timeout_dur, execution).await {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                let mut combined = String::new();
                if !stdout.is_empty() {
                    combined.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    if !combined.is_empty() {
                        combined.push('\n');
                    }
                    combined.push_str(&format!("[STDERR]\n{}", stderr));
                }

                let truncated = truncate_output(&combined);

                if output.status.success() {
                    Ok(ToolResult::success(
                        call_id.clone(),
                        "exec_command",
                        truncated,
                    ))
                } else {
                    Ok(ToolResult::error(
                        call_id.clone(),
                        "exec_command",
                        format!(
                            "Command failed with exit code {}:\n{}",
                            exit_code, truncated
                        ),
                    ))
                }
            }
            Ok(Err(e)) => Ok(ToolResult::error(
                call_id.clone(),
                "exec_command",
                format!("Failed to spawn process: {}", e),
            )),
            Err(_) => Ok(ToolResult::error(
                call_id.clone(),
                "exec_command",
                format!("Command timed out after {} seconds.", timeout_dur.as_secs()),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ox_security::{EnvScrubber, PathJail};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_exec_command_echo() {
        let dir = tempdir().unwrap();
        let jail = PathJail::new(dir.path()).unwrap();
        let ctx = ToolContext::new(jail, EnvScrubber::new());

        let tool = ExecCommandTool;
        let res = tool
            .execute(
                &ToolCallId::new("1"),
                &serde_json::json!({ "command": "echo 'ox-test'" }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(!res.is_error);
        assert!(res.content.contains("ox-test"));
    }

    #[test]
    fn test_truncate_under_threshold() {
        let short_text = "Line 1\nLine 2\nLine 3";
        assert_eq!(truncate_output(short_text), short_text);
    }

    #[test]
    fn test_truncate_over_300_lines() {
        let mut lines = Vec::new();
        for i in 1..=500 {
            lines.push(format!("Line {}", i));
        }
        let full_text = lines.join("\n");
        let truncated = truncate_output(&full_text);

        assert!(truncated.contains("Line 1"));
        assert!(truncated.contains("Line 80"));
        assert!(
            truncated.contains("... [Output truncated: 270 lines omitted to preserve context] ...")
        );
        assert!(truncated.contains("Line 351"));
        assert!(truncated.contains("Line 500"));
        assert!(!truncated.contains("Line 81\n"));
    }

    #[test]
    fn test_truncate_over_12kb_single_line() {
        let large_text = "x".repeat(20_000);
        let truncated = truncate_output(&large_text);

        assert!(truncated.contains("... [Output truncated:"));
        assert!(truncated.len() < large_text.len());
    }
}
