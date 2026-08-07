use crate::context::budget::TokenBudgeter;
use crate::types::{ContentBlock, Message, Role};

/// Compresses message history to fit within a specified token budget while preserving critical context.
pub struct ContextCompactor {
    pub budgeter: TokenBudgeter,
}

impl ContextCompactor {
    pub fn new(budgeter: TokenBudgeter) -> Self {
        Self { budgeter }
    }

    /// Truncates or compacts message history if it exceeds the available context budget.
    /// Preserves system messages and the most recent turns while trimming oldest intermediate tool outputs.
    pub fn compact(&self, messages: &[Message]) -> Vec<Message> {
        if self.budgeter.is_within_budget(messages) || messages.len() <= 2 {
            return messages.to_vec();
        }

        let mut compacted = messages.to_vec();
        let target_budget = self
            .budgeter
            .max_context_tokens
            .saturating_sub(self.budgeter.max_output_tokens);

        // Step 1: Compress large intermediate tool outputs in historical messages
        let len = compacted.len();
        if len > 3 {
            for msg in compacted.iter_mut().take(len - 2).skip(1) {
                if msg.role == Role::Tool {
                    if let Some(ContentBlock::ToolResult(res)) = msg.content.first_mut() {
                        if res.content.len() > 300 {
                            res.content = format!(
                                "{}... [Output truncated to save context]",
                                &res.content[..200]
                            );
                        }
                    }
                }
            }
        }

        // Step 2: If still overflowing, apply sliding window from the front (preserving system message at index 0)
        while TokenBudgeter::estimate_messages_total(&compacted) > target_budget
            && compacted.len() > 3
        {
            // Keep index 0 (system message), drop index 1 (oldest turn)
            compacted.remove(1);
        }

        compacted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_preserves_under_budget() {
        let budgeter = TokenBudgeter::new(100_000, 4_000);
        let compactor = ContextCompactor::new(budgeter);

        let messages = vec![
            Message::system("System instructions"),
            Message::user("Hello"),
            Message::assistant("Hi there"),
        ];

        let result = compactor.compact(&messages);
        assert_eq!(result.len(), 3);
    }
}
