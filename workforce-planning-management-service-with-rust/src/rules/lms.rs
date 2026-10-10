//! Pure rules for **LMS completion sync** (WPM-T54): validating one
//! completion event pushed by a learning-management system, deciding what
//! it does to a training enrollment, and converting any CPD credit it
//! carries. DB-free and clock-free.
//!
//! WPM is not the LMS (WPM-D10): the LMS owns courses and learners, and
//! reports completions here. Every event carries an `external_ref` so a
//! redelivery is recognised and changes nothing.

use chrono::NaiveDate;

use crate::rules::cpd::to_hundredths;

/// The most completions accepted in one batch.
pub const MAX_BATCH: usize = 500;

/// The longest accepted external reference.
pub const MAX_EXTERNAL_REF_LEN: usize = 200;

/// Validate one completion event.
///
/// # Errors
/// A message naming the offending field.
pub fn validate_completion(
    external_ref: &str,
    completed_on: NaiveDate,
    certificate_expires_on: Option<NaiveDate>,
    today: NaiveDate,
) -> Result<(), String> {
    let reference = external_ref.trim();
    if reference.is_empty() {
        return Err("external_ref is required".to_string());
    }
    if reference.chars().count() > MAX_EXTERNAL_REF_LEN {
        return Err(format!(
            "external_ref is longer than {MAX_EXTERNAL_REF_LEN} characters"
        ));
    }
    if completed_on > today {
        return Err("completed_on cannot be in the future".to_string());
    }
    if certificate_expires_on.is_some_and(|expiry| expiry < completed_on) {
        return Err("certificate_expires_on is before completed_on".to_string());
    }
    Ok(())
}

/// The CPD credit a completion carries: `(unit, hundredths)`, or `None`
/// when it carries none. At most one of hours / points may be given.
///
/// # Errors
/// A message when both are given or an amount is not a positive number.
pub fn cpd_credit(
    hours: Option<f64>,
    points: Option<f64>,
) -> Result<Option<(&'static str, i64)>, String> {
    match (hours, points) {
        (None, None) => Ok(None),
        (Some(h), None) => to_hundredths(h)
            .map(|v| Some(("hours", v)))
            .ok_or_else(|| "hours must be a positive amount".to_string()),
        (None, Some(p)) => to_hundredths(p)
            .map(|v| Some(("points", v)))
            .ok_or_else(|| "points must be a positive amount".to_string()),
        (Some(_), Some(_)) => Err("give hours or points, not both".to_string()),
    }
}

/// What a completion does to a worker's enrollment in a course.
#[derive(Debug, PartialEq, Eq)]
pub enum EnrollmentAction {
    /// No enrollment exists: create one, already completed.
    Create,
    /// An enrollment exists but is not (or differently) completed: update it.
    Complete,
    /// Already completed with the same dates: a redelivery, nothing to do.
    Unchanged,
}

/// An existing enrollment, as far as the decision needs.
pub struct Existing<'a> {
    /// Its status.
    pub status: &'a str,
    /// When it was completed.
    pub completed_on: Option<NaiveDate>,
    /// Certificate expiry.
    pub certificate_expires_on: Option<NaiveDate>,
}

/// Decide what a completion does to the matching enrollment, if any.
#[must_use]
pub fn enrollment_action(
    existing: Option<&Existing<'_>>,
    completed_on: NaiveDate,
    certificate_expires_on: Option<NaiveDate>,
) -> EnrollmentAction {
    match existing {
        None => EnrollmentAction::Create,
        Some(e)
            if e.status == "completed"
                && e.completed_on == Some(completed_on)
                && e.certificate_expires_on == certificate_expires_on =>
        {
            EnrollmentAction::Unchanged
        }
        Some(_) => EnrollmentAction::Complete,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    /// An event needs a reference, a non-future completion, and a sane expiry.
    #[test]
    fn completions_are_validated() {
        let today = day(2026, 10, 2);
        assert!(validate_completion("evt-1", today, None, today).is_ok());
        assert!(validate_completion("  ", today, None, today).is_err());
        assert!(validate_completion(&"x".repeat(201), today, None, today).is_err());
        assert!(validate_completion("e", day(2026, 10, 3), None, today).is_err());
        assert!(validate_completion("e", today, Some(day(2026, 1, 1)), today).is_err());
        assert!(validate_completion("e", today, Some(day(2028, 1, 1)), today).is_ok());
    }

    /// Credit is hours or points, never both, and positive.
    #[test]
    fn credit_is_hours_or_points() {
        assert_eq!(cpd_credit(None, None), Ok(None));
        assert_eq!(cpd_credit(Some(1.5), None), Ok(Some(("hours", 150))));
        assert_eq!(cpd_credit(None, Some(20.0)), Ok(Some(("points", 2000))));
        assert!(cpd_credit(Some(1.0), Some(1.0)).is_err());
        assert!(cpd_credit(Some(0.0), None).is_err());
        assert!(cpd_credit(None, Some(-3.0)).is_err());
    }

    /// A redelivery changes nothing; new dates update; no enrollment creates.
    #[test]
    fn enrollment_decisions() {
        let (done, exp) = (day(2026, 9, 1), Some(day(2027, 9, 1)));
        assert_eq!(enrollment_action(None, done, exp), EnrollmentAction::Create);
        let same = Existing {
            status: "completed",
            completed_on: Some(done),
            certificate_expires_on: exp,
        };
        assert_eq!(
            enrollment_action(Some(&same), done, exp),
            EnrollmentAction::Unchanged
        );
        let enrolled = Existing {
            status: "enrolled",
            completed_on: None,
            certificate_expires_on: None,
        };
        assert_eq!(
            enrollment_action(Some(&enrolled), done, exp),
            EnrollmentAction::Complete
        );
        let renewed = Existing {
            status: "completed",
            completed_on: Some(day(2025, 9, 1)),
            certificate_expires_on: Some(day(2026, 9, 1)),
        };
        assert_eq!(
            enrollment_action(Some(&renewed), done, exp),
            EnrollmentAction::Complete,
            "a renewal updates"
        );
        let failed = Existing {
            status: "failed",
            completed_on: None,
            certificate_expires_on: None,
        };
        assert_eq!(
            enrollment_action(Some(&failed), done, None),
            EnrollmentAction::Complete
        );
    }
}
