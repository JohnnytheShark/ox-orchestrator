use crate::error::ToolError;
use crate::tool::{Tool, ToolContext};
use async_trait::async_trait;
use ox_core::types::{ToolCallId, ToolDefinition, ToolResult};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FindFilesTool;

#[derive(Deserialize)]
struct FindFilesArgs {
    path: Option<String>,
    pattern: Option<String>,
    max_results: Option<usize>,
}

#[async_trait]
impl Tool for FindFilesTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "find_files",
            "Find files and directories within the workspace matching an optional pattern.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Optional subdirectory to start search from."
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Optional substring pattern to filter filenames."
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of files to return (default 100)."
                    }
                }
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
        let args: FindFilesArgs =
            serde_json::from_value(arguments.clone()).map_err(|e| ToolError::InvalidArguments {
                tool: "find_files".to_string(),
                details: e.to_string(),
            })?;

        let search_root = match args.path {
            Some(ref p) => context.path_jail.resolve_and_verify(p)?,
            None => context.path_jail.root().to_path_buf(),
        };

        let max_results = args.max_results.unwrap_or(100);
        let pattern_lower = args.pattern.as_ref().map(|p| p.to_lowercase());

        let mut collected = Vec::new();
        walk_dir(
            &search_root,
            &mut collected,
            max_results * 2,
            &context.ignore_filter,
        );

        let mut matching_paths = Vec::new();
        for path in collected {
            if matching_paths.len() >= max_results {
                break;
            }

            let rel_path = path
                .strip_prefix(context.path_jail.root())
                .unwrap_or(&path)
                .display()
                .to_string();

            if let Some(ref pat) = pattern_lower {
                if rel_path.to_lowercase().contains(pat) {
                    matching_paths.push(rel_path);
                }
            } else {
                matching_paths.push(rel_path);
            }
        }

        if matching_paths.is_empty() {
            Ok(ToolResult::success(
                call_id.clone(),
                "find_files",
                "No matching files found.",
            ))
        } else {
            let count = matching_paths.len();
            let summary = format!("Found {} file(s):\n{}", count, matching_paths.join("\n"));
            Ok(ToolResult::success(call_id.clone(), "find_files", summary))
        }
    }
}

fn walk_dir(
    dir: &Path,
    results: &mut Vec<PathBuf>,
    max_limit: usize,
    ignore_filter: &crate::ignore::IgnoreFilter,
) {
    if results.len() >= max_limit {
        return;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if results.len() >= max_limit {
                break;
            }

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
                walk_dir(&path, results, max_limit, ignore_filter);
            } else if path.is_file() {
                results.push(path);
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
    async fn test_find_files_basic() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("main.rs"), "fn main() {}").unwrap();
        fs::write(src.join("lib.rs"), "pub fn run() {}").unwrap();

        let jail = PathJail::new(dir.path()).unwrap();
        let ctx = ToolContext::new(jail, EnvScrubber::new());

        let tool = FindFilesTool;
        let res = tool
            .execute(
                &ToolCallId::new("1"),
                &serde_json::json!({ "pattern": "main" }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(!res.is_error);
        assert!(res.content.contains("main.rs"));
        assert!(!res.content.contains("lib.rs"));
    }

    #[tokio::test]
    async fn test_find_files_masks_oxignore() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".oxignore"), "*.secret\nignored_dir/\n").unwrap();
        fs::write(dir.path().join("passwords.secret"), "12345").unwrap();
        fs::write(dir.path().join("visible.txt"), "hello").unwrap();

        let ignored_dir = dir.path().join("ignored_dir");
        fs::create_dir(&ignored_dir).unwrap();
        fs::write(ignored_dir.join("subfile.txt"), "hidden").unwrap();

        let jail = PathJail::new(dir.path()).unwrap();
        let ctx = ToolContext::new(jail, EnvScrubber::new());

        let tool = FindFilesTool;
        let res = tool
            .execute(&ToolCallId::new("1"), &serde_json::json!({}), &ctx)
            .await
            .unwrap();

        assert!(!res.is_error);
        assert!(res.content.contains("visible.txt"));
        assert!(!res.content.contains("passwords.secret"));
        assert!(!res.content.contains("subfile.txt"));
    }
}
