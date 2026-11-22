//! Pure rules for the **CPD ledger** (continuing professional
//! development): units, categories, entry and requirement validation,
//! progress against a requirement, and registration expiry.
//!
//! DB-free and clock-free (dates are always supplied). Amounts are whole
//! **hundredths** of a unit (hours or points) so no float ever rounds a
//! total, and a worker with no entries has `0` recorded of a known
//! requirement — an honest zero, since "required" is defined, unlike a
//! ratio with nothing to divide.

use jiff::civil::Date;

/// What a requirement and an entry are measured in.
pub const UNITS: &[&str] = &["hours", "points"];

/// What kind of activity an entry records.
pub const CATEGORIES: &[&str] = &[
    "course",
    "conference",
    "reading",
    "mentoring",
    "on_the_job",
    "self_study",
    "other",
];

/// Where an entry came from.
pub const SOURCES: &[&str] = &["manual", "lms"];

/// The most one entry may record, in whole units (a sanity bound).
pub const MAX_UNITS_PER_ENTRY: i64 = 1000;

/// Convert a human amount (hours or points, e.g. `1.5`) to hundredths;
/// `None` for non-finite, non-positive, or implausibly large values.
#[must_use]
pub fn to_hundredths(amount: f64) -> Option<i64> {
    if !amount.is_finite() || amount <= 0.0_f64 {
        return None;
    }
    let hundredths = crate::metrics::rounded_i64(amount * 100.0)?;
    (1..=MAX_UNITS_PER_ENTRY * 100)
        .contains(&hundredths)
        .then_some(hundredths)
}

/// Validate one entry: known unit and category, a positive amount, and
/// an activity date that is not in the future.
///
/// # Errors
/// A message naming the offending field.
pub fn validate_entry(
    unit: &str,
    category: &str,
    amount_hundredths: i64,
    entry_date: Date,
    today: Date,
) -> Result<(), String> {
    if !UNITS.contains(&unit) {
        return Err(format!("unit must be one of {}", UNITS.join(", ")));
    }
    if !CATEGORIES.contains(&category) {
        return Err(format!("category must be one of {}", CATEGORIES.join(", ")));
    }
    if amount_hundredths <= 0 {
        return Err("amount must be positive".to_string());
    }
    if entry_date > today {
        return Err("entry_date cannot be in the future".to_string());
    }
    Ok(())
}

/// Validate a requirement: known unit, a positive amount, and a period
/// that does not end before it starts.
///
/// # Errors
/// A message naming the offending field.
pub fn validate_requirement(
    unit: &str,
    required_hundredths: i64,
    period_start: Date,
    period_end: Date,
) -> Result<(), String> {
    if !UNITS.contains(&unit) {
        return Err(format!("unit must be one of {}", UNITS.join(", ")));
    }
    if required_hundredths <= 0 {
        return Err("required amount must be positive".to_string());
    }
    if period_end < period_start {
        return Err("period_end must not be before period_start".to_string());
    }
    Ok(())
}

/// Whether `date` falls in `[start, end]` (inclusive).
#[must_use]
pub fn in_period(date: Date, start: Date, end: Date) -> bool {
    date >= start && date <= end
}

/// Whether a requirement scoped to `requirement_job_title` applies to a
/// worker with `worker_job_title`: `None` applies to everyone; a title
/// matches case-insensitively.
#[must_use]
pub fn applies_to(requirement_job_title: Option<&str>, worker_job_title: &str) -> bool {
    requirement_job_title.is_none_or(|t| t.trim().eq_ignore_ascii_case(worker_job_title.trim()))
}

/// Progress against a requirement, in hundredths of a unit.
#[derive(Debug, PartialEq, Eq)]
pub struct Progress {
    /// Everything recorded in the period.
    pub recorded: i64,
    /// The part of it that has been verified.
    pub verified: i64,
    /// What the requirement asks for.
    pub required: i64,
}

impl Progress {
    /// Still to do on recorded entries (never negative); `None` when the difference does not fit an `i64`.
    #[must_use]
    pub fn remaining(&self) -> Option<i64> {
        Some(self.required.checked_sub(self.recorded)?.max(0))
    }

    /// Whether recorded entries meet the requirement.
    #[must_use]
    pub fn met(&self) -> bool {
        self.recorded >= self.required
    }

    /// Whether *verified* entries alone meet the requirement.
    #[must_use]
    pub fn met_verified(&self) -> bool {
        self.verified >= self.required
    }
}

/// Sum `(amount_hundredths, verified)` entries against a requirement; `None` when a total does not fit an `i64` (unknown, not a
/// wrapped or capped total).
#[must_use]
pub fn progress(entries: &[(i64, bool)], required: i64) -> Option<Progress> {
    Some(Progress {
        recorded: entries
            .iter()
            .try_fold(0_i64, |total, (amount, _)| total.checked_add(*amount))?,
        verified: entries
            .iter()
            .filter(|(_, verified)| *verified)
            .try_fold(0_i64, |total, (amount, _)| total.checked_add(*amount))?,
        required,
    })
}

/// The default warning window for an expiring registration (days).
pub const DEFAULT_WARN_DAYS: i64 = 90;

/// Status of a registration on `today`: `no_expiry`, `valid`, `expiring`
/// (within `warn_days`), or `expired`.
#[must_use]
pub fn registration_status(expires_on: Option<Date>, today: Date, warn_days: i64) -> &'static str {
    match expires_on {
        None => "no_expiry",
        Some(date) if date < today => "expired",
        Some(date) if crate::dates::days_between(today, date) <= warn_days => "expiring",
        Some(_) => "valid",
    }
}

/// Where a worker stands against a requirement's period (WPM-D92).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaceStatus {
    /// Recorded at least the required amount.
    Met,
    /// The period has not begun.
    NotStarted,
    /// At or ahead of the straight line.
    OnTrack,
    /// Less than the straight line gives by now.
    Behind,
    /// The period is over and the requirement was not met.
    Missed,
}

/// How the recorded amount compares with the straight line through the period.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Pace {
    /// The comparison.
    pub status: PaceStatus,
    /// Calendar days left in the period, counting today; `0` once it is over.
    pub days_left: i64,
    /// The remaining amount (hundredths) per 30 days left, rounded up; absent when met or over.
    pub needed_per_30_days_hundredths: Option<i64>,
    /// What the straight line gives by today (hundredths), when the period is under way.
    pub expected_by_today_hundredths: Option<i64>,
}

/// What [`Pace`] rests on, to show beside it.
pub const PACE_BASIS: &str = "a straight line from the first to the last day of the period, both counted: \
behind means less recorded than that line gives by today. Real CPD comes in lumps, so behind is a prompt \
to look, not a finding. Verification is not used.";

/// Compare `recorded` with a requirement of `required` over `start..=end`, as of `today`. `None` when a figure does not fit an
/// `i64`: the pace is then unknown, not a capped number.
#[must_use]
pub fn pace(recorded: i64, required: i64, start: Date, end: Date, today: Date) -> Option<Pace> {
    let days_left = if today > end {
        0
    } else {
        crate::dates::days_between(today.max(start), end).checked_add(1)?
    };
    let none = |status| Pace {
        status,
        days_left,
        needed_per_30_days_hundredths: None,
        expected_by_today_hundredths: None,
    };
    if recorded >= required {
        return Some(none(PaceStatus::Met));
    }
    if today > end {
        return Some(none(PaceStatus::Missed));
    }
    if today < start {
        return Some(none(PaceStatus::NotStarted));
    }
    let total_days = crate::dates::days_between(start, end).checked_add(1)?;
    let elapsed = crate::dates::days_between(start, today).checked_add(1)?;
    // required × elapsed ÷ total, rounded down: a worker exactly on the line is on track
    let expected = required.checked_mul(elapsed)?.checked_div(total_days)?;
    let remaining = required.checked_sub(recorded)?;
    let needed = remaining
        .checked_mul(30)?
        .checked_add(days_left)?
        .checked_sub(1)?
        .checked_div(days_left)?;
    Some(Pace {
        status: if recorded >= expected {
            PaceStatus::OnTrack
        } else {
            PaceStatus::Behind
        },
        days_left,
        needed_per_30_days_hundredths: Some(needed),
        expected_by_today_hundredths: Some(expected),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(y: i32, m: u32, d: u32) -> Date {
        crate::dates::ymd(y, m, d).expect("valid date")
    }

    /// Amounts convert to hundredths; non-positive, non-finite, and
    /// implausibly large amounts are refused.
    #[test]
    fn amounts_become_hundredths() {
        assert_eq!(to_hundredths(1.5), Some(150));
        assert_eq!(to_hundredths(0.25), Some(25));
        assert_eq!(to_hundredths(2.0), Some(200));
        assert_eq!(to_hundredths(0.0), None);
        assert_eq!(to_hundredths(-1.0), None);
        assert_eq!(to_hundredths(f64::NAN), None);
        assert_eq!(to_hundredths(0.001), None, "rounds to zero");
        assert_eq!(to_hundredths(1000.0), Some(100_000));
        assert_eq!(to_hundredths(1000.01), None);
    }

    /// Entries need known tokens, a positive amount, and a non-future date.
    #[test]
    fn entries_are_validated() {
        let today = day(2026, 10, 2);
        assert!(validate_entry("hours", "course", 150, today, today).is_ok());
        assert!(
            validate_entry("days", "course", 150, today, today)
                .unwrap_err()
                .contains("unit")
        );
        assert!(
            validate_entry("hours", "party", 150, today, today)
                .unwrap_err()
                .contains("category")
        );
        assert!(validate_entry("hours", "course", 0, today, today).is_err());
        assert!(validate_entry("hours", "course", 150, day(2026, 10, 3), today).is_err());
    }

    /// Requirements need a known unit, a positive amount, and an ordered period.
    #[test]
    fn requirements_are_validated() {
        let (a, b) = (day(2026, 1, 1), day(2026, 12, 31));
        assert!(validate_requirement("points", 3000, a, b).is_ok());
        assert!(
            validate_requirement("points", 3000, a, a).is_ok(),
            "a one-day period is fine"
        );
        assert!(validate_requirement("points", 0, a, b).is_err());
        assert!(validate_requirement("days", 3000, a, b).is_err());
        assert!(validate_requirement("points", 3000, b, a).is_err());
    }

    /// Period membership is inclusive; job-title scoping is case-insensitive.
    #[test]
    fn periods_and_scoping() {
        let (a, b) = (day(2026, 1, 1), day(2026, 12, 31));
        assert!(in_period(a, a, b) && in_period(b, a, b));
        assert!(!in_period(day(2025, 12, 31), a, b));
        assert!(applies_to(None, "Nurse"));
        assert!(applies_to(Some(" nurse "), "Nurse"));
        assert!(!applies_to(Some("Doctor"), "Nurse"));
    }

    /// Progress sums recorded and verified separately; a worker with no
    /// entries has a real 0 of a known requirement.
    #[test]
    fn progress_sums() {
        let p = progress(&[(300, true), (200, false), (100, true)], 1000).expect("fits");
        assert_eq!((p.recorded, p.verified, p.required), (600, 400, 1000));
        assert_eq!(p.remaining(), Some(400));
        assert!(!p.met() && !p.met_verified());
        let done = progress(&[(1000, false)], 1000).expect("fits");
        assert!(
            done.met() && !done.met_verified(),
            "recorded meets it; verified does not yet"
        );
        assert_eq!(done.remaining(), Some(0));
        assert!(
            progress(&[(1000, true)], 1000)
                .expect("fits")
                .met_verified()
        );
        let none = progress(&[], 500).expect("fits");
        assert_eq!((none.recorded, none.remaining()), (0, Some(500)));
        assert_eq!(
            progress(&[(900, true)], 500).expect("fits").remaining(),
            Some(0),
            "never negative"
        );
        assert!(progress(&[(i64::MAX, false), (1, false)], 1).is_none());
        assert!(progress(&[(i64::MAX, true), (1, true)], 1).is_none());
        assert!(progress(&[(i64::MAX, false)], 1).is_some());
        assert_eq!(to_hundredths(1.5), Some(150));
        assert_eq!(to_hundredths(1e300), None);
    }

    /// Registration status bands, inclusive at the warning edge.
    #[test]
    fn registration_bands() {
        let today = day(2026, 10, 2);
        assert_eq!(registration_status(None, today, 90), "no_expiry");
        assert_eq!(
            registration_status(Some(day(2026, 10, 1)), today, 90),
            "expired"
        );
        assert_eq!(
            registration_status(Some(today), today, 90),
            "expiring",
            "expires today: still valid but flagged"
        );
        assert_eq!(
            registration_status(Some(day(2026, 12, 31)), today, 90),
            "expiring",
            "exactly 90 days is inside"
        );
        assert_eq!(
            registration_status(Some(day(2027, 1, 1)), today, 90),
            "valid",
            "91 days is outside"
        );
        assert_eq!(
            registration_status(Some(day(2027, 3, 1)), today, 90),
            "valid"
        );
    }

    fn d(m: u32, day: u32) -> Date {
        crate::dates::ymd(2026, m, day).unwrap()
    }

    fn pace(recorded: i64, required: i64, start: Date, end: Date, today: Date) -> Pace {
        super::pace(recorded, required, start, end, today).unwrap()
    }

    #[test]
    fn a_pace_or_remainder_that_does_not_fit_is_unknown() {
        let (start, end, today) = (d(1, 1), d(1, 30), d(1, 15));
        assert_eq!(super::pace(0, i64::MAX, start, end, today), None);
        assert_eq!(super::pace(i64::MIN, 1, start, end, today), None);
        let one_day = d(1, 1);
        assert_eq!(
            super::pace(0, i64::MAX / 20, one_day, one_day, one_day),
            None,
            "the amount needed per 30 days overflows"
        );
        let p = Progress {
            recorded: i64::MIN,
            verified: 0,
            required: 1,
        };
        assert_eq!(p.remaining(), None);
    }

    #[test]
    fn pace_is_met_before_anything_else() {
        // met wins even before the period starts or after it ends
        for today in [d(12, 31), d(1, 15), d(3, 1)] {
            let p = pace(3_000, 3_000, d(1, 1), d(2, 28), today);
            assert_eq!(p.status, PaceStatus::Met);
            assert_eq!(p.needed_per_30_days_hundredths, None);
        }
    }

    #[test]
    fn pace_before_during_and_after_the_period() {
        let (start, end) = (d(1, 1), d(1, 30)); // 30 days
        assert_eq!(
            pace(0, 3_000, d(3, 1), d(3, 30), d(2, 1)).status,
            PaceStatus::NotStarted
        );
        assert_eq!(
            pace(0, 3_000, start, end, d(2, 1)).status,
            PaceStatus::Missed
        );
        let during = pace(0, 3_000, start, end, d(1, 10));
        assert_eq!(during.status, PaceStatus::Behind);
        assert_eq!(during.days_left, 21); // the 10th to the 30th, both counted
    }

    #[test]
    fn exactly_on_the_line_is_on_track_and_one_hundredth_short_is_behind() {
        let (start, end) = (d(1, 1), d(1, 30));
        // day 15 of 30: the line gives 3,000 × 15 ÷ 30 = 1,500
        let on = pace(1_500, 3_000, start, end, d(1, 15));
        assert_eq!(on.status, PaceStatus::OnTrack);
        assert_eq!(on.expected_by_today_hundredths, Some(1_500));
        assert_eq!(
            pace(1_499, 3_000, start, end, d(1, 15)).status,
            PaceStatus::Behind
        );
    }

    #[test]
    fn the_needed_pace_rounds_up_and_uses_the_days_left() {
        // 1,000 remaining over 16 days left (15th to 30th): 1,000 × 30 ÷ 16 = 1,875
        let p = pace(2_000, 3_000, d(1, 1), d(1, 30), d(1, 15));
        assert_eq!(p.days_left, 16);
        assert_eq!(p.needed_per_30_days_hundredths, Some(1_875));
        // 100 remaining over 7 days left: 3,000 ÷ 7 = 428.57…, rounded up
        let q = pace(2_900, 3_000, d(1, 1), d(1, 30), d(1, 24));
        assert_eq!(q.needed_per_30_days_hundredths, Some(429));
    }

    #[test]
    fn a_one_day_period_is_handled() {
        let day = d(5, 5);
        assert_eq!(pace(0, 100, day, day, day).status, PaceStatus::Behind);
        assert_eq!(pace(100, 100, day, day, day).status, PaceStatus::Met);
    }
}
