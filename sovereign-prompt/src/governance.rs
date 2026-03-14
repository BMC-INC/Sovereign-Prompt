use crate::types::{FeedbackItem, Severity};
use regex::Regex;
use std::sync::OnceLock;

pub struct GovernancePolicy {
    pub version: String,
}

impl GovernancePolicy {
    pub fn current() -> Self {
        Self {
            version: "v1.0.0".to_string(),
        }
    }

    /// Governance check: detect sensitive data patterns in a prompt.
    pub fn validate_prompt(prompt: &str) -> Vec<FeedbackItem> {
        let mut feedback = Vec::new();

        check_sensitive_patterns(prompt, &mut feedback);
        check_pii_references(prompt, &mut feedback);

        feedback
    }

    /// Determine initial approval status based on governance feedback.
    pub fn determine_status(governance_feedback: &[FeedbackItem]) -> String {
        if governance_feedback
            .iter()
            .any(|f| matches!(f.severity, Severity::Critical))
        {
            "rejected".to_string()
        } else if governance_feedback
            .iter()
            .any(|f| matches!(f.severity, Severity::Warning))
        {
            "pending".to_string()
        } else {
            "approved".to_string()
        }
    }
}

fn ssn_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap())
}

fn credit_card_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b").unwrap())
}

fn credential_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)(api[_-]?key|secret[_-]?key|password)\s*[:=]\s*\S+").unwrap()
    })
}

fn check_sensitive_patterns(prompt: &str, feedback: &mut Vec<FeedbackItem>) {
    let checks: Vec<(&Regex, &str)> = vec![
        (ssn_regex(), "SSN pattern"),
        (credit_card_regex(), "credit card pattern"),
        (credential_regex(), "credential pattern"),
    ];

    for (re, label) in checks {
        if re.is_match(prompt) {
            feedback.push(FeedbackItem {
                category: "Governance".to_string(),
                severity: Severity::Critical,
                message: format!("Sensitive data detected: {}", label),
                suggestion: Some(
                    "Remove sensitive data before optimization. This violates governance policy."
                        .to_string(),
                ),
            });
        }
    }
}

fn check_pii_references(prompt: &str, feedback: &mut Vec<FeedbackItem>) {
    let pii_terms = [
        "social security",
        "date of birth",
        "passport number",
        "bank account",
    ];
    let lower = prompt.to_lowercase();
    let found: Vec<&&str> = pii_terms.iter().filter(|t| lower.contains(*t)).collect();

    if !found.is_empty() {
        feedback.push(FeedbackItem {
            category: "Governance".to_string(),
            severity: Severity::Warning,
            message: format!(
                "PII references detected: {}",
                found.iter().map(|s| **s).collect::<Vec<_>>().join(", ")
            ),
            suggestion: Some(
                "Ensure PII handling complies with data governance policy.".to_string(),
            ),
        });
    }
}
