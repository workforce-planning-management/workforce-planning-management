//! **The state history of a position** (WPM-R288, WPM-R289, WPM-D233, WPM-D234, WPM-D235), DB-free and clock-free. Every move of a
//! position is a row that is added and never changed; from the rows the age of an open vacancy is worked out, which a time-to-fill
//! figure cannot do, since it counts only the posts that were filled. A post that was vacant before the history began has a start
//! that is **unknown, never zero**. A freeze carries a reason, the role that decided and a review date. Abolished posts are shown
//! beside the ages, because a vacancy age can be cut by abolishing the post. No holder is named anywhere in this module.

use jiff::civil::Date;
use serde::Serialize;
use std::collections::BTreeMap;

/// The reasons a post is frozen (WPM-R289).
pub const FREEZE_REASONS: &[&str] = &["reorganization", "funding", "demand_review", "other"];

/// The age bands of a vacancy, in days, as (label, from, to inclusive); the last has no end.
pub const AGE_BANDS: &[(&str, i64, Option<i64>)] = &[
    ("0-30", 0, Some(30)),
    ("31-90", 31, Some(90)),
    ("91-180", 91, Some(180)),
    ("181+", 181, None),
];

/// The longest role name or reason note, in characters.
pub const MAX_ROLE: usize = 80;

/// One move of a position, as recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move {
    /// The state moved to.
    pub to: String,
    /// The date of the move.
    pub on: Date,
    /// Whether the date is the date of the move. A post that already had its state when the history began has one row whose date
    /// is only the date it was backfilled, so the start is unknown (WPM-D234).
    pub start_known: bool,
}

/// What a freeze is recorded with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Freeze<'a> {
    /// One of [`FREEZE_REASONS`].
    pub reason: Option<&'a str>,
    /// The role that decided, not a person.
    pub decided_by_role: Option<&'a str>,
    /// The date the freeze is to be looked at again.
    pub review_on: Option<Date>,
}

/// Check what a move to `frozen` is made with (WPM-R289): a reason from the list, the deciding role, and a review date that is
/// not already past. A freeze with no review date is refused.
///
/// # Errors
///
/// The problems found, each saying what to add.
pub fn check_freeze(freeze: &Freeze<'_>, today: Date) -> Result<(), Vec<String>> {
    let mut out = Vec::new();
    match freeze.reason {
        Some(r) if FREEZE_REASONS.contains(&r) => {}
        _ => out.push(format!(
            "a freeze needs a reason: one of {}",
            FREEZE_REASONS.join(", ")
        )),
    }
    match freeze.decided_by_role.map(str::trim) {
        Some(r) if (1..=MAX_ROLE).contains(&r.chars().count()) => {}
        _ => out.push(format!(
            "a freeze needs the role that decided it, 1 to {MAX_ROLE} characters"
        )),
    }
    match freeze.review_on {
        None => out.push("a freeze needs a review date".to_string()),
        Some(d) if d <= today => {
            out.push("the review date of a freeze is in the future".to_string());
        }
        Some(_) => {}
    }
    if out.is_empty() { Ok(()) } else { Err(out) }
}

/// Whether a state is one in which a post is vacant when nobody holds it: approved, open or frozen. A planned post is not yet a
/// vacancy; a filled one is held; an abolished one is gone.
#[must_use]
pub fn is_vacant_state(state: &str) -> bool {
    matches!(state, "approved" | "open" | "frozen")
}

/// Since when a post has been vacant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VacantSince {
    /// The date the current run of vacant states began.
    Since {
        /// That date.
        on: Date,
    },
    /// The post was vacant before the history began (WPM-D234): the age is unknown, never zero.
    StartUnknown,
    /// The post is not vacant: it is planned, filled or abolished.
    NotVacant,
}

/// Since when a post is vacant, from its moves in the order they were made. The run is every move back from the last whose state is
/// vacant; it begins where the post came from planned or from filled. Hands out `StartUnknown` when the run begins with a backfilled
/// row.
#[must_use]
pub fn vacant_since(moves: &[Move]) -> VacantSince {
    let run: Vec<&Move> = moves
        .iter()
        .rev()
        .take_while(|m| is_vacant_state(&m.to))
        .collect();
    match run.last() {
        None => VacantSince::NotVacant,
        Some(first) if first.start_known => VacantSince::Since { on: first.on },
        Some(_) => VacantSince::StartUnknown,
    }
}

/// The age of a vacancy in whole days, or `None` where the start is unknown or the post is not vacant. A date in the future is 0.
#[must_use]
pub fn age_days(since: VacantSince, today: Date) -> Option<i64> {
    match since {
        VacantSince::Since { on } => Some(crate::dates::days_between(on, today).max(0)),
        VacantSince::StartUnknown | VacantSince::NotVacant => None,
    }
}

/// One post as the report reads it. No holder is in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Post {
    /// The cost centre, which stands for the unit.
    pub cost_centre: String,
    /// The state now.
    pub state: String,
    /// Whether a worker is linked as the holder.
    pub held: bool,
    /// The moves, oldest first.
    pub moves: Vec<Move>,
    /// The review date of the freeze now in force, where the post is frozen.
    pub review_on: Option<Date>,
}

/// A count that is a number, or is withheld because the cut is under the floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum Count {
    /// A count.
    Known(usize),
    /// Withheld.
    Withheld(Withheld),
}

/// The marker of a withheld figure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Withheld {
    /// Under the small-group floor.
    Withheld,
}

/// The vacancies of one cost centre.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unit {
    /// The cost centre.
    pub cost_centre: String,
    /// Open vacancies: approved, open or frozen posts nobody holds.
    pub vacancies: Count,
    /// Vacancies by age band, in the order of [`AGE_BANDS`].
    pub by_age: BTreeMap<String, Count>,
    /// Vacancies whose start was not recorded.
    pub start_unknown: Count,
    /// Frozen vacancies.
    pub frozen: Count,
    /// Frozen vacancies with no review date (a post frozen before the review date was asked for).
    pub frozen_without_review_date: Count,
    /// Frozen vacancies whose review date has passed: a finding (WPM-R289).
    pub review_overdue: Count,
    /// Posts abolished, beside the ages (WPM-D235).
    pub abolished: usize,
    /// Whether a figure above is withheld.
    pub withheld: bool,
}

/// The report: the vacancies by cost centre. `floor` withholds the figures of a cost centre with fewer posts than the floor (and
/// at least one), the way the establishment does; `None` shows everything. A cost centre is withheld on the posts it has, never on
/// the holders, so no holder is named.
#[must_use]
pub fn report(posts: &[Post], today: Date, floor: Option<usize>) -> Vec<Unit> {
    let mut centres: BTreeMap<&str, Vec<&Post>> = BTreeMap::new();
    for p in posts {
        centres.entry(p.cost_centre.as_str()).or_default().push(p);
    }
    centres
        .into_iter()
        .map(|(name, all)| unit(name, &all, today, floor))
        .collect()
}

fn unit(name: &str, all: &[&Post], today: Date, floor: Option<usize>) -> Unit {
    let live = all.iter().filter(|p| p.state != "abolished").count();
    let small = floor.is_some_and(|f| live > 0 && live < f);
    // Each vacancy with its age in days, where the start is known.
    let vacancies: Vec<(&Post, Option<i64>)> = all
        .iter()
        .filter(|p| !p.held && is_vacant_state(&p.state))
        .map(|p| (*p, age_days(vacant_since(&p.moves), today)))
        .collect();
    let frozen: Vec<&Post> = vacancies
        .iter()
        .map(|(p, _)| *p)
        .filter(|p| p.state == "frozen")
        .collect();
    let in_band = |b: &(&str, i64, Option<i64>)| {
        vacancies
            .iter()
            .filter(|(_, age)| age.is_some_and(|d| d >= b.1 && b.2.is_none_or(|end| d <= end)))
            .count()
    };
    let show = |n: usize| {
        if small {
            Count::Withheld(Withheld::Withheld)
        } else {
            Count::Known(n)
        }
    };
    Unit {
        cost_centre: name.to_string(),
        vacancies: show(vacancies.len()),
        by_age: AGE_BANDS
            .iter()
            .map(|b| (b.0.to_string(), show(in_band(b))))
            .collect(),
        start_unknown: show(vacancies.iter().filter(|(_, age)| age.is_none()).count()),
        frozen: show(frozen.len()),
        frozen_without_review_date: show(frozen.iter().filter(|p| p.review_on.is_none()).count()),
        review_overdue: show(
            frozen
                .iter()
                .filter(|p| p.review_on.is_some_and(|d| d < today))
                .count(),
        ),
        // A count of posts that exist or are gone carries no holder; it is shown whole, as the establishment shows its post count.
        abolished: all.iter().filter(|p| p.state == "abolished").count(),
        withheld: small,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    fn mv(to: &str, on: Date, known: bool) -> Move {
        Move {
            to: to.into(),
            on,
            start_known: known,
        }
    }

    fn freeze<'a>(
        reason: Option<&'a str>,
        role: Option<&'a str>,
        review: Option<Date>,
    ) -> Freeze<'a> {
        Freeze {
            reason,
            decided_by_role: role,
            review_on: review,
        }
    }

    #[test]
    fn a_freeze_needs_a_reason_a_role_and_a_future_review_date() {
        let today = date(2026, 11, 26);
        assert!(
            check_freeze(
                &freeze(
                    Some("funding"),
                    Some("Finance director"),
                    Some(date(2027, 1, 1))
                ),
                today
            )
            .is_ok()
        );
        let e = check_freeze(&freeze(None, None, None), today).unwrap_err();
        assert_eq!(e.len(), 3);
        let e = check_freeze(&freeze(Some("because"), Some("  "), Some(today)), today).unwrap_err();
        assert_eq!(e.len(), 3);
        assert!(e[2].contains("future"));
        let long = "x".repeat(MAX_ROLE + 1);
        assert!(
            check_freeze(
                &freeze(Some("other"), Some(&long), Some(date(2027, 1, 1))),
                today
            )
            .is_err()
        );
    }

    #[test]
    fn a_vacancy_runs_from_the_move_that_made_the_post_vacant() {
        let m = [
            mv("planned", date(2026, 1, 1), true),
            mv("approved", date(2026, 2, 1), true),
            mv("open", date(2026, 3, 1), true),
            mv("filled", date(2026, 4, 1), true),
            mv("approved", date(2026, 6, 1), true),
            mv("frozen", date(2026, 7, 1), true),
        ];
        // The run is approved, frozen: it began on 1 June, not on 1 February.
        assert_eq!(
            vacant_since(&m),
            VacantSince::Since {
                on: date(2026, 6, 1)
            }
        );
        assert_eq!(age_days(vacant_since(&m), date(2026, 6, 11)), Some(10));
        assert_eq!(
            age_days(vacant_since(&m), date(2026, 5, 1)),
            Some(0),
            "never below zero"
        );
    }

    #[test]
    fn a_post_vacant_before_the_history_began_has_an_unknown_start_never_zero() {
        let m = [mv("approved", date(2026, 11, 26), false)];
        assert_eq!(vacant_since(&m), VacantSince::StartUnknown);
        assert_eq!(age_days(vacant_since(&m), date(2026, 12, 30)), None);
        // Once it moves on within the vacant run the start is still unknown.
        let m = [
            mv("approved", date(2026, 11, 26), false),
            mv("open", date(2026, 12, 1), true),
        ];
        assert_eq!(vacant_since(&m), VacantSince::StartUnknown);
    }

    #[test]
    fn a_post_that_is_not_in_a_vacant_state_is_not_vacant() {
        for state in ["planned", "filled", "abolished"] {
            assert_eq!(
                vacant_since(&[mv(state, date(2026, 1, 1), true)]),
                VacantSince::NotVacant
            );
        }
        assert_eq!(vacant_since(&[]), VacantSince::NotVacant);
        assert_eq!(age_days(VacantSince::NotVacant, date(2026, 1, 1)), None);
    }

    fn post(cc: &str, state: &str, held: bool, moves: Vec<Move>, review_on: Option<Date>) -> Post {
        Post {
            cost_centre: cc.into(),
            state: state.into(),
            held,
            moves,
            review_on,
        }
    }

    #[test]
    fn the_report_counts_open_vacancies_by_age_with_abolished_posts_beside_them() {
        let today = date(2026, 11, 26);
        let posts = vec![
            post(
                "A",
                "approved",
                false,
                vec![mv("approved", date(2026, 11, 20), true)],
                None,
            ),
            post(
                "A",
                "open",
                false,
                vec![
                    mv("approved", date(2026, 8, 1), true),
                    mv("open", date(2026, 9, 1), true),
                ],
                None,
            ),
            post(
                "A",
                "approved",
                false,
                vec![mv("approved", date(2025, 1, 1), true)],
                None,
            ),
            post(
                "A",
                "approved",
                false,
                vec![mv("approved", date(2026, 11, 26), false)],
                None,
            ),
            post(
                "A",
                "frozen",
                false,
                vec![mv("frozen", date(2026, 10, 1), true)],
                Some(date(2026, 11, 1)),
            ),
            post(
                "A",
                "frozen",
                false,
                vec![mv("frozen", date(2026, 10, 1), true)],
                None,
            ),
            post(
                "A",
                "frozen",
                false,
                vec![mv("frozen", date(2026, 10, 1), true)],
                Some(date(2027, 3, 1)),
            ),
            post(
                "A",
                "filled",
                true,
                vec![mv("filled", date(2026, 1, 1), true)],
                None,
            ),
            post(
                "A",
                "abolished",
                false,
                vec![mv("abolished", date(2026, 5, 1), true)],
                None,
            ),
        ];
        let out = report(&posts, today, None);
        assert_eq!(out.len(), 1);
        let a = &out[0];
        assert_eq!(a.vacancies, Count::Known(7));
        assert_eq!(a.by_age["0-30"], Count::Known(1));
        assert_eq!(
            a.by_age["31-90"],
            Count::Known(3),
            "the three frozen posts, 56 days"
        );
        assert_eq!(a.by_age["91-180"], Count::Known(1));
        assert_eq!(a.by_age["181+"], Count::Known(1));
        assert_eq!(
            a.start_unknown,
            Count::Known(1),
            "unknown is its own figure, in no band"
        );
        assert_eq!(a.frozen, Count::Known(3));
        assert_eq!(a.frozen_without_review_date, Count::Known(1));
        assert_eq!(a.review_overdue, Count::Known(1));
        assert_eq!(
            a.abolished, 1,
            "beside the ages, so abolishing cannot cut an age unseen"
        );
        assert!(!a.withheld);
    }

    #[test]
    fn a_small_cost_centre_is_withheld_and_says_so() {
        let today = date(2026, 11, 26);
        let posts = vec![
            post(
                "Small",
                "approved",
                false,
                vec![mv("approved", date(2026, 11, 1), true)],
                None,
            ),
            post(
                "Gone",
                "abolished",
                false,
                vec![mv("abolished", date(2026, 5, 1), true)],
                None,
            ),
        ];
        let out = report(&posts, today, Some(5));
        let small = &out[1];
        assert_eq!(small.cost_centre, "Small");
        assert!(small.withheld);
        assert_eq!(small.vacancies, Count::Withheld(Withheld::Withheld));
        assert_eq!(small.by_age["0-30"], Count::Withheld(Withheld::Withheld));
        // A centre with nothing but abolished posts has nothing to hide.
        assert!(!out[0].withheld);
        assert_eq!(out[0].vacancies, Count::Known(0));
        assert_eq!(
            serde_json::to_value(small.vacancies).unwrap(),
            serde_json::json!("withheld")
        );
    }

    #[test]
    fn a_held_post_is_not_a_vacancy_even_in_a_vacant_state() {
        let posts = vec![post(
            "A",
            "open",
            true,
            vec![mv("open", date(2026, 1, 1), true)],
            None,
        )];
        assert_eq!(
            report(&posts, date(2026, 11, 26), None)[0].vacancies,
            Count::Known(0)
        );
    }
}
