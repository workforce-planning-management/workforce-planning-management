//! Pure rules for learning & development: proficiency scale, the
//! mentorship lifecycle, and the honest path-progress derivation. No
//! I/O; progress counts only real course completions, never
//! interpolated.

/// Skill categories.
pub const SKILL_CATEGORIES: &[&str] = &["technical", "leadership", "compliance", "domain", "other"];

/// Ordered keyword rules for suggesting a skill's category from its name:
/// `(category, keywords)`; the first category with a keyword found in the
/// lower-cased name wins. Coverage is partial by design (about 45% of the
/// UK GDAD PCF skills on first use) — the rest stay `other`. A **suggestion** with its reason, for a planner to
/// accept or correct — never applied automatically.
const CATEGORY_RULES: &[(&str, &[&str])] = &[
    (
        "compliance",
        &[
            "compliance",
            "legal",
            "regulat",
            "ethic",
            "privacy",
            "data protection",
            "governance",
            "audit",
            "risk",
            "safeguard",
        ],
    ),
    (
        "leadership",
        &[
            "leadership",
            "coaching",
            "mentoring",
            "people management",
            "line management",
            "strategic thinking",
            "stakeholder",
            "influenc",
            "negotiat",
            "vision",
        ],
    ),
    // Domain before technical: "Content concepts and prototyping" is a content
    // design skill, not a technical one.
    (
        "domain",
        &[
            "user research",
            "content",
            "service design",
            "interaction design",
            "product",
            "delivery",
            "agile",
            "business analysis",
            "procurement",
            "commercial",
            "finance",
            "performance analysis",
            "user focus",
            "accessibility",
        ],
    ),
    (
        "technical",
        &[
            "programming",
            "software",
            "development",
            "engineering",
            "architecture",
            "testing",
            "network",
            "infrastructure",
            "database",
            "cloud",
            "security",
            "machine learning",
            "automation",
            "prototyping",
            "systems design",
            "modelling",
            "analytics",
            "coding",
            "devops",
            "deployment",
            "monitoring",
            "incident",
            "service support",
            "data analysis",
            "data visuali",
            "configuration management",
            "capacity management",
            "statistic",
            "applied maths",
        ],
    ),
];

/// Suggest a category and the keyword that triggered it; `None` when no rule
/// matches (the skill stays `other`).
#[must_use]
pub fn suggest_category(name: &str) -> Option<(&'static str, &'static str)> {
    let lower = name.to_lowercase();
    CATEGORY_RULES.iter().find_map(|(category, keywords)| {
        keywords
            .iter()
            .find(|keyword| lower.contains(**keyword))
            .map(|keyword| (*category, *keyword))
    })
}

/// Trim and validate a skill name for a rename (non-empty, bounded).
///
/// # Errors
/// A message naming the problem.
pub fn normalize_skill_name(input: &str) -> Result<String, String> {
    let name = input.trim();
    if name.is_empty() {
        return Err("name is required".to_string());
    }
    if name.chars().count() > 200 {
        return Err("name is longer than 200 characters".to_string());
    }
    Ok(name.to_string())
}

/// The proficiency / target scale (declared 1–5).
#[must_use]
pub fn valid_proficiency(level: i32) -> bool {
    (1..=5).contains(&level)
}

/// Mentorship statuses.
pub const MENTORSHIP_STATUSES: &[&str] = &["proposed", "active", "completed", "ended"];

/// The mentorship lifecycle: proposed → active → completed, and
/// `ended` reachable from proposed or active. Terminal states do not
/// transition.
///
/// # Errors
///
/// A human-readable refusal naming the legal moves.
pub fn mentorship_transition(current: &str, to: &str) -> Result<(), String> {
    if !MENTORSHIP_STATUSES.contains(&to) {
        return Err(format!(
            "unknown status `{to}` (statuses: {MENTORSHIP_STATUSES:?})"
        ));
    }
    let ok = matches!(
        (current, to),
        ("proposed", "active" | "ended") | ("active", "completed" | "ended")
    );
    if ok {
        Ok(())
    } else {
        Err(format!(
            "illegal transition `{current}` → `{to}` (proposed→active→completed, or end an open one)"
        ))
    }
}

/// Path progress: how many of `step_courses` a member has completed,
/// given the set of course refs they have **completed** enrolments
/// for. Returns `(completed, total)`; `total` is the step count.
/// Counts real completions only.
#[must_use]
pub fn path_progress(step_courses: &[String], completed_courses: &[String]) -> (usize, usize) {
    let done = step_courses
        .iter()
        .filter(|course| completed_courses.iter().any(|c| c == *course))
        .count();
    (done, step_courses.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Suggestions come with their reason, follow the rule order, and a
    /// name no rule matches gets none.
    #[test]
    fn category_suggestions() {
        assert_eq!(
            suggest_category("Programming and build (software engineering)"),
            Some(("technical", "programming"))
        );
        assert_eq!(
            suggest_category("Data compliance and security"),
            Some(("compliance", "compliance")),
            "compliance outranks security"
        );
        assert_eq!(
            suggest_category("Stakeholder relationship management"),
            Some(("leadership", "stakeholder"))
        );
        assert_eq!(
            suggest_category("User research"),
            Some(("domain", "user research"))
        );
        assert_eq!(suggest_category("Origami"), None);
        assert_eq!(
            suggest_category("Strategic THINKING"),
            Some(("leadership", "strategic thinking")),
            "case-insensitive"
        );
        for (category, _) in CATEGORY_RULES {
            assert!(SKILL_CATEGORIES.contains(category));
        }
    }

    #[test]
    fn skill_names_are_normalised() {
        assert_eq!(normalize_skill_name("  Rust "), Ok("Rust".to_string()));
        assert!(normalize_skill_name("   ").is_err());
        assert!(normalize_skill_name(&"x".repeat(201)).is_err());
    }

    #[test]
    fn proficiency_is_one_to_five() {
        assert!(valid_proficiency(1) && valid_proficiency(5));
        assert!(!valid_proficiency(0) && !valid_proficiency(6));
    }

    #[test]
    fn mentorship_lifecycle() {
        assert!(mentorship_transition("proposed", "active").is_ok());
        assert!(mentorship_transition("active", "completed").is_ok());
        assert!(mentorship_transition("proposed", "ended").is_ok());
        assert!(
            mentorship_transition("proposed", "completed").is_err(),
            "must activate first"
        );
        assert!(
            mentorship_transition("completed", "active").is_err(),
            "terminal"
        );
        assert!(
            mentorship_transition("active", "sideways").is_err(),
            "unknown"
        );
    }

    #[test]
    fn progress_counts_only_real_completions() {
        let steps = vec![
            "course:a".to_string(),
            "course:b".to_string(),
            "course:c".to_string(),
        ];
        let done = vec![
            "course:a".to_string(),
            "course:c".to_string(),
            "course:z".to_string(),
        ];
        assert_eq!(path_progress(&steps, &done), (2, 3));
        assert_eq!(path_progress(&steps, &[]), (0, 3));
    }
}
