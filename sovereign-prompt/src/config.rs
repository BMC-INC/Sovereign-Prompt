use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SovereignConfig {
    #[serde(default)]
    pub heuristics: HeuristicsConfig,
    #[serde(default)]
    pub injection: InjectionConfig,
}

#[derive(Debug, Clone, Deserialize)]
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
    pub fn load() -> Self {
        let path = std::env::var("SOVEREIGN_CONFIG_PATH")
            .unwrap_or_else(|_| "./sovereign_prompt.toml".to_string());

        if Path::new(&path).exists() {
            match std::fs::read_to_string(&path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => {
                        tracing::info!("Loaded config from {}", path);
                        return config;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse config file {}: {}", path, e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read config file {}: {}", path, e);
                }
            }
        }

        Self::default()
    }
}
