//! Policies and acknowledgements (WPM-R156, WPM-D105). DB-free and clock-free.
//!
//! A policy has numbered versions. A version has a title, a reference to the document (WPM holds no policy text), an effective day,
//! an audience and a number of calendar days to acknowledge it. A worker is asked about the **newest version in force**; an
//! acknowledgement of an older one does not count for it. A late acknowledgement is flagged as overdue, never forced. A manager
//! sees how many of their team have acknowledged, from the floor; the names are HR's.

use chrono::{Days, NaiveDate};
use serde::Serialize;

/// Who a version is for.
pub const AUDIENCES: &[&str] = &["all", "department", "job_title"];

/// The longest title, reference or audience value.
pub const TITLE_MAX: usize = 160;
/// The longest document reference.
pub const REF_MAX: usize = 300;
/// The days to acknowledge when none is given.
pub const DEFAULT_DAYS: i32 = 30;

/// Problems with a version, one sentence each.
#[must_use]
pub fn version_problems(
    title: &str,
    document_ref: &str,
    audience_kind: &str,
    audience_value: Option<&str>,
    days: i32,
) -> Vec<String> {
    let mut out = Vec::new();
    let t = title.trim().chars().count();
    if t == 0 || t > TITLE_MAX {
        out.push(format!("title must be 1 to {TITLE_MAX} characters"));
    }
    let r = document_ref.trim();
    if r.is_empty() || r.chars().count() > REF_MAX || r.chars().any(char::is_whitespace) {
        out.push(format!(
            "document_ref must be a reference of 1 to {REF_MAX} characters with no spaces (WPM holds no policy text)"
        ));
    }
    if AUDIENCES.contains(&audience_kind) {
        let value = audience_value.map(str::trim).filter(|v| !v.is_empty());
        match (audience_kind, value) {
            ("all", Some(_)) => out.push("an audience of all takes no value".to_string()),
            ("all", None) => {}
            (_, None) => out.push(format!("an audience of {audience_kind} needs a value")),
            (_, Some(v)) if v.chars().count() > TITLE_MAX => {
                out.push(format!("the audience value must be at most {TITLE_MAX} characters"));
            }
            _ => {}
        }
    } else {
        out.push(format!("audience must be one of: {}", AUDIENCES.join(", ")));
    }
    if !(1..=365).contains(&days) {
        out.push("acknowledge_within_days must be between 1 and 365".to_string());
    }
    out
}

/// Whether a worker is in an audience (case-insensitive, trimmed).
#[must_use]
pub fn in_audience(kind: &str, value: Option<&str>, department: &str, job_title: &str) -> bool {
    let same = |a: &str, b: Option<&str>| b.is_some_and(|b| a.trim().eq_ignore_ascii_case(b.trim()));
    match kind {
        "all" => true,
        "department" => same(department, value),
        "job_title" => same(job_title, value),
        _ => false,
    }
}

/// A version as the rule needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fact {
    /// The version number.
    pub version: i32,
    /// The day it comes into force.
    pub effective_on: NaiveDate,
}

/// The index of the newest version in force on `today`; `None` when none is yet.
#[must_use]
pub fn current(facts: &[Fact], today: NaiveDate) -> Option<usize> {
    facts
        .iter()
        .enumerate()
        .filter(|(_, f)| f.effective_on <= today)
        .max_by_key(|(_, f)| f.version)
        .map(|(i, _)| i)
}

/// Where a worker stands on a version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Standing {
    /// Acknowledged.
    Done {
        /// The day.
        on: NaiveDate,
    },
    /// Waiting, and not yet late.
    Waiting {
        /// The last day it is not late.
        due_on: NaiveDate,
    },
    /// Waiting, and past the day.
    Overdue {
        /// The day it was due.
        due_on: NaiveDate,
        /// Whole calendar days past it.
        calendar_days_late: i64,
    },
}

/// Where a worker stands. The day due is the effective day plus the calendar days allowed.
#[must_use]
pub fn standing(
    effective_on: NaiveDate,
    days: i32,
    acknowledged_on: Option<NaiveDate>,
    today: NaiveDate,
) -> Standing {
    if let Some(on) = acknowledged_on {
        return Standing::Done { on };
    }
    let due_on = effective_on
        .checked_add_days(Days::new(u64::try_from(days).unwrap_or(0)))
        .unwrap_or(effective_on);
    if today > due_on {
        Standing::Overdue {
            due_on,
            calendar_days_late: (today - due_on).num_days(),
        }
    } else {
        Standing::Waiting { due_on }
    }
}

/// How many calendar days before the day due a worker is reminded, when none is given.
pub const REMIND_DAYS: i64 = 3;

/// Whether to remind a worker now: they have not acknowledged and the day due is within `days_ahead` calendar days (or has passed).
/// Someone who has acknowledged is never reminded. A reminder is sent once per version, so a late reminder is not repeated.
#[must_use]
pub fn remind_now(standing: Standing, days_ahead: i64, today: NaiveDate) -> bool {
    match standing {
        Standing::Done { .. } => false,
        Standing::Overdue { .. } => true,
        Standing::Waiting { due_on } => (due_on - today).num_days() <= days_ahead,
    }
}

/// A team's count for a manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Count {
    /// Shown.
    Shown {
        /// Acknowledged.
        acknowledged: usize,
        /// Not yet.
        waiting: usize,
    },
    /// The team in the audience is under the floor: no count, since two numbers would name them.
    Withheld,
    /// Nobody in the team is in the audience.
    NoOne,
}

/// What a manager sees: both numbers from the floor, else nothing.
#[must_use]
pub fn team_count(in_audience: usize, acknowledged: usize, floor: usize) -> Count {
    if in_audience == 0 {
        Count::NoOne
    } else if in_audience < floor.max(1) {
        Count::Withheld
    } else {
        let acknowledged = acknowledged.min(in_audience);
        Count::Shown {
            acknowledged,
            waiting: in_audience - acknowledged,
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
    fn a_version_needs_a_title_a_reference_without_spaces_an_audience_and_days() {
        let ok = |t: &str, r: &str, k: &str, v: Option<&str>, n: i32| version_problems(t, r, k, v, n);
        assert_eq!(ok("Code of conduct", "doc:conduct-v2", "all", None, 30), Vec::<String>::new());
        assert_eq!(ok("", "doc:x", "all", None, 30).len(), 1);
        assert_eq!(ok("T", "has space", "all", None, 30).len(), 1, "no text is held, so no sentence fits");
        assert_eq!(ok("T", "", "all", None, 30).len(), 1);
        assert_eq!(ok("T", "doc:x", "everyone", None, 30).len(), 1);
        assert_eq!(ok("T", "doc:x", "department", None, 30).len(), 1);
        assert_eq!(ok("T", "doc:x", "all", Some("care"), 30).len(), 1);
        assert_eq!(ok("T", "doc:x", "job_title", Some("Nurse"), 0).len(), 1);
        assert_eq!(ok("T", "doc:x", "job_title", Some("Nurse"), 365), Vec::<String>::new());
        assert_eq!(ok("T", "doc:x", "job_title", Some("Nurse"), 366).len(), 1);
    }

    #[test]
    fn a_reminder_goes_to_someone_who_has_not_acknowledged_and_is_near_or_past_the_day() {
        let today = d("2026-10-10");
        let wait = |due: &str| Standing::Waiting { due_on: d(due) };
        assert!(!remind_now(Standing::Done { on: d("2026-10-01") }, 3, today), "someone who has acknowledged is left alone");
        assert!(!remind_now(wait("2026-10-14"), 3, today), "four calendar days left is outside three");
        assert!(remind_now(wait("2026-10-13"), 3, today), "three calendar days left is inside three");
        assert!(remind_now(wait("2026-10-10"), 0, today), "the day itself, with none ahead");
        assert!(remind_now(Standing::Overdue { due_on: d("2026-10-01"), calendar_days_late: 9 }, 0, today), "overdue always");
    }

    #[test]
    fn an_audience_matches_without_regard_to_case_or_edge_spaces() {
        assert!(in_audience("all", None, "care", "Nurse"));
        assert!(in_audience("department", Some(" Care "), "care", "Nurse"));
        assert!(!in_audience("department", Some("legal"), "care", "Nurse"));
        assert!(in_audience("job_title", Some("nurse"), "care", "Nurse"));
        assert!(!in_audience("job_title", None, "care", "Nurse"));
        assert!(!in_audience("anything else", None, "care", "Nurse"));
    }

    #[test]
    fn the_newest_version_in_force_is_the_one_asked_about() {
        let facts = [
            Fact { version: 1, effective_on: d("2026-01-01") },
            Fact { version: 3, effective_on: d("2026-12-01") },
            Fact { version: 2, effective_on: d("2026-06-01") },
        ];
        assert_eq!(current(&facts, d("2026-10-24")), Some(2), "version 2; version 3 is not in force yet");
        assert_eq!(current(&facts, d("2026-12-01")), Some(1), "in force on its own day");
        assert_eq!(current(&facts, d("2025-12-31")), None);
        assert_eq!(current(&[], d("2026-10-24")), None);
    }

    #[test]
    fn a_worker_is_done_waiting_or_overdue() {
        let eff = d("2026-10-01");
        assert_eq!(
            standing(eff, 30, Some(d("2026-10-05")), d("2026-12-31")),
            Standing::Done { on: d("2026-10-05") },
            "late and done is done"
        );
        assert_eq!(standing(eff, 30, None, d("2026-10-31")), Standing::Waiting { due_on: d("2026-10-31") });
        assert_eq!(
            standing(eff, 30, None, d("2026-11-01")),
            Standing::Overdue { due_on: d("2026-10-31"), calendar_days_late: 1 }
        );
        assert_eq!(standing(eff, 30, None, d("2026-09-01")), Standing::Waiting { due_on: d("2026-10-31") });
    }

    #[test]
    fn a_manager_sees_both_numbers_from_the_floor_and_nothing_below_it() {
        assert_eq!(team_count(10, 7, 10), Count::Shown { acknowledged: 7, waiting: 3 });
        assert_eq!(team_count(9, 7, 10), Count::Withheld, "one under");
        assert_eq!(team_count(0, 0, 10), Count::NoOne);
        assert_eq!(team_count(10, 99, 10), Count::Shown { acknowledged: 10, waiting: 0 });
        assert_eq!(team_count(10, 0, 10), Count::Shown { acknowledged: 0, waiting: 10 });
    }

    #[test]
    fn a_count_for_a_manager_carries_no_name() {
        let json = serde_json::to_string(&team_count(12, 5, 10)).unwrap();
        assert!(!json.contains("pid") && !json.contains("name"), "{json}");
    }
}
