//! Pure rules for the **derived oversight measures** (WPM-R67, WPM-T114): the monthly figures the
//! service computes itself instead of taking a reported reading. DB-free and clock-free.
//!
//! Every derivation returns `None` when the data cannot say, never `0`: no one to divide by, fewer
//! people than the floor, a department row from before the split by basis, or a pool-month a partner
//! has not answered. An oversight criterion treats `None` as not met, so an unknown is never a pass.

use crate::rules::capacity::CellStatus;
use crate::rules::funding::SplitRow;

/// A computation that can supply a measure's readings.
#[derive(Debug, Clone, Copy)]
pub struct Derivation {
    /// The stored key.
    pub key: &'static str,
    /// The unit of the figure; a derived measure takes it, so a threshold cannot be read in another.
    pub unit: &'static str,
    /// What is counted, what is left out and the period, in words a reader can check.
    pub definition: &'static str,
}

/// The derivations available. Each is dated, so it can be computed for a past month.
///
/// Single points of failure and governance hours are not here: neither is recorded with a date, so a
/// past month cannot be derived, and a figure computed today and filed under last month would be an
/// invention (they join when the monthly pack freezes them, WPM-T116).
pub const DERIVATIONS: &[Derivation] = &[
    Derivation {
        key: "turnover_rate",
        unit: "basis points",
        definition: "Leavers in the calendar month, doubled and divided by the sum of the headcount the \
                     day before the month and on its last day (the turnover rate of the metrics layer, \
                     for one month), in basis points. Employed workers of the organization only; \
                     soft-deleted records are left out. Absent when the larger of the two headcounts is \
                     below the aggregate floor, or when both are zero.",
    },
    Derivation {
        key: "contingent_share",
        unit: "basis points",
        definition: "Contractor plus fixed-term FTE as a share of all FTE, in basis points, from the \
                     last headcount snapshot dated in the calendar month. Absent when the month has no \
                     snapshot, when any department's row on that date predates the split by basis, when \
                     fewer people than the aggregate floor were employed, or when the FTE is zero.",
    },
    Derivation {
        key: "overcommitted_pools",
        unit: "pools",
        definition: "The number of skill pools whose own supply plus firm partner capacity falls short \
                     of the demand from active programme claims in the calendar month. Absent when the \
                     organization has no pool, or when any pool-month cannot be said because a partner \
                     has not answered. Computed today from the supply, claims and leave now on file, so \
                     a later correction to them changes a past month.",
    },
    Derivation {
        key: "time_to_report",
        unit: "calendar days",
        definition: "The longest time, in calendar days, from the first sweep that saw an early-warning \
                     condition to a person acknowledging it, over the warnings first seen in the calendar \
                     month. Absent when no warning was first seen in the month, and absent while any of \
                     them is unacknowledged: the figure is not known yet, and a slow report must not read \
                     as a fast one.",
    },
];

/// The derivation with this key.
#[must_use]
pub fn find(key: &str) -> Option<&'static Derivation> {
    DERIVATIONS.iter().find(|d| d.key == key)
}

/// One month's turnover in basis points, rounded half up. `None` when the larger headcount is below
/// `floor` or there is nothing to divide by.
#[must_use]
pub fn turnover_bp(leavers: usize, opening: usize, closing: usize, floor: usize) -> Option<i64> {
    if opening.max(closing) < floor || opening + closing == 0 {
        return None;
    }
    let leavers = i64::try_from(leavers).ok()?;
    let sum = i64::try_from(opening + closing).ok()?;
    Some((leavers * 40_000 + sum) / (2 * sum))
}

/// The contingent share, in basis points, of the department rows of one snapshot date. `None` for no
/// rows, a row without the split, fewer people than `floor`, or no FTE.
#[must_use]
pub fn contingent_bp(rows: &[SplitRow], floor: usize) -> Option<i64> {
    if rows.is_empty() {
        return None;
    }
    let mut contingent = 0;
    for r in rows {
        contingent += r.contractor_fte? + r.fixed_term_fte?;
    }
    let people: i64 = rows.iter().map(|r| r.headcount).sum();
    let fte: i64 = rows.iter().map(|r| r.fte).sum();
    if usize::try_from(people).ok()? < floor || fte <= 0 {
        return None;
    }
    Some((contingent * 20_000 + fte) / (2 * fte))
}

/// How many pools are over in a month, given each pool's cell. `None` for no pools, or when any cell
/// cannot be said (the count would be a lower bound, and a lower bound passes an at-most threshold).
#[must_use]
pub fn over_committed(cells: &[CellStatus]) -> Option<i64> {
    if cells.is_empty() || cells.contains(&CellStatus::Unknown) {
        return None;
    }
    i64::try_from(cells.iter().filter(|c| **c == CellStatus::Over).count()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn row(headcount: i64, fte: i64, c: Option<i64>, f: Option<i64>) -> SplitRow {
        SplitRow {
            as_of: NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
            headcount,
            fte,
            contractor_fte: c,
            fixed_term_fte: f,
        }
    }

    #[test]
    fn turnover_is_leavers_over_the_mean_headcount_in_basis_points() {
        // 2 leavers, mean headcount 100 -> 2 %.
        assert_eq!(turnover_bp(2, 100, 100, 10), Some(200));
        assert_eq!(turnover_bp(0, 100, 100, 10), Some(0));
        // 1 / mean 15 = 6.67 % -> 667, rounded half up.
        assert_eq!(turnover_bp(1, 14, 16, 10), Some(667));
    }

    #[test]
    fn turnover_is_absent_below_the_floor_or_with_nothing_to_divide_by() {
        assert_eq!(turnover_bp(1, 9, 9, 10), None);
        assert_eq!(turnover_bp(0, 0, 0, 0), None);
        // Either end reaching the floor is enough.
        assert_eq!(turnover_bp(1, 9, 10, 10), Some(1053));
    }

    #[test]
    fn contingent_share_sums_the_departments_before_dividing() {
        let rows = [
            row(10, 1000, Some(100), Some(0)),
            row(10, 500, Some(0), Some(150)),
        ];
        // (100 + 150) / 1500 = 16.67 % -> 1667.
        assert_eq!(contingent_bp(&rows, 10), Some(1667));
        assert_eq!(
            contingent_bp(&[row(10, 1000, Some(0), Some(0))], 10),
            Some(0)
        );
    }

    #[test]
    fn contingent_share_is_absent_when_it_cannot_be_said() {
        assert_eq!(contingent_bp(&[], 10), None);
        // One row from before the split spoils the date, however good the others are.
        let rows = [row(10, 1000, Some(100), Some(0)), row(10, 500, None, None)];
        assert_eq!(contingent_bp(&rows, 10), None);
        assert_eq!(contingent_bp(&[row(10, 1000, Some(100), None)], 10), None);
        assert_eq!(contingent_bp(&[row(9, 900, Some(0), Some(0))], 10), None);
        assert_eq!(contingent_bp(&[row(10, 0, Some(0), Some(0))], 10), None);
    }

    #[test]
    fn over_committed_counts_the_pools_that_are_over() {
        use CellStatus::{Fits, FitsWithPartners, Over, Unknown};
        assert_eq!(over_committed(&[Fits, FitsWithPartners]), Some(0));
        assert_eq!(over_committed(&[Over, Fits, Over]), Some(2));
        assert_eq!(over_committed(&[Over, Unknown]), None);
        assert_eq!(over_committed(&[]), None);
    }

    #[test]
    fn every_derivation_is_findable_and_has_a_unit() {
        for d in DERIVATIONS {
            assert_eq!(find(d.key).map(|f| f.key), Some(d.key));
            assert!(!d.unit.is_empty() && !d.definition.is_empty());
        }
        assert!(find("single_points_of_failure").is_none());
    }
}
