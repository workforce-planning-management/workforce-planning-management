//! The personal development review (WPM-R172, WPM-D73 spirit). DB-free and clock-free: the as-of date is an argument.
//!
//! A review has a kind, `performance` or `padr`. A PADR is **complete when both the reviewer and the reviewee have signed**; once
//! either has signed its content is frozen and a correction is a new version. It is **due** twelve months after the last completed
//! PADR, or a configured number of months after the start of the primary assignment for someone who has never had one; with
//! neither, the due date is unknown, never invented. Whether a wellbeing conversation was *offered* is a yes or no; nothing here
//! can hold what was said.

use chrono::{Months, NaiveDate};

/// The kinds of review.
pub const KINDS: &[&str] = &["performance", "padr"];

/// Months between one completed PADR and the next being due.
pub const CYCLE_MONTHS: u32 = 12;

/// The two people who sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signer {
    /// The person doing the review.
    Reviewer,
    /// The person being reviewed.
    Reviewee,
}

/// The sign-offs on one version of a PADR.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SignOffs {
    /// The day the reviewer signed.
    pub reviewer: Option<NaiveDate>,
    /// The day the reviewee signed.
    pub reviewee: Option<NaiveDate>,
}

impl SignOffs {
    /// Both have signed.
    #[must_use]
    pub fn complete(&self) -> bool {
        self.reviewer.is_some() && self.reviewee.is_some()
    }

    /// The day it became complete: the later of the two. `None` until both have signed.
    #[must_use]
    pub fn completed_on(&self) -> Option<NaiveDate> {
        self.reviewer.zip(self.reviewee).map(|(a, b)| a.max(b))
    }

    /// Content is frozen once either has signed. A reviewee's comment is not content and is not governed by this.
    #[must_use]
    pub fn frozen(&self) -> bool {
        self.reviewer.is_some() || self.reviewee.is_some()
    }
}

/// Record a signature. A signer signs once, and not on a day to come.
///
/// # Errors
/// A sentence saying why the signature cannot be recorded.
pub fn sign(
    current: SignOffs,
    who: Signer,
    on: NaiveDate,
    today: NaiveDate,
) -> Result<SignOffs, String> {
    if on > today {
        return Err("a sign-off cannot be dated in the future".to_string());
    }
    let mut next = current;
    let slot = match who {
        Signer::Reviewer => &mut next.reviewer,
        Signer::Reviewee => &mut next.reviewee,
    };
    if slot.is_some() {
        return Err("this version is already signed by that person".to_string());
    }
    *slot = Some(on);
    Ok(next)
}

/// Whether the content of a version may still be edited: not once either has signed.
///
/// # Errors
/// A sentence saying a correction is a new version.
pub fn check_editable(s: SignOffs) -> Result<(), String> {
    if s.frozen() {
        Err("a signed review is frozen; record a correction as a new version".to_string())
    } else {
        Ok(())
    }
}

/// Problems with the kind.
#[must_use]
pub fn kind_problems(kind: &str) -> Vec<String> {
    if KINDS.contains(&kind) {
        Vec::new()
    } else {
        vec![format!("kind must be one of: {}", KINDS.join(", "))]
    }
}

/// Where a person stands against the cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// There is no completed PADR and no primary start with a configured lead time, so no due date exists.
    Unknown,
    /// Due on or after the as-of date, on this day.
    Current(NaiveDate),
    /// The due date passed, on this day.
    Overdue(NaiveDate),
}

/// The due date. `last_completed` is the day the latest PADR was completed; `primary_start` the start of the primary assignment;
/// `first_after_months` the configured lead time for someone who has never had one (no default shipped).
#[must_use]
pub fn due_on(
    last_completed: Option<NaiveDate>,
    primary_start: Option<NaiveDate>,
    first_after_months: Option<u32>,
) -> Option<NaiveDate> {
    match last_completed {
        Some(d) => d.checked_add_months(Months::new(CYCLE_MONTHS)),
        None => primary_start
            .zip(first_after_months)
            .and_then(|(s, m)| s.checked_add_months(Months::new(m))),
    }
}

/// Where the person stands on `as_of`. Due on the as-of day is still current.
#[must_use]
pub fn standing(
    last_completed: Option<NaiveDate>,
    primary_start: Option<NaiveDate>,
    first_after_months: Option<u32>,
    as_of: NaiveDate,
) -> Standing {
    match due_on(last_completed, primary_start, first_after_months) {
        None => Standing::Unknown,
        Some(d) if as_of > d => Standing::Overdue(d),
        Some(d) => Standing::Current(d),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    const NOW: fn() -> NaiveDate = || d(2026, 10, 28);

    #[test]
    fn complete_means_both_signed_and_the_later_day_is_the_completion() {
        let none = SignOffs::default();
        assert!(!none.complete());
        let one = sign(none, Signer::Reviewer, d(2026, 10, 1), NOW()).unwrap();
        assert!(!one.complete());
        assert_eq!(one.completed_on(), None);
        let both = sign(one, Signer::Reviewee, d(2026, 10, 5), NOW()).unwrap();
        assert!(both.complete());
        assert_eq!(both.completed_on(), Some(d(2026, 10, 5)));
        let other_order = sign(
            sign(none, Signer::Reviewee, d(2026, 10, 9), NOW()).unwrap(),
            Signer::Reviewer,
            d(2026, 10, 3),
            NOW(),
        )
        .unwrap();
        assert_eq!(other_order.completed_on(), Some(d(2026, 10, 9)));
    }

    #[test]
    fn a_signer_signs_once_and_not_in_the_future() {
        let one = sign(SignOffs::default(), Signer::Reviewer, d(2026, 10, 1), NOW()).unwrap();
        assert!(sign(one, Signer::Reviewer, d(2026, 10, 2), NOW()).is_err());
        assert!(
            sign(
                SignOffs::default(),
                Signer::Reviewee,
                d(2026, 10, 29),
                NOW()
            )
            .is_err()
        );
        assert!(sign(SignOffs::default(), Signer::Reviewee, NOW(), NOW()).is_ok());
    }

    #[test]
    fn the_first_signature_freezes_the_content() {
        assert!(check_editable(SignOffs::default()).is_ok());
        let one = sign(SignOffs::default(), Signer::Reviewee, d(2026, 10, 1), NOW()).unwrap();
        assert!(check_editable(one).unwrap_err().contains("new version"));
    }

    #[test]
    fn a_review_has_a_known_kind() {
        assert_eq!(kind_problems("padr"), Vec::<String>::new());
        assert_eq!(kind_problems("performance"), Vec::<String>::new());
        assert_eq!(kind_problems("probation").len(), 1);
    }

    #[test]
    fn due_twelve_months_after_the_last_completed_one() {
        assert_eq!(
            due_on(Some(d(2025, 10, 28)), None, None),
            Some(d(2026, 10, 28))
        );
        assert_eq!(
            due_on(Some(d(2024, 2, 29)), None, None),
            Some(d(2025, 2, 28))
        );
        assert_eq!(
            due_on(Some(d(2025, 10, 28)), Some(d(2020, 1, 1)), Some(3)),
            Some(d(2026, 10, 28)),
            "the lead time is only for someone who has never had one"
        );
    }

    #[test]
    fn never_reviewed_is_due_a_configured_time_after_the_primary_start() {
        assert_eq!(
            due_on(None, Some(d(2026, 1, 15)), Some(6)),
            Some(d(2026, 7, 15))
        );
    }

    #[test]
    fn with_nothing_to_go_on_the_due_date_is_unknown() {
        assert_eq!(
            due_on(None, Some(d(2026, 1, 15)), None),
            None,
            "no lead time is shipped"
        );
        assert_eq!(due_on(None, None, Some(6)), None);
        assert_eq!(standing(None, None, None, NOW()), Standing::Unknown);
    }

    #[test]
    fn due_today_is_current_and_the_day_after_is_overdue() {
        let last = Some(d(2025, 10, 28));
        assert_eq!(
            standing(last, None, None, NOW()),
            Standing::Current(d(2026, 10, 28))
        );
        let last = Some(d(2025, 10, 27));
        assert_eq!(
            standing(last, None, None, NOW()),
            Standing::Overdue(d(2026, 10, 27))
        );
        let last = Some(d(2026, 1, 1));
        assert_eq!(
            standing(last, None, None, NOW()),
            Standing::Current(d(2027, 1, 1))
        );
    }
}
