//! Pure rules for **workforce capability analysis** — the strategic
//! read of declared skills: "do we have enough depth in the skills we
//! need?", as distinct from the per-worker and per-department gap
//! reports.
//!
//! DB-free and clock-free. Like every workforce-intelligence rule, it
//! counts **declarations** (a worker's self-declared 1–5 proficiency),
//! never inferred ability, and it never turns a missing declaration
//! into a zero measurement.

use crate::rules::learning::valid_proficiency;

/// Default proficiency at which a declaration counts as *working*
/// capability (3 of 5).
pub const DEFAULT_MIN_PROFICIENCY: i32 = 3;

/// Default number of workers at that proficiency a skill needs before
/// the organisation is not dependent on a single person.
pub const DEFAULT_MIN_DEPTH: usize = 2;

/// The depth statuses, from worst to best.
pub const DEPTH_STATUSES: &[&str] = &["undeclared", "no_proficient", "thin", "adequate"];

/// Why a threshold was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum ThresholdError {
    /// `min_proficiency` is outside the 1–5 scale.
    Proficiency,
    /// `min_depth` is zero — a skill cannot need nobody.
    Depth,
}

/// Validate the analysis thresholds.
///
/// # Errors
/// [`ThresholdError`] naming the offending threshold.
pub fn validate_thresholds(min_proficiency: i32, min_depth: usize) -> Result<(), ThresholdError> {
    if !valid_proficiency(min_proficiency) {
        return Err(ThresholdError::Proficiency);
    }
    if min_depth == 0 {
        return Err(ThresholdError::Depth);
    }
    Ok(())
}

/// Classify a skill's depth.
///
/// - `undeclared` — nobody has declared it (we do not know, which is
///   not the same as "nobody has it").
/// - `no_proficient` — declared, but nobody at the proficiency bar.
/// - `thin` — some proficient workers, fewer than `min_depth`.
/// - `adequate` — at least `min_depth` proficient workers.
#[must_use]
pub fn depth_status(declared: usize, proficient: usize, min_depth: usize) -> &'static str {
    if declared == 0 {
        "undeclared"
    } else if proficient == 0 {
        "no_proficient"
    } else if proficient < min_depth {
        "thin"
    } else {
        "adequate"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each band is reachable, and "nobody declared" is kept apart from
    /// "declared but nobody proficient".
    #[test]
    fn depth_status_bands() {
        assert_eq!(depth_status(0, 0, 2), "undeclared");
        assert_eq!(depth_status(4, 0, 2), "no_proficient");
        assert_eq!(depth_status(4, 1, 2), "thin");
        assert_eq!(depth_status(4, 2, 2), "adequate");
        assert_eq!(depth_status(4, 9, 2), "adequate");
    }

    /// A depth bar of one makes any proficient holder adequate.
    #[test]
    fn depth_one_accepts_a_single_holder() {
        assert_eq!(depth_status(1, 1, 1), "adequate");
    }

    /// Thresholds outside the scale are refused, in-scale accepted.
    #[test]
    fn thresholds_are_validated() {
        assert_eq!(validate_thresholds(0, 2), Err(ThresholdError::Proficiency));
        assert_eq!(validate_thresholds(6, 2), Err(ThresholdError::Proficiency));
        assert_eq!(validate_thresholds(3, 0), Err(ThresholdError::Depth));
        assert!(validate_thresholds(1, 1).is_ok());
        assert!(validate_thresholds(5, 10).is_ok());
    }

    /// The default thresholds are themselves valid.
    #[test]
    fn defaults_are_valid() {
        assert!(validate_thresholds(DEFAULT_MIN_PROFICIENCY, DEFAULT_MIN_DEPTH).is_ok());
    }
}
