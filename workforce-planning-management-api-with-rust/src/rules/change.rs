//! Pure rules for the **AI-driven change tracker** (WPM-D28): an
//! operational record of an automation or AI initiative and its impact on
//! *roles* and *skills* — which roles are displaced, reshaped, or created,
//! which skills are rising or declining, and how ready the affected
//! workforce is to move.
//!
//! The tracker is about positions and capabilities, never about people:
//! every figure it derives is an aggregate, and no individual is named or
//! ranked. DB-free; the controller only wires these.

use crate::rules::lifecycle;

/// What kind of change an initiative is.
pub const KINDS: &[&str] = &[
    "automation",
    "ai_assistance",
    "process_redesign",
    "restructure",
    "other",
];

/// Initiative lifecycle statuses.
pub const STATUSES: &[&str] = &["draft", "active", "completed", "cancelled"];

/// The legal initiative transitions; `completed` and `cancelled` are
/// terminal.
pub const MACHINE: &[(&str, &str)] = &[
    ("draft", "active"),
    ("draft", "cancelled"),
    ("active", "completed"),
    ("active", "cancelled"),
];

/// How a role is affected.
pub const IMPACTS: &[&str] = &["displaced", "reshaped", "created"];

/// When the impact is expected.
pub const TIMEFRAMES: &[&str] = &["now", "within_1y", "one_to_three_years", "beyond"];

/// Which way a skill's demand moves.
pub const DIRECTIONS: &[&str] = &["rising", "declining"];

/// The longest accepted name / note.
pub const MAX_TEXT_LEN: usize = 200;

/// Check an initiative status transition (the family `422` message on
/// refusal).
///
/// # Errors
/// A message naming the current state.
pub fn check_transition(from: &str, to: &str) -> Result<(), String> {
    lifecycle::check("change initiative", MACHINE, from, to)
}

/// Validate a role impact's tokens.
///
/// # Errors
/// A message naming the offending field.
pub fn validate_role_impact(impact: &str, timeframe: &str) -> Result<(), String> {
    if !IMPACTS.contains(&impact) {
        return Err(format!("impact must be one of {}", IMPACTS.join(", ")));
    }
    if !TIMEFRAMES.contains(&timeframe) {
        return Err(format!(
            "timeframe must be one of {}",
            TIMEFRAMES.join(", ")
        ));
    }
    Ok(())
}

/// Whether an impact means the existing role-holders must be supported to
/// move — i.e. the role is displaced or reshaped, not created.
#[must_use]
pub fn affects_current_holders(impact: &str) -> bool {
    matches!(impact, "displaced" | "reshaped")
}

/// Counts of affected workers by readiness for a rising skill.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct SkillReadiness {
    /// Declared at or above the bar.
    pub meeting: usize,
    /// Declared, but under the bar.
    pub below: usize,
    /// Not declared (unknown, not below).
    pub undeclared: usize,
}

/// Tally declared proficiencies against a bar; `None` is undeclared.
#[must_use]
pub fn skill_readiness(declared: &[Option<i32>], bar: i32) -> SkillReadiness {
    let mut out = SkillReadiness::default();
    for level in declared {
        match crate::rules::gap::grade(*level, bar) {
            "met" => out.meeting += 1,
            "below" => out.below += 1,
            _ => out.undeclared += 1,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only forward transitions exist; the terminals have no exit.
    #[test]
    fn lifecycle_machine() {
        assert!(check_transition("draft", "active").is_ok());
        assert!(check_transition("active", "completed").is_ok());
        assert!(check_transition("active", "cancelled").is_ok());
        assert!(
            check_transition("draft", "completed").is_err(),
            "must be activated first"
        );
        assert!(check_transition("completed", "active").is_err());
        assert!(
            check_transition("cancelled", "draft")
                .unwrap_err()
                .contains("cancelled")
        );
    }

    /// Every transition uses tokens from the status vocabulary.
    #[test]
    fn machine_uses_vocabulary_tokens() {
        for (from, to) in MACHINE {
            assert!(STATUSES.contains(from) && STATUSES.contains(to));
        }
    }

    /// Role impacts need known tokens; only displaced/reshaped affect holders.
    #[test]
    fn role_impacts() {
        assert!(validate_role_impact("reshaped", "within_1y").is_ok());
        assert!(
            validate_role_impact("erased", "now")
                .unwrap_err()
                .contains("impact")
        );
        assert!(
            validate_role_impact("created", "soon")
                .unwrap_err()
                .contains("timeframe")
        );
        assert!(affects_current_holders("displaced") && affects_current_holders("reshaped"));
        assert!(!affects_current_holders("created"));
    }

    /// Undeclared is kept apart from below.
    #[test]
    fn readiness_tally() {
        let r = skill_readiness(&[Some(4), Some(3), Some(2), None, None], 3);
        assert_eq!(
            r,
            SkillReadiness {
                meeting: 2,
                below: 1,
                undeclared: 2
            }
        );
        assert_eq!(skill_readiness(&[], 3), SkillReadiness::default());
    }
}
