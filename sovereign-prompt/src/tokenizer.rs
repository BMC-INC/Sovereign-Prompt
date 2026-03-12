use anyhow::Result;
use std::collections::BTreeMap;
use tiktoken_rs::{cl100k_base, o200k_base, p50k_base, r50k_base, CoreBPE};

pub const DEFAULT_TOKEN_MODEL: &str = "cl100k_base";

pub struct Tokenizer {
    models: BTreeMap<String, CoreBPE>,
}

impl Tokenizer {
    pub fn new() -> Result<Self> {
        let mut models = BTreeMap::new();
        models.insert("cl100k_base".to_string(), cl100k_base()?);
        models.insert("o200k_base".to_string(), o200k_base()?);
        models.insert("p50k_base".to_string(), p50k_base()?);
        models.insert("r50k_base".to_string(), r50k_base()?);
        Ok(Self { models })
    }

    pub fn count(&self, text: &str) -> i64 {
        self.count_for_model(DEFAULT_TOKEN_MODEL, text).unwrap_or(0)
    }

    pub fn count_for_model(&self, model: &str, text: &str) -> Option<i64> {
        self.models
            .get(model)
            .map(|bpe| bpe.encode_with_special_tokens(text).len() as i64)
    }

    pub fn count_across_models(&self, text: &str) -> BTreeMap<String, i64> {
        self.models
            .iter()
            .map(|(name, bpe)| {
                (
                    name.clone(),
                    bpe.encode_with_special_tokens(text).len() as i64,
                )
            })
            .collect()
    }

    pub fn available_models(&self) -> Vec<String> {
        self.models.keys().cloned().collect()
    }

    pub fn is_supported_model(&self, model: &str) -> bool {
        self.models.contains_key(model)
    }
}
