use serde::{Deserialize, Serialize};

/// Cumulative and per-turn token usage statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub cache_read_tokens: Option<usize>,
    pub cache_write_tokens: Option<usize>,
}

impl TokenUsage {
    pub fn new(input_tokens: usize, output_tokens: usize) -> Self {
        Self {
            input_tokens,
            output_tokens,
            cache_read_tokens: None,
            cache_write_tokens: None,
        }
    }

    pub fn total_tokens(&self) -> usize {
        self.input_tokens + self.output_tokens
    }

    pub fn add(&mut self, other: &TokenUsage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        if let Some(r) = other.cache_read_tokens {
            *self.cache_read_tokens.get_or_insert(0) += r;
        }
        if let Some(w) = other.cache_write_tokens {
            *self.cache_write_tokens.get_or_insert(0) += w;
        }
    }
}
