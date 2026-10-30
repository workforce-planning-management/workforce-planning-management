//! **People-impact for a programme** (WPM-R211, WPM-T361), the pull PPM makes of WPM. DB-free and clock-free: today and the programme's
//! end are arguments.
//!
//! Three sub-counts, each **known or unknown with a reason**, never a zero standing in for "WPM does not know":
//!
//! - **Workers committed**, by pool: the people employed today whose post is funded by the programme (WPM-D59: a programme reaches a
//!   person only through post funding), counted in each pool they count towards, and once more as "in no pool" when they count towards
//!   none. Unknown when no post is recorded as funded by the programme.
//! - **Contingent engagements ending before the programme's end**, by pool: those of the committed people who are fixed-term or
//!   contractors and whose engagement ends on or after today and before the end. Unknown when WPM is given no end date (it holds none
//!   for a programme), or when no post is funded by it. A contingent person with no engagement end recorded is counted apart, not as
//!   ending.
//! - **Over-committed pool-months**: of the pool-months in which the programme holds a claim the pool counts (WPM-D140), how many are
//!   over and how many cannot be said. Unknown when the programme holds no such claim.
//!
//! A count of people under the floor is hidden ([`crate::rules::suppression`]). **No total across pools is given**: a person can count
//! towards more than one pool, so a total would count them twice, and beside the cells it could give a hidden one away. A pool-month
//! is not a person, so the third sub-count is not floored.

use std::collections::BTreeMap;

use chrono::NaiveDate;

use crate::rules::capacity::{CellStatus, ends_before_programme};
use crate::rules::suppression::visible;

/// The floor for every count of people in the answer: the floor of the programme roll-up over the same funding data (WPM-R83), since
/// this answer goes to a caller who need not be HR.
pub const FLOOR: usize = crate::rules::equality_monitoring::DEFAULT_FLOOR;

/// Why the committed workers cannot be counted.
pub const NO_FUNDING: &str = "WPM links a person to a programme only through the funding of their post (WPM-D59), and no post of a \
person employed today is recorded as funded by this programme in the organizations you may see";
/// Why the engagements ending cannot be counted without an end date.
pub const NO_END_DATE: &str = "WPM holds no end date for a programme; pass ends_on (the date the programme's owner gives) to count \
the engagements that end before it";
/// Why the over-committed months cannot be counted.
pub const NO_DEMAND: &str = "the programme holds no demand claim that a pool counts, in the organizations you may see and the months \
asked about";

/// One person employed today whose post the programme funds. No name, no pid, no pay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Committed {
    /// The pools (their keys) the person counts towards; empty for none.
    pub pools: Vec<String>,
    /// Whether the engagement is fixed-term or a contract.
    pub contingent: bool,
    /// When the engagement ends, if recorded.
    pub engagement_ends: Option<NaiveDate>,
}

/// One pool-month in which the programme holds a claim the pool counts, and what the month comes to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimedMonth {
    /// The pool's key.
    pub pool: String,
    /// The month (its first day).
    pub month: NaiveDate,
    /// What the pool-month comes to in the capacity view.
    pub status: CellStatus,
}

/// A sub-count: known, or unknown with the reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sub<T> {
    /// What was counted.
    Known(T),
    /// Why it cannot be.
    Unknown(&'static str),
}

/// Counts of people by pool, each `None` where hidden by the floor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByPool {
    /// Each pool asked about, in the order given.
    pub pools: Vec<(String, Option<usize>)>,
    /// The people who count towards no pool.
    pub in_no_pool: Option<usize>,
}

/// Engagements ending before the programme's end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ending {
    /// By pool, and in no pool.
    pub by_pool: ByPool,
    /// Committed fixed-term and contract engagements with no end recorded, across the programme: not counted as ending.
    pub end_not_recorded: Option<usize>,
}

/// Over-committed pool-months.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Over {
    /// The pool-months in which the programme holds a counted claim.
    pub considered: usize,
    /// Of those, the months that are over.
    pub over: usize,
    /// Of those, the months that cannot be said.
    pub unknown: usize,
    /// By pool (in key order): the months over and the months unknown. A pool with neither is left out.
    pub pools: Vec<(String, Vec<NaiveDate>, Vec<NaiveDate>)>,
}

fn floored(n: usize, floor: usize) -> Option<usize> {
    visible(n, floor).then_some(n)
}

fn by_pool<'a>(
    people: impl Iterator<Item = &'a Committed> + Clone,
    pools: &[String],
    floor: usize,
) -> ByPool {
    ByPool {
        pools: pools
            .iter()
            .map(|p| {
                let n = people.clone().filter(|c| c.pools.contains(p)).count();
                (p.clone(), floored(n, floor))
            })
            .collect(),
        in_no_pool: floored(people.filter(|c| c.pools.is_empty()).count(), floor),
    }
}

/// The committed people by pool. Unknown when there are none: WPM cannot tell "nobody" from "not recorded" here.
#[must_use]
pub fn workers_committed(committed: &[Committed], pools: &[String], floor: usize) -> Sub<ByPool> {
    if committed.is_empty() {
        return Sub::Unknown(NO_FUNDING);
    }
    Sub::Known(by_pool(committed.iter(), pools, floor))
}

/// The committed fixed-term and contract engagements that end on or after `today` and before `ends_on`, by pool.
#[must_use]
pub fn contingent_ending(
    committed: &[Committed],
    pools: &[String],
    today: NaiveDate,
    ends_on: Option<NaiveDate>,
    floor: usize,
) -> Sub<Ending> {
    let Some(end) = ends_on else {
        return Sub::Unknown(NO_END_DATE);
    };
    if committed.is_empty() {
        return Sub::Unknown(NO_FUNDING);
    }
    let ending = committed.iter().filter(|c| {
        c.contingent
            && c.engagement_ends
                .is_some_and(|e| ends_before_programme(e, today, end))
    });
    let not_recorded = committed
        .iter()
        .filter(|c| c.contingent && c.engagement_ends.is_none())
        .count();
    Sub::Known(Ending {
        by_pool: by_pool(ending, pools, floor),
        end_not_recorded: floored(not_recorded, floor),
    })
}

/// The pool-months the programme holds a counted claim in, and how many are over or cannot be said.
#[must_use]
pub fn over_committed(months: &[ClaimedMonth]) -> Sub<Over> {
    if months.is_empty() {
        return Sub::Unknown(NO_DEMAND);
    }
    let mut pools: BTreeMap<&str, (Vec<NaiveDate>, Vec<NaiveDate>)> = BTreeMap::new();
    let mut seen = std::collections::BTreeSet::new();
    for m in months {
        if !seen.insert((m.pool.as_str(), m.month)) {
            continue;
        }
        let entry = pools.entry(&m.pool).or_default();
        match m.status {
            CellStatus::Over => entry.0.push(m.month),
            CellStatus::Unknown => entry.1.push(m.month),
            CellStatus::Fits | CellStatus::FitsWithPartners => {}
        }
    }
    let over = pools.values().map(|(o, _)| o.len()).sum();
    let unknown = pools.values().map(|(_, u)| u.len()).sum();
    Sub::Known(Over {
        considered: seen.len(),
        over,
        unknown,
        pools: pools
            .into_iter()
            .filter(|(_, (o, u))| !o.is_empty() || !u.is_empty())
            .map(|(p, (mut o, mut u))| {
                o.sort_unstable();
                u.sort_unstable();
                (p.to_string(), o, u)
            })
            .collect(),
    })
}

/// How the answer explains itself.
pub const DERIVATION: &str = "Workers committed = people employed today whose post is recorded as funded by the programme and whose \
funding has not ended (WPM-D59), counted in each pool they count towards (a person can count in more than one, so there is no total) \
and in_no_pool when in none. Engagements ending = of those, the fixed-term and contract engagements that end on or after today and \
before ends_on, which the caller gives because WPM holds no end date for a programme; an engagement with no end recorded is counted \
in end_not_recorded, not as ending. Over-committed pool-months = of the months in which the programme holds a claim the pool counts \
(a linked pool counts the peer's claims, an unlinked one its own), those over in the capacity view and those that cannot be said; \
over is worked out from active claims (a proposed claim is shown, not counted, until a start decision). A count of people under the \
floor is hidden (null, hidden true); zero is shown.";

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn person(pools: &[&str], contingent: bool, ends: Option<NaiveDate>) -> Committed {
        Committed {
            pools: pools.iter().map(ToString::to_string).collect(),
            contingent,
            engagement_ends: ends,
        }
    }

    fn keys() -> Vec<String> {
        vec!["a".to_string(), "b".to_string(), "c".to_string()]
    }

    #[test]
    fn nobody_funded_is_unknown_not_zero() {
        assert_eq!(
            workers_committed(&[], &keys(), FLOOR),
            Sub::Unknown(NO_FUNDING)
        );
        assert_eq!(
            contingent_ending(&[], &keys(), d(2026, 10, 30), Some(d(2027, 6, 1)), FLOOR),
            Sub::Unknown(NO_FUNDING)
        );
    }

    #[test]
    fn workers_are_counted_in_every_pool_they_count_towards_and_small_cells_are_hidden() {
        let mut people: Vec<Committed> =
            (0..10).map(|_| person(&["a", "b"], false, None)).collect();
        people.extend((0..3).map(|_| person(&["b"], false, None)));
        people.extend((0..2).map(|_| person(&[], false, None)));
        let Sub::Known(got) = workers_committed(&people, &keys(), FLOOR) else {
            panic!("known")
        };
        assert_eq!(
            got.pools,
            vec![
                ("a".to_string(), Some(10)),
                ("b".to_string(), Some(13)),
                ("c".to_string(), Some(0))
            ],
            "the ten in a and b count in both; zero is shown"
        );
        assert_eq!(got.in_no_pool, None, "two in no pool is under the floor");
        let few: Vec<Committed> = (0..4).map(|_| person(&["a"], false, None)).collect();
        let Sub::Known(few) = workers_committed(&few, &keys(), FLOOR) else {
            panic!("known")
        };
        assert_eq!(few.pools[0], ("a".to_string(), None));
        assert_eq!(few.in_no_pool, Some(0));
    }

    #[test]
    fn with_no_end_date_the_ending_count_is_unknown() {
        let people = vec![person(&["a"], true, Some(d(2026, 12, 31)))];
        assert_eq!(
            contingent_ending(&people, &keys(), d(2026, 10, 30), None, FLOOR),
            Sub::Unknown(NO_END_DATE)
        );
    }

    #[test]
    fn engagements_ending_are_contingent_on_or_after_today_and_before_the_end() {
        let today = d(2026, 10, 30);
        let end = d(2027, 6, 1);
        let mut people: Vec<Committed> = (0..10)
            .map(|_| person(&["a"], true, Some(d(2027, 1, 31))))
            .collect();
        people.push(person(&["a"], true, Some(end))); // on the end itself covers it
        people.push(person(&["a"], true, Some(d(2026, 10, 29)))); // already over
        people.push(person(&["a"], false, Some(d(2027, 1, 31)))); // permanent
        people.extend((0..10).map(|_| person(&["b"], true, None))); // no end recorded
        people.push(person(&["a"], true, Some(today))); // today counts
        let Sub::Known(got) = contingent_ending(&people, &keys(), today, Some(end), 1) else {
            panic!("known")
        };
        assert_eq!(got.by_pool.pools[0], ("a".to_string(), Some(11)));
        assert_eq!(
            got.by_pool.pools[1],
            ("b".to_string(), Some(0)),
            "no end recorded is not ending"
        );
        assert_eq!(got.end_not_recorded, Some(10));
        let Sub::Known(floored) = contingent_ending(&people, &keys(), today, Some(end), FLOOR)
        else {
            panic!("known")
        };
        assert_eq!(floored.by_pool.pools[0].1, Some(11));
        assert_eq!(floored.by_pool.in_no_pool, Some(0));
    }

    #[test]
    fn over_committed_months_are_counted_once_and_unknown_is_apart() {
        assert_eq!(over_committed(&[]), Sub::Unknown(NO_DEMAND));
        let m = |pool: &str, month: u32, status| ClaimedMonth {
            pool: pool.to_string(),
            month: d(2026, month, 1),
            status,
        };
        let got = over_committed(&[
            m("b", 11, CellStatus::Over),
            m("b", 10, CellStatus::Over),
            m("b", 10, CellStatus::Over),
            m("a", 10, CellStatus::Fits),
            m("a", 11, CellStatus::Unknown),
            m("c", 12, CellStatus::FitsWithPartners),
        ]);
        let Sub::Known(got) = got else {
            panic!("known")
        };
        assert_eq!((got.considered, got.over, got.unknown), (5, 2, 1));
        assert_eq!(
            got.pools,
            vec![
                ("a".to_string(), vec![], vec![d(2026, 11, 1)]),
                (
                    "b".to_string(),
                    vec![d(2026, 10, 1), d(2026, 11, 1)],
                    vec![]
                ),
            ],
            "a pool that fits throughout is left out"
        );
    }

    #[test]
    fn the_floor_is_the_programme_roll_up_floor() {
        assert_eq!(FLOOR, 10);
    }
}
