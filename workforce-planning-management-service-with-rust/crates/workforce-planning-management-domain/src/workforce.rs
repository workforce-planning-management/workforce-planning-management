//! Time & attendance and scheduling rules (WPM-R4, WPM-R6), DB-free.
//!
//! Time is recorded in whole **minutes** (no float hours). Overtime is
//! **derived** against the contracted day, scaled by FTE (WPM-D3).

use jiff::Timestamp;

/// Minutes in a day — the hard cap on one day's recorded time.
pub const DAY_MINUTES: i32 = 24 * 60;

/// The example contracted full-time working day, in minutes (7.5 h): a
/// starting point, no regime's. The deployment's own day is passed to
/// [`contracted_day_minutes`] and [`overtime_minutes`] (WPM-T383).
pub const CONTRACTED_DAY_MINUTES: i32 = 450;

/// Check a contracted full-time day: 1 to [`DAY_MINUTES`] minutes.
///
/// # Errors
///
/// A refusal naming the range.
pub fn validate_contracted_day(minutes: i32) -> Result<(), String> {
    if (1..=DAY_MINUTES).contains(&minutes) {
        Ok(())
    } else {
        Err(format!(
            "a contracted day must be 1 to {DAY_MINUTES} minutes"
        ))
    }
}

/// Validate one time entry's minutes: positive, and a single day's
/// total (existing + new) may not exceed 24 h (WPM-R4).
///
/// # Errors
///
/// A human-readable refusal (`422` material).
pub fn check_day_minutes(existing_minutes: i32, new_minutes: i32) -> Result<(), String> {
    if new_minutes <= 0 {
        return Err("minutes must be positive".to_string());
    }
    if new_minutes > DAY_MINUTES {
        return Err(format!(
            "minutes {new_minutes} exceeds a day ({DAY_MINUTES})"
        ));
    }
    match existing_minutes.checked_add(new_minutes) {
        Some(total) if total <= DAY_MINUTES => Ok(()),
        Some(total) => Err(format!(
            "day total {total} minutes exceeds 24h ({existing_minutes} already recorded)"
        )),
        // A sum too large for the type is far past 24 h (WPM-R4): refused, with no invented total.
        None => Err(format!(
            "day total exceeds 24h ({existing_minutes} already recorded)"
        )),
    }
}

/// The contracted day for a worker at `fte_percent` (100 ⇒ full
/// time) of a full-time day of `full_day_minutes`, rounded down to
/// whole minutes. `None` when the FTE is outside 0 to 100 or the full day outside 0 to [`DAY_MINUTES`]: such a figure is not
/// trusted and not clamped (RCA-INT-6).
#[must_use]
pub fn contracted_day_minutes(fte_percent: i32, full_day_minutes: i32) -> Option<i32> {
    if !(0..=100).contains(&fte_percent) || !(0..=DAY_MINUTES).contains(&full_day_minutes) {
        return None;
    }
    full_day_minutes.checked_mul(fte_percent)?.checked_div(100)
}

/// Overtime is never negative (WPM-D3): the floor of a derived or explicit overtime figure.
const NO_OVERTIME: i32 = 0;

/// Derived overtime for one day: recorded `regular` minutes beyond the
/// FTE-scaled contracted day of `full_day_minutes`, plus all explicit
/// `overtime` minutes. `None` when the contracted day cannot be worked out ([`contracted_day_minutes`]) or the total does
/// not fit: unknown, not a number nobody recorded.
#[must_use]
pub fn overtime_minutes(
    regular_minutes: i32,
    explicit_overtime_minutes: i32,
    fte_percent: i32,
    full_day_minutes: i32,
) -> Option<i32> {
    let contracted = contracted_day_minutes(fte_percent, full_day_minutes)?;
    // A difference below the type's minimum is negative whatever it is, so no overtime (WPM-D3: overtime is never negative).
    let derived = regular_minutes
        .checked_sub(contracted)
        .map_or(NO_OVERTIME, |over| over.max(NO_OVERTIME));
    derived.checked_add(explicit_overtime_minutes.max(NO_OVERTIME))
}

/// Whether two half-open time windows `[start, end)` overlap — the
/// shift double-booking check (WPM-R6).
#[must_use]
pub fn windows_overlap(
    a_start: Timestamp,
    a_end: Timestamp,
    b_start: Timestamp,
    b_end: Timestamp,
) -> bool {
    a_start < b_end && b_start < a_end
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(h: u32) -> Timestamp {
        crate::dates::utc(2026, 8, 3, h, 0, 0).unwrap()
    }

    /// Day-minute caps: zero/negative refused, a single >24h entry
    /// refused, and the running day total enforced.
    #[test]
    fn day_minute_caps() {
        assert!(check_day_minutes(0, 480).is_ok());
        assert!(check_day_minutes(0, 0).is_err());
        assert!(check_day_minutes(0, -5).is_err());
        assert!(check_day_minutes(0, DAY_MINUTES + 1).is_err());
        assert!(check_day_minutes(1000, 441).is_err());
        assert!(check_day_minutes(1000, 440).is_ok());
        // A sum too large for the type is refused, naming no total.
        assert_eq!(
            check_day_minutes(i32::MAX, 1),
            Err(format!(
                "day total exceeds 24h ({} already recorded)",
                i32::MAX
            ))
        );
    }

    /// The contracted day scales by FTE and clamps silly percentages.
    #[test]
    fn contracted_day_scales_by_fte() {
        let day = CONTRACTED_DAY_MINUTES;
        assert_eq!(contracted_day_minutes(100, day), Some(450));
        assert_eq!(contracted_day_minutes(50, day), Some(225));
        assert_eq!(contracted_day_minutes(0, day), Some(0));
        // A percentage outside 0 to 100 is unknown, not clamped.
        assert_eq!(contracted_day_minutes(150, day), None);
        assert_eq!(contracted_day_minutes(-10, day), None);
        // Another deployment's day: 8 hours, and 9 hours 36 minutes.
        assert_eq!(contracted_day_minutes(100, 480), Some(480));
        assert_eq!(contracted_day_minutes(50, 576), Some(288));
        // A day longer than a day, or below zero, is unknown, never trusted.
        assert_eq!(contracted_day_minutes(100, DAY_MINUTES + 60), None);
        assert_eq!(contracted_day_minutes(100, -1), None);
    }

    /// The contracted day is 1 minute to 24 hours.
    #[test]
    fn a_contracted_day_is_checked() {
        assert_eq!(validate_contracted_day(CONTRACTED_DAY_MINUTES), Ok(()));
        assert_eq!(validate_contracted_day(1), Ok(()));
        assert_eq!(validate_contracted_day(DAY_MINUTES), Ok(()));
        assert!(validate_contracted_day(0).is_err());
        assert!(validate_contracted_day(DAY_MINUTES + 1).is_err());
    }

    /// Overtime = regular beyond the FTE-scaled day + explicit
    /// overtime; never negative.
    #[test]
    fn overtime_derivation() {
        let day = CONTRACTED_DAY_MINUTES;
        assert_eq!(overtime_minutes(450, 0, 100, day), Some(0));
        assert_eq!(overtime_minutes(510, 0, 100, day), Some(60));
        assert_eq!(overtime_minutes(300, 0, 50, day), Some(75)); // 225 contracted at 0.5 FTE
        assert_eq!(overtime_minutes(200, 30, 100, day), Some(30)); // under contract + explicit
        assert_eq!(overtime_minutes(500, -10, 100, day), Some(50)); // junk explicit ignored
        // An 8-hour day: 510 minutes is 30 over, not 60.
        assert_eq!(overtime_minutes(510, 0, 100, 480), Some(30));
        // An unknown contracted day, or a total that does not fit, is unknown.
        assert_eq!(overtime_minutes(510, 0, 150, day), None);
        assert_eq!(overtime_minutes(i32::MAX, i32::MAX, 100, day), None);
        // A difference below the type's minimum is negative, so no overtime.
        assert_eq!(overtime_minutes(i32::MIN, 0, 100, day), Some(0));
    }

    /// Half-open windows: back-to-back shifts do NOT overlap;
    /// containment and partial overlap do.
    #[test]
    fn window_overlap() {
        assert!(!windows_overlap(t(8), t(16), t(16), t(23)));
        assert!(windows_overlap(t(8), t(16), t(15), t(23)));
        assert!(windows_overlap(t(8), t(20), t(9), t(10)));
        assert!(!windows_overlap(t(8), t(9), t(10), t(11)));
    }
}
