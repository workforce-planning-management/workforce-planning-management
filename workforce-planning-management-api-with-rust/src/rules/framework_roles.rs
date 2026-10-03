//! Pure rules for a person **selecting their current role and skills** in an
//! external framework (UK GDAD PCF or ESCO): which frameworks are
//! selectable, and validating a batch of skill choices.
//!
//! DB-free. A choice carries a reference (a skill id for the PCF, a concept
//! URI for ESCO) and a proficiency on WPM's 1–5 scale — or `None` to
//! deselect. The person sets their own level; neither framework's wording
//! decides it for them.

use crate::rules::learning::valid_proficiency;

/// The selectable frameworks.
pub const FRAMEWORKS: &[&str] = &["uk-gdad-pcf", "esco"];

/// The most skill choices in one request.
pub const MAX_CHOICES: usize = 500;

/// Whether `slug` names a selectable framework.
#[must_use]
pub fn valid_framework(slug: &str) -> bool {
    FRAMEWORKS.contains(&slug)
}

/// One skill choice: a framework reference and a level (`None` deselects).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// The framework reference (PCF skill pid / ESCO concept URI).
    pub reference: String,
    /// WPM proficiency 1–5, or `None` to remove the declaration.
    pub proficiency: Option<i32>,
}

/// Validate a batch: bounded, no blank or repeated references, and every
/// level on the 1–5 scale.
///
/// # Errors
/// A message naming the problem.
pub fn validate_choices(choices: &[Choice]) -> Result<(), String> {
    if choices.len() > MAX_CHOICES {
        return Err(format!("at most {MAX_CHOICES} skills per request"));
    }
    let mut seen = std::collections::BTreeSet::new();
    for choice in choices {
        let reference = choice.reference.trim();
        if reference.is_empty() {
            return Err("every choice needs a ref".to_string());
        }
        if !seen.insert(reference) {
            return Err(format!("`{reference}` appears more than once"));
        }
        if let Some(level) = choice.proficiency
            && !valid_proficiency(level)
        {
            return Err("proficiency must be between 1 and 5".to_string());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choice(reference: &str, proficiency: Option<i32>) -> Choice {
        Choice {
            reference: reference.to_string(),
            proficiency,
        }
    }

    #[test]
    fn frameworks() {
        assert!(valid_framework("uk-gdad-pcf") && valid_framework("esco"));
        assert!(!valid_framework("sfia") && !valid_framework(""));
    }

    #[test]
    fn choices_are_validated() {
        assert!(validate_choices(&[]).is_ok(), "an empty batch is fine");
        assert!(validate_choices(&[choice("a", Some(3)), choice("b", None)]).is_ok());
        assert!(
            validate_choices(&[choice(" ", Some(3))]).is_err(),
            "blank ref"
        );
        assert!(
            validate_choices(&[choice("a", Some(3)), choice("a", None)])
                .unwrap_err()
                .contains("more than once")
        );
        assert!(validate_choices(&[choice("a", Some(0))]).is_err());
        assert!(validate_choices(&[choice("a", Some(6))]).is_err());
        let many: Vec<Choice> = (0..=MAX_CHOICES)
            .map(|i| choice(&i.to_string(), Some(1)))
            .collect();
        assert!(validate_choices(&many).is_err(), "bounded");
    }
}
