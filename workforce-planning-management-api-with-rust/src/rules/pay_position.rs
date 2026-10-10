//! A worker's **place on a pay scale** (WPM-R54), DB-free: which band and step
//! they are on, since when, what that pays, and when they become eligible for
//! the next step.
//!
//! Eligibility is **only** the circular's years on the step (see
//! [`super::pay_scale`]): the date is `step_since` plus those whole years. Whether
//! the employer's own conditions for progression are met is not modelled, so the
//! wording is always "eligible", never "will move". All dates are arguments: the
//! module never reads a clock.

use chrono::{Months, NaiveDate};
use serde::Serialize;

use super::grade;
use super::pay_scale::{self, PayScale};

/// A position resolved against the reference scale.
///
/// # Errors
/// A message when the scale or band is unknown, or the step is not on the band.
pub fn validate(
    scale: &str,
    band: &str,
    step: usize,
) -> Result<(&'static str, &'static str, usize), String> {
    let (scale_id, band_code) = grade::resolve_band(scale, band)?;
    let found =
        pay_scale::find(scale_id).ok_or_else(|| format!("unknown pay scale {scale_id:?}"))?;
    let steps = found.band(band_code).map_or(0, |b| b.steps.len());
    if step == 0 || step > steps {
        return Err(format!(
            "band {band_code} has {steps} step(s); step is 1-based"
        ));
    }
    Ok((scale_id, band_code, step))
}

/// A step is held *since* a date, so that date cannot be after `today`.
///
/// # Errors
/// A message when `step_since` is in the future.
pub fn validate_step_since(step_since: NaiveDate, today: NaiveDate) -> Result<(), String> {
    if step_since > today {
        Err("step_since cannot be in the future".to_string())
    } else {
        Ok(())
    }
}

/// The date a person on `step` of `band` becomes eligible for the next step:
/// `step_since` plus the step's whole years. `None` on the top step or a single
/// rate (nowhere to go), when `step` is not on the band, or on a date overflow.
#[must_use]
pub fn eligible_on(
    band: &pay_scale::Band,
    step: usize,
    step_since: NaiveDate,
) -> Option<NaiveDate> {
    let current = band.steps.get(step.checked_sub(1)?)?;
    band.steps.get(step)?;
    let years = current.years_to_next?;
    step_since.checked_add_months(Months::new(u32::from(years) * 12))
}

/// Where a person stands on progression, with dates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Standing {
    /// On the top step (or a single rate): nowhere to go.
    AtTop,
    /// Eligible now (since `eligible_on`).
    Due {
        /// The date they became eligible.
        eligible_on: NaiveDate,
        /// Annual pay on the next step, in pence.
        next_annual_minor: i64,
    },
    /// Not yet.
    NotYet {
        /// The date they become eligible.
        eligible_on: NaiveDate,
        /// Whole days from today until then.
        days_remaining: i64,
        /// Annual pay on the next step, in pence.
        next_annual_minor: i64,
    },
}

/// Progression standing on `today` for someone on `step` since `step_since`.
/// `None` when the step is not on the band.
#[must_use]
pub fn standing(
    band: &pay_scale::Band,
    step: usize,
    step_since: NaiveDate,
    today: NaiveDate,
) -> Option<Standing> {
    band.steps.get(step.checked_sub(1)?)?;
    let (Some(on), Some(next)) = (eligible_on(band, step, step_since), band.steps.get(step)) else {
        return Some(Standing::AtTop);
    };
    Some(if today >= on {
        Standing::Due {
            eligible_on: on,
            next_annual_minor: next.annual_minor,
        }
    } else {
        Standing::NotYet {
            eligible_on: on,
            days_remaining: (on - today).num_days(),
            next_annual_minor: next.annual_minor,
        }
    })
}

/// The annual pay of a step, in pence.
#[must_use]
pub fn annual_minor(scale: &PayScale, band: &str, step: usize) -> Option<i64> {
    scale
        .band(band)?
        .steps
        .get(step.checked_sub(1)?)
        .map(|s| s.annual_minor)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn national() -> PayScale {
        pay_scale::national_2026_27()
    }

    /// A position must exist on the scale: unknown scale, band, or step is refused
    /// with the reason; the canonical codes come back.
    #[test]
    fn positions_are_validated_against_the_scale() {
        assert_eq!(
            validate("national-2026-27", "6", 2),
            Ok(("national-2026-27", "6", 2))
        );
        assert_eq!(
            validate("national-2026-27", "8A", 1),
            Ok(("national-2026-27", "8a", 1))
        );
        assert!(validate("national-2025-26", "6", 1).is_err());
        assert!(validate("national-2026-27", "14", 1).is_err());
        assert!(
            validate("national-2026-27", "6", 0)
                .unwrap_err()
                .contains("1-based")
        );
        assert!(
            validate("national-2026-27", "6", 4)
                .unwrap_err()
                .contains("3 step")
        );
        // Band 2 is a single rate: only step 1.
        assert!(validate("national-2026-27", "2", 1).is_ok());
        assert!(validate("national-2026-27", "2", 2).is_err());
    }

    #[test]
    fn a_step_is_not_held_since_the_future() {
        assert!(validate_step_since(d(2026, 10, 6), d(2026, 10, 6)).is_ok());
        assert!(validate_step_since(d(2026, 10, 7), d(2026, 10, 6)).is_err());
    }

    /// Eligibility is the step's whole years after `step_since`; the top step,
    /// a single rate and a step off the band have none.
    #[test]
    fn eligibility_dates() {
        let scale = national();
        let b6 = scale.band("6").unwrap();
        // Entry: 2 years. Intermediate: 3 years.
        assert_eq!(eligible_on(b6, 1, d(2024, 4, 1)), Some(d(2026, 4, 1)));
        assert_eq!(eligible_on(b6, 2, d(2024, 4, 1)), Some(d(2027, 4, 1)));
        assert_eq!(eligible_on(b6, 3, d(2024, 4, 1)), None, "top step");
        assert_eq!(eligible_on(b6, 0, d(2024, 4, 1)), None);
        assert_eq!(eligible_on(b6, 4, d(2024, 4, 1)), None);
        assert_eq!(
            eligible_on(scale.band("2").unwrap(), 1, d(2024, 4, 1)),
            None
        );
        // A leap day clamps to the end of February rather than overflowing.
        assert_eq!(eligible_on(b6, 1, d(2024, 2, 29)), Some(d(2026, 2, 28)));
    }

    /// Not yet, then due *on* the date, then long after; at the top there is no
    /// progression; a step off the band is `None`.
    #[test]
    fn standing_on_a_day() {
        let scale = national();
        let b6 = scale.band("6").unwrap();
        let since = d(2024, 4, 1);
        assert_eq!(
            standing(b6, 1, since, d(2026, 3, 31)),
            Some(Standing::NotYet {
                eligible_on: d(2026, 4, 1),
                days_remaining: 1,
                next_annual_minor: 4_280_500
            })
        );
        assert_eq!(
            standing(b6, 1, since, d(2026, 4, 1)),
            Some(Standing::Due {
                eligible_on: d(2026, 4, 1),
                next_annual_minor: 4_280_500
            })
        );
        assert!(matches!(
            standing(b6, 1, since, d(2030, 1, 1)),
            Some(Standing::Due { .. })
        ));
        assert_eq!(standing(b6, 3, since, d(2030, 1, 1)), Some(Standing::AtTop));
        assert_eq!(standing(b6, 9, since, d(2030, 1, 1)), None);
    }

    #[test]
    fn a_steps_pay() {
        let scale = national();
        assert_eq!(annual_minor(&scale, "5", 2), Some(3_511_400));
        assert_eq!(annual_minor(&scale, "5", 4), None);
        assert_eq!(annual_minor(&scale, "99", 1), None);
    }
}
