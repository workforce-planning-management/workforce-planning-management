//! Equipment issued and returned (WPM-R155). DB-free and clock-free.
//!
//! An asset has a kind, a tag and an owner role. It is **not located**: nothing here is a position, a cost or a reading. An
//! issue ties an asset to a worker from a day; a return or a write-off closes it, and a write-off needs a reason. Equipment still
//! out is what a leaver's record cannot be completed over.

use chrono::NaiveDate;

/// The kinds of asset offered.
pub const KINDS: &[&str] = &["laptop", "phone", "pass", "key", "other"];

/// How an issue ends.
pub const OUTCOMES: &[&str] = &["returned", "written_off"];

/// The longest tag, role or note.
pub const TAG_MAX: usize = 60;
/// The longest note.
pub const NOTE_MAX: usize = 300;

/// Problems with an asset, one sentence each.
#[must_use]
pub fn asset_problems(kind: &str, tag: &str, owner_role: &str) -> Vec<String> {
    let mut out = Vec::new();
    if !KINDS.contains(&kind) {
        out.push(format!("kind must be one of: {}", KINDS.join(", ")));
    }
    for (name, text) in [("tag", tag), ("owner_role", owner_role)] {
        let n = text.trim().chars().count();
        if n == 0 || n > TAG_MAX {
            out.push(format!("{name} must be 1 to {TAG_MAX} characters"));
        }
    }
    out
}

/// Problems with issuing: not in the future, and not before the worker's first day.
#[must_use]
pub fn issue_problems(issued_on: NaiveDate, today: NaiveDate, hired_on: NaiveDate) -> Vec<String> {
    let mut out = Vec::new();
    if issued_on > today {
        out.push("issued_on must not be in the future".to_string());
    }
    if issued_on < hired_on {
        out.push("issued_on must not be before the worker's first day".to_string());
    }
    out
}

/// Problems with ending an issue: a known outcome, a day inside the issue and not in the future, and a reason for a write-off.
#[must_use]
pub fn return_problems(
    outcome: &str,
    returned_on: NaiveDate,
    issued_on: NaiveDate,
    today: NaiveDate,
    note: Option<&str>,
) -> Vec<String> {
    let mut out = Vec::new();
    if !OUTCOMES.contains(&outcome) {
        out.push(format!("outcome must be one of: {}", OUTCOMES.join(", ")));
    }
    if returned_on > today {
        out.push("returned_on must not be in the future".to_string());
    }
    if returned_on < issued_on {
        out.push("returned_on must not be before the day it was issued".to_string());
    }
    let reason = note.map_or(0, |n| n.trim().chars().count());
    if outcome == "written_off" && reason == 0 {
        out.push("a write-off needs a reason".to_string());
    }
    if reason > NOTE_MAX {
        out.push(format!("the note must be at most {NOTE_MAX} characters"));
    }
    out
}

/// What a manager sees of their team's equipment: how many of their direct reports still hold some, from the floor; under it, nothing,
/// since a count of two or three would name them. Never a list, an item or a kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TeamHolding {
    /// Shown.
    Shown {
        /// Reports holding at least one item.
        holding: usize,
        /// Reports holding none.
        none: usize,
    },
    /// The team is under the floor: no count.
    Withheld,
    /// The manager has no direct reports in work.
    NoOne,
}

/// The count for a team of `team` people, `holding` of whom hold equipment.
#[must_use]
pub fn team_holding(team: usize, holding: usize, floor: usize) -> TeamHolding {
    if team == 0 {
        TeamHolding::NoOne
    } else if team < floor.max(1) {
        TeamHolding::Withheld
    } else {
        let holding = holding.min(team);
        TeamHolding::Shown {
            holding,
            none: team - holding,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    #[test]
    fn an_asset_needs_a_known_kind_a_tag_and_an_owner_role() {
        assert_eq!(asset_problems("laptop", "LT-1", "IT"), Vec::<String>::new());
        assert_eq!(asset_problems("drone", "LT-1", "IT").len(), 1);
        assert_eq!(asset_problems("laptop", "  ", "IT").len(), 1);
        assert_eq!(asset_problems("laptop", "LT-1", &"x".repeat(61)).len(), 1);
        assert_eq!(
            asset_problems("laptop", &"x".repeat(60), "IT"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn an_issue_is_not_in_the_future_or_before_the_first_day() {
        let today = d("2026-10-24");
        assert_eq!(
            issue_problems(today, today, d("2026-10-24")),
            Vec::<String>::new()
        );
        assert_eq!(
            issue_problems(d("2026-10-25"), today, d("2020-01-01")).len(),
            1
        );
        assert_eq!(
            issue_problems(d("2026-10-01"), today, d("2026-10-02")).len(),
            1
        );
    }

    #[test]
    fn a_return_sits_between_the_issue_and_today() {
        let today = d("2026-10-24");
        let issued = d("2026-10-01");
        assert_eq!(
            return_problems("returned", issued, issued, today, None),
            Vec::<String>::new()
        );
        assert_eq!(
            return_problems("returned", today, issued, today, None),
            Vec::<String>::new()
        );
        assert_eq!(
            return_problems("returned", d("2026-09-30"), issued, today, None).len(),
            1
        );
        assert_eq!(
            return_problems("returned", d("2026-10-25"), issued, today, None).len(),
            1
        );
        assert_eq!(return_problems("lost", today, issued, today, None).len(), 1);
    }

    #[test]
    fn a_write_off_needs_a_reason_and_a_return_does_not() {
        let today = d("2026-10-24");
        assert_eq!(
            return_problems("written_off", today, today, today, None).len(),
            1
        );
        assert_eq!(
            return_problems("written_off", today, today, today, Some("   ")).len(),
            1
        );
        assert_eq!(
            return_problems("written_off", today, today, today, Some("lost on a train")),
            Vec::<String>::new()
        );
        assert_eq!(
            return_problems("returned", today, today, today, None),
            Vec::<String>::new()
        );
        assert_eq!(
            return_problems("returned", today, today, today, Some(&"x".repeat(301))).len(),
            1
        );
    }

    #[test]
    fn nothing_here_is_a_place_a_cost_or_a_reading() {
        let src = include_str!("equipment.rs");
        let code = &src[..src.find("#[cfg(test)]").unwrap()];
        for word in [
            "location",
            "latitude",
            "cost",
            "value",
            "telemetry",
            "usage",
        ] {
            assert!(
                !code
                    .lines()
                    .filter(|l| !l.trim_start().starts_with("//"))
                    .any(|l| l.contains(word)),
                "{word}"
            );
        }
    }

    #[test]
    fn a_manager_sees_a_count_only_from_the_floor() {
        assert_eq!(team_holding(0, 0, 10), TeamHolding::NoOne);
        for team in 1..10 {
            assert_eq!(
                team_holding(team, team, 10),
                TeamHolding::Withheld,
                "{team}"
            );
        }
        assert_eq!(
            team_holding(10, 3, 10),
            TeamHolding::Shown {
                holding: 3,
                none: 7
            }
        );
        assert_eq!(
            team_holding(10, 0, 10),
            TeamHolding::Shown {
                holding: 0,
                none: 10
            }
        );
        assert_eq!(
            team_holding(10, 99, 10),
            TeamHolding::Shown {
                holding: 10,
                none: 0
            },
            "never more than the team"
        );
        assert_eq!(
            team_holding(1, 1, 0),
            TeamHolding::Shown {
                holding: 1,
                none: 0
            },
            "a floor of 0 is read as 1"
        );
    }
}
