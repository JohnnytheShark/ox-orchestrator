use crate::error::SecurityError;
use std::path::{Component, Path, PathBuf};

/// Enforces a strict workspace boundary for all file and directory operations.
/// Prevents path traversal attacks (`../`), symlink escapes, and unauthorized access.
#[derive(Debug, Clone)]
pub struct PathJail {
    workspace_root: PathBuf,
}

impl PathJail {
    /// Creates a new `PathJail` rooted at `workspace_root`.
    /// The root is canonicalized to resolve any symbolic links or relative segments.
    pub fn new(workspace_root: impl AsRef<Path>) -> Result<Self, SecurityError> {
        let root = workspace_root.as_ref();
        let canonical_root = root
            .canonicalize()
            .map_err(|e| SecurityError::SymlinkError {
                path: root.to_path_buf(),
                source: e,
            })?;

        Ok(Self {
            workspace_root: canonical_root,
        })
    }

    /// Returns a reference to the canonicalized workspace root.
    pub fn root(&self) -> &Path {
        &self.workspace_root
    }

    /// Sanitizes and validates an input path (relative or absolute).
    ///
    /// If the path exists:
    ///   - Canonicalizes it and verifies it begins with the workspace root.
    ///
    /// If the path does NOT exist yet (e.g., for creating a new file):
    ///   - Traverses up to find the closest existing parent directory,
    ///     canonicalizes the parent, verifies containment, and reconstructs the sanitized path.
    ///
    /// Returns the verified canonical/normalized `PathBuf` on success,
    /// or `SecurityError::PathEscape` if the path escapes the jail.
    pub fn resolve_and_verify(
        &self,
        user_path: impl AsRef<Path>,
    ) -> Result<PathBuf, SecurityError> {
        let input = user_path.as_ref();

        // 1. If absolute, check directly; if relative, join with workspace root
        let combined = if input.is_absolute() {
            input.to_path_buf()
        } else {
            self.workspace_root.join(input)
        };

        // 2. Lexical normalization to reject obvious traversal
        let normalized = self.lexical_normalize(&combined);

        // 3. Canonical validation
        if normalized.exists() {
            let canonical = normalized
                .canonicalize()
                .map_err(|e| SecurityError::SymlinkError {
                    path: normalized.clone(),
                    source: e,
                })?;

            if !self.is_contained(&canonical) {
                return Err(SecurityError::PathEscape {
                    target: canonical,
                    root: self.workspace_root.clone(),
                });
            }

            Ok(canonical)
        } else {
            // For new files, resolve closest existing ancestor
            let mut current = normalized.as_path();
            let mut remaining_parts: Vec<&Path> = Vec::new();

            while !current.exists() {
                if let Some(filename) = current.file_name() {
                    remaining_parts.push(Path::new(filename));
                }
                match current.parent() {
                    Some(parent) => current = parent,
                    None => break,
                }
            }

            let canonical_parent = if current.exists() {
                current
                    .canonicalize()
                    .map_err(|e| SecurityError::SymlinkError {
                        path: current.to_path_buf(),
                        source: e,
                    })?
            } else {
                self.workspace_root.clone()
            };

            if !self.is_contained(&canonical_parent) {
                return Err(SecurityError::PathEscape {
                    target: canonical_parent,
                    root: self.workspace_root.clone(),
                });
            }

            let mut final_path = canonical_parent;
            for part in remaining_parts.into_iter().rev() {
                final_path.push(part);
            }

            // Final containment check
            let clean_final = self.lexical_normalize(&final_path);
            if !self.is_contained(&clean_final) {
                return Err(SecurityError::PathEscape {
                    target: clean_final,
                    root: self.workspace_root.clone(),
                });
            }

            Ok(clean_final)
        }
    }

    /// Checks whether `candidate` is equal to or a sub-path of `workspace_root`.
    fn is_contained(&self, candidate: &Path) -> bool {
        // Use normalized paths with case-insensitivity on Windows if applicable
        #[cfg(target_os = "windows")]
        {
            let root_str = self.workspace_root.to_string_lossy().to_lowercase();
            let cand_str = candidate.to_string_lossy().to_lowercase();
            cand_str.starts_with(&root_str)
        }

        #[cfg(not(target_os = "windows"))]
        {
            candidate.starts_with(&self.workspace_root)
        }
    }

    /// Normalizes path components without touching the filesystem.
    fn lexical_normalize(&self, path: &Path) -> PathBuf {
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                Component::Prefix(p) => components.push(Component::Prefix(p)),
                Component::RootDir => components.push(Component::RootDir),
                Component::CurDir => {}
                Component::ParentDir => {
                    if let Some(last) = components.last() {
                        if !matches!(last, Component::RootDir | Component::Prefix(_)) {
                            components.pop();
                        }
                    }
                }
                Component::Normal(c) => components.push(Component::Normal(c)),
            }
        }
        components.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_valid_subpath_resolution() {
        let dir = tempdir().unwrap();
        let jail = PathJail::new(dir.path()).unwrap();

        let subfile = dir.path().join("test.txt");
        std::fs::write(&subfile, "content").unwrap();

        let resolved = jail.resolve_and_verify("test.txt").unwrap();
        assert_eq!(resolved, subfile.canonicalize().unwrap());
    }

    #[test]
    fn test_blocks_directory_traversal() {
        let dir = tempdir().unwrap();
        let jail = PathJail::new(dir.path()).unwrap();

        let escape_attempt = "../../etc/passwd";
        let result = jail.resolve_and_verify(escape_attempt);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SecurityError::PathEscape { .. }
        ));
    }

    #[test]
    fn test_new_file_in_nested_dir_allowed() {
        let dir = tempdir().unwrap();
        let jail = PathJail::new(dir.path()).unwrap();

        let new_file_path = "src/nested/new_file.rs";
        let resolved = jail.resolve_and_verify(new_file_path).unwrap();
        assert!(resolved.to_string_lossy().contains("new_file.rs"));
    }
}
