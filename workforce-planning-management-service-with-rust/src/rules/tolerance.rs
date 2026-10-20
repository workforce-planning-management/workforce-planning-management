//! Tolerances on a workforce plan (WPM-R140, WPM-D93): the largest deviation allowed in time, cost,
//! resources and risk, and the red, amber or green status that follows.
//!
//! Pure: the deviations are worked out by the caller from the plan's own forecast, cost and alignment
//! views and handed in as numbers. A deviation that cannot be worked out is `None`, and `None` is never
//! green.

use chrono::NaiveDate;
use serde::Serialize;

/// What is measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    /// Calendar days an unmet demand line is past its target date.
    Time,
    /// Projected cost above budget, in basis points of the budget.
    Cost,
    /// The largest headcount gap, in basis points of that line's planned demand.
    Resources,
    /// Critical roles in the plan's departments with no successor ready.
    Risk,
}

impl Dimension {
    /// Every dimension, in the order they are shown.
    pub const ALL: [Self; 4] = [Self::Time, Self::Cost, Self::Resources, Self::Risk];

    /// The token used in URLs and payloads.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Time => "time",
            Self::Cost => "cost",
            Self::Resources => "resources",
            Self::Risk => "risk",
        }
    }

    /// Parse a token.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|d| d.as_str() == text)
    }

    /// The unit of the deviation and of the limit.
    #[must_use]
    pub const fn unit(self) -> &'static str {
        match self {
            Self::Time => "calendar_days",
            Self::Cost | Self::Resources => "basis_points",
            Self::Risk => "roles",
        }
    }
}

/// Where a deviation stands against its tolerance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Rag {
    /// No tolerance has been set. Ordered lowest: it says nothing.
    NotSet,
    /// Inside the tolerance with room to spare.
    Green,
    /// The deviation cannot be worked out. Above green: it has not shown it is within tolerance.
    Unknown,
    /// Using `amber_percent` or more of the tolerance, up to and including the limit.
    Amber,
    /// Over the limit.
    Red,
}

/// The default share of a tolerance at which green turns amber.
pub const DEFAULT_AMBER_PERCENT: i32 = 75;

/// The largest limit accepted, to keep arithmetic well inside `i64`.
pub const MAX_LIMIT: i64 = 1_000_000;

/// Check a tolerance before it is stored.
///
/// # Errors
///
/// What is wrong with the limit or the amber share.
pub fn validate(limit: i64, amber_percent: i32) -> Result<(), String> {
    if !(0..=MAX_LIMIT).contains(&limit) {
        return Err(format!(
            "limit must be a whole number from 0 to {MAX_LIMIT}"
        ));
    }
    if !(1..=99).contains(&amber_percent) {
        return Err("amber_percent must be from 1 to 99".to_string());
    }
    Ok(())
}

/// The status of `deviation` against `limit`. A limit of `0` makes any deviation red.
#[must_use]
pub fn assess(deviation: Option<i64>, limit: i64, amber_percent: i32) -> Rag {
    let Some(deviation) = deviation else {
        return Rag::Unknown;
    };
    if deviation <= 0 {
        return Rag::Green;
    }
    if deviation > limit {
        return Rag::Red;
    }
    // deviation / limit >= amber_percent / 100, without dividing
    if i128::from(deviation) * 100 >= i128::from(limit) * i128::from(amber_percent) {
        Rag::Amber
    } else {
        Rag::Green
    }
}

/// The plan's status from its dimensions' statuses: red if any is, else amber, else unknown, else green
/// if at least one tolerance is set, else not set.
#[must_use]
pub fn overall(statuses: &[Rag]) -> Rag {
    statuses.iter().copied().max().unwrap_or(Rag::NotSet)
}

/// One demand line as the forecast gives it.
#[derive(Debug, Clone, Copy)]
pub struct Line {
    /// The date by which the headcount is wanted.
    pub target_on: NaiveDate,
    /// What is planned.
    pub planned: i64,
    /// Planned minus projected supply; `None` when no attrition assumption exists. Positive is short.
    pub gap: Option<i64>,
}

/// Time: the most calendar days an unmet line is past its date. `Some(0)` when every known line is
/// on time or met; `None` when there are no lines or no line's gap is known.
#[must_use]
pub fn time_deviation(lines: &[Line], today: NaiveDate) -> Option<i64> {
    let known: Vec<&Line> = lines.iter().filter(|l| l.gap.is_some()).collect();
    if known.is_empty() {
        return None;
    }
    Some(
        known
            .iter()
            .filter(|l| l.gap.is_some_and(|g| g > 0))
            .map(|l| (today - l.target_on).num_days())
            .max()
            .unwrap_or(0)
            .max(0),
    )
}

/// Resources: the largest shortfall as basis points of its line's planned demand, rounded down.
/// `None` when no line has both a known gap and a demand above zero.
#[must_use]
pub fn resources_deviation(lines: &[Line]) -> Option<i64> {
    let shares: Vec<i64> = lines
        .iter()
        .filter(|l| l.planned > 0)
        .filter_map(|l| l.gap.map(|g| g.max(0) * 10_000 / l.planned))
        .collect();
    shares.into_iter().max()
}

/// Cost: how far the projected cost is above the budget, in basis points of the budget, rounded down; `0` when
/// within it. A total that leaves groups out (`complete` false) can show an overspend, which only gets bigger
/// with the rest, but never "within budget": that is `None`. Also `None` when the total or the budget is
/// missing or the budget is zero (a share of nothing is not a number).
#[must_use]
pub fn cost_deviation(total: Option<i64>, budget: Option<i64>, complete: bool) -> Option<i64> {
    let (total, budget) = (total?, budget?);
    if budget <= 0 || (total <= budget && !complete) {
        return None;
    }
    let over = i128::from((total - budget).max(0)) * 10_000 / i128::from(budget);
    Some(i64::try_from(over).unwrap_or(i64::MAX))
}

/// Risk: the critical roles with no ready successor, when there is at least one succession plan to
/// have looked at in the plan's departments; otherwise `None` (no plans is not the same as no risk).
#[must_use]
pub fn risk_deviation(plans_in_departments: usize, without_bench: usize) -> Option<i64> {
    (plans_in_departments > 0).then(|| i64::try_from(without_bench).unwrap_or(i64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, m, d).unwrap()
    }

    #[test]
    fn a_dimension_round_trips_through_its_token() {
        for d in Dimension::ALL {
            assert_eq!(Dimension::parse(d.as_str()), Some(d));
        }
        assert_eq!(Dimension::parse("quality"), None);
    }

    #[test]
    fn status_edges_at_the_amber_and_red_points() {
        // limit 100, amber from 75
        assert_eq!(assess(Some(0), 100, 75), Rag::Green);
        assert_eq!(assess(Some(-5), 100, 75), Rag::Green);
        assert_eq!(assess(Some(74), 100, 75), Rag::Green);
        assert_eq!(assess(Some(75), 100, 75), Rag::Amber);
        assert_eq!(assess(Some(100), 100, 75), Rag::Amber);
        assert_eq!(assess(Some(101), 100, 75), Rag::Red);
    }

    #[test]
    fn a_zero_limit_makes_any_deviation_red_and_none_green() {
        assert_eq!(assess(Some(1), 0, 75), Rag::Red);
        assert_eq!(assess(Some(0), 0, 75), Rag::Green);
    }

    #[test]
    fn an_unknown_deviation_is_unknown_never_green() {
        assert_eq!(assess(None, 100, 75), Rag::Unknown);
        assert!(Rag::Unknown > Rag::Green);
    }

    #[test]
    fn amber_is_a_share_of_the_limit_so_small_limits_work() {
        // limit 3 roles, amber from 75%: 3 × 0.75 = 2.25, so 2 is green and 3 is amber
        assert_eq!(assess(Some(2), 3, 75), Rag::Green);
        assert_eq!(assess(Some(3), 3, 75), Rag::Amber);
        assert_eq!(assess(Some(4), 3, 75), Rag::Red);
        // a low amber point
        assert_eq!(assess(Some(1), 10, 10), Rag::Amber);
    }

    #[test]
    fn the_overall_status_is_the_worst_and_unknown_is_not_hidden() {
        assert_eq!(overall(&[]), Rag::NotSet);
        assert_eq!(overall(&[Rag::NotSet, Rag::NotSet]), Rag::NotSet);
        assert_eq!(overall(&[Rag::Green, Rag::NotSet]), Rag::Green);
        assert_eq!(overall(&[Rag::Green, Rag::Unknown]), Rag::Unknown);
        assert_eq!(overall(&[Rag::Unknown, Rag::Amber]), Rag::Amber);
        assert_eq!(overall(&[Rag::Amber, Rag::Red, Rag::Green]), Rag::Red);
    }

    #[test]
    fn validation_bounds() {
        assert!(validate(0, 75).is_ok());
        assert!(validate(MAX_LIMIT, 1).is_ok());
        assert!(validate(-1, 75).is_err());
        assert!(validate(MAX_LIMIT + 1, 75).is_err());
        assert!(validate(10, 0).is_err());
        assert!(validate(10, 100).is_err());
    }

    fn line(target: NaiveDate, planned: i64, gap: Option<i64>) -> Line {
        Line {
            target_on: target,
            planned,
            gap,
        }
    }

    #[test]
    fn time_is_the_longest_overdue_unmet_line() {
        let today = day(10, 20);
        let lines = [
            line(day(10, 10), 5, Some(2)), // 10 days past, short
            line(day(9, 1), 5, Some(0)),   // long past but met
            line(day(10, 19), 5, Some(1)), // 1 day past, short
            line(day(11, 30), 5, Some(3)), // not yet due
        ];
        assert_eq!(time_deviation(&lines, today), Some(10));
        // nothing unmet and overdue: a known zero
        assert_eq!(
            time_deviation(&[line(day(11, 1), 5, Some(2))], today),
            Some(0)
        );
        // no lines, or no known gap: unknown
        assert_eq!(time_deviation(&[], today), None);
        assert_eq!(time_deviation(&[line(day(10, 1), 5, None)], today), None);
    }

    #[test]
    fn resources_is_the_worst_shortfall_share_rounded_down() {
        let t = day(12, 1);
        let lines = [
            line(t, 10, Some(1)),
            line(t, 3, Some(1)),
            line(t, 8, Some(-4)),
            line(t, 0, Some(5)),
            line(t, 5, None),
        ];
        assert_eq!(resources_deviation(&lines), Some(3_333)); // 1 of 3
        assert_eq!(resources_deviation(&[line(t, 5, Some(-2))]), Some(0)); // surplus is no shortfall
        assert_eq!(
            resources_deviation(&[line(t, 0, Some(5)), line(t, 5, None)]),
            None
        );
    }

    #[test]
    fn cost_is_the_overspend_as_a_share_of_budget() {
        assert_eq!(cost_deviation(Some(1_150), Some(1_000), true), Some(1_500));
        assert_eq!(cost_deviation(Some(1_000), Some(1_000), true), Some(0));
        assert_eq!(cost_deviation(Some(500), Some(1_000), true), Some(0));
        assert_eq!(cost_deviation(None, Some(1_000), true), None);
        assert_eq!(cost_deviation(Some(500), None, true), None);
        assert_eq!(cost_deviation(Some(500), Some(0), true), None);
        // a huge overspend saturates and stays red; it never becomes an unknown
        assert_eq!(
            cost_deviation(Some(i64::MAX / 2), Some(1), true),
            Some(i64::MAX)
        );
    }

    #[test]
    fn an_incomplete_total_can_show_an_overspend_but_never_within_budget() {
        assert_eq!(cost_deviation(Some(1_150), Some(1_000), false), Some(1_500));
        assert_eq!(cost_deviation(Some(500), Some(1_000), false), None);
        assert_eq!(cost_deviation(Some(1_000), Some(1_000), false), None);
    }

    #[test]
    fn risk_needs_something_to_have_looked_at() {
        assert_eq!(risk_deviation(0, 0), None);
        assert_eq!(risk_deviation(4, 0), Some(0));
        assert_eq!(risk_deviation(4, 3), Some(3));
    }
}
