use crate::tokenizer::Tokenizer;
use crate::types::{FeedbackItem, PromptVariant, Severity};
use regex::Regex;
use std::sync::OnceLock;

fn politeness_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(please|kindly|could you|would you mind|if you don't mind|thank you|thanks)\b\s*,?\s*"
        ).unwrap()
    })
}

fn whitespace_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\s{2,}").unwrap())
}

pub struct PromptOptimizer;

impl PromptOptimizer {
    pub fn refine(prompt: &str, feedback: &[FeedbackItem], _tokenizer: &Tokenizer) -> String {
        // Strip politeness tokens (case-insensitive)
        let mut refined = politeness_regex().replace_all(prompt, "").to_string();

        // Normalize whitespace
        refined = whitespace_regex()
            .replace_all(&refined, " ")
            .trim()
            .to_string();

        // Append format instruction if missing
        let has_critical = feedback
            .iter()
            .any(|f| matches!(f.severity, Severity::Critical));

        if !has_critical {
            let has_format_signal = ["list", "json", "table", "bullet", "code", "paragraph"]
                .iter()
                .any(|s| refined.to_lowercase().contains(s));

            if !has_format_signal && refined.len() > 30 {
                refined.push_str(" Respond concisely and directly.");
            }
        }

        refined
    }

    pub fn generate_variants(prompt: &str, tokenizer: &Tokenizer) -> Vec<PromptVariant> {
        Self::generate_variants_with_model(prompt, tokenizer, crate::tokenizer::DEFAULT_TOKEN_MODEL)
    }

    pub fn generate_variants_with_model(
        prompt: &str,
        tokenizer: &Tokenizer,
        model: &str,
    ) -> Vec<PromptVariant> {
        let precision_text = format!("{} Be exact, technical, and minimal.", prompt);
        let creative_text = format!("{} Think broadly and explore multiple angles.", prompt);
        let concise_text: String = prompt
            .split_whitespace()
            .take(prompt.split_whitespace().count() / 2 + 1)
            .collect::<Vec<_>>()
            .join(" ");

        vec![
            PromptVariant {
                label: "Precision".to_string(),
                prompt: precision_text.clone(),
                token_count: tokenizer
                    .count_for_model(model, &precision_text)
                    .unwrap_or_else(|| tokenizer.count(&precision_text)),
                use_case: "Engineering, code generation, structured output".to_string(),
            },
            PromptVariant {
                label: "Creative".to_string(),
                prompt: creative_text.clone(),
                token_count: tokenizer
                    .count_for_model(model, &creative_text)
                    .unwrap_or_else(|| tokenizer.count(&creative_text)),
                use_case: "Brainstorming, ideation, content generation".to_string(),
            },
            PromptVariant {
                label: "Concise".to_string(),
                prompt: concise_text.clone(),
                token_count: tokenizer
                    .count_for_model(model, &concise_text)
                    .unwrap_or_else(|| tokenizer.count(&concise_text)),
                use_case: "Quick lookups, simple commands, minimal context".to_string(),
            },
        ]
    }
}
