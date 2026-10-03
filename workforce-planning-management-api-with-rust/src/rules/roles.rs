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

/// What changes between a role level and the next one up.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Progression {
    /// Skills the next level requires that this one does not: `(skill, min)`.
    pub added: Vec<(String, i32)>,
    /// Skills required at both levels, higher at the next: `(skill, from, to)`.
    pub raised: Vec<(String, i32, i32)>,
    /// Skills required at both levels at the same or a lower minimum.
    pub unchanged: usize,
    /// Skills this level requires that the next does not list.
    pub dropped: Vec<String>,
}

/// Compare the `(skill, minimum)` requirements of a level with those of the
/// next level up. Lists are sorted by skill name so the output is stable.
#[must_use]
pub fn progression_diff(current: &[(String, i32)], next: &[(String, i32)]) -> Progression {
    use std::collections::BTreeMap;
    let now: BTreeMap<&str, i32> = current.iter().map(|(s, m)| (s.as_str(), *m)).collect();
    let then: BTreeMap<&str, i32> = next.iter().map(|(s, m)| (s.as_str(), *m)).collect();
    let mut out = Progression::default();
    for (skill, to) in &then {
        match now.get(skill) {
            None => out.added.push(((*skill).to_string(), *to)),
            Some(from) if to > from => out.raised.push(((*skill).to_string(), *from, *to)),
            Some(_) => out.unchanged += 1,
        }
    }
    out.dropped = now
        .keys()
        .filter(|skill| !then.contains_key(*skill))
        .map(|s| (*s).to_string())
        .collect();
    out
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

    /// Progression names what is new, what rises, and what is unchanged.
    #[test]
    fn progression_between_levels() {
        let level = |pairs: &[(&str, i32)]| -> Vec<(String, i32)> {
            pairs.iter().map(|(s, m)| ((*s).to_string(), *m)).collect()
        };
        let now = level(&[("Testing", 2), ("Design", 3), ("Support", 3), ("Legacy", 1)]);
        let next = level(&[
            ("Testing", 4),
            ("Design", 3),
            ("Support", 2),
            ("Coaching", 3),
        ]);
        let diff = progression_diff(&now, &next);
        assert_eq!(diff.added, [("Coaching".to_string(), 3)]);
        assert_eq!(diff.raised, [("Testing".to_string(), 2, 4)]);
        assert_eq!(
            diff.unchanged, 2,
            "Design is equal and Support is lower: neither is a step up"
        );
        assert_eq!(diff.dropped, ["Legacy".to_string()]);
        assert_eq!(progression_diff(&[], &[]), Progression::default());
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
