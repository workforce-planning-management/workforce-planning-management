//! Pure rules for the **on-call rota**: a rotation of workers where, every
//! `period_days`, the duty passes to the next member in order, counting from
//! `starts_on`. Swaps are **overrides** — a worker on call for a date
//! window regardless of the rotation (a later override wins over an
//! earlier one). A scheduled member who is unavailable that day (on
//! approved leave, or no longer employed) is **skipped**: the duty goes to
//! the next available member in rotation order, and the assignment says so.
//! If nobody is available the day is unassigned — said plainly, not guessed.
//!
//! DB-free and clock-free. Availability is a caller-supplied predicate.

use chrono::{Duration, NaiveDate};
use serde::Serialize;
use std::collections::BTreeMap;
use uuid::Uuid;

/// Most members one rota can hold.
pub const MAX_MEMBERS: usize = 30;

/// Longest window the schedule view will compute, in days.
pub const MAX_WINDOW_DAYS: i64 = 92;

/// A rotation.
#[derive(Debug, Clone)]
pub struct Rota {
    /// Days each member stays on call before the next takes over (1–31).
    pub period_days: i64,
    /// First day of the first member's turn.
    pub starts_on: NaiveDate,
    /// Members in rotation order.
    pub members: Vec<Uuid>,
}

/// A swap: `worker` is on call from `starts_on` to `ends_on` inclusive.
#[derive(Debug, Clone, Copy)]
pub struct Override {
    /// Who is on call.
    pub worker: Uuid,
    /// First day.
    pub starts_on: NaiveDate,
    /// Last day.
    pub ends_on: NaiveDate,
}

/// Why a worker is on call on a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The rotation's own turn.
    Rotation,
    /// The scheduled member was unavailable; this is the next available one.
    Skipped,
    /// An override (swap).
    Override,
}

/// Who is on call on one day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Assignment {
    /// The day.
    pub date: NaiveDate,
    /// Who is on call; `None` when nobody can be.
    pub worker: Option<Uuid>,
    /// Why; `None` exactly when `worker` is.
    pub source: Option<Source>,
}

/// Validate a rota's shape.
///
/// # Errors
///
/// A message when the period is outside 1–31 days, there are no members, too
/// many, or a member appears twice.
pub fn validate_rota(period_days: i64, members: &[Uuid]) -> Result<(), String> {
    if !(1..=31).contains(&period_days) {
        return Err("period_days must be between 1 and 31".to_string());
    }
    if members.is_empty() {
        return Err("a rota needs at least one member".to_string());
    }
    if members.len() > MAX_MEMBERS {
        return Err(format!("at most {MAX_MEMBERS} members"));
    }
    let mut seen = std::collections::BTreeSet::new();
    if !members.iter().all(|m| seen.insert(*m)) {
        return Err("a member appears more than once".to_string());
    }
    Ok(())
}

/// The index into `members` whose turn it is on `date`, counting from
/// `starts_on`; `None` before the rota starts or with no members.
fn turn_index(rota: &Rota, date: NaiveDate) -> Option<usize> {
    if rota.members.is_empty() || date < rota.starts_on || rota.period_days < 1 {
        return None;
    }
    let turns = (date - rota.starts_on).num_days() / rota.period_days;
    let n = i64::try_from(rota.members.len()).ok()?;
    usize::try_from(turns.rem_euclid(n)).ok()
}

/// Who is *scheduled* on `date` by the rotation alone (ignoring overrides
/// and availability).
#[must_use]
pub fn scheduled(rota: &Rota, date: NaiveDate) -> Option<Uuid> {
    turn_index(rota, date).map(|i| rota.members[i])
}

/// Who is on call on `date`: an override if one covers it (the last one
/// listed wins); else the scheduled member, or the next available member in
/// rotation order if they are unavailable. `unavailable(worker, date)` says
/// who cannot take the duty.
#[must_use]
pub fn assign(
    rota: &Rota,
    overrides: &[Override],
    unavailable: &dyn Fn(Uuid, NaiveDate) -> bool,
    date: NaiveDate,
) -> Assignment {
    if date < rota.starts_on {
        return Assignment { date, worker: None, source: None };
    }
    if let Some(o) = overrides
        .iter()
        .rev()
        .find(|o| o.starts_on <= date && date <= o.ends_on)
    {
        return Assignment { date, worker: Some(o.worker), source: Some(Source::Override) };
    }
    let Some(start) = turn_index(rota, date) else {
        return Assignment { date, worker: None, source: None };
    };
    let n = rota.members.len();
    for step in 0..n {
        let member = rota.members[(start + step) % n];
        if !unavailable(member, date) {
            let source = if step == 0 { Source::Rotation } else { Source::Skipped };
            return Assignment { date, worker: Some(member), source: Some(source) };
        }
    }
    Assignment { date, worker: None, source: None }
}

/// Who a turn **starts** for on `today`: the person on call today who was
/// not on call the day before (a new turn, a swap beginning, a skip-in, or
/// the rota's first day). `None` when nobody is on call today or the same
/// person simply carries on.
#[must_use]
pub fn turn_starts(yesterday: &Assignment, today: &Assignment) -> Option<Uuid> {
    today.worker.filter(|w| yesterday.worker != Some(*w))
}

/// Swap-request statuses.
pub const SWAP_STATUSES: &[&str] = &["requested", "accepted", "declined", "cancelled"];

/// Whether a swap request may move from `from` to `to`: only a **requested**
/// swap can be decided — accepted or declined by the taker, cancelled by the
/// requester. Everything else is final.
#[must_use]
pub fn swap_can_move(from: &str, to: &str) -> bool {
    from == "requested" && matches!(to, "accepted" | "declined" | "cancelled")
}

/// Validate a swap request's shape.
///
/// # Errors
///
/// A message when requester and taker are the same person, the window is
/// inverted, or it ends before `today`.
pub fn validate_swap(
    requester: Uuid,
    taker: Uuid,
    starts_on: NaiveDate,
    ends_on: NaiveDate,
    today: NaiveDate,
) -> Result<(), String> {
    if requester == taker {
        return Err("a person cannot swap with themself".to_string());
    }
    if ends_on < starts_on {
        return Err("ends_on must not be before starts_on".to_string());
    }
    if ends_on < today {
        return Err("a swap must include today or a later day".to_string());
    }
    if (ends_on - starts_on).num_days() + 1 > MAX_WINDOW_DAYS {
        return Err(format!("a swap may cover at most {MAX_WINDOW_DAYS} days"));
    }
    Ok(())
}

/// The stretches `worker` is on call in `assignments` — what a swap request
/// can hand over (only the requester's own on-call days move, never someone
/// else's inside the same window).
#[must_use]
pub fn stretches_for(assignments: &[Assignment], worker: Uuid) -> Vec<Run> {
    runs(assignments)
        .into_iter()
        .filter(|r| r.worker == Some(worker))
        .collect()
}

/// The assignments for every day in `from..=to`.
#[must_use]
pub fn schedule(
    rota: &Rota,
    overrides: &[Override],
    unavailable: &dyn Fn(Uuid, NaiveDate) -> bool,
    from: NaiveDate,
    to: NaiveDate,
) -> Vec<Assignment> {
    let mut out = Vec::new();
    let mut day = from;
    while day <= to {
        out.push(assign(rota, overrides, unavailable, day));
        day += Duration::days(1);
    }
    out
}

/// A run of consecutive days with the same on-call worker and reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Run {
    /// First day.
    pub from: NaiveDate,
    /// Last day.
    pub to: NaiveDate,
    /// Who; `None` for a stretch with nobody available.
    pub worker: Option<Uuid>,
    /// Why; `None` exactly when `worker` is.
    pub source: Option<Source>,
}

/// Merge consecutive equal assignments into runs (assignments are in date order).
#[must_use]
pub fn runs(assignments: &[Assignment]) -> Vec<Run> {
    let mut out: Vec<Run> = Vec::new();
    for a in assignments {
        match out.last_mut() {
            Some(last)
                if last.worker == a.worker
                    && last.source == a.source
                    && last.to + Duration::days(1) == a.date =>
            {
                last.to = a.date;
            }
            _ => out.push(Run { from: a.date, to: a.date, worker: a.worker, source: a.source }),
        }
    }
    out
}

/// Days on call per worker over `assignments` — the fairness view.
#[must_use]
pub fn load(assignments: &[Assignment]) -> BTreeMap<Uuid, usize> {
    let mut counts = BTreeMap::new();
    for a in assignments {
        if let Some(w) = a.worker {
            *counts.entry(w).or_insert(0) += 1;
        }
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
    }
    fn w(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }
    fn weekly(members: &[u128]) -> Rota {
        Rota {
            period_days: 7,
            starts_on: d(5),
            members: members.iter().map(|n| w(*n)).collect(),
        }
    }
    const NOBODY_AWAY: &dyn Fn(Uuid, NaiveDate) -> bool = &|_, _| false;

    #[test]
    fn the_duty_passes_every_period_and_wraps() {
        let rota = weekly(&[1, 2, 3]);
        assert_eq!(scheduled(&rota, d(5)), Some(w(1)));
        assert_eq!(scheduled(&rota, d(11)), Some(w(1)));
        assert_eq!(scheduled(&rota, d(12)), Some(w(2)));
        assert_eq!(scheduled(&rota, d(19)), Some(w(3)));
        assert_eq!(scheduled(&rota, d(26)), Some(w(1)), "wraps to the first");
    }

    #[test]
    fn nothing_before_the_rota_starts_or_with_no_members() {
        assert_eq!(scheduled(&weekly(&[1]), d(4)), None);
        assert_eq!(scheduled(&weekly(&[]), d(6)), None);
        let a = assign(&weekly(&[1]), &[], NOBODY_AWAY, d(4));
        assert_eq!((a.worker, a.source), (None, None));
    }

    #[test]
    fn an_unavailable_member_is_skipped_to_the_next_available() {
        let rota = weekly(&[1, 2, 3]);
        let away = |who: Uuid, _| who == w(1);
        let a = assign(&rota, &[], &away, d(6));
        assert_eq!((a.worker, a.source), (Some(w(2)), Some(Source::Skipped)));
        // The skip passes over several people in order.
        let two_away = |who: Uuid, _| who == w(1) || who == w(2);
        assert_eq!(assign(&rota, &[], &two_away, d(6)).worker, Some(w(3)));
        // And the available scheduled member is the plain rotation.
        let a = assign(&rota, &[], &away, d(12));
        assert_eq!((a.worker, a.source), (Some(w(2)), Some(Source::Rotation)));
    }

    #[test]
    fn nobody_available_leaves_the_day_unassigned() {
        let rota = weekly(&[1, 2]);
        let everyone_away = |_: Uuid, _| true;
        let a = assign(&rota, &[], &everyone_away, d(6));
        assert_eq!((a.worker, a.source), (None, None));
    }

    #[test]
    fn an_override_wins_and_the_last_one_listed_wins_over_earlier() {
        let rota = weekly(&[1, 2]);
        let swap = Override { worker: w(9), starts_on: d(7), ends_on: d(8) };
        let later = Override { worker: w(8), starts_on: d(8), ends_on: d(8) };
        let all = [swap, later];
        assert_eq!(assign(&rota, &all, NOBODY_AWAY, d(6)).worker, Some(w(1)));
        let a = assign(&rota, &all, NOBODY_AWAY, d(7));
        assert_eq!((a.worker, a.source), (Some(w(9)), Some(Source::Override)));
        assert_eq!(assign(&rota, &all, NOBODY_AWAY, d(8)).worker, Some(w(8)));
        assert_eq!(assign(&rota, &all, NOBODY_AWAY, d(9)).worker, Some(w(1)));
    }

    #[test]
    fn days_merge_into_runs_and_count_into_load() {
        let rota = weekly(&[1, 2]);
        let days = schedule(&rota, &[], NOBODY_AWAY, d(5), d(18));
        let r = runs(&days);
        assert_eq!(r.len(), 2);
        assert_eq!((r[0].from, r[0].to, r[0].worker), (d(5), d(11), Some(w(1))));
        assert_eq!((r[1].from, r[1].to, r[1].worker), (d(12), d(18), Some(w(2))));
        let counts = load(&days);
        assert_eq!(counts[&w(1)], 7);
        assert_eq!(counts[&w(2)], 7);
    }

    #[test]
    fn a_skip_splits_a_run_because_the_reason_changes() {
        let rota = weekly(&[1, 2]);
        // Member 1 is away only on the 8th.
        let away = |who: Uuid, day: NaiveDate| who == w(1) && day == d(8);
        let r = runs(&schedule(&rota, &[], &away, d(7), d(9)));
        assert_eq!(r.len(), 3);
        assert_eq!(r[1].worker, Some(w(2)));
        assert_eq!(r[1].source, Some(Source::Skipped));
    }

    #[test]
    fn a_turn_starts_when_the_person_on_call_changes() {
        let rota = weekly(&[1, 2]);
        let days = schedule(&rota, &[], NOBODY_AWAY, d(4), d(13));
        let starts: Vec<Option<Uuid>> =
            days.windows(2).map(|p| turn_starts(&p[0], &p[1])).collect();
        // d(4) before the rota: nobody. d(5) starts member 1; d(12) starts 2.
        assert_eq!(starts[0], Some(w(1)));
        assert_eq!(starts[1..7], [None; 6]);
        assert_eq!(starts[7], Some(w(2)));
        assert_eq!(starts[8], None);
    }

    #[test]
    fn nobody_on_call_means_no_turn_starts() {
        let rota = weekly(&[1]);
        let everyone_away = |_: Uuid, _| true;
        let days = schedule(&rota, &[], &everyone_away, d(5), d(6));
        assert_eq!(turn_starts(&days[0], &days[1]), None);
    }

    #[test]
    fn swaps_are_decided_once() {
        for to in ["accepted", "declined", "cancelled"] {
            assert!(swap_can_move("requested", to));
            assert!(!swap_can_move(to, "accepted"), "{to} is final");
        }
        assert!(!swap_can_move("requested", "requested"));
        assert!(!swap_can_move("requested", "bogus"));
    }

    #[test]
    fn a_swap_needs_two_people_and_a_live_window() {
        let (a, b) = (w(1), w(2));
        assert!(validate_swap(a, b, d(6), d(8), d(5)).is_ok());
        assert!(validate_swap(a, a, d(6), d(8), d(5)).is_err());
        assert!(validate_swap(a, b, d(8), d(6), d(5)).is_err());
        assert!(validate_swap(a, b, d(1), d(4), d(5)).is_err(), "all in the past");
        assert!(validate_swap(a, b, d(4), d(6), d(5)).is_ok(), "spans today");
        assert!(validate_swap(a, b, d(5), d(5) + Duration::days(92), d(5)).is_err());
    }

    #[test]
    fn only_the_requesters_own_days_can_move() {
        let rota = weekly(&[1, 2]);
        let days = schedule(&rota, &[], NOBODY_AWAY, d(5), d(25));
        // Weeks: 1 (5–11), 2 (12–18), 1 (19–25).
        let mine = stretches_for(&days, w(1));
        assert_eq!(mine.len(), 2);
        assert_eq!((mine[0].from, mine[0].to), (d(5), d(11)));
        assert_eq!((mine[1].from, mine[1].to), (d(19), d(25)));
        assert_eq!(stretches_for(&days, w(9)), Vec::<Run>::new());
    }

    #[test]
    fn rota_shape_is_validated() {
        let m = [w(1), w(2)];
        assert!(validate_rota(7, &m).is_ok());
        assert!(validate_rota(0, &m).is_err());
        assert!(validate_rota(32, &m).is_err());
        assert!(validate_rota(7, &[]).is_err());
        assert!(validate_rota(7, &[w(1), w(1)]).is_err());
        let many: Vec<Uuid> = (0..31).map(w).collect();
        assert!(validate_rota(7, &many).is_err());
    }
}
