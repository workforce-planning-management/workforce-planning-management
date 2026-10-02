//! Pure rules for **role profiles** (WPM-R34): what a role requires, so a
//! competency gap can be measured against a role rather than a person.
//! DB-free; the controller only wires these.

use crate::rules::learning::valid_proficiency;

/// How much a required skill matters to the role.
pub const IMPORTANCES: &[&str] = &["critical", "important", "useful"];

/// The longest accepted job title / free-text note.
pub const MAX_TITLE_LEN: usize = 100;

/// Normalise a job title: trimmed, non-empty, at most [`MAX_TITLE_LEN`]
/// characters.
///
/// # Errors
/// A message naming the problem.
pub fn normalize_job_title(input: &str) -> Result<String, String> {
    let title = input.trim();
    if title.is_empty() {
        return Err("job_title is required".to_string());
    }
    if title.chars().count() > MAX_TITLE_LEN {
        return Err(format!(
            "job_title is longer than {MAX_TITLE_LEN} characters"
        ));
    }
    Ok(title.to_string())
}

/// Validate one requirement: minimum proficiency on the 1–5 scale and a
/// known importance.
///
/// # Errors
/// A message naming the offending field.
pub fn validate_requirement(min_proficiency: i32, importance: &str) -> Result<(), String> {
    if !valid_proficiency(min_proficiency) {
        return Err("min_proficiency must be between 1 and 5".to_string());
    }
    if !IMPORTANCES.contains(&importance) {
        return Err(format!(
            "importance must be one of {}",
            IMPORTANCES.join(", ")
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Titles are trimmed; blank and over-long are refused.
    #[test]
    fn job_titles_are_normalised() {
        assert_eq!(normalize_job_title("  Nurse "), Ok("Nurse".to_string()));
        assert!(normalize_job_title("   ").is_err());
        assert!(normalize_job_title(&"x".repeat(100)).is_ok());
        assert!(normalize_job_title(&"x".repeat(101)).is_err());
    }

    /// The proficiency scale and the importance vocabulary are enforced.
    #[test]
    fn requirements_are_validated() {
        assert!(validate_requirement(1, "critical").is_ok());
        assert!(validate_requirement(5, "useful").is_ok());
        assert!(
            validate_requirement(0, "critical")
                .unwrap_err()
                .contains("min_proficiency")
        );
        assert!(validate_requirement(6, "critical").is_err());
        assert!(
            validate_requirement(3, "essential")
                .unwrap_err()
                .contains("importance")
        );
    }
}
