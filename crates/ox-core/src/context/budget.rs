use crate::types::{ContentBlock, Message};

/// Lightweight, deterministic token estimator and context budget validator.
#[derive(Debug, Clone)]
pub struct TokenBudgeter {
    pub max_context_tokens: usize,
    pub max_output_tokens: usize,
}

impl Default for TokenBudgeter {
    fn default() -> Self {
        Self {
            max_context_tokens: 128_000,
            max_output_tokens: 4_096,
        }
    }
}

impl TokenBudgeter {
    pub fn new(max_context_tokens: usize, max_output_tokens: usize) -> Self {
        Self {
            max_context_tokens,
            max_output_tokens,
        }
    }

    /// Fast, robust heuristic token counter (approx 3.7 characters per token for code/prose, plus block overhead).
    pub fn estimate_text_tokens(text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        // Base character count heuristic with whitespace weighting
        let char_count = text.chars().count();
        let words = text.split_whitespace().count();

        // Heuristic: weighted average of char length and word count + 1 token baseline
        let est = (char_count + words * 2) / 4 + 1;
        est.max(1)
    }

    /// Estimates the total token count of a message including JSON metadata overhead.
    pub fn estimate_message_tokens(msg: &Message) -> usize {
        let mut total = 4; // Base per-message framing overhead

        for block in &msg.content {
            match block {
                ContentBlock::Text { text } => {
                    total += Self::estimate_text_tokens(text);
                }
                ContentBlock::Thinking { thinking } => {
                    total += Self::estimate_text_tokens(thinking);
                }
                ContentBlock::ToolCall(call) => {
                    total += Self::estimate_text_tokens(&call.name);
                    let args_str = call.arguments.to_string();
                    total += Self::estimate_text_tokens(&args_str) + 4;
                }
                ContentBlock::ToolResult(res) => {
                    total += Self::estimate_text_tokens(&res.tool_name);
                    total += Self::estimate_text_tokens(&res.content) + 4;
                }
            }
        }

        total
    }

    /// Estimates total tokens across a slice of messages.
    pub fn estimate_messages_total(messages: &[Message]) -> usize {
        messages.iter().map(Self::estimate_message_tokens).sum()
    }

    /// Checks if a conversation exceeds the available token budget.
    pub fn is_within_budget(&self, messages: &[Message]) -> bool {
        let total = Self::estimate_messages_total(messages);
        total + self.max_output_tokens <= self.max_context_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_estimation() {
        let text = "Hello world! This is a simple test.";
        let tokens = TokenBudgeter::estimate_text_tokens(text);
        assert!((6..=15).contains(&tokens));

        let msg = Message::user(text);
        let msg_tokens = TokenBudgeter::estimate_message_tokens(&msg);
        assert!(msg_tokens > tokens);
    }
}
