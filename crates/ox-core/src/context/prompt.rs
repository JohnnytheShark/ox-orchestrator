use std::path::Path;

/// Constructs the system prompt with workspace context, operational guidelines,
/// and repository instruction files (`AGENTS.md`, `CLAUDE.md`, `OX.md`, `.ox/rules.md`).
pub struct SystemPromptBuilder;

impl SystemPromptBuilder {
    pub const INSTRUCTION_CANDIDATES: &[&str] =
        &["AGENTS.md", "CLAUDE.md", "OX.md", ".ox/rules.md"];

    /// Searches the workspace root for repository instruction / rules files.
    pub fn discover_instructions(workspace_root: &Path) -> Option<(String, String)> {
        for candidate in Self::INSTRUCTION_CANDIDATES {
            let path = workspace_root.join(candidate);
            if path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        return Some((candidate.to_string(), trimmed.to_string()));
                    }
                }
            }
        }
        None
    }

    /// Builds the full system prompt, prepending discovered project guidelines.
    pub fn build(workspace_root: &Path) -> String {
        let os_name = std::env::consts::OS;
        let root_str = workspace_root.display().to_string();

        let base_prompt = format!(
            r#"You are ox, a high-performance, minimalist AI coding assistant and agent harness.
Operating System: {os_name}
Workspace Root: {root_str}

Guidelines:
1. Be concise, direct, and pragmatic. Avoid unnecessary preamble or conversational filler.
2. Use tools to inspect files, edit code, and execute commands. Always verify assumptions against actual codebase content.
3. When modifying files, prefer precise surgical edits or atomic overwrites.
4. Explain what you are doing before calling tools when appropriate.
5. Adhere strictly to the workspace boundary and practice secure coding habits."#
        );

        if let Some((filename, instructions)) = Self::discover_instructions(workspace_root) {
            format!(
                r#"# Project Instructions ({filename})
{instructions}

---

{base_prompt}"#
            )
        } else {
            base_prompt
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_build_without_instructions() {
        let dir = tempdir().unwrap();
        let prompt = SystemPromptBuilder::build(dir.path());
        assert!(prompt.contains("You are ox"));
        assert!(!prompt.contains("# Project Instructions"));
    }

    #[test]
    fn test_discover_and_inject_agents_md() {
        let dir = tempdir().unwrap();
        let agents_file = dir.path().join("AGENTS.md");
        fs::write(&agents_file, "Always run cargo clippy before committing.").unwrap();

        let (filename, content) = SystemPromptBuilder::discover_instructions(dir.path()).unwrap();
        assert_eq!(filename, "AGENTS.md");
        assert_eq!(content, "Always run cargo clippy before committing.");

        let prompt = SystemPromptBuilder::build(dir.path());
        assert!(prompt.contains("# Project Instructions (AGENTS.md)"));
        assert!(prompt.contains("Always run cargo clippy before committing."));
        assert!(prompt.contains("You are ox"));
    }

    #[test]
    fn test_discover_ox_rules_md() {
        let dir = tempdir().unwrap();
        let ox_dir = dir.path().join(".ox");
        fs::create_dir(&ox_dir).unwrap();
        let rules_file = ox_dir.join("rules.md");
        fs::write(&rules_file, "Follow rust 2021 edition idioms.").unwrap();

        let (filename, content) = SystemPromptBuilder::discover_instructions(dir.path()).unwrap();
        assert_eq!(filename, ".ox/rules.md");
        assert_eq!(content, "Follow rust 2021 edition idioms.");

        let prompt = SystemPromptBuilder::build(dir.path());
        assert!(prompt.contains("# Project Instructions (.ox/rules.md)"));
        assert!(prompt.contains("Follow rust 2021 edition idioms."));
    }

    #[test]
    fn test_candidate_precedence() {
        let dir = tempdir().unwrap();
        // Create both AGENTS.md and CLAUDE.md
        fs::write(dir.path().join("AGENTS.md"), "Priority 1").unwrap();
        fs::write(dir.path().join("CLAUDE.md"), "Priority 2").unwrap();

        let (filename, content) = SystemPromptBuilder::discover_instructions(dir.path()).unwrap();
        assert_eq!(filename, "AGENTS.md");
        assert_eq!(content, "Priority 1");
    }
}
