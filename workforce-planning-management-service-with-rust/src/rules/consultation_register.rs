//! The consultation register (WPM-R105, WPM-D68). DB-free and clock-free.
//!
//! A record per proposal that may trigger a collective duty: the body consulted, the proposal as counts and dates, the topics, a reference
//! to the documents shared, when consultation began and ended, and the outcome. It answers "what did we consult on, with whom and when?".
//! It holds **no attendee, representative or minute**: the consultation is with a body, and what was said is not stored.

use chrono::NaiveDate;

use super::consultation::Assessment;

/// What consultation may be about.
pub const TOPICS: &[&str] = &[
    "redundancy",
    "restructuring",
    "transfer",
    "pay",
    "working_time",
    "other",
];

/// How a consultation ended.
pub const OUTCOMES: &[&str] = &["agreed", "no_agreement", "withdrawn"];

/// The longest establishment name.
pub const ESTABLISHMENT_MAX: usize = 80;
/// The longest document reference.
pub const REF_MAX: usize = 200;

/// The topics as stored: known, unique, in the order of [`TOPICS`]; or the problem.
///
/// # Errors
///
/// When there is none, or one is unknown.
pub fn clean_topics(given: &[String]) -> Result<String, String> {
    if given.is_empty() {
        return Err(format!(
            "topics needs at least one of: {}",
            TOPICS.join(", ")
        ));
    }
    if let Some(bad) = given.iter().find(|t| !TOPICS.contains(&t.as_str())) {
        return Err(format!(
            "topic `{bad}` is not one of: {}",
            TOPICS.join(", ")
        ));
    }
    Ok(TOPICS
        .iter()
        .filter(|t| given.iter().any(|g| g == **t))
        .copied()
        .collect::<Vec<_>>()
        .join(","))
}

/// A document reference: one token with no spaces (WPM holds no document text).
fn reference_problem(text: &str) -> Option<String> {
    let r = text.trim();
    (r.is_empty() || r.chars().count() > REF_MAX || r.contains(char::is_whitespace))
        .then(|| format!("document_ref must be a reference of 1 to {REF_MAX} characters with no spaces (WPM holds no document text)"))
}

/// Problems with opening a record.
#[must_use]
pub fn open_problems(
    establishment: &str,
    establishment_size: i32,
    dismissals: i32,
    document_ref: Option<&str>,
    topics: &[String],
) -> Vec<String> {
    let mut out = Vec::new();
    let n = establishment.trim().chars().count();
    if n == 0 || n > ESTABLISHMENT_MAX {
        out.push(format!(
            "establishment must be 1 to {ESTABLISHMENT_MAX} characters"
        ));
    }
    if establishment_size < 0 {
        out.push("establishment_size must not be negative".to_string());
    }
    if dismissals < 1 {
        out.push("dismissals must be at least 1".to_string());
    }
    out.extend(document_ref.and_then(reference_problem));
    if let Err(problem) = clean_topics(topics) {
        out.push(problem);
    }
    out
}

/// Problems with recording the day consultation began: not in the future, and not twice.
#[must_use]
pub fn begin_problems(started_on: NaiveDate, today: NaiveDate, already_begun: bool) -> Vec<String> {
    let mut out = Vec::new();
    if already_begun {
        out.push("consultation has already begun".to_string());
    }
    if started_on > today {
        out.push("started_on must not be in the future".to_string());
    }
    out
}

/// Problems with closing: a known outcome, a day not in the future and not before it began; consultation that never began can only be withdrawn.
#[must_use]
pub fn close_problems(
    outcome: &str,
    ended_on: NaiveDate,
    started_on: Option<NaiveDate>,
    today: NaiveDate,
    has_agreement: bool,
) -> Vec<String> {
    let mut out = Vec::new();
    if !OUTCOMES.contains(&outcome) {
        out.push(format!("outcome must be one of: {}", OUTCOMES.join(", ")));
    }
    if ended_on > today {
        out.push("ended_on must not be in the future".to_string());
    }
    if started_on.is_some_and(|start| ended_on < start) {
        out.push("ended_on must not be before consultation began".to_string());
    }
    if started_on.is_none() && outcome != "withdrawn" {
        out.push("consultation that never began can only be withdrawn".to_string());
    }
    if has_agreement && outcome != "agreed" {
        out.push("an agreement can only be linked to an outcome of agreed".to_string());
    }
    out
}

/// Where a consultation stands against the latest day it should have begun.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Timeliness {
    /// No threshold is met, so nothing is due (this is not "no duty").
    NothingDue,
    /// A duty may apply but the profile gives no fixed consultation period (see the notice date instead).
    NoFixedPeriod,
    /// Not begun, with days left (0 on the last day).
    NotBegun {
        /// Days to the latest start.
        days_left: i64,
    },
    /// Not begun and the latest day has passed.
    NotBegunAndLate {
        /// Days past it.
        days_late: i64,
    },
    /// Began on or before the latest day.
    BegunInTime,
    /// Began after the latest day.
    BegunLate {
        /// Days after it.
        days_late: i64,
    },
}

/// Compare when consultation began (or, if not, `today`) with the latest start the assessment gave.
#[must_use]
pub fn timeliness(
    assessment: &Assessment,
    started_on: Option<NaiveDate>,
    today: NaiveDate,
) -> Timeliness {
    if !assessment.may_apply {
        return Timeliness::NothingDue;
    }
    let Some(latest) = assessment.latest_consultation_start else {
        return Timeliness::NoFixedPeriod;
    };
    match started_on {
        Some(start) if start <= latest => Timeliness::BegunInTime,
        Some(start) => Timeliness::BegunLate {
            days_late: (start - latest).num_days(),
        },
        None if today <= latest => Timeliness::NotBegun {
            days_left: (latest - today).num_days(),
        },
        None => Timeliness::NotBegunAndLate {
            days_late: (today - latest).num_days(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::consultation::{Proposal, assess, great_britain};

    fn d(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    fn topics(t: &[&str]) -> Vec<String> {
        t.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn topics_are_known_unique_and_in_a_fixed_order() {
        assert_eq!(
            clean_topics(&topics(&["pay", "redundancy", "pay"])),
            Ok("redundancy,pay".to_string())
        );
        assert!(clean_topics(&[]).is_err());
        assert!(
            clean_topics(&topics(&["redundancy", "gossip"]))
                .unwrap_err()
                .contains("gossip")
        );
    }

    #[test]
    fn opening_needs_an_establishment_a_dismissal_and_a_reference_with_no_spaces() {
        let t = topics(&["redundancy"]);
        assert_eq!(
            open_problems("Leeds site", 300, 25, Some("DOC-123"), &t),
            Vec::<String>::new()
        );
        assert_eq!(open_problems(" ", 300, 25, None, &t).len(), 1);
        assert_eq!(open_problems("Leeds", -1, 25, None, &t).len(), 1);
        assert_eq!(open_problems("Leeds", 300, 0, None, &t).len(), 1);
        assert_eq!(
            open_problems("Leeds", 300, 25, Some("two words"), &t).len(),
            1
        );
        assert_eq!(open_problems("Leeds", 300, 25, None, &[]).len(), 1);
    }

    #[test]
    fn beginning_is_once_and_not_in_the_future() {
        assert_eq!(
            begin_problems(d("2026-05-01"), d("2026-05-01"), false),
            Vec::<String>::new()
        );
        assert_eq!(
            begin_problems(d("2026-05-02"), d("2026-05-01"), false).len(),
            1
        );
        assert_eq!(
            begin_problems(d("2026-05-01"), d("2026-05-01"), true).len(),
            1
        );
    }

    #[test]
    fn closing_needs_a_known_outcome_and_days_in_order_and_an_agreement_only_when_agreed() {
        let (s, t) = (Some(d("2026-05-01")), d("2026-06-01"));
        assert_eq!(
            close_problems("agreed", d("2026-05-20"), s, t, true),
            Vec::<String>::new()
        );
        assert_eq!(
            close_problems("no_agreement", t, s, t, false),
            Vec::<String>::new(),
            "today is fine"
        );
        assert_eq!(
            close_problems("open", d("2026-05-20"), s, t, false).len(),
            1
        );
        assert_eq!(
            close_problems("agreed", d("2026-06-02"), s, t, false).len(),
            1
        );
        assert_eq!(
            close_problems("agreed", d("2026-04-30"), s, t, false).len(),
            1
        );
        assert_eq!(
            close_problems("no_agreement", d("2026-05-20"), s, t, true).len(),
            1
        );
        assert_eq!(
            close_problems("withdrawn", d("2026-05-20"), None, t, false),
            Vec::<String>::new()
        );
        assert_eq!(
            close_problems("agreed", d("2026-05-20"), None, t, false).len(),
            1,
            "never begun: only withdrawn"
        );
    }

    fn gb_assessment(dismissals: u32) -> Assessment {
        let p = Proposal {
            establishment_size: 500,
            dismissals,
            earliest_termination: d("2026-09-30"),
            others: vec![],
            agreed_period_days: None,
        };
        assess(&great_britain(), &p, d("2026-01-01"))
    }

    #[test]
    fn timeliness_is_read_against_the_latest_start() {
        // 25 dismissals: 30 days before 2026-09-30 is 2026-08-31.
        let a = gb_assessment(25);
        assert_eq!(
            timeliness(&a, Some(d("2026-08-31")), d("2026-10-01")),
            Timeliness::BegunInTime,
            "the last day counts"
        );
        assert_eq!(
            timeliness(&a, Some(d("2026-09-02")), d("2026-10-01")),
            Timeliness::BegunLate { days_late: 2 }
        );
        assert_eq!(
            timeliness(&a, None, d("2026-08-31")),
            Timeliness::NotBegun { days_left: 0 }
        );
        assert_eq!(
            timeliness(&a, None, d("2026-08-01")),
            Timeliness::NotBegun { days_left: 30 }
        );
        assert_eq!(
            timeliness(&a, None, d("2026-09-03")),
            Timeliness::NotBegunAndLate { days_late: 3 }
        );
    }

    #[test]
    fn nothing_is_due_under_the_thresholds_and_a_profile_with_no_period_has_no_deadline() {
        assert_eq!(
            timeliness(&gb_assessment(3), None, d("2026-09-01")),
            Timeliness::NothingDue
        );
        let eu = assess(
            &crate::rules::consultation::eu_directive_98_59(),
            &Proposal {
                establishment_size: 300,
                dismissals: 30,
                earliest_termination: d("2026-09-30"),
                others: vec![],
                agreed_period_days: None,
            },
            d("2026-01-01"),
        );
        assert_eq!(
            timeliness(&eu, None, d("2026-09-01")),
            Timeliness::NoFixedPeriod
        );
    }
}
