//! Decision gates and facility time (WPM-R107, WPM-R108, WPM-D68). DB-free and clock-free.
//!
//! A **decision gate** says a kind of change needs consultation with a recognised body. When someone makes such a change, WPM points at the gate
//! and asks for the consultation reference; it does not stop them (the law and the agreements decide, not this tool). **Facility time** is
//! minutes released for representative duties, per body per period, so a budget can be agreed. Neither has a person in it.

use chrono::NaiveDate;

/// The kinds of change a gate may be set for.
pub const CHANGES: &[&str] = &[
    "redundancy_proposal",
    "shift_pattern_change",
    "new_monitoring",
    "new_or_changed_score",
    "pay_structure_change",
    "other",
];

/// The longest note.
pub const NOTE_MAX: usize = 200;

/// Problems with a gate.
#[must_use]
pub fn gate_problems(change_kind: &str, note: Option<&str>) -> Vec<String> {
    let mut out = Vec::new();
    if !CHANGES.contains(&change_kind) {
        out.push(format!(
            "change_kind must be one of: {}",
            CHANGES.join(", ")
        ));
    }
    if note.is_some_and(|n| n.chars().count() > NOTE_MAX) {
        out.push(format!("note must be at most {NOTE_MAX} characters"));
    }
    out
}

/// A gate as the rules see it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate<'a> {
    /// The kind of change.
    pub change_kind: &'a str,
    /// The body to consult.
    pub body: &'a str,
}

/// The bodies to consult for `change`, once each and in order.
#[must_use]
pub fn bodies_for<'a>(gates: &[Gate<'a>], change: &str) -> Vec<&'a str> {
    let mut out: Vec<&str> = gates
        .iter()
        .filter(|g| g.change_kind == change)
        .map(|g| g.body)
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// Where a change stands against its gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Standing {
    /// No gate is set for this kind of change.
    NoGate,
    /// A gate is set and no consultation reference was given: ask for it.
    ReferenceNeeded {
        /// How many bodies are to be consulted.
        bodies: usize,
    },
    /// A gate is set and a reference was given (WPM does not judge whether it is enough).
    ReferenceGiven,
}

/// The standing of a change: whether a gate applies and whether a consultation reference came with it.
#[must_use]
pub fn standing(bodies: usize, reference_given: bool) -> Standing {
    match (bodies, reference_given) {
        (0, _) => Standing::NoGate,
        (_, true) => Standing::ReferenceGiven,
        (n, false) => Standing::ReferenceNeeded { bodies: n },
    }
}

/// The longest a period of facility time may be.
pub const PERIOD_MAX_DAYS: i64 = 366;
/// The most minutes one record may hold (100,000 hours).
pub const MINUTES_MAX: i32 = 6_000_000;

/// Problems with a facility-time record. `existing` are the `(start, end)` periods already recorded for the body.
#[must_use]
pub fn facility_problems(
    start: NaiveDate,
    end: NaiveDate,
    minutes: i32,
    budget_minutes: Option<i32>,
    existing: &[(NaiveDate, NaiveDate)],
) -> Vec<String> {
    let mut out = Vec::new();
    if end < start {
        out.push("period_end must not be before period_start".to_string());
    } else if (end - start).num_days() + 1 > PERIOD_MAX_DAYS {
        out.push(format!("a period must be at most {PERIOD_MAX_DAYS} days"));
    }
    for (name, value) in [
        ("minutes", Some(minutes)),
        ("budget_minutes", budget_minutes),
    ] {
        if value.is_some_and(|v| !(0..=MINUTES_MAX).contains(&v)) {
            out.push(format!("{name} must be 0 to {MINUTES_MAX}"));
        }
    }
    if end >= start && existing.iter().any(|(s, e)| start <= *e && *s <= end) {
        out.push("this period overlaps one already recorded for the body".to_string());
    }
    out
}

/// Whether the time released went over the budget; `None` when no budget was agreed (an unknown is not "within").
#[must_use]
pub fn over_budget(minutes: i32, budget_minutes: Option<i32>) -> Option<bool> {
    budget_minutes.map(|b| minutes > b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    #[test]
    fn a_gate_needs_a_known_change_and_a_short_note() {
        assert_eq!(
            gate_problems("new_or_changed_score", Some("Works council")),
            Vec::<String>::new()
        );
        assert_eq!(gate_problems("coffee_machine", None).len(), 1);
        assert_eq!(gate_problems("other", Some(&"n".repeat(201))).len(), 1);
    }

    #[test]
    fn the_bodies_for_a_change_are_those_of_its_gates_once_each() {
        let gates = [
            Gate {
                change_kind: "new_or_changed_score",
                body: "b",
            },
            Gate {
                change_kind: "new_or_changed_score",
                body: "a",
            },
            Gate {
                change_kind: "new_or_changed_score",
                body: "a",
            },
            Gate {
                change_kind: "new_monitoring",
                body: "c",
            },
        ];
        assert_eq!(bodies_for(&gates, "new_or_changed_score"), ["a", "b"]);
        assert_eq!(
            bodies_for(&gates, "pay_structure_change"),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn a_change_is_ungated_asked_for_a_reference_or_given_one() {
        assert_eq!(standing(0, false), Standing::NoGate);
        assert_eq!(standing(0, true), Standing::NoGate);
        assert_eq!(standing(2, false), Standing::ReferenceNeeded { bodies: 2 });
        assert_eq!(standing(2, true), Standing::ReferenceGiven);
    }

    #[test]
    fn facility_time_is_a_dated_period_with_minutes_and_no_overlap() {
        let none: Vec<(NaiveDate, NaiveDate)> = vec![];
        assert_eq!(
            facility_problems(
                d("2026-04-01"),
                d("2027-03-31"),
                120_000,
                Some(150_000),
                &none
            ),
            Vec::<String>::new()
        );
        assert_eq!(
            facility_problems(d("2026-04-01"), d("2026-03-31"), 0, None, &none).len(),
            1
        );
        assert_eq!(
            facility_problems(d("2026-01-01"), d("2027-01-02"), 0, None, &none).len(),
            1,
            "367 days"
        );
        assert_eq!(
            facility_problems(d("2026-01-01"), d("2026-12-31"), 0, None, &none),
            Vec::<String>::new(),
            "365 days"
        );
        assert_eq!(
            facility_problems(
                d("2026-04-01"),
                d("2026-04-30"),
                -1,
                Some(MINUTES_MAX + 1),
                &none
            )
            .len(),
            2
        );
        let held = vec![(d("2026-04-01"), d("2026-06-30"))];
        assert_eq!(
            facility_problems(d("2026-06-30"), d("2026-09-30"), 0, None, &held).len(),
            1,
            "one shared day overlaps"
        );
        assert_eq!(
            facility_problems(d("2026-07-01"), d("2026-09-30"), 0, None, &held),
            Vec::<String>::new()
        );
    }

    #[test]
    fn over_budget_is_unknown_without_a_budget() {
        assert_eq!(over_budget(100, None), None);
        assert_eq!(over_budget(100, Some(100)), Some(false));
        assert_eq!(over_budget(101, Some(100)), Some(true));
    }
}
