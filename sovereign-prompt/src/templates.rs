use crate::types::PromptTemplateSummary;

pub struct PromptTemplateLibrary;

impl PromptTemplateLibrary {
    pub fn available_domains() -> Vec<&'static str> {
        vec![
            "general",
            "backend",
            "frontend",
            "data",
            "security",
            "product",
            "documentation",
        ]
    }

    pub fn normalize_domain(domain: Option<&str>) -> String {
        let normalized = domain
            .unwrap_or("general")
            .trim()
            .to_lowercase()
            .replace('-', "_");

        if Self::available_domains().contains(&normalized.as_str()) {
            normalized
        } else {
            "general".to_string()
        }
    }

    pub fn apply(domain: &str, prompt: &str) -> (String, PromptTemplateSummary) {
        let domain = Self::normalize_domain(Some(domain));
        let (template_name, strategy, constraints) = match domain.as_str() {
            "backend" => (
                "Backend Precision",
                "Enforce reproducibility and production-safe delivery.",
                vec![
                    "State concrete file paths, APIs, and data contracts.",
                    "Include error handling, edge cases, and test coverage expectations.",
                    "Prefer deterministic outputs with explicit acceptance criteria.",
                ],
            ),
            "frontend" => (
                "Frontend Experience",
                "Optimize for UX clarity, accessibility, and responsive behavior.",
                vec![
                    "Define target layout states for desktop and mobile.",
                    "Specify semantic HTML and accessibility requirements.",
                    "Include visual QA checks and interaction behaviors.",
                ],
            ),
            "data" => (
                "Data Reliability",
                "Prioritize schema correctness and measurable quality checks.",
                vec![
                    "Specify source, transformations, and destination schema.",
                    "Define null/duplicate handling and validation rules.",
                    "Request metrics for correctness and performance.",
                ],
            ),
            "security" => (
                "Security Hardening",
                "Bias toward least privilege and explicit threat mitigation.",
                vec![
                    "Call out attack surfaces and trust boundaries.",
                    "Define required controls and verification steps.",
                    "Require no secret leakage and safe failure behavior.",
                ],
            ),
            "product" => (
                "Product Delivery",
                "Align implementation with user outcomes and measurable success.",
                vec![
                    "Define user persona, workflow, and expected outcome.",
                    "Specify success metrics and rollout constraints.",
                    "Request concise rationale for tradeoffs.",
                ],
            ),
            "documentation" => (
                "Documentation Clarity",
                "Structure content for discoverability and direct action.",
                vec![
                    "Use crisp section hierarchy and concrete examples.",
                    "Include prerequisites and exact commands.",
                    "Call out pitfalls and verification steps.",
                ],
            ),
            _ => (
                "General Precision",
                "Apply clear scope, format constraints, and completion checks.",
                vec![
                    "Define exact scope and desired output format.",
                    "Request direct, concise, and actionable output.",
                    "Include completion criteria for verification.",
                ],
            ),
        };

        let mut templated = String::new();
        templated.push_str(prompt.trim());
        templated.push_str("\n\nConstraints:\n");
        for (idx, item) in constraints.iter().enumerate() {
            templated.push_str(&format!("{}. {}\n", idx + 1, item));
        }
        templated.push_str("Return only what is required to complete the task.\n");

        (
            templated,
            PromptTemplateSummary {
                domain,
                template_name: template_name.to_string(),
                strategy: strategy.to_string(),
                constraints: constraints.iter().map(|s| s.to_string()).collect(),
            },
        )
    }
}
