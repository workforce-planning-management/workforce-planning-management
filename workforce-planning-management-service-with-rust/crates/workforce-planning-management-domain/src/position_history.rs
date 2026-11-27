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

/// The longest name of the role that decided a freeze, in characters.
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
    /// The row records that the holder was unlinked while the state stayed vacant (approved, open or frozen), not a change of state.
    pub holder_left: bool,
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
/// vacant; it begins where the post came from planned or from filled, or where the holder left within the run, whichever is later
/// (a post held in a vacant state was not vacant). Hands out `StartUnknown` when the run begins with a backfilled row.
#[must_use]
pub fn vacant_since(moves: &[Move]) -> VacantSince {
    let run: Vec<&Move> = moves
        .iter()
        .rev()
        .take_while(|m| is_vacant_state(&m.to))
        .collect();
    // The run is newest first: the latest holder leaving wins over the move that began the run.
    let start = run.iter().find(|m| m.holder_left).or(run.last());
    match start {
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
    /// The grade's band as a report label (`scale:band`), or `None` when no grade is recorded.
    pub band: Option<String>,
    /// HR has marked the post critical (WPM-D237).
    pub critical: bool,
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

/// The label a post without a recorded grade is grouped under in the report by band.
pub const UNKNOWN_BAND: &str = "unknown";

/// The vacancies of one group of posts: a cost centre, a band, or the critical posts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unit {
    /// The cost centre, the band label (`scale:band`, or [`UNKNOWN_BAND`]), or `critical`.
    pub group: String,
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

/// The report: the vacancies by cost centre. `floor` withholds the figures of a group with fewer live posts than the floor (and at
/// least one), the way the establishment does; `None` shows everything. A group is withheld on the posts it has, never on the
/// holders, so no holder is named.
#[must_use]
pub fn report(posts: &[Post], today: Date, floor: Option<usize>) -> Vec<Unit> {
    grouped(posts, today, floor, |p| Some(p.cost_centre.clone()))
}

/// The same figures by band (WPM-R288): a post with no recorded grade is in [`UNKNOWN_BAND`], never in a band.
#[must_use]
pub fn report_by_band(posts: &[Post], today: Date, floor: Option<usize>) -> Vec<Unit> {
    grouped(posts, today, floor, |p| {
        Some(p.band.clone().unwrap_or_else(|| UNKNOWN_BAND.to_string()))
    })
}

/// The same figures for the critical posts together (WPM-R288), or `None` when the organization has marked none.
#[must_use]
pub fn report_critical(posts: &[Post], today: Date, floor: Option<usize>) -> Option<Unit> {
    grouped(posts, today, floor, |p| {
        p.critical.then(|| "critical".to_string())
    })
    .into_iter()
    .next()
}

/// Group the posts by a key (a post whose key is `None` is left out) and count each group, in key order.
fn grouped(
    posts: &[Post],
    today: Date,
    floor: Option<usize>,
    key: impl Fn(&Post) -> Option<String>,
) -> Vec<Unit> {
    let mut groups: BTreeMap<String, Vec<&Post>> = BTreeMap::new();
    for p in posts {
        if let Some(k) = key(p) {
            groups.entry(k).or_default().push(p);
        }
    }
    groups
        .into_iter()
        .map(|(name, all)| unit(&name, &all, today, floor))
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
        group: name.to_string(),
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

/// The window the continuity of a post is counted over, in months (WPM-R290).
pub const CONTINUITY_MONTHS: i32 = 36;

/// One tenure of a post's holder, as the count needs it. Who held it is not in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tenure {
    /// The day the holder was linked (for a backfilled tenure, a day on or after the real start).
    pub started_on: Date,
    /// The day the holder was unlinked, or `None` while they hold it.
    pub ended_on: Option<Date>,
}

/// The continuity of one post over a window (WPM-R290).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Continuity {
    /// The first day of the window asked for.
    pub window_from: Date,
    /// The first day counted: the window's, or the day the post's history begins if that is later.
    pub counted_from: Date,
    /// Whether the whole window is counted. When it is not, the figures are for `counted_from` to today and say nothing earlier.
    pub complete: bool,
    /// Holders whose tenure falls in the counted days, each tenure once (one person holding twice counts twice).
    pub tenures: usize,
    /// Days in the counted span with somebody linked; `None` (unknown) only if the total does not fit.
    pub days_held: Option<i64>,
    /// Days in the counted span with nobody linked; `None` as for `days_held`.
    pub days_empty: Option<i64>,
}

/// Count the continuity of a post from its tenures: how many fall in the last `months` months, and how many days it was held and empty.
/// `known_from` is the day the post's history begins (its creation when that was recorded, otherwise the day the history began);
/// nothing before it is counted, and the result says so (WPM-D234). Days run from a start up to, not including, an end, so a holder
/// replaced the day another leaves leaves no empty day.
#[must_use]
pub fn continuity(tenures: &[Tenure], known_from: Date, today: Date, months: i32) -> Continuity {
    let window_from = today
        .checked_sub(jiff::Span::new().months(months))
        .unwrap_or(Date::MIN);
    let counted_from = window_from.max(known_from).min(today);
    // Each tenure clipped to the counted span, in order of start.
    let mut spans: Vec<(Date, Date)> = tenures
        .iter()
        // A tenure counts when it starts inside the counted span (even one linked and unlinked the same day) or runs into it.
        .filter(|t| {
            t.started_on <= today
                && (t.started_on >= counted_from || t.ended_on.is_none_or(|e| e > counted_from))
        })
        .map(|t| {
            (
                t.started_on.max(counted_from),
                t.ended_on.unwrap_or(today).min(today),
            )
        })
        .collect();
    let count = spans.len();
    spans.sort_unstable();
    // Merge the overlaps, so a day is held once however many tenures cover it.
    let mut merged: Vec<(Date, Date)> = Vec::new();
    for (from, to) in spans {
        match merged.last_mut() {
            Some(last) if from <= last.1 => last.1 = last.1.max(to),
            _ => merged.push((from, to)),
        }
    }
    // A total that does not fit is unknown, never wrapped (RCA-AGG-1); with spans of a few years it always fits.
    let days_held = merged.iter().try_fold(0_i64, |acc, (a, b)| {
        acc.checked_add(crate::dates::days_between(*a, *b))
    });
    let days_empty = days_held
        .and_then(|held| crate::dates::days_between(counted_from, today).checked_sub(held));
    Continuity {
        window_from,
        counted_from,
        complete: counted_from == window_from,
        tenures: count,
        days_held,
        days_empty,
    }
}

/// The continuity of one critical post, as the report by group reads it. Who holds it is not in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CriticalPost {
    /// The post's continuity over the window.
    pub continuity: Continuity,
    /// Somebody holds the post today.
    pub held_now: bool,
}

/// The continuity of a group of critical posts (WPM-R290): a cost centre, a band, or all of them. Every figure is `None` when the
/// group is withheld.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CriticalGroup {
    /// The cost centre, the band label (or [`UNKNOWN_BAND`]), or `all`.
    pub group: String,
    /// Withheld: too few posts, or hidden so that a withheld group cannot be worked out from the total.
    pub withheld: bool,
    /// Critical posts in the group.
    pub posts: Option<usize>,
    /// Tenures in the window, added over the posts.
    pub tenures: Option<usize>,
    /// Days held in the window, added over the posts; also `None` if the total does not fit.
    pub days_held: Option<i64>,
    /// Days empty in the window, added over the posts; as `days_held`.
    pub days_empty: Option<i64>,
    /// Posts held today.
    pub held_now: Option<usize>,
    /// Posts whose history is shorter than the window, so their figures start later (WPM-D234).
    pub counted_partly: Option<usize>,
}

/// The report of critical posts: each group, and all of them together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CriticalReport {
    /// The groups, in key order.
    pub groups: Vec<CriticalGroup>,
    /// All the critical posts together.
    pub total: CriticalGroup,
}

/// Add up the continuity of critical posts by group and withhold what the floor asks (WPM-R290, WPM-D235): a group with fewer posts
/// than the floor is withheld, and so is the smallest other group when exactly one is withheld and the total is shown, so the withheld
/// one cannot be had by subtraction (`suppression::with_total`). A senior group is small, so this withholds often, and says so.
#[must_use]
pub fn critical_report(posts: &[(String, CriticalPost)], floor: usize) -> CriticalReport {
    let mut groups: BTreeMap<&str, Vec<&CriticalPost>> = BTreeMap::new();
    for (key, post) in posts {
        groups.entry(key.as_str()).or_default().push(post);
    }
    let cells: Vec<(&str, usize)> = groups.iter().map(|(k, v)| (*k, v.len())).collect();
    let suppressed = crate::suppression::with_total(&cells, floor);
    let all: Vec<&CriticalPost> = posts.iter().map(|(_, p)| p).collect();
    CriticalReport {
        groups: suppressed
            .cells
            .iter()
            .map(|(key, shown)| {
                let members = groups.get(key).map_or(&[][..], Vec::as_slice);
                figures(key, members, shown.is_none())
            })
            .collect(),
        total: figures("all", &all, suppressed.total.is_none()),
    }
}

fn figures(group: &str, posts: &[&CriticalPost], withheld: bool) -> CriticalGroup {
    if withheld {
        return CriticalGroup {
            group: group.to_string(),
            withheld: true,
            posts: None,
            tenures: None,
            days_held: None,
            days_empty: None,
            held_now: None,
            counted_partly: None,
        };
    }
    let tenures = posts
        .iter()
        .try_fold(0_usize, |sum, p| sum.checked_add(p.continuity.tenures));
    let days = |pick: fn(&Continuity) -> Option<i64>| {
        posts.iter().try_fold(0_i64, |sum, p| {
            pick(&p.continuity).and_then(|d| sum.checked_add(d))
        })
    };
    CriticalGroup {
        group: group.to_string(),
        withheld: false,
        posts: Some(posts.len()),
        tenures,
        days_held: days(|c| c.days_held),
        days_empty: days(|c| c.days_empty),
        held_now: Some(posts.iter().filter(|p| p.held_now).count()),
        counted_partly: Some(posts.iter().filter(|p| !p.continuity.complete).count()),
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
            holder_left: false,
        }
    }

    fn left(to: &str, on: Date) -> Move {
        Move {
            holder_left: true,
            ..mv(to, on, true)
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
            band: None,
            critical: false,
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
        assert_eq!(small.group, "Small");
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

    #[test]
    fn a_review_date_today_is_not_yet_overdue_and_every_past_one_is() {
        let today = date(2026, 11, 26);
        let frozen = |review| {
            post(
                "A",
                "frozen",
                false,
                vec![mv("frozen", date(2026, 10, 1), true)],
                Some(review),
            )
        };
        let posts = vec![
            frozen(today),
            frozen(date(2026, 11, 25)),
            frozen(date(2026, 1, 1)),
            frozen(date(2026, 11, 27)),
            frozen(date(2026, 10, 2)),
        ];
        let a = &report(&posts, today, None)[0];
        assert_eq!(a.frozen, Count::Known(5));
        assert_eq!(
            a.review_overdue,
            Count::Known(3),
            "the three past dates; today and the future are not overdue"
        );
        assert_eq!(a.frozen_without_review_date, Count::Known(0));
    }

    #[test]
    fn a_cost_centre_with_exactly_the_floor_of_posts_is_shown() {
        let today = date(2026, 11, 26);
        let approved = || {
            post(
                "A",
                "approved",
                false,
                vec![mv("approved", date(2026, 11, 1), true)],
                None,
            )
        };
        let at_floor: Vec<Post> = (0..3).map(|_| approved()).collect();
        let a = &report(&at_floor, today, Some(3))[0];
        assert!(!a.withheld);
        assert_eq!(a.vacancies, Count::Known(3));
        let under: Vec<Post> = (0..2).map(|_| approved()).collect();
        assert!(report(&under, today, Some(3))[0].withheld);
        // Abolished posts do not count towards the floor: two live posts and an abolished one are still under it.
        let mut with_gone = under.clone();
        with_gone.push(post(
            "A",
            "abolished",
            false,
            vec![mv("abolished", date(2026, 5, 1), true)],
            None,
        ));
        assert!(report(&with_gone, today, Some(3))[0].withheld);
    }

    #[test]
    fn the_age_bands_meet_without_a_gap_or_an_overlap() {
        let today = date(2026, 11, 26);
        for (days, band) in [
            (0, "0-30"),
            (30, "0-30"),
            (31, "31-90"),
            (90, "31-90"),
            (91, "91-180"),
            (180, "91-180"),
            (181, "181+"),
            (4000, "181+"),
        ] {
            let on = today - jiff::Span::new().days(days);
            let a = &report(
                &[post(
                    "A",
                    "approved",
                    false,
                    vec![mv("approved", on, true)],
                    None,
                )],
                today,
                None,
            )[0];
            for (label, n) in &a.by_age {
                let want = usize::from(label == band);
                assert_eq!(*n, Count::Known(want), "{days} days in {label}");
            }
        }
    }

    #[test]
    fn a_holder_leaving_a_post_in_a_vacant_state_starts_the_vacancy_there() {
        let m = [
            mv("approved", date(2026, 1, 1), true),
            left("approved", date(2026, 9, 1)),
            mv("open", date(2026, 10, 1), true),
        ];
        assert_eq!(
            vacant_since(&m),
            VacantSince::Since {
                on: date(2026, 9, 1)
            }
        );
        // The latest leaving counts, and it gives a known start even after a backfilled row.
        let m = [
            mv("frozen", date(2026, 1, 1), false),
            left("frozen", date(2026, 3, 1)),
            left("frozen", date(2026, 6, 1)),
        ];
        assert_eq!(
            vacant_since(&m),
            VacantSince::Since {
                on: date(2026, 6, 1)
            }
        );
        // A leaving before the run began is not in the run.
        let m = [
            left("approved", date(2026, 2, 1)),
            mv("open", date(2026, 3, 1), true),
            mv("filled", date(2026, 4, 1), true),
            mv("approved", date(2026, 8, 1), true),
        ];
        assert_eq!(
            vacant_since(&m),
            VacantSince::Since {
                on: date(2026, 8, 1)
            }
        );
    }

    fn tenure(from: Date, to: Option<Date>) -> Tenure {
        Tenure {
            started_on: from,
            ended_on: to,
        }
    }

    #[test]
    fn continuity_counts_tenures_and_merges_overlaps_and_gaps() {
        let today = date(2026, 11, 26);
        let t = [
            tenure(date(2020, 1, 1), Some(date(2023, 6, 1))), // ends before the window
            tenure(date(2023, 6, 1), Some(date(2024, 6, 1))), // begins before the window: clipped
            tenure(date(2024, 9, 1), Some(date(2025, 3, 1))),
            tenure(date(2025, 2, 1), Some(date(2025, 4, 1))), // overlaps the one before
            tenure(date(2026, 1, 1), None),                   // holds it now
        ];
        let c = continuity(&t, date(2019, 1, 1), today, CONTINUITY_MONTHS);
        assert_eq!(c.window_from, date(2023, 11, 26));
        assert!(c.complete);
        assert_eq!(c.tenures, 4);
        let held = crate::dates::days_between(date(2023, 11, 26), date(2024, 6, 1))
            + crate::dates::days_between(date(2024, 9, 1), date(2025, 4, 1))
            + crate::dates::days_between(date(2026, 1, 1), today);
        assert_eq!(c.days_held, Some(held));
        assert_eq!(
            c.days_held.zip(c.days_empty).map(|(h, e)| h + e),
            Some(crate::dates::days_between(c.window_from, today))
        );
    }

    #[test]
    fn continuity_before_the_history_began_is_not_counted_and_says_so() {
        let today = date(2026, 11, 26);
        let c = continuity(
            &[tenure(date(2026, 11, 1), None)],
            date(2026, 11, 1),
            today,
            CONTINUITY_MONTHS,
        );
        assert_eq!(c.counted_from, date(2026, 11, 1));
        assert!(!c.complete);
        assert_eq!(
            (c.tenures, c.days_held, c.days_empty),
            (1, Some(25), Some(0))
        );
        // A post nobody has held is all empty, from where the count begins.
        let c = continuity(&[], date(2026, 11, 20), today, CONTINUITY_MONTHS);
        assert_eq!(
            (c.tenures, c.days_held, c.days_empty),
            (0, Some(0), Some(6))
        );
        // A history that begins in the future of the window is clipped to today, not negative.
        let c = continuity(&[], date(2027, 1, 1), today, CONTINUITY_MONTHS);
        assert_eq!((c.counted_from, c.days_empty), (today, Some(0)));
    }

    #[test]
    fn a_hand_over_on_the_same_day_leaves_no_empty_day_and_a_tenure_ending_at_the_window_is_out() {
        let today = date(2026, 11, 26);
        let from = date(2026, 1, 1);
        let t = [
            tenure(date(2025, 6, 1), Some(from)), // ends the day the count begins: not in it
            tenure(from, Some(date(2026, 6, 1))),
            tenure(date(2026, 6, 1), None),
        ];
        let c = continuity(&t, from, today, CONTINUITY_MONTHS);
        assert_eq!(c.tenures, 2);
        assert_eq!(c.days_empty, Some(0));
        // A tenure that starts after today is not counted.
        let c = continuity(
            &[tenure(date(2027, 1, 1), None)],
            from,
            today,
            CONTINUITY_MONTHS,
        );
        assert_eq!((c.tenures, c.days_held), (0, Some(0)));
    }

    #[test]
    fn a_tenure_linked_and_unlinked_the_same_day_is_a_holder_with_no_day_held() {
        let today = date(2026, 11, 26);
        let t = [
            tenure(today, Some(today)),
            tenure(today, Some(today)),
            tenure(today, None),
        ];
        let c = continuity(&t, today, today, CONTINUITY_MONTHS);
        assert_eq!(
            (c.tenures, c.days_held, c.days_empty),
            (3, Some(0), Some(0))
        );
    }

    #[test]
    fn the_report_by_band_keeps_an_unrecorded_grade_apart_and_the_critical_posts_are_one_group() {
        let today = date(2026, 11, 26);
        let graded = |cc: &str, band: Option<&str>, critical: bool| Post {
            band: band.map(ToString::to_string),
            critical,
            ..post(
                cc,
                "approved",
                false,
                vec![mv("approved", date(2026, 11, 1), true)],
                None,
            )
        };
        let posts = vec![
            graded("A", Some("s:5"), true),
            graded("B", Some("s:5"), false),
            graded("A", Some("s:6"), true),
            graded("B", None, false),
        ];
        let bands = report_by_band(&posts, today, None);
        let names: Vec<&str> = bands.iter().map(|u| u.group.as_str()).collect();
        assert_eq!(names, ["s:5", "s:6", UNKNOWN_BAND]);
        assert_eq!(bands[0].vacancies, Count::Known(2));
        assert_eq!(bands[2].vacancies, Count::Known(1));
        let critical = report_critical(&posts, today, None).expect("two critical posts");
        assert_eq!(
            (critical.group.as_str(), critical.vacancies),
            ("critical", Count::Known(2))
        );
        // Under the floor the critical group is withheld and says so; with none marked there is no group at all.
        assert!(report_critical(&posts, today, Some(3)).unwrap().withheld);
        let none: Vec<Post> = posts
            .iter()
            .map(|p| Post {
                critical: false,
                ..p.clone()
            })
            .collect();
        assert_eq!(report_critical(&none, today, None), None);
    }

    fn critical(
        tenures: usize,
        held: i64,
        empty: i64,
        complete: bool,
        held_now: bool,
    ) -> CriticalPost {
        CriticalPost {
            continuity: Continuity {
                window_from: date(2023, 11, 26),
                counted_from: if complete {
                    date(2023, 11, 26)
                } else {
                    date(2026, 1, 1)
                },
                complete,
                tenures,
                days_held: Some(held),
                days_empty: Some(empty),
            },
            held_now,
        }
    }

    #[test]
    fn critical_posts_add_up_by_group_and_a_small_group_is_withheld_with_its_complement() {
        let mut posts: Vec<(String, CriticalPost)> = (0..5)
            .map(|_| ("A".to_string(), critical(2, 900, 196, true, true)))
            .collect();
        posts.extend((0..4).map(|_| ("B".to_string(), critical(1, 1000, 96, false, false))));
        posts.push(("C".to_string(), critical(3, 500, 596, true, true)));
        let r = critical_report(&posts, 4);
        let by: BTreeMap<&str, &CriticalGroup> =
            r.groups.iter().map(|g| (g.group.as_str(), g)).collect();
        assert!(by["C"].withheld, "one post is under the floor");
        assert_eq!(by["C"].tenures, None);
        assert!(
            by["B"].withheld,
            "the smallest other group goes too, or C is the total less A and B"
        );
        assert!(!by["A"].withheld);
        assert_eq!(by["A"].posts, Some(5));
        assert_eq!(by["A"].tenures, Some(10));
        assert_eq!(by["A"].days_held, Some(4500));
        assert_eq!(by["A"].days_empty, Some(980));
        assert_eq!(by["A"].held_now, Some(5));
        assert_eq!(by["A"].counted_partly, Some(0));
        assert!(!r.total.withheld);
        assert_eq!(
            (r.total.group.as_str(), r.total.posts, r.total.tenures),
            ("all", Some(10), Some(17))
        );
        assert_eq!(r.total.counted_partly, Some(4));
        assert_eq!(r.total.held_now, Some(6));
    }

    #[test]
    fn with_no_group_under_the_floor_everything_is_shown_and_a_lone_small_group_hides_the_total() {
        let posts: Vec<(String, CriticalPost)> = (0..6)
            .map(|i| {
                (
                    if i < 3 { "A" } else { "B" }.to_string(),
                    critical(1, 10, 0, true, true),
                )
            })
            .collect();
        let r = critical_report(&posts, 3);
        assert!(r.groups.iter().all(|g| !g.withheld));
        assert!(!r.total.withheld);
        let lone = vec![("A".to_string(), critical(1, 10, 0, true, true))];
        let r = critical_report(&lone, 3);
        assert!(r.groups[0].withheld);
        assert!(r.total.withheld, "the total is the one group");
        assert_eq!(critical_report(&[], 3).groups, Vec::new());
    }

    #[test]
    fn an_unknown_day_count_makes_the_group_sum_unknown_not_a_smaller_number() {
        let mut unknown = critical(1, 10, 0, true, true);
        unknown.continuity.days_held = None;
        let posts: Vec<(String, CriticalPost)> = vec![
            ("A".into(), critical(1, 10, 0, true, true)),
            ("A".into(), unknown),
        ];
        let r = critical_report(&posts, 1);
        assert_eq!(r.groups[0].days_held, None);
        assert_eq!(r.groups[0].days_empty, Some(0));
    }
}
