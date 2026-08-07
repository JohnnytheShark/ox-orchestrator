use crate::error::ToolError;
use crate::tool::{Tool, ToolContext};
use async_trait::async_trait;
use ox_core::types::{ToolCallId, ToolDefinition, ToolResult};
use regex::RegexBuilder;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub struct GrepSearchTool;

#[derive(Deserialize)]
struct GrepSearchArgs {
    query: String,
    path: Option<String>,
    case_sensitive: Option<bool>,
    is_regex: Option<bool>,
    max_results: Option<usize>,
}

#[async_trait]
impl Tool for GrepSearchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "grep_search",
            "Search for text or regex patterns across files in the workspace.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Text substring or regex pattern to search for."
                    },
                    "path": {
                        "type": "string",
                        "description": "Optional subdirectory or file to limit search to."
                    },
                    "case_sensitive": {
                        "type": "boolean",
                        "description": "Whether to match case sensitively (default false)."
                    },
                    "is_regex": {
                        "type": "boolean",
                        "description": "Whether query should be treated as regex (default false)."
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of matching lines to return (default 50)."
                    }
                },
                "required": ["query"]
            }),
            false, // Read-only / safe
        )
    }

    async fn execute(
        &self,
        call_id: &ToolCallId,
        arguments: &Value,
        context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let args: GrepSearchArgs =
            serde_json::from_value(arguments.clone()).map_err(|e| ToolError::InvalidArguments {
                tool: "grep_search".to_string(),
                details: e.to_string(),
            })?;

        let search_root = match args.path {
            Some(ref p) => context.path_jail.resolve_and_verify(p)?,
            None => context.path_jail.root().to_path_buf(),
        };

        let case_insensitive = !args.case_sensitive.unwrap_or(false);
        let pattern = if args.is_regex.unwrap_or(false) {
            args.query.clone()
        } else {
            regex::escape(&args.query)
        };

        let re = match RegexBuilder::new(&pattern)
            .case_insensitive(case_insensitive)
            .build()
        {
            Ok(r) => r,
            Err(e) => {
                return Ok(ToolResult::error(
                    call_id.clone(),
                    "grep_search",
                    format!("Invalid regex query: {}", e),
                ))
            }
        };

        let max_results = args.max_results.unwrap_or(50);
        let mut matches = Vec::new();
        let mut files_to_scan = Vec::new();

        collect_files(&search_root, &mut files_to_scan, &context.ignore_filter);

        for file_path in files_to_scan {
            if matches.len() >= max_results {
                break;
            }

            if let Ok(content) = fs::read_to_string(&file_path) {
                let rel_path = file_path
                    .strip_prefix(context.path_jail.root())
                    .unwrap_or(&file_path)
                    .display()
                    .to_string();

                for (line_num, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        matches.push(format!("{}:{}: {}", rel_path, line_num + 1, line.trim()));
                        if matches.len() >= max_results {
                            break;
                        }
                    }
                }
            }
        }

        if matches.is_empty() {
            Ok(ToolResult::success(
                call_id.clone(),
                "grep_search",
                "No matches found.",
            ))
        } else {
            let count = matches.len();
            let summary = format!("Found {} match(es):\n{}", count, matches.join("\n"));
            Ok(ToolResult::success(call_id.clone(), "grep_search", summary))
        }
    }
}

fn collect_files(
    dir: &Path,
    files: &mut Vec<std::path::PathBuf>,
    ignore_filter: &crate::ignore::IgnoreFilter,
) {
    if dir.is_file() {
        if !ignore_filter.is_ignored(dir, false) {
            files.push(dir.to_path_buf());
        }
        return;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_dir = path.is_dir();

            if ignore_filter.is_ignored(&path, is_dir) {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            if name.starts_with('.') && name != ".ox" {
                continue;
            }

            if is_dir {
                collect_files(&path, files, ignore_filter);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ox_security::{EnvScrubber, PathJail};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_grep_search_basic() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");
        fs::write(&file_path, "fn hello_world() {\n    println!(\"hi\");\n}").unwrap();

        let jail = PathJail::new(dir.path()).unwrap();
        let ctx = ToolContext::new(jail, EnvScrubber::new());

        let tool = GrepSearchTool;
        let res = tool
            .execute(
                &ToolCallId::new("1"),
                &serde_json::json!({ "query": "hello_world" }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(!res.is_error);
        assert!(res.content.contains("main.rs:1: fn hello_world()"));
    }

    #[tokio::test]
    async fn test_grep_search_masks_oxignore() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".oxignore"), "keys.txt\nsecrets/\n").unwrap();
        fs::write(dir.path().join("keys.txt"), "PASSWORD=secret123").unwrap();
        fs::write(
            dir.path().join("regular.txt"),
            "normal content with secret123",
        )
        .unwrap();

        let secrets_dir = dir.path().join("secrets");
        fs::create_dir(&secrets_dir).unwrap();
        fs::write(secrets_dir.join("db.conf"), "db_pass=secret123").unwrap();

        let jail = PathJail::new(dir.path()).unwrap();
        let ctx = ToolContext::new(jail, EnvScrubber::new());

        let tool = GrepSearchTool;
        let res = tool
            .execute(
                &ToolCallId::new("1"),
                &serde_json::json!({ "query": "secret123" }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(!res.is_error);
        assert!(res
            .content
            .contains("regular.txt:1: normal content with secret123"));
        assert!(!res.content.contains("keys.txt"));
        assert!(!res.content.contains("secrets"));
    }
}
