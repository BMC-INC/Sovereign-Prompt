use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SovereignConfig {
    #[serde(default)]
    pub heuristics: HeuristicsConfig,
    #[serde(default)]
    pub injection: InjectionConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeuristicsConfig {
    #[serde(default = "default_true")]
    pub vagueness: bool,
    #[serde(default = "default_true")]
    pub redundancy: bool,
    #[serde(default = "default_true")]
    pub missing_context: bool,
    #[serde(default = "default_true")]
    pub politeness: bool,
    #[serde(default = "default_true")]
    pub injection: bool,
    #[serde(default = "default_true")]
    pub task_separation: bool,
    #[serde(default = "default_true")]
    pub output_format: bool,
    #[serde(default = "default_true")]
    pub ambiguous_pronouns: bool,
    #[serde(default = "default_true")]
    pub governance: bool,

    #[serde(default = "default_redundancy_word_repeat")]
    pub redundancy_word_repeat: usize,
    #[serde(default = "default_pronoun_threshold")]
    pub pronoun_threshold: usize,
    #[serde(default = "default_context_min_length")]
    pub context_min_length: usize,
    #[serde(default = "default_conjunction_threshold")]
    pub conjunction_threshold: usize,
    #[serde(default = "default_format_min_length")]
    pub format_min_length: usize,

    #[serde(default)]
    pub extra_vague_terms: Vec<String>,
    #[serde(default)]
    pub extra_injection_patterns: Vec<String>,
    #[serde(default)]
    pub extra_polite_terms: Vec<String>,

    #[serde(default)]
    pub custom_checks: Vec<CustomCheck>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomCheck {
    pub name: String,
    pub pattern: String,
    #[serde(default = "default_custom_severity")]
    pub severity: String,
    pub message: String,
    #[serde(default)]
    pub suggestion: Option<String>,
}

fn default_custom_severity() -> String {
    "warning".to_string()
}

impl Default for HeuristicsConfig {
    fn default() -> Self {
        Self {
            vagueness: true,
            redundancy: true,
            missing_context: true,
            politeness: true,
            injection: true,
            task_separation: true,
            output_format: true,
            ambiguous_pronouns: true,
            governance: true,
            redundancy_word_repeat: 3,
            pronoun_threshold: 3,
            context_min_length: 50,
            conjunction_threshold: 2,
            format_min_length: 30,
            extra_vague_terms: Vec::new(),
            extra_injection_patterns: Vec::new(),
            extra_polite_terms: Vec::new(),
            custom_checks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum InjectionMode {
    #[default]
    Warn,
    Rewrite,
    Reject,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InjectionConfig {
    #[serde(default)]
    pub mode: InjectionMode,
}

impl Default for InjectionConfig {
    fn default() -> Self {
        Self {
            mode: InjectionMode::Warn,
        }
    }
}

fn default_true() -> bool {
    true
}
fn default_redundancy_word_repeat() -> usize {
    3
}
fn default_pronoun_threshold() -> usize {
    3
}
fn default_context_min_length() -> usize {
    50
}
fn default_conjunction_threshold() -> usize {
    2
}
fn default_format_min_length() -> usize {
    30
}

impl SovereignConfig {
    pub fn load() -> Result<Self> {
        match std::env::var_os("SOVEREIGN_CONFIG_PATH") {
            Some(path) => Self::load_file(Path::new(&path), false),
            None => Self::load_file(Path::new("./sovereign_prompt.toml"), true),
        }
    }

    fn load_file(path: &Path, allow_missing: bool) -> Result<Self> {
        let contents = match std::fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(error) if allow_missing && error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("Cannot read config {}", path.display()));
            }
        };
        let config = toml::from_str(&contents)
            .with_context(|| format!("Invalid config {}", path.display()))?;
        tracing::info!("Loaded config from {}", path.display());
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_optional_config_uses_defaults_but_explicit_path_is_required() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.toml");
        assert_eq!(
            SovereignConfig::load_file(&path, true)
                .unwrap()
                .injection
                .mode,
            InjectionMode::Warn
        );
        assert!(SovereignConfig::load_file(&path, false).is_err());
    }

    #[test]
    fn invalid_or_misspelled_policy_never_falls_back_to_warn() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("policy.toml");
        for contents in [
            "[injection",
            "[injection]\nmode = 'rejcet'",
            "[injection]\nmod = 'reject'",
            "[heuristics]\ngovernnace = false",
            "[injecion]\nmode = 'reject'",
        ] {
            std::fs::write(&path, contents).unwrap();
            assert!(
                SovereignConfig::load_file(&path, true).is_err(),
                "{contents}"
            );
            assert!(
                SovereignConfig::load_file(&path, false).is_err(),
                "{contents}"
            );
        }
    }

    #[test]
    fn valid_rejection_policy_is_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("policy.toml");
        std::fs::write(&path, "[injection]\nmode = 'reject'").unwrap();
        assert_eq!(
            SovereignConfig::load_file(&path, false)
                .unwrap()
                .injection
                .mode,
            InjectionMode::Reject
        );
    }

    #[test]
    fn unreadable_config_never_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        assert!(SovereignConfig::load_file(dir.path(), true).is_err());
    }
}
