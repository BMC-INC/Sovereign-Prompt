use crate::config::HeuristicsConfig;
use crate::governance::GovernancePolicy;
use crate::types::{FeedbackItem, HeuristicExplanation, Severity};
use regex::Regex;

const VAGUE_TERMS: &[&str] = &[
    "something",
    "somehow",
    "kind of",
    "sort of",
    "maybe",
    "perhaps",
    "possibly",
    "might",
    "stuff",
    "things",
    "etc",
    "and so on",
    "whatever",
    "anything",
];

const INJECTION_PATTERNS: &[&str] = &[
    "ignore previous",
    "ignore all",
    "disregard",
    "forget everything",
    "new instruction",
    "system:",
    "assistant:",
    "jailbreak",
];

const POLITE_TERMS: &[&str] = &[
    "please",
    "kindly",
    "could you",
    "would you mind",
    "if you don't mind",
    "thank you",
    "thanks",
];

const ACTION_WORDS: &[&str] = &["fix", "update", "change", "modify", "edit", "improve", "make"];

const CONJUNCTIONS: &[&str] = &[
    "and then",
    "also",
    "additionally",
    "furthermore",
    "as well as",
];

const FORMAT_SIGNALS: &[&str] = &[
    "list",
    "table",
    "json",
    "markdown",
    "bullet",
    "numbered",
    "summary",
    "paragraph",
    "code",
    "csv",
];

pub struct PromptAnalyzer;

impl PromptAnalyzer {
    /// Analyze with default config (backward compatible)
    pub fn analyze(prompt: &str) -> Vec<FeedbackItem> {
        let config = HeuristicsConfig::default();
        Self::analyze_with_config(prompt, &config)
    }

    /// Analyze with custom config
    pub fn analyze_with_config(prompt: &str, config: &HeuristicsConfig) -> Vec<FeedbackItem> {
        let mut feedback = Vec::new();

        if config.vagueness {
            check_vagueness(prompt, &mut feedback, config);
        }
        if config.redundancy {
            check_redundancy(prompt, &mut feedback, config);
        }
        if config.missing_context {
            check_missing_context(prompt, &mut feedback, config);
        }
        if config.politeness {
            check_politeness_tokens(prompt, &mut feedback, config);
        }
        if config.injection {
            check_prompt_injection(prompt, &mut feedback, config);
        }
        if config.task_separation {
            check_task_separation(prompt, &mut feedback, config);
        }
        if config.output_format {
            check_output_format(prompt, &mut feedback, config);
        }
        if config.ambiguous_pronouns {
            check_ambiguous_pronouns(prompt, &mut feedback, config);
        }
        if config.governance {
            check_governance_policy(prompt, &mut feedback);
        }

        // Run custom checks (user-defined plugins)
        run_custom_checks(prompt, &mut feedback, config);

        feedback
    }

    /// Analyze with explanations for each heuristic
    pub fn analyze_explained(
        prompt: &str,
        config: &HeuristicsConfig,
    ) -> (Vec<FeedbackItem>, Vec<HeuristicExplanation>) {
        let feedback = Self::analyze_with_config(prompt, config);
        let explanations = build_explanations(prompt, config);
        (feedback, explanations)
    }
}

fn build_explanations(prompt: &str, config: &HeuristicsConfig) -> Vec<HeuristicExplanation> {
    let lower = prompt.to_lowercase();
    let mut explanations = Vec::new();

    // 1. Vagueness
    {
        let mut all_terms: Vec<&str> = VAGUE_TERMS.to_vec();
        let extra: Vec<&str> = config.extra_vague_terms.iter().map(|s| s.as_str()).collect();
        all_terms.extend(extra);
        let found: Vec<String> = all_terms
            .iter()
            .filter(|t| lower.contains(*t))
            .map(|t| t.to_string())
            .collect();
        explanations.push(HeuristicExplanation {
            check_name: "vagueness_detection".to_string(),
            fired: config.vagueness && !found.is_empty(),
            reason: if found.is_empty() {
                None
            } else {
                Some(format!("Found {} vague terms", found.len()))
            },
            matched_patterns: found,
            threshold: Some(format!("Any of {} vague terms present", all_terms.len())),
        });
    }

    // 2. Redundancy
    {
        let words: Vec<&str> = prompt.split_whitespace().collect();
        let mut seen = std::collections::HashMap::new();
        for word in &words {
            let w = word.to_lowercase();
            let w = w.trim_matches(|c: char| !c.is_alphanumeric());
            if w.len() > 4 {
                *seen.entry(w.to_string()).or_insert(0usize) += 1;
            }
        }
        let duplicates: Vec<String> = seen
            .iter()
            .filter(|(_, &count)| count > config.redundancy_word_repeat)
            .map(|(word, _)| word.clone())
            .collect();
        explanations.push(HeuristicExplanation {
            check_name: "redundancy_analysis".to_string(),
            fired: config.redundancy && !duplicates.is_empty(),
            reason: if duplicates.is_empty() {
                None
            } else {
                Some(format!("Found {} repeated words", duplicates.len()))
            },
            matched_patterns: duplicates,
            threshold: Some(format!(
                "Words repeated more than {} times",
                config.redundancy_word_repeat
            )),
        });
    }

    // 3. Missing context
    {
        let has_action = ACTION_WORDS.iter().any(|w| lower.contains(w));
        let has_specifics = prompt.len() > config.context_min_length;
        let fired = config.missing_context && has_action && !has_specifics;
        let matched: Vec<String> = if has_action {
            ACTION_WORDS
                .iter()
                .filter(|w| lower.contains(*w))
                .map(|w| w.to_string())
                .collect()
        } else {
            Vec::new()
        };
        explanations.push(HeuristicExplanation {
            check_name: "missing_context".to_string(),
            fired,
            reason: if fired {
                Some(format!(
                    "Action verb found but prompt is only {} chars",
                    prompt.len()
                ))
            } else {
                None
            },
            matched_patterns: matched,
            threshold: Some(format!(
                "Action verb present with <{} chars context",
                config.context_min_length
            )),
        });
    }

    // 4. Politeness
    {
        let mut all_terms: Vec<&str> = POLITE_TERMS.to_vec();
        let extra: Vec<&str> = config.extra_polite_terms.iter().map(|s| s.as_str()).collect();
        all_terms.extend(extra);
        let found: Vec<String> = all_terms
            .iter()
            .filter(|t| lower.contains(*t))
            .map(|t| t.to_string())
            .collect();
        explanations.push(HeuristicExplanation {
            check_name: "politeness_tokens".to_string(),
            fired: config.politeness && !found.is_empty(),
            reason: if found.is_empty() {
                None
            } else {
                Some(format!("Found {} politeness tokens", found.len()))
            },
            matched_patterns: found,
            threshold: Some(format!("Any of {} polite terms present", all_terms.len())),
        });
    }

    // 5. Injection
    {
        let mut all_patterns: Vec<&str> = INJECTION_PATTERNS.to_vec();
        let extra: Vec<&str> = config
            .extra_injection_patterns
            .iter()
            .map(|s| s.as_str())
            .collect();
        all_patterns.extend(extra);
        let found: Vec<String> = all_patterns
            .iter()
            .filter(|p| lower.contains(*p))
            .map(|p| p.to_string())
            .collect();
        explanations.push(HeuristicExplanation {
            check_name: "prompt_injection".to_string(),
            fired: config.injection && !found.is_empty(),
            reason: if found.is_empty() {
                None
            } else {
                Some(format!("Found {} injection patterns", found.len()))
            },
            matched_patterns: found,
            threshold: Some(format!("Any of {} injection patterns present", all_patterns.len())),
        });
    }

    // 6. Task separation
    {
        let found: Vec<String> = CONJUNCTIONS
            .iter()
            .filter(|c| lower.contains(*c))
            .map(|c| c.to_string())
            .collect();
        let fired = config.task_separation && found.len() >= config.conjunction_threshold;
        explanations.push(HeuristicExplanation {
            check_name: "task_separation".to_string(),
            fired,
            reason: if fired {
                Some(format!("Found {} task-bundling conjunctions", found.len()))
            } else {
                None
            },
            matched_patterns: found,
            threshold: Some(format!(
                ">={} conjunctions indicate bundled tasks",
                config.conjunction_threshold
            )),
        });
    }

    // 7. Output format
    {
        let has_format = FORMAT_SIGNALS.iter().any(|f| lower.contains(f));
        let fired = config.output_format && !has_format && prompt.len() > config.format_min_length;
        explanations.push(HeuristicExplanation {
            check_name: "output_format".to_string(),
            fired,
            reason: if fired {
                Some("No output format signal detected".to_string())
            } else {
                None
            },
            matched_patterns: Vec::new(),
            threshold: Some(format!(
                "No format signal in prompts >{} chars",
                config.format_min_length
            )),
        });
    }

    // 8. Ambiguous pronouns
    {
        let re = ambiguous_pronoun_regex();
        let matches: Vec<String> = re
            .find_iter(prompt)
            .map(|m| m.as_str().to_string())
            .collect();
        let fired = config.ambiguous_pronouns && matches.len() >= config.pronoun_threshold;
        explanations.push(HeuristicExplanation {
            check_name: "ambiguous_pronouns".to_string(),
            fired,
            reason: if fired {
                Some(format!("Found {} ambiguous pronouns", matches.len()))
            } else {
                None
            },
            matched_patterns: matches,
            threshold: Some(format!(
                ">={} unresolved pronouns",
                config.pronoun_threshold
            )),
        });
    }

    // 9. Governance
    {
        let gov_feedback = GovernancePolicy::validate_prompt(prompt);
        let fired = config.governance && !gov_feedback.is_empty();
        let matched: Vec<String> = gov_feedback.iter().map(|f| f.message.clone()).collect();
        explanations.push(HeuristicExplanation {
            check_name: "governance_policy".to_string(),
            fired,
            reason: if fired {
                Some(format!("Found {} governance violations", gov_feedback.len()))
            } else {
                None
            },
            matched_patterns: matched,
            threshold: Some("Any governance policy violation".to_string()),
        });
    }

    // Custom checks
    for check in &config.custom_checks {
        if let Ok(re) = Regex::new(&check.pattern) {
            let matches: Vec<String> = re
                .find_iter(prompt)
                .map(|m| m.as_str().to_string())
                .collect();
            explanations.push(HeuristicExplanation {
                check_name: format!("custom:{}", check.name),
                fired: !matches.is_empty(),
                reason: if matches.is_empty() {
                    None
                } else {
                    Some(format!("Matched {} patterns", matches.len()))
                },
                matched_patterns: matches,
                threshold: Some(format!("Regex: {}", check.pattern)),
            });
        }
    }

    explanations
}

fn check_vagueness(prompt: &str, feedback: &mut Vec<FeedbackItem>, config: &HeuristicsConfig) {
    let lower = prompt.to_lowercase();
    let mut found: Vec<&str> = VAGUE_TERMS
        .iter()
        .filter(|t| lower.contains(*t))
        .copied()
        .collect();

    for extra in &config.extra_vague_terms {
        if lower.contains(extra.as_str()) {
            found.push(extra.as_str());
        }
    }

    if !found.is_empty() {
        feedback.push(FeedbackItem {
            category: "Clarity".to_string(),
            severity: Severity::Warning,
            message: format!("Vague language detected: {}", found.join(", ")),
            suggestion: Some("Replace vague terms with specific, measurable language.".to_string()),
        });
    }
}

fn check_redundancy(prompt: &str, feedback: &mut Vec<FeedbackItem>, config: &HeuristicsConfig) {
    let words: Vec<&str> = prompt.split_whitespace().collect();
    let mut seen = std::collections::HashMap::new();
    let mut duplicates = Vec::new();

    for word in &words {
        let lower = word.to_lowercase();
        let lower = lower.trim_matches(|c: char| !c.is_alphanumeric());
        if lower.len() > 4 {
            *seen.entry(lower.to_string()).or_insert(0) += 1;
        }
    }

    for (word, count) in &seen {
        if *count > config.redundancy_word_repeat {
            duplicates.push(word.clone());
        }
    }

    if !duplicates.is_empty() {
        feedback.push(FeedbackItem {
            category: "Redundancy".to_string(),
            severity: Severity::Info,
            message: format!("Repeated words detected: {}", duplicates.join(", ")),
            suggestion: Some("Consolidate repeated concepts to reduce token waste.".to_string()),
        });
    }
}

fn check_missing_context(prompt: &str, feedback: &mut Vec<FeedbackItem>, config: &HeuristicsConfig) {
    let lower = prompt.to_lowercase();
    let has_action = ACTION_WORDS.iter().any(|w| lower.contains(w));
    let has_specifics = prompt.len() > config.context_min_length;

    if has_action && !has_specifics {
        feedback.push(FeedbackItem {
            category: "Context".to_string(),
            severity: Severity::Critical,
            message: "Action requested but context is minimal.".to_string(),
            suggestion: Some(
                "Add: what specifically, where it is, what the expected result should be."
                    .to_string(),
            ),
        });
    }
}

fn check_politeness_tokens(
    prompt: &str,
    feedback: &mut Vec<FeedbackItem>,
    config: &HeuristicsConfig,
) {
    let lower = prompt.to_lowercase();
    let mut found: Vec<&str> = POLITE_TERMS
        .iter()
        .filter(|t| lower.contains(*t))
        .copied()
        .collect();

    for extra in &config.extra_polite_terms {
        if lower.contains(extra.as_str()) {
            found.push(extra.as_str());
        }
    }

    if !found.is_empty() {
        feedback.push(FeedbackItem {
            category: "Token Efficiency".to_string(),
            severity: Severity::Info,
            message: format!("Politeness tokens detected: {}", found.join(", ")),
            suggestion: Some("LLMs don't require politeness. Remove to save tokens.".to_string()),
        });
    }
}

fn check_prompt_injection(
    prompt: &str,
    feedback: &mut Vec<FeedbackItem>,
    config: &HeuristicsConfig,
) {
    let lower = prompt.to_lowercase();
    let mut found: Vec<&str> = INJECTION_PATTERNS
        .iter()
        .filter(|p| lower.contains(*p))
        .copied()
        .collect();

    for extra in &config.extra_injection_patterns {
        if lower.contains(extra.as_str()) {
            found.push(extra.as_str());
        }
    }

    if !found.is_empty() {
        feedback.push(FeedbackItem {
            category: "Security".to_string(),
            severity: Severity::Critical,
            message: format!(
                "Potential prompt injection pattern detected: {}",
                found.join(", ")
            ),
            suggestion: Some("Review prompt for injection risks before submission.".to_string()),
        });
    }
}

fn check_task_separation(
    prompt: &str,
    feedback: &mut Vec<FeedbackItem>,
    config: &HeuristicsConfig,
) {
    let lower = prompt.to_lowercase();
    let count = CONJUNCTIONS.iter().filter(|c| lower.contains(*c)).count();

    if count >= config.conjunction_threshold {
        feedback.push(FeedbackItem {
            category: "Task Separation".to_string(),
            severity: Severity::Warning,
            message: "Multiple tasks detected in a single prompt.".to_string(),
            suggestion: Some(
                "Split into separate prompts for each task to improve accuracy.".to_string(),
            ),
        });
    }
}

fn check_output_format(prompt: &str, feedback: &mut Vec<FeedbackItem>, config: &HeuristicsConfig) {
    let lower = prompt.to_lowercase();
    let has_format = FORMAT_SIGNALS.iter().any(|f| lower.contains(f));

    if !has_format && prompt.len() > config.format_min_length {
        feedback.push(FeedbackItem {
            category: "Output Format".to_string(),
            severity: Severity::Info,
            message: "No output format specified.".to_string(),
            suggestion: Some(
                "Specify desired format (e.g., JSON, bullet list, paragraph) for consistent results."
                    .to_string(),
            ),
        });
    }
}

fn ambiguous_pronoun_regex() -> &'static Regex {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(it|this|that|they|them|those|these)\b").unwrap())
}

fn check_ambiguous_pronouns(
    prompt: &str,
    feedback: &mut Vec<FeedbackItem>,
    config: &HeuristicsConfig,
) {
    let re = ambiguous_pronoun_regex();
    let matches: Vec<_> = re.find_iter(prompt).collect();

    if matches.len() >= config.pronoun_threshold {
        feedback.push(FeedbackItem {
            category: "Clarity".to_string(),
            severity: Severity::Warning,
            message: "Excessive ambiguous pronouns detected.".to_string(),
            suggestion: Some(
                "Replace pronouns with explicit nouns to remove ambiguity.".to_string(),
            ),
        });
    }
}

fn check_governance_policy(prompt: &str, feedback: &mut Vec<FeedbackItem>) {
    let gov_feedback = GovernancePolicy::validate_prompt(prompt);
    feedback.extend(gov_feedback);
}

fn run_custom_checks(prompt: &str, feedback: &mut Vec<FeedbackItem>, config: &HeuristicsConfig) {
    for check in &config.custom_checks {
        if let Ok(re) = Regex::new(&check.pattern) {
            let matches: Vec<String> = re
                .find_iter(prompt)
                .map(|m| m.as_str().to_string())
                .collect();
            if !matches.is_empty() {
                let severity = match check.severity.to_lowercase().as_str() {
                    "critical" | "crit" => Severity::Critical,
                    "info" => Severity::Info,
                    _ => Severity::Warning,
                };
                feedback.push(FeedbackItem {
                    category: format!("Custom: {}", check.name),
                    severity,
                    message: format!("{}: {}", check.message, matches.join(", ")),
                    suggestion: check.suggestion.clone(),
                });
            }
        }
    }
}
