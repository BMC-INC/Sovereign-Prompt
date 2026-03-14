use crate::governance::GovernancePolicy;
use crate::types::{FeedbackItem, Severity};
use regex::Regex;

pub struct PromptAnalyzer;

impl PromptAnalyzer {
    pub fn analyze(prompt: &str) -> Vec<FeedbackItem> {
        let mut feedback = Vec::new();

        check_vagueness(prompt, &mut feedback);
        check_redundancy(prompt, &mut feedback);
        check_missing_context(prompt, &mut feedback);
        check_politeness_tokens(prompt, &mut feedback);
        check_prompt_injection(prompt, &mut feedback);
        check_task_separation(prompt, &mut feedback);
        check_output_format(prompt, &mut feedback);
        check_ambiguous_pronouns(prompt, &mut feedback);
        check_governance_policy(prompt, &mut feedback);

        feedback
    }
}

fn check_vagueness(prompt: &str, feedback: &mut Vec<FeedbackItem>) {
    let vague_terms = [
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
    let lower = prompt.to_lowercase();
    let found: Vec<&str> = vague_terms
        .iter()
        .filter(|t| lower.contains(*t))
        .copied()
        .collect();

    if !found.is_empty() {
        feedback.push(FeedbackItem {
            category: "Clarity".to_string(),
            severity: Severity::Warning,
            message: format!("Vague language detected: {}", found.join(", ")),
            suggestion: Some("Replace vague terms with specific, measurable language.".to_string()),
        });
    }
}

fn check_redundancy(prompt: &str, feedback: &mut Vec<FeedbackItem>) {
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
        if *count > 2 {
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

fn check_missing_context(prompt: &str, feedback: &mut Vec<FeedbackItem>) {
    let action_words = [
        "fix", "update", "change", "modify", "edit", "improve", "make",
    ];
    let lower = prompt.to_lowercase();
    let has_action = action_words.iter().any(|w| lower.contains(w));
    let has_specifics = prompt.len() > 50;

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

fn check_politeness_tokens(prompt: &str, feedback: &mut Vec<FeedbackItem>) {
    let polite_terms = [
        "please",
        "kindly",
        "could you",
        "would you mind",
        "if you don't mind",
        "thank you",
        "thanks",
    ];
    let lower = prompt.to_lowercase();
    let found: Vec<&str> = polite_terms
        .iter()
        .filter(|t| lower.contains(*t))
        .copied()
        .collect();

    if !found.is_empty() {
        feedback.push(FeedbackItem {
            category: "Token Efficiency".to_string(),
            severity: Severity::Info,
            message: format!("Politeness tokens detected: {}", found.join(", ")),
            suggestion: Some("LLMs don't require politeness. Remove to save tokens.".to_string()),
        });
    }
}

fn check_prompt_injection(prompt: &str, feedback: &mut Vec<FeedbackItem>) {
    let injection_patterns = [
        "ignore previous",
        "ignore all",
        "disregard",
        "forget everything",
        "new instruction",
        "system:",
        "assistant:",
        "jailbreak",
    ];
    let lower = prompt.to_lowercase();
    let found: Vec<&str> = injection_patterns
        .iter()
        .filter(|p| lower.contains(*p))
        .copied()
        .collect();

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

fn check_task_separation(prompt: &str, feedback: &mut Vec<FeedbackItem>) {
    let conjunctions = [
        "and then",
        "also",
        "additionally",
        "furthermore",
        "as well as",
    ];
    let lower = prompt.to_lowercase();
    let count = conjunctions.iter().filter(|c| lower.contains(*c)).count();

    if count >= 2 {
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

fn check_output_format(prompt: &str, feedback: &mut Vec<FeedbackItem>) {
    let format_signals = [
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
    let lower = prompt.to_lowercase();
    let has_format = format_signals.iter().any(|f| lower.contains(f));

    if !has_format && prompt.len() > 30 {
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

fn check_ambiguous_pronouns(prompt: &str, feedback: &mut Vec<FeedbackItem>) {
    let re = ambiguous_pronoun_regex();
    let matches: Vec<_> = re.find_iter(prompt).collect();

    if matches.len() >= 3 {
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
