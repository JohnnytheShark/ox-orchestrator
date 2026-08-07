use crate::types::{Message, TokenUsage};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a node within the session DAG.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn new_from_str(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn short(&self) -> &str {
        if self.0.len() >= 8 {
            &self.0[..8]
        } else {
            &self.0
        }
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A node in the session DAG representing a conversational step / turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionNode {
    pub id: NodeId,
    pub parent_id: Option<NodeId>,
    pub message: Message,
    pub usage: Option<TokenUsage>,
    pub timestamp: DateTime<Utc>,
}

impl SessionNode {
    pub fn root(message: Message) -> Self {
        Self {
            id: NodeId::new(),
            parent_id: None,
            message,
            usage: None,
            timestamp: Utc::now(),
        }
    }

    pub fn child(parent_id: NodeId, message: Message, usage: Option<TokenUsage>) -> Self {
        Self {
            id: NodeId::new(),
            parent_id: Some(parent_id),
            message,
            usage,
            timestamp: Utc::now(),
        }
    }
}
