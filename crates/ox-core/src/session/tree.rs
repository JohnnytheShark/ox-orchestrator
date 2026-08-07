use crate::error::CoreError;
use crate::session::node::{NodeId, SessionNode};
use crate::types::{Message, TokenUsage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A Directed Acyclic Graph (DAG) representing the complete branched history of a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTree {
    pub id: String,
    pub title: String,
    pub nodes: HashMap<NodeId, SessionNode>,
    pub current_leaf_id: Option<NodeId>,
    pub root_id: Option<NodeId>,
}

impl SessionTree {
    /// Creates a new empty `SessionTree`.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            nodes: HashMap::new(),
            current_leaf_id: None,
            root_id: None,
        }
    }

    /// Appends a new message as a child of the current leaf node and advances the leaf pointer.
    pub fn append(&mut self, message: Message, usage: Option<TokenUsage>) -> NodeId {
        let new_node = match &self.current_leaf_id {
            Some(parent_id) => SessionNode::child(parent_id.clone(), message, usage),
            None => {
                let root = SessionNode::root(message);
                self.root_id = Some(root.id.clone());
                root
            }
        };

        let node_id = new_node.id.clone();
        self.nodes.insert(node_id.clone(), new_node);
        self.current_leaf_id = Some(node_id.clone());
        node_id
    }

    /// Reconstructs the linear message sequence from the root to the current active leaf.
    pub fn linear_history(&self) -> Vec<Message> {
        let mut messages = Vec::new();
        let mut current_id = self.current_leaf_id.clone();

        while let Some(id) = current_id {
            if let Some(node) = self.nodes.get(&id) {
                messages.push(node.message.clone());
                current_id = node.parent_id.clone();
            } else {
                break;
            }
        }

        messages.reverse();
        messages
    }

    /// Reconstructs the linear sequence of `SessionNode`s from root to current active leaf.
    pub fn active_path(&self) -> Vec<&SessionNode> {
        let mut path = Vec::new();
        let mut current_id = self.current_leaf_id.clone();

        while let Some(id) = current_id {
            if let Some(node) = self.nodes.get(&id) {
                path.push(node);
                current_id = node.parent_id.clone();
            } else {
                break;
            }
        }

        path.reverse();
        path
    }

    /// Switches the active branch pointer to `target_id`.
    /// Enables rewinding to an earlier state or branching into a new conversation path.
    pub fn checkout(&mut self, target_id: &NodeId) -> Result<(), CoreError> {
        if !self.nodes.contains_key(target_id) {
            return Err(CoreError::NodeNotFound(target_id.to_string()));
        }
        self.current_leaf_id = Some(target_id.clone());
        Ok(())
    }

    /// Steps back one turn in history (moves leaf pointer to parent).
    pub fn undo(&mut self) -> Result<Option<NodeId>, CoreError> {
        if let Some(current) = &self.current_leaf_id {
            let parent_id = self.nodes.get(current).and_then(|n| n.parent_id.clone());

            self.current_leaf_id = parent_id.clone();
            Ok(parent_id)
        } else {
            Ok(None)
        }
    }

    /// Calculates cumulative token usage along the active linear path.
    pub fn active_token_usage(&self) -> TokenUsage {
        let mut total = TokenUsage::default();
        for node in self.active_path() {
            if let Some(u) = &node.usage {
                total.add(u);
            }
        }
        total
    }

    /// Returns a list of all leaf nodes (branch tips) in the session.
    pub fn find_all_leaves(&self) -> Vec<NodeId> {
        let mut parent_set = std::collections::HashSet::new();
        for node in self.nodes.values() {
            if let Some(p) = &node.parent_id {
                parent_set.insert(p.clone());
            }
        }

        self.nodes
            .keys()
            .filter(|id| !parent_set.contains(id))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_tree_linear_append() {
        let mut tree = SessionTree::new("test-session", "Test Session");
        let n1 = tree.append(Message::user("Hello"), None);
        let n2 = tree.append(Message::assistant("Hi there!"), None);

        let history = tree.linear_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].text_content(), "Hello");
        assert_eq!(history[1].text_content(), "Hi there!");
        assert_eq!(tree.current_leaf_id, Some(n2));
        assert_eq!(tree.root_id, Some(n1));
    }

    #[test]
    fn test_session_tree_fork_and_branch() {
        let mut tree = SessionTree::new("test-fork", "Fork Test");
        let root = tree.append(Message::user("Task: write tests"), None);
        let branch_a = tree.append(Message::assistant("Branch A attempt"), None);

        assert_eq!(tree.linear_history().len(), 2);

        // Fork from root
        tree.checkout(&root).unwrap();
        let branch_b = tree.append(Message::assistant("Branch B alternative"), None);

        let history_b = tree.linear_history();
        assert_eq!(history_b.len(), 2);
        assert_eq!(history_b[1].text_content(), "Branch B alternative");

        let leaves = tree.find_all_leaves();
        assert_eq!(leaves.len(), 2);
        assert!(leaves.contains(&branch_a));
        assert!(leaves.contains(&branch_b));
    }

    #[test]
    fn test_undo() {
        let mut tree = SessionTree::new("test-undo", "Undo Test");
        let root = tree.append(Message::user("1"), None);
        let _child = tree.append(Message::assistant("2"), None);

        let prev = tree.undo().unwrap();
        assert_eq!(prev, Some(root));
        assert_eq!(tree.linear_history().len(), 1);
    }
}
