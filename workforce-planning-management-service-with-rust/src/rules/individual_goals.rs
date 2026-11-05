//! Pure rules for **individual goals** (WPM-R224 to WPM-R226, WPM-D163 to WPM-D166). DB-free and clock-free.
//!
//! A goal is a record of intent and evidence, not a score. Nothing here computes attainment, a percentage, a rank or a rating,
//! within one goal or across people: a measure shows its own start, current and target, and an unknown value stays unknown. The
//! scoring guard in [`super::scoring`] names any function that would.

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;
use serde::Serialize;
use uuid::Uuid;

/// A goal's statuses.
pub const STATUSES: &[&str] = &[
    "proposed",
    "agreed",
    "in_progress",
    "achieved",
    "partly_achieved",
    "not_achieved",
    "withdrawn",
];

/// Where a goal may end up, which is also what an outcome in a review may say.
pub const OUTCOMES: &[&str] = &["achieved", "partly_achieved", "not_achieved", "withdrawn"];

/// How a check-in says a goal stands.
pub const CHECK_IN_STATUSES: &[&str] = &["on_track", "at_risk", "off_track"];

/// What a measure is. A milestone is yes or no; there is no free-form score.
pub const MEASURE_KINDS: &[&str] = &["number", "milestone"];

/// The longest title or measure name.
pub const MAX_TITLE_CHARS: usize = 200;

/// The longest description or check-in note.
pub const MAX_NOTE_CHARS: usize = 2_000;

/// The longest alignment reference.
pub const MAX_REF_CHARS: usize = 300;

/// The fewest people a department's counts may rest on.
pub const ROLLUP_FLOOR: usize = 5;

/// Who someone is to a goal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Party {
    /// The person the goal belongs to.
    Worker,
    /// A manager in their chain.
    Manager,
}

impl Party {
    /// The stored name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Worker => "worker",
            Self::Manager => "manager",
        }
    }
}

/// Check a goal's title and period.
///
/// # Errors
///
/// A reason when the title is blank or too long, or the period ends before it starts.
pub fn valid_goal(title: &str, start: NaiveDate, end: NaiveDate) -> Result<(), String> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > MAX_TITLE_CHARS {
        return Err(format!(
            "title is required (up to {MAX_TITLE_CHARS} characters)"
        ));
    }
    if end < start {
        return Err("period_end must not be before period_start".to_string());
    }
    Ok(())
}

/// Tidy optional text: trimmed, blank is none, too long is refused.
///
/// # Errors
///
/// A reason when it is longer than `max` characters.
pub fn clean_text(text: Option<&str>, max: usize) -> Result<Option<String>, String> {
    match text.map(str::trim).filter(|t| !t.is_empty()) {
        Some(t) if t.chars().count() > max => Err(format!("text is limited to {max} characters")),
        other => Ok(other.map(ToString::to_string)),
    }
}

/// An alignment reference is an opaque `EntityRef` URN WPM never resolves (WPM-D166): only its shape is checked.
///
/// # Errors
///
/// A reason when it has no `kind:id` shape, or is too long.
pub fn valid_alignment(reference: &str) -> Result<(), String> {
    let fine = reference.len() <= MAX_REF_CHARS
        && !reference.chars().any(char::is_whitespace)
        && reference
            .split_once(':')
            .is_some_and(|(kind, id)| !kind.is_empty() && !id.is_empty());
    if fine {
        Ok(())
    } else {
        Err("aligns_to must be a reference such as objective:Q3-reliability".to_string())
    }
}

/// A measure as proposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasureShape<'a> {
    /// Its name.
    pub name: &'a str,
    /// `number` or `milestone`.
    pub kind: &'a str,
    /// Any start, target or current value.
    pub has_number: bool,
    /// Whether a yes or no is stated.
    pub has_done: bool,
}

/// Check a measure: a number carries no yes or no, and a milestone no number. Every value is optional.
///
/// # Errors
///
/// A reason when the name is blank or too long, the kind is unknown, or the values do not fit the kind.
pub fn valid_measure(shape: MeasureShape<'_>) -> Result<(), String> {
    let name = shape.name.trim();
    if name.is_empty() || name.chars().count() > MAX_TITLE_CHARS {
        return Err(format!(
            "a measure's name is required (up to {MAX_TITLE_CHARS} characters)"
        ));
    }
    match shape.kind {
        "number" if shape.has_done => Err("a number has no yes or no".to_string()),
        "milestone" if shape.has_number => Err("a milestone has no number".to_string()),
        "number" | "milestone" => Ok(()),
        _ => Err("kind must be number or milestone".to_string()),
    }
}

/// Whether a goal may move from one status to another. `proposed` to `agreed` is [`may_agree`]'s, with its own rule.
#[must_use]
pub fn may_move(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("proposed" | "agreed" | "in_progress", "withdrawn")
            | ("agreed", "in_progress")
            | (
                "agreed" | "in_progress",
                "achieved" | "partly_achieved" | "not_achieved"
            )
    )
}

/// Whether a goal is still open for check-ins.
#[must_use]
pub fn is_open(status: &str) -> bool {
    matches!(status, "agreed" | "in_progress")
}

/// Whether a goal may still be edited: until it is agreed, after which it is what both agreed to.
#[must_use]
pub fn is_editable(status: &str) -> bool {
    status == "proposed"
}

/// The other party agrees: whoever proposed a goal cannot agree it, and HR is neither party.
#[must_use]
pub fn may_agree(status: &str, proposed_by: &str, party: Party) -> bool {
    status == "proposed" && proposed_by != party.as_str()
}

/// Whether a goal's period overlaps a review's.
#[must_use]
pub fn overlaps(goal: (NaiveDate, NaiveDate), review: (NaiveDate, NaiveDate)) -> bool {
    goal.0 <= review.1 && review.0 <= goal.1
}

/// One goal, for the roll-up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollupRow {
    /// The person's department.
    pub department: String,
    /// The person.
    pub worker: Uuid,
    /// The goal's status.
    pub status: String,
}

/// Whether a goal is an objective of a PADR with this period (WPM-R225): it overlaps, and it has not been withdrawn.
#[must_use]
pub fn is_objective(
    status: &str,
    goal: (NaiveDate, NaiveDate),
    padr: (NaiveDate, NaiveDate),
) -> bool {
    status != "withdrawn" && overlaps(goal, padr)
}

/// Who a proposal waits for (WPM-R232): the person, for a manager's proposal; their direct manager, for theirs. A person with no
/// manager on record has nobody to tell, so the answer is `None`, never a guess.
#[must_use]
pub const fn waiting_on(by: Party, worker: Uuid, manager: Option<Uuid>) -> Option<Uuid> {
    match by {
        Party::Worker => manager,
        Party::Manager => Some(worker),
    }
}

/// One department's counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RollupLine {
    /// The department.
    pub department: String,
    /// Goals per status, or `None` under the floor. Counts only: no rate, and nobody named.
    pub by_status: Option<BTreeMap<String, usize>>,
}

/// Counts by status and department. A department with goals from fewer than [`ROLLUP_FLOOR`] people is withheld; so is the
/// next smallest when exactly one is, so a total could not give it away.
#[must_use]
pub fn rollup(rows: &[RollupRow]) -> Vec<RollupLine> {
    let mut people: BTreeMap<&str, BTreeSet<Uuid>> = BTreeMap::new();
    let mut counts: BTreeMap<&str, BTreeMap<String, usize>> = BTreeMap::new();
    for row in rows {
        people
            .entry(&row.department)
            .or_default()
            .insert(row.worker);
        *counts
            .entry(&row.department)
            .or_default()
            .entry(row.status.clone())
            .or_insert(0) += 1;
    }
    let groups: Vec<(String, usize, i64)> = people
        .iter()
        .map(|(name, who)| ((*name).to_string(), who.len(), 0))
        .collect();
    super::bargaining_pack::suppress(groups, ROLLUP_FLOOR)
        .into_iter()
        .map(|group| RollupLine {
            by_status: group
                .fte_hundredths
                .and_then(|_| counts.get(group.group.as_str()).cloned()),
            department: group.group,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(month: u32, date: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, month, date).expect("a date")
    }

    fn measure<'a>(name: &'a str, kind: &'a str, number: bool, done: bool) -> MeasureShape<'a> {
        MeasureShape {
            name,
            kind,
            has_number: number,
            has_done: done,
        }
    }

    #[test]
    fn a_withdrawn_goal_is_not_an_objective_and_neither_is_one_outside_the_period() {
        let padr = (day(1, 1), day(12, 31));
        assert!(is_objective("agreed", (day(3, 1), day(4, 1)), padr));
        assert!(is_objective("proposed", (day(12, 31), day(12, 31)), padr));
        assert!(!is_objective("withdrawn", (day(3, 1), day(4, 1)), padr));
        let before = (
            NaiveDate::from_ymd_opt(2025, 1, 1).expect("a date"),
            NaiveDate::from_ymd_opt(2025, 12, 31).expect("a date"),
        );
        assert!(!is_objective("agreed", before, padr));
    }

    #[test]
    fn a_proposal_waits_for_the_other_party() {
        let worker = Uuid::from_u128(1);
        let manager = Uuid::from_u128(2);
        assert_eq!(
            waiting_on(Party::Worker, worker, Some(manager)),
            Some(manager)
        );
        assert_eq!(
            waiting_on(Party::Manager, worker, Some(manager)),
            Some(worker)
        );
        assert_eq!(
            waiting_on(Party::Worker, worker, None),
            None,
            "nobody to tell, and no guess"
        );
        assert_eq!(waiting_on(Party::Manager, worker, None), Some(worker));
    }

    fn rows(department: &str, people: u128) -> Vec<RollupRow> {
        (0..people)
            .map(|n| RollupRow {
                department: department.to_string(),
                worker: Uuid::from_u128(n + 1),
                status: "agreed".to_string(),
            })
            .collect()
    }

    #[test]
    fn a_goal_has_a_title_and_a_period() {
        assert!(valid_goal("Ship it", day(1, 1), day(12, 31)).is_ok());
        assert!(valid_goal("Ship it", day(1, 1), day(1, 1)).is_ok());
        assert!(valid_goal("   ", day(1, 1), day(12, 31)).is_err());
        assert!(valid_goal(&"x".repeat(201), day(1, 1), day(12, 31)).is_err());
        assert!(valid_goal("Ship it", day(2, 1), day(1, 1)).is_err());
    }

    #[test]
    fn a_number_and_a_milestone_keep_to_their_kind() {
        assert!(valid_measure(measure("Pages", "number", true, false)).is_ok());
        assert!(valid_measure(measure("Pages", "number", false, false)).is_ok());
        assert!(valid_measure(measure("Signed off", "milestone", false, true)).is_ok());
        assert!(valid_measure(measure("Signed off", "milestone", false, false)).is_ok());
        assert!(valid_measure(measure("Pages", "number", true, true)).is_err());
        assert!(valid_measure(measure("Signed off", "milestone", true, false)).is_err());
        assert!(valid_measure(measure("Mood", "score", true, false)).is_err());
        assert!(valid_measure(measure(" ", "number", true, false)).is_err());
    }

    #[test]
    fn blank_text_is_none_and_long_text_is_refused() {
        assert_eq!(clean_text(Some("  a  "), 5), Ok(Some("a".to_string())));
        assert_eq!(clean_text(Some("   "), 5), Ok(None));
        assert_eq!(clean_text(None, 5), Ok(None));
        assert!(clean_text(Some("toolong"), 5).is_err());
    }

    #[test]
    fn an_alignment_is_a_reference_and_nothing_more() {
        assert!(valid_alignment("objective:Q3-reliability").is_ok());
        assert!(valid_alignment("no-colon").is_err());
        assert!(valid_alignment(":x").is_err());
        assert!(valid_alignment("x:").is_err());
        assert!(valid_alignment("a b:c").is_err());
        assert!(valid_alignment(&format!("x:{}", "y".repeat(300))).is_err());
    }

    #[test]
    fn every_status_is_reachable_and_the_ends_are_final() {
        assert!(may_move("agreed", "in_progress"));
        assert!(may_move("agreed", "achieved"));
        for end in ["achieved", "partly_achieved", "not_achieved", "withdrawn"] {
            assert!(may_move("in_progress", end), "{end}");
            for status in STATUSES {
                assert!(!may_move(end, status), "{end} to {status}");
            }
        }
        assert!(may_move("proposed", "withdrawn"));
        assert!(!may_move("proposed", "achieved"), "agree it first");
        assert!(!may_move("proposed", "agreed"), "that is may_agree's");
        assert!(!may_move("in_progress", "agreed"));
    }

    #[test]
    fn only_the_other_party_agrees() {
        assert!(may_agree("proposed", "worker", Party::Manager));
        assert!(may_agree("proposed", "manager", Party::Worker));
        assert!(!may_agree("proposed", "worker", Party::Worker));
        assert!(!may_agree("proposed", "manager", Party::Manager));
        assert!(!may_agree("agreed", "worker", Party::Manager));
    }

    #[test]
    fn a_goal_is_open_while_it_is_being_worked_on() {
        assert!(is_open("agreed") && is_open("in_progress"));
        assert!(!is_open("proposed") && !is_open("achieved") && !is_open("withdrawn"));
        assert!(is_editable("proposed") && !is_editable("agreed"));
    }

    #[test]
    fn periods_overlap_at_their_edges() {
        let review = (day(4, 1), day(6, 30));
        assert!(overlaps((day(1, 1), day(4, 1)), review));
        assert!(overlaps((day(6, 30), day(12, 31)), review));
        assert!(overlaps((day(5, 1), day(5, 2)), review));
        assert!(!overlaps((day(1, 1), day(3, 31)), review));
        assert!(!overlaps((day(7, 1), day(12, 31)), review));
    }

    #[test]
    fn the_rollup_counts_and_names_nobody() {
        let mut all = rows("engineering", 6);
        all.extend(rows("finance", 7));
        let lines = rollup(&all);
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[0].by_status.as_ref().and_then(|c| c.get("agreed")),
            Some(&6)
        );
    }

    #[test]
    fn a_small_department_is_withheld_and_so_is_the_next_smallest() {
        let mut all = rows("engineering", 6);
        all.extend(rows("finance", 7));
        all.extend(rows("legal", 2));
        let lines = rollup(&all);
        let shown = |name: &str| {
            lines
                .iter()
                .find(|l| l.department == name)
                .is_some_and(|l| l.by_status.is_some())
        };
        assert!(!shown("legal"));
        assert!(!shown("engineering"), "the total would give legal away");
        assert!(shown("finance"));
    }

    #[test]
    fn the_floor_counts_people_not_goals() {
        let mut all = rows("legal", 1);
        all.extend(rows("legal", 1));
        all.extend(rows("legal", 1));
        let lines = rollup(&all);
        assert_eq!(lines[0].by_status, None);
    }
}
