use crate::error::CoreError;
use crate::session::tree::SessionTree;
use std::fs;
use std::path::{Path, PathBuf};

/// Handles persistence of session trees to disk with atomic write guarantees.
pub struct SessionStorage {
    base_dir: PathBuf,
}

impl SessionStorage {
    /// Creates a new storage manager rooted at the specified directory (e.g., `.ox/sessions`).
    pub fn new(base_dir: impl AsRef<Path>) -> Result<Self, CoreError> {
        let path = base_dir.as_ref().to_path_buf();
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        Ok(Self { base_dir: path })
    }

    /// Saves a session tree atomically using a tempfile swap.
    pub fn save(&self, tree: &SessionTree) -> Result<PathBuf, CoreError> {
        let target_file = self.base_dir.join(format!("{}.json", tree.id));
        let temp_file =
            self.base_dir
                .join(format!("{}.json.tmp.{}", tree.id, uuid::Uuid::new_v4()));

        let json = serde_json::to_string_pretty(tree)?;
        fs::write(&temp_file, json)?;
        fs::rename(&temp_file, &target_file)?;

        Ok(target_file)
    }

    /// Loads a session tree by its session ID.
    pub fn load(&self, session_id: &str) -> Result<SessionTree, CoreError> {
        let file_path = self.base_dir.join(format!("{}.json", session_id));
        if !file_path.exists() {
            return Err(CoreError::NodeNotFound(format!(
                "Session file '{}' not found",
                session_id
            )));
        }

        let content = fs::read_to_string(&file_path)?;
        let tree: SessionTree = serde_json::from_str(&content)?;
        Ok(tree)
    }

    /// Lists all stored session IDs.
    pub fn list_sessions(&self) -> Result<Vec<String>, CoreError> {
        let mut session_ids = Vec::new();
        if self.base_dir.exists() {
            for entry in fs::read_dir(&self.base_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        session_ids.push(stem.to_string());
                    }
                }
            }
        }
        Ok(session_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Message;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_session() {
        let dir = tempdir().unwrap();
        let storage = SessionStorage::new(dir.path()).unwrap();

        let mut tree = SessionTree::new("session-123", "Unit Test Session");
        tree.append(Message::user("Hello storage"), None);

        storage.save(&tree).unwrap();

        let loaded = storage.load("session-123").unwrap();
        assert_eq!(loaded.id, "session-123");
        assert_eq!(loaded.linear_history().len(), 1);
        assert_eq!(loaded.linear_history()[0].text_content(), "Hello storage");
    }
}
