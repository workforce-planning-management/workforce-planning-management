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
}
