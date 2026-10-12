//! **Oversight criteria and the sustain rule** (WPM-R66, WPM-D48, WPM-D49): whether a measure has met
//! its threshold for N consecutive periods, and whether a criterion's measures all have.
//!
//! Pure and clock-free. Periods are whole numbers the caller numbers consecutively (for example
//! months), so "consecutive" is a plain `+1`. Thresholds and readings are whole numbers in whatever
//! unit the measure states (basis points, hundredths, a count). Nothing here decides a level: the
//! rules report evidence, and a person records any change of level (WPM-D49). A reading that is not
//! there, or is `null`, is never read as meeting the threshold and never as breaching it: it breaks
//! the run and the answer says so.

use std::collections::BTreeMap;

/// Which side of the threshold is good.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// The reading meets the threshold when it is at or below it (a rate that should stay low).
    AtMost,
    /// The reading meets the threshold when it is at or above it (a coverage that should stay high).
    AtLeast,
}

impl Direction {
    /// Parse the stored token.
    #[must_use]
    pub fn parse(token: &str) -> Option<Self> {
        match token {
            "at_most" => Some(Self::AtMost),
            "at_least" => Some(Self::AtLeast),
            _ => None,
        }
    }
}

/// The stored direction tokens.
pub const DIRECTIONS: &[&str] = &["at_most", "at_least"];
/// The origins a measure may be created with today. Derived measures arrive with their
/// computation (WPM-T114); until then a derived measure would have no readings and could never be met.
pub const AVAILABLE_ORIGINS: &[&str] = &["reported"];
/// The most periods a sustain rule may ask for.
pub const MAX_SUSTAIN: u32 = 120;

/// Where a reading came from. Shown, never merged silently (WPM-D49).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Computed from WPM data.
    Derived,
    /// Entered by a person with an evidence note and a named source document.
    Reported,
}

/// One period's reading of a measure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reading {
    /// The period, numbered consecutively by the caller.
    pub period: i64,
    /// The value, or `None` when the data is missing (`null`).
    pub value: Option<i64>,
    /// Where it came from.
    pub origin: Origin,
}

/// A measure's threshold and sustain rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spec {
    /// The threshold.
    pub threshold: i64,
    /// Which side is good.
    pub direction: Direction,
    /// Consecutive periods that must meet it.
    pub sustain: u32,
}

/// The period number of a month: months counted from year 0, so consecutive months are consecutive
/// numbers.
#[must_use]
pub fn month_period(year: i32, month: u32) -> i64 {
    i64::from(year) * 12 + i64::from(month) - 1
}

/// Check a spec before it is stored.
///
/// # Errors
///
/// A sentence saying what is wrong.
pub fn validate_spec(direction: &str, sustain: i64) -> Result<(), String> {
    if Direction::parse(direction).is_none() {
        return Err(format!("direction must be one of {DIRECTIONS:?}"));
    }
    if !(1..=i64::from(MAX_SUSTAIN)).contains(&sustain) {
        return Err(format!("sustain must be 1 to {MAX_SUSTAIN} periods"));
    }
    Ok(())
}

/// A reported reading needs an evidence note and a named source document (WPM-R66).
///
/// # Errors
///
/// A sentence saying which is missing.
pub fn validate_reported(evidence: &str, source: &str) -> Result<(), String> {
    if evidence.trim().is_empty() {
        return Err("a reported reading needs an evidence note".into());
    }
    if source.trim().is_empty() {
        return Err("a reported reading needs the source document".into());
    }
    Ok(())
}

/// Does `value` meet the threshold? The threshold itself meets it.
#[must_use]
pub fn meets(value: i64, spec: &Spec) -> bool {
    match spec.direction {
        Direction::AtMost => value <= spec.threshold,
        Direction::AtLeast => value >= spec.threshold,
    }
}

/// The answer for one measure or one criterion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The sustain rule is met.
    Met,
    /// A reading in the window breaches the threshold.
    NotMet,
    /// It cannot be said, because a reading is missing or `null` (see [`Reason`]).
    Unknown,
}

impl Status {
    /// The token the API returns.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Met => "met",
            Self::NotMet => "not_met",
            Self::Unknown => "unknown",
        }
    }
}

/// What stopped the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// The run reached the sustain length.
    Sustained,
    /// This period breached the threshold.
    Breach(i64),
    /// This period has no reading.
    NoReading(i64),
    /// This period's reading is `null`.
    NullReading(i64),
    /// A criterion with no linked measure cannot be met.
    NoMeasures,
}

/// The verdict on one measure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasureVerdict {
    /// Met, not met or unknown.
    pub status: Status,
    /// Consecutive periods meeting the threshold, ending at the latest period.
    pub run: u32,
    /// The periods required.
    pub required: u32,
    /// What stopped the run.
    pub reason: Reason,
    /// The periods counted in the run, oldest first.
    pub periods: Vec<i64>,
    /// How many of those periods rest on a reported reading rather than a derived one.
    pub reported_periods: u32,
}

/// Judge one measure as of `latest`, looking back one period at a time.
#[must_use]
pub fn evaluate_measure(spec: &Spec, readings: &[Reading], latest: i64) -> MeasureVerdict {
    let by_period: BTreeMap<i64, &Reading> = readings.iter().map(|r| (r.period, r)).collect();
    let mut counted: Vec<i64> = Vec::new();
    let mut reported = 0u32;
    let mut period = latest;
    let reason = loop {
        if u32::try_from(counted.len()).unwrap_or(u32::MAX) >= spec.sustain {
            break Reason::Sustained;
        }
        match by_period.get(&period) {
            None => break Reason::NoReading(period),
            Some(Reading { value: None, .. }) => break Reason::NullReading(period),
            Some(Reading {
                value: Some(v),
                origin,
                ..
            }) => {
                if !meets(*v, spec) {
                    break Reason::Breach(period);
                }
                counted.push(period);
                if *origin == Origin::Reported {
                    reported += 1;
                }
            }
        }
        period -= 1;
    };
    counted.reverse();
    let status = match reason {
        Reason::Sustained => Status::Met,
        Reason::Breach(_) => Status::NotMet,
        _ => Status::Unknown,
    };
    MeasureVerdict {
        status,
        run: u32::try_from(counted.len()).unwrap_or(u32::MAX),
        required: spec.sustain,
        reason,
        periods: counted,
        reported_periods: reported,
    }
}

/// The verdict on a criterion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionVerdict {
    /// Met only when every linked measure is met.
    pub status: Status,
    /// Each measure's verdict, in the order given, so the answer says which measures and which
    /// periods.
    pub measures: Vec<MeasureVerdict>,
}

/// Judge a criterion: met when every linked measure is met; not met when any is definitely not;
/// otherwise unknown. A criterion with no measure is unknown, never vacuously met.
#[must_use]
pub fn evaluate_criterion(measures: &[(Spec, Vec<Reading>)], latest: i64) -> CriterionVerdict {
    let verdicts: Vec<MeasureVerdict> = measures
        .iter()
        .map(|(spec, readings)| evaluate_measure(spec, readings, latest))
        .collect();
    let status = if verdicts.is_empty() {
        Status::Unknown
    } else if verdicts.iter().any(|v| v.status == Status::NotMet) {
        Status::NotMet
    } else if verdicts.iter().any(|v| v.status == Status::Unknown) {
        Status::Unknown
    } else {
        Status::Met
    };
    CriterionVerdict {
        status,
        measures: verdicts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(threshold: i64, direction: Direction, sustain: u32) -> Spec {
        Spec {
            threshold,
            direction,
            sustain,
        }
    }

    fn derived(period: i64, value: Option<i64>) -> Reading {
        Reading {
            period,
            value,
            origin: Origin::Derived,
        }
    }

    fn reported(period: i64, value: i64) -> Reading {
        Reading {
            period,
            value: Some(value),
            origin: Origin::Reported,
        }
    }

    #[test]
    fn consecutive_months_are_consecutive_numbers_across_a_year_end() {
        assert_eq!(month_period(2027, 1), month_period(2026, 12) + 1);
        assert_eq!(month_period(2026, 2), month_period(2026, 1) + 1);
    }

    #[test]
    fn the_threshold_itself_meets_it_on_either_side() {
        let low = spec(10, Direction::AtMost, 1);
        assert!(meets(10, &low) && meets(9, &low) && !meets(11, &low));
        let high = spec(10, Direction::AtLeast, 1);
        assert!(meets(10, &high) && meets(11, &high) && !meets(9, &high));
    }

    #[test]
    fn three_periods_in_a_row_meet_a_sustain_of_three() {
        let s = spec(10, Direction::AtMost, 3);
        let r = [
            derived(1, Some(9)),
            derived(2, Some(10)),
            derived(3, Some(8)),
        ];
        let v = evaluate_measure(&s, &r, 3);
        assert_eq!((v.status, v.run, v.required), (Status::Met, 3, 3));
        assert_eq!(v.periods, [1, 2, 3]);
        assert_eq!(v.reason, Reason::Sustained);
    }

    #[test]
    fn a_run_shorter_than_the_sustain_is_not_met_when_a_breach_ended_it() {
        let s = spec(10, Direction::AtMost, 3);
        let r = [
            derived(1, Some(20)),
            derived(2, Some(9)),
            derived(3, Some(8)),
        ];
        let v = evaluate_measure(&s, &r, 3);
        assert_eq!((v.status, v.run), (Status::NotMet, 2));
        assert_eq!(v.reason, Reason::Breach(1));
    }

    #[test]
    fn only_the_latest_periods_count_so_an_old_breach_does_not_matter() {
        let s = spec(10, Direction::AtMost, 2);
        let r = [
            derived(1, Some(99)),
            derived(2, Some(9)),
            derived(3, Some(8)),
        ];
        assert_eq!(evaluate_measure(&s, &r, 3).status, Status::Met);
    }

    #[test]
    fn a_null_reading_breaks_the_run_and_says_so() {
        let s = spec(10, Direction::AtMost, 3);
        let r = [derived(1, Some(1)), derived(2, None), derived(3, Some(1))];
        let v = evaluate_measure(&s, &r, 3);
        assert_eq!((v.status, v.run), (Status::Unknown, 1));
        assert_eq!(v.reason, Reason::NullReading(2));
    }

    #[test]
    fn a_missing_period_breaks_the_run_and_is_not_a_zero() {
        let s = spec(10, Direction::AtMost, 3);
        let r = [derived(1, Some(1)), derived(3, Some(1))];
        let v = evaluate_measure(&s, &r, 3);
        assert_eq!(
            (v.status, v.reason),
            (Status::Unknown, Reason::NoReading(2))
        );
        // Nothing for the latest period at all.
        let none = evaluate_measure(&s, &r, 4);
        assert_eq!(
            (none.status, none.run, none.reason),
            (Status::Unknown, 0, Reason::NoReading(4))
        );
        // A missing reading is not read as meeting an at-least threshold either.
        assert_eq!(
            evaluate_measure(&spec(1, Direction::AtLeast, 1), &[], 1).status,
            Status::Unknown
        );
    }

    #[test]
    fn a_breach_after_a_gap_is_still_a_breach() {
        // Looking back: period 3 breaches before the gap at 2 is reached.
        let s = spec(10, Direction::AtMost, 3);
        let r = [derived(1, Some(1)), derived(3, Some(50))];
        assert_eq!(evaluate_measure(&s, &r, 3).status, Status::NotMet);
    }

    #[test]
    fn too_little_history_is_unknown_not_met() {
        let s = spec(10, Direction::AtMost, 6);
        let r = [derived(5, Some(1)), derived(6, Some(1))];
        let v = evaluate_measure(&s, &r, 6);
        assert_eq!(
            (v.status, v.run, v.reason),
            (Status::Unknown, 2, Reason::NoReading(4))
        );
    }

    #[test]
    fn reported_periods_are_counted_apart_from_derived_ones() {
        let s = spec(10, Direction::AtMost, 3);
        let r = [derived(1, Some(1)), reported(2, 2), reported(3, 3)];
        let v = evaluate_measure(&s, &r, 3);
        assert_eq!((v.status, v.reported_periods), (Status::Met, 2));
    }

    #[test]
    fn a_criterion_is_met_only_when_every_measure_is() {
        let a = spec(10, Direction::AtMost, 2);
        let b = spec(5, Direction::AtLeast, 2);
        let good_a: Vec<_> = vec![derived(1, Some(1)), derived(2, Some(1))];
        let good_b = vec![reported(1, 6), reported(2, 7)];
        let bad_b = vec![reported(1, 6), reported(2, 1)];
        let null_b = vec![reported(1, 6), derived(2, None)];
        let met = evaluate_criterion(&[(a, good_a.clone()), (b, good_b)], 2);
        assert_eq!(met.status, Status::Met);
        assert_eq!(met.measures.len(), 2, "it says which measures");
        assert_eq!(
            evaluate_criterion(&[(a, good_a.clone()), (b, bad_b.clone())], 2).status,
            Status::NotMet
        );
        assert_eq!(
            evaluate_criterion(&[(a, good_a.clone()), (b, null_b.clone())], 2).status,
            Status::Unknown
        );
        assert_eq!(
            evaluate_criterion(&[(a, good_a), (b, bad_b), (b, null_b)], 2).status,
            Status::NotMet,
            "a definite breach outweighs an unknown"
        );
    }

    #[test]
    fn a_criterion_with_no_measure_is_unknown_not_met() {
        let v = evaluate_criterion(&[], 1);
        assert_eq!(v.status, Status::Unknown);
    }

    #[test]
    fn a_spec_and_a_reported_reading_are_checked() {
        assert!(validate_spec("at_most", 1).is_ok());
        assert!(validate_spec("at_least", i64::from(MAX_SUSTAIN)).is_ok());
        assert!(validate_spec("below", 3).is_err());
        assert!(validate_spec("at_most", 0).is_err());
        assert!(validate_spec("at_most", i64::from(MAX_SUSTAIN) + 1).is_err());
        assert!(validate_reported("seen in the minutes", "Board minutes, March").is_ok());
        assert!(validate_reported("  ", "Board minutes").is_err());
        assert!(validate_reported("seen", "").is_err());
    }

    #[test]
    fn the_verdict_has_no_level_in_it() {
        // The rules answer met, not met or unknown. There is no value that proposes a level change.
        for s in [Status::Met, Status::NotMet, Status::Unknown] {
            assert!(!s.as_str().contains("level"));
        }
    }
}
