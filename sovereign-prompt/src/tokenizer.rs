use anyhow::Result;
use tiktoken_rs::{cl100k_base, CoreBPE};

pub struct Tokenizer {
    bpe: CoreBPE,
}

impl Tokenizer {
    pub fn new() -> Result<Self> {
        let bpe = cl100k_base()?;
        Ok(Self { bpe })
    }

    pub fn count(&self, text: &str) -> i64 {
        self.bpe.encode_with_special_tokens(text).len() as i64
    }
}
