//! Qualifications and the qualification gap (WPM-R171). DB-free and clock-free: the as-of date is an argument.
//!
//! A qualification a person adds is `declared`; HR may mark it `verified`. A role requirement is `met` only by a **verified** match
//! in date; a declared match is shown as `declared_only`, never as met; a match past its expiry is `expired`; otherwise `missing`.
//! **Unknown is never met**, and nothing here is a score: the outcome is a fact about a record, not a rating of a person.

use chrono::NaiveDate;

/// The kinds a qualification can have.
pub const KINDS: &[&str] = &["academic", "professional", "vocational", "licence"];

/// A qualification a person holds, as the rules read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Held {
    /// The title as written.
    pub title: String,
    /// One of [`KINDS`].
    pub kind: String,
    /// The named framework a level is on (for example `RQF`), if any.
    pub framework: Option<String>,
    /// The level on that framework, if any.
    pub level: Option<i32>,
    /// The last day it is valid, if it expires.
    pub expires_on: Option<NaiveDate>,
    /// Whether HR has verified it.
    pub verified: bool,
}

/// What a role asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    /// The title asked for; matched ignoring case and surrounding space.
    pub title: String,
    /// A kind it must be, or any.
    pub kind: Option<String>,
    /// The framework a minimum level is on.
    pub framework: Option<String>,
    /// The lowest level accepted on that framework.
    pub min_level: Option<i32>,
}

/// How one requirement stands for one person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// A verified match, in date.
    Met,
    /// A match the person declared and HR has not verified.
    DeclaredOnly,
    /// A match whose expiry has passed.
    Expired,
    /// No match.
    Missing,
}

fn same(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

/// Whether a held qualification is the one a requirement asks for (date and verification aside).
#[must_use]
pub fn matches(held: &Held, req: &Requirement) -> bool {
    if !same(&held.title, &req.title) {
        return false;
    }
    if req.kind.as_ref().is_some_and(|k| !same(k, &held.kind)) {
        return false;
    }
    match (&req.framework, req.min_level) {
        (Some(f), Some(min)) => {
            held.framework.as_ref().is_some_and(|hf| same(hf, f))
                && held.level.is_some_and(|l| l >= min)
        }
        _ => true,
    }
}

/// How `req` stands given what a person holds on `today`. The best matching qualification decides.
#[must_use]
pub fn standing(req: &Requirement, held: &[Held], today: NaiveDate) -> Standing {
    let mut best = Standing::Missing;
    for h in held.iter().filter(|h| matches(h, req)) {
        let in_date = h.expires_on.is_none_or(|e| e >= today);
        let s = match (in_date, h.verified) {
            (true, true) => Standing::Met,
            (true, false) => Standing::DeclaredOnly,
            (false, _) => Standing::Expired,
        };
        best = better(best, s);
    }
    best
}

fn rank(s: Standing) -> u8 {
    match s {
        Standing::Met => 3,
        Standing::DeclaredOnly => 2,
        Standing::Expired => 1,
        Standing::Missing => 0,
    }
}

fn better(a: Standing, b: Standing) -> Standing {
    if rank(b) > rank(a) { b } else { a }
}

/// Counts per standing for the people who hold one role, against one requirement of that role.
#[derive(Debug, Clone, PartialEq)]
pub enum Rollup {
    /// The group is smaller than the floor: nothing is shown.
    Hidden,
    /// Counts per standing and the share that is met.
    Shown {
        /// A verified match in date.
        met: usize,
        /// A match declared and not verified.
        declared_only: usize,
        /// A match past its expiry.
        expired: usize,
        /// No match.
        missing: usize,
        /// `met / everyone in the group`; the group is never empty here.
        rate: f64,
    },
}

/// Roll a group up. A group smaller than `floor` (0 is read as 1) shows nothing, and neither does an empty one.
#[must_use]
pub fn rollup(standings: &[Standing], floor: usize) -> Rollup {
    if standings.is_empty() || !super::suppression::visible(standings.len(), floor) {
        return Rollup::Hidden;
    }
    let n = |s: Standing| standings.iter().filter(|x| **x == s).count();
    let met = n(Standing::Met);
    #[allow(clippy::cast_precision_loss)]
    let rate = met as f64 / standings.len() as f64;
    Rollup::Shown {
        met,
        declared_only: n(Standing::DeclaredOnly),
        expired: n(Standing::Expired),
        missing: n(Standing::Missing),
        rate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn held(title: &str, verified: bool, expires_on: Option<NaiveDate>) -> Held {
        Held {
            title: title.into(),
            kind: "professional".into(),
            framework: Some("RQF".into()),
            level: Some(5),
            expires_on,
            verified,
        }
    }

    fn req(title: &str) -> Requirement {
        Requirement {
            title: title.into(),
            kind: None,
            framework: None,
            min_level: None,
        }
    }

    #[test]
    fn a_verified_match_in_date_is_met() {
        let h = [held("Chartered Engineer", true, None)];
        assert_eq!(
            standing(&req(" chartered engineer "), &h, d(2026, 11, 2)),
            Standing::Met
        );
    }

    #[test]
    fn a_declared_match_is_never_met() {
        let h = [held("Chartered Engineer", false, None)];
        assert_eq!(
            standing(&req("Chartered Engineer"), &h, d(2026, 11, 2)),
            Standing::DeclaredOnly
        );
    }

    #[test]
    fn a_match_past_its_expiry_is_expired_and_the_last_day_still_counts() {
        let h = [held("First Aid", true, Some(d(2026, 11, 1)))];
        assert_eq!(
            standing(&req("First Aid"), &h, d(2026, 11, 2)),
            Standing::Expired
        );
        assert_eq!(
            standing(&req("First Aid"), &h, d(2026, 11, 1)),
            Standing::Met
        );
    }

    #[test]
    fn nothing_held_is_missing() {
        assert_eq!(
            standing(&req("First Aid"), &[], d(2026, 11, 2)),
            Standing::Missing
        );
    }

    #[test]
    fn the_best_match_decides() {
        let h = [
            held("First Aid", true, Some(d(2020, 1, 1))),
            held("First Aid", false, None),
            held("First Aid", true, None),
        ];
        assert_eq!(
            standing(&req("First Aid"), &h, d(2026, 11, 2)),
            Standing::Met
        );
    }

    #[test]
    fn a_minimum_level_needs_the_same_framework_and_a_recorded_level() {
        let r = Requirement {
            title: "Degree".into(),
            kind: None,
            framework: Some("RQF".into()),
            min_level: Some(6),
        };
        let mut h = held("Degree", true, None);
        h.level = Some(5);
        assert_eq!(
            standing(&r, std::slice::from_ref(&h), d(2026, 11, 2)),
            Standing::Missing
        );
        h.level = Some(6);
        assert_eq!(
            standing(&r, std::slice::from_ref(&h), d(2026, 11, 2)),
            Standing::Met
        );
        h.framework = Some("EQF".into());
        assert_eq!(
            standing(&r, std::slice::from_ref(&h), d(2026, 11, 2)),
            Standing::Missing
        );
        h.framework = Some("RQF".into());
        h.level = None;
        assert_eq!(
            standing(&r, std::slice::from_ref(&h), d(2026, 11, 2)),
            Standing::Missing,
            "an unrecorded level is not a pass"
        );
    }

    #[test]
    fn a_required_kind_must_match() {
        let r = Requirement {
            title: "Degree".into(),
            kind: Some("academic".into()),
            framework: None,
            min_level: None,
        };
        assert_eq!(
            standing(&r, &[held("Degree", true, None)], d(2026, 11, 2)),
            Standing::Missing
        );
    }

    #[test]
    fn a_group_under_the_floor_shows_nothing_and_at_the_floor_shows_counts() {
        let few = vec![Standing::Met; 9];
        assert_eq!(rollup(&few, 10), Rollup::Hidden);
        let mut ten = vec![Standing::Met; 6];
        ten.extend([
            Standing::DeclaredOnly,
            Standing::DeclaredOnly,
            Standing::Expired,
            Standing::Missing,
        ]);
        match rollup(&ten, 10) {
            Rollup::Shown {
                met,
                declared_only,
                expired,
                missing,
                rate,
            } => {
                assert_eq!((met, declared_only, expired, missing), (6, 2, 1, 1));
                assert!((rate - 0.6).abs() < 1e-9);
            }
            Rollup::Hidden => panic!("ten people meet a floor of ten"),
        }
    }

    #[test]
    fn an_empty_group_shows_nothing_even_with_no_floor() {
        assert_eq!(rollup(&[], 0), Rollup::Hidden);
    }

    #[test]
    fn declared_only_does_not_count_toward_the_met_share() {
        let all_declared = vec![Standing::DeclaredOnly; 10];
        match rollup(&all_declared, 10) {
            Rollup::Shown { met, rate, .. } => {
                assert_eq!(met, 0);
                assert!(rate.abs() < f64::EPSILON);
            }
            Rollup::Hidden => panic!("shown"),
        }
    }
}
