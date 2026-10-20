//! Pure rules for **LMS completion sync** (WPM-T54): validating one
//! completion event pushed by a learning-management system, deciding what
//! it does to a training enrollment, and converting any CPD credit it
//! carries. DB-free and clock-free.
//!
//! WPM is not the LMS (WPM-D10): the LMS owns courses and learners, and
//! reports completions here. Every event carries an `external_ref` so a
//! redelivery is recognised and changes nothing.

use chrono::NaiveDate;
use std::fmt::Write;

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

/// The longest course-link template accepted.
pub const MAX_TEMPLATE_LEN: usize = 500;

/// Where the course id goes in a course-link template.
pub const PLACEHOLDER: &str = "{course}";

/// Check a deployer's course-link template, such as `https://lms.example.org/course/{course}`.
///
/// It must be an `https` link with the placeholder exactly once, no spaces and no user-info
/// (`https://user@host/` is how a link hides where it goes). The template is the deployer's: a caller
/// of the API never supplies one, so a link cannot be made to point anywhere the deployer did not choose.
///
/// # Errors
///
/// What is wrong with the template.
pub fn parse_template(raw: &str) -> Result<String, String> {
    let template = raw.trim();
    if template.chars().count() > MAX_TEMPLATE_LEN {
        return Err(format!("longer than {MAX_TEMPLATE_LEN} characters"));
    }
    let Some(rest) = template.strip_prefix("https://") else {
        return Err("must start with https://".to_string());
    };
    if template
        .chars()
        .any(|c| c.is_whitespace() || c.is_control())
    {
        return Err("must not contain spaces or control characters".to_string());
    }
    if template.matches(PLACEHOLDER).count() != 1 {
        return Err(format!("must contain {PLACEHOLDER} exactly once"));
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.is_empty() || authority.contains('@') || authority.contains(PLACEHOLDER) {
        return Err("needs a host, with no user-info and no placeholder in it".to_string());
    }
    Ok(template.to_string())
}

/// The link to a course in the LMS: the template with the course's id in place of the placeholder.
/// The id is the part of the `course:` or `courseinstance:` URN after the first colon, percent-encoded
/// so it cannot add a path, a query or a second host. `None` when there is no id.
#[must_use]
pub fn course_url(template: &str, course_ref: &str) -> Option<String> {
    let id = course_ref
        .split_once(':')
        .map_or(course_ref, |(_, id)| id)
        .trim();
    if id.is_empty() {
        return None;
    }
    let mut encoded = String::with_capacity(id.len());
    for byte in id.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    Some(template.replacen(PLACEHOLDER, &encoded, 1))
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

    const TEMPLATE: &str = "https://lms.example.org/course/{course}";

    #[test]
    fn a_template_must_be_https_with_one_placeholder_and_a_plain_host() {
        assert_eq!(
            parse_template(&format!("  {TEMPLATE} ")),
            Ok(TEMPLATE.to_string())
        );
        for bad in [
            "http://lms.example.org/{course}",
            "https://lms.example.org/course",
            "https://lms.example.org/{course}/{course}",
            "https://user@lms.example.org/{course}",
            "https://{course}.example.org/",
            "https:///{course}",
            "https://lms.example.org/a b/{course}",
            "javascript:alert(1)//{course}",
            "",
        ] {
            assert!(parse_template(bad).is_err(), "{bad}");
        }
        let long = format!("https://a.org/{}{{course}}", "x".repeat(MAX_TEMPLATE_LEN));
        assert!(parse_template(&long).is_err());
    }

    #[test]
    fn the_course_id_is_the_urn_after_its_first_colon() {
        let id = "3f2a1c9e-1111-4222-8333-444455556666";
        assert_eq!(
            course_url(TEMPLATE, &format!("course:{id}")),
            Some(format!("https://lms.example.org/course/{id}"))
        );
        assert_eq!(
            course_url(TEMPLATE, &format!("courseinstance:{id}")),
            Some(format!("https://lms.example.org/course/{id}"))
        );
        assert_eq!(course_url(TEMPLATE, "course:"), None);
        assert_eq!(course_url(TEMPLATE, "  "), None);
    }

    #[test]
    fn a_course_id_cannot_change_where_the_link_goes() {
        let url = course_url(TEMPLATE, "course:../../admin?x=1#y&z=https://evil.example/").unwrap();
        assert!(url.starts_with("https://lms.example.org/course/"));
        let tail = &url["https://lms.example.org/course/".len()..];
        assert!(!tail.contains(['/', '?', '#', '&', ':', ' ']), "{tail}");
        assert_eq!(
            course_url(TEMPLATE, "course:a b/é"),
            Some("https://lms.example.org/course/a%20b%2F%C3%A9".to_string())
        );
    }
}
