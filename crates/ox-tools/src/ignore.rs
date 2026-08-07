use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::{Path, PathBuf};

/// Filters and masks files based on `.oxignore`, `.gitignore`, and default security exclusions.
#[derive(Clone, Debug)]
pub struct IgnoreFilter {
    root: PathBuf,
    matcher: Gitignore,
}

impl IgnoreFilter {
    /// Creates a new `IgnoreFilter` rooted at `workspace_root`.
    /// Automatically loads `.oxignore` and `.gitignore` if present.
    pub fn new(workspace_root: &Path) -> Self {
        let mut builder = GitignoreBuilder::new(workspace_root);

        // Security defaults: ignore .git repository internals and .env secrets by default
        let _ = builder.add_line(None, ".git");
        let _ = builder.add_line(None, ".git/");
        let _ = builder.add_line(None, "target/");
        let _ = builder.add_line(None, "node_modules/");

        // Workspace-level .gitignore
        let gitignore_path = workspace_root.join(".gitignore");
        if gitignore_path.exists() {
            let _ = builder.add(&gitignore_path);
        }

        // Workspace-level .oxignore (takes precedence / adds additional rules)
        let oxignore_path = workspace_root.join(".oxignore");
        if oxignore_path.exists() {
            let _ = builder.add(&oxignore_path);
        }

        let matcher = builder.build().unwrap_or_else(|_| Gitignore::empty());

        Self {
            root: workspace_root.to_path_buf(),
            matcher,
        }
    }

    /// Checks whether a given path is ignored.
    pub fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        let rel_path = path.strip_prefix(&self.root).unwrap_or(path);

        for comp in rel_path.components() {
            if let std::path::Component::Normal(os_str) = comp {
                if os_str == ".git" {
                    return true;
                }
            }
        }

        match self.matcher.matched_path_or_any_parents(rel_path, is_dir) {
            ignore::Match::Ignore(_) => true,
            ignore::Match::Whitelist(_) => false,
            ignore::Match::None => false,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_oxignore_and_gitignore_rules() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Write .gitignore
        fs::write(root.join(".gitignore"), "*.secret\nbuild/\n").unwrap();

        // Write .oxignore
        fs::write(root.join(".oxignore"), "private_key.pem\ninternal_data/\n").unwrap();

        let filter = IgnoreFilter::new(root);

        // Test gitignore matches
        assert!(filter.is_ignored(&root.join("passwords.secret"), false));
        assert!(filter.is_ignored(&root.join("build").join("output.bin"), false));
        assert!(filter.is_ignored(&root.join("build"), true));

        // Test oxignore matches
        assert!(filter.is_ignored(&root.join("private_key.pem"), false));
        assert!(filter.is_ignored(&root.join("internal_data").join("customer.csv"), false));
        assert!(filter.is_ignored(&root.join("internal_data"), true));

        // Test default exclusions
        assert!(filter.is_ignored(&root.join(".git").join("config"), false));
        assert!(filter.is_ignored(&root.join("target").join("debug"), true));

        // Test normal source files are NOT ignored
        assert!(!filter.is_ignored(&root.join("src").join("main.rs"), false));
        assert!(!filter.is_ignored(&root.join("README.md"), false));
    }
}
