use crate::types::TokenUsage;

/// Pricing rates for an LLM model in USD per 1,000,000 tokens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelRates {
    /// Cost per 1,000,000 input tokens in USD.
    pub input_per_million: f64,
    /// Cost per 1,000,000 output tokens in USD.
    pub output_per_million: f64,
    /// Optional cost per 1,000,000 cache read tokens in USD.
    pub cache_read_per_million: Option<f64>,
}

impl ModelRates {
    pub const fn new(input_per_million: f64, output_per_million: f64) -> Self {
        Self {
            input_per_million,
            output_per_million,
            cache_read_per_million: None,
        }
    }

    pub const fn with_cache_read(mut self, cache_read_per_million: f64) -> Self {
        self.cache_read_per_million = Some(cache_read_per_million);
        self
    }

    /// Computes the total USD cost for the given token usage.
    pub fn calculate_cost(&self, usage: &TokenUsage) -> f64 {
        let input_cost = (usage.input_tokens as f64 / 1_000_000.0) * self.input_per_million;
        let output_cost = (usage.output_tokens as f64 / 1_000_000.0) * self.output_per_million;
        let cache_cost = match (usage.cache_read_tokens, self.cache_read_per_million) {
            (Some(tokens), Some(rate)) => (tokens as f64 / 1_000_000.0) * rate,
            _ => 0.0,
        };
        input_cost + output_cost + cache_cost
    }
}

/// Catalog of model pricing tiers and cost estimation helper.
pub struct ModelPricing;

impl ModelPricing {
    /// Returns the pricing tier for a model name using case-insensitive prefix/substring match.
    pub fn get_rates(model_name: &str) -> ModelRates {
        let name = model_name.to_lowercase();

        // Local / Ollama models ($0.00)
        if name.starts_with("ollama")
            || name.contains("llama")
            || name.contains("mistral")
            || name.contains("qwen")
            || name.contains("phi")
            || name.contains("local")
        {
            return ModelRates::new(0.0, 0.0);
        }

        // Anthropic Claude
        if name.contains("claude-3-7") || name.contains("claude-3.7") {
            return ModelRates::new(3.00, 15.00).with_cache_read(0.30);
        }
        if name.contains("claude-3-5-sonnet") || name.contains("claude-3.5-sonnet") {
            return ModelRates::new(3.00, 15.00).with_cache_read(0.30);
        }
        if name.contains("claude-3-5-haiku") || name.contains("claude-3.5-haiku") {
            return ModelRates::new(0.80, 4.00).with_cache_read(0.08);
        }
        if name.contains("claude-3-opus") || name.contains("claude-3.0-opus") {
            return ModelRates::new(15.00, 75.00);
        }
        if name.contains("claude-3-sonnet") {
            return ModelRates::new(3.00, 15.00);
        }
        if name.contains("claude-3-haiku") {
            return ModelRates::new(0.25, 1.25);
        }
        if name.contains("claude") {
            return ModelRates::new(3.00, 15.00);
        }

        // OpenAI GPT-4o / o1 / o3
        if name.contains("gpt-4o-mini") {
            return ModelRates::new(0.15, 0.60);
        }
        if name.contains("gpt-4o") {
            return ModelRates::new(2.50, 10.00);
        }
        if name.contains("o1-mini") {
            return ModelRates::new(1.10, 4.40);
        }
        if name.contains("o1") {
            return ModelRates::new(15.00, 60.00);
        }
        if name.contains("o3-mini") || name.contains("o3") {
            return ModelRates::new(1.10, 4.40);
        }
        if name.contains("gpt-4") {
            return ModelRates::new(10.00, 30.00);
        }

        // Google Gemini
        if name.contains("gemini-2.0-flash") || name.contains("gemini-2-flash") {
            return ModelRates::new(0.10, 0.40);
        }
        if name.contains("gemini-2.0")
            || name.contains("gemini-2")
            || name.contains("gemini-1.5-pro")
            || name.contains("gemini-pro")
        {
            return ModelRates::new(1.25, 5.00);
        }
        if name.contains("gemini-1.5-flash") {
            return ModelRates::new(0.075, 0.30);
        }
        if name.contains("gemini") {
            return ModelRates::new(0.10, 0.40);
        }

        // DeepSeek
        if name.contains("deepseek") {
            return ModelRates::new(0.14, 0.28);
        }

        // Fallback default
        ModelRates::new(3.00, 15.00)
    }

    /// Convenience calculation of total cost for a given model and token usage.
    pub fn calculate_cost(model_name: &str, usage: &TokenUsage) -> f64 {
        let rates = Self::get_rates(model_name);
        rates.calculate_cost(usage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_37_pricing() {
        let usage = TokenUsage::new(1_000_000, 1_000_000);
        let cost = ModelPricing::calculate_cost("claude-3-7-sonnet-20250219", &usage);
        assert!((cost - 18.00).abs() < 1e-6);
    }

    #[test]
    fn test_gpt4o_pricing() {
        let usage = TokenUsage::new(100_000, 10_000);
        // 100,000 / 1M * 2.50 = 0.25
        // 10,000 / 1M * 10.00 = 0.10
        // Total = 0.35
        let cost = ModelPricing::calculate_cost("gpt-4o", &usage);
        assert!((cost - 0.35).abs() < 1e-6);
    }

    #[test]
    fn test_gemini_2_flash_pricing() {
        let usage = TokenUsage::new(1_000_000, 1_000_000);
        let cost = ModelPricing::calculate_cost("gemini-2.0-flash-exp", &usage);
        assert!((cost - 0.50).abs() < 1e-6);
    }

    #[test]
    fn test_ollama_zero_cost() {
        let usage = TokenUsage::new(500_000, 500_000);
        let cost = ModelPricing::calculate_cost("ollama/llama3.3", &usage);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_o1_o3_pricing() {
        let usage = TokenUsage::new(1_000_000, 1_000_000);
        let o1_cost = ModelPricing::calculate_cost("o1", &usage);
        assert!((o1_cost - 75.00).abs() < 1e-6);

        let o3_mini_cost = ModelPricing::calculate_cost("o3-mini", &usage);
        assert!((o3_mini_cost - 5.50).abs() < 1e-6);
    }
}
