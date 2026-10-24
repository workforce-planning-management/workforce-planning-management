//! **Positions** (WPM-R152, WPM-D104), DB-free and clock-free. A position is a budgeted seat that exists whether or not anyone
//! holds it. It carries a budget in FTE, a funding kind and a state, and no amount of pay. Who holds it is a link, so the
//! establishment (the approved number of posts) can be counted without naming a person. Spec: `spec/positions.md`.
//!
//! Vacancy and over-establishment are arithmetic over positions, never records, and an over-establishment on one position is
//! never netted against a vacancy on another. An unknown holder FTE is an unknown vacancy, not the whole budget.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::Serialize;

use crate::rules::lifecycle;

/// The states of a position.
pub const STATES: &[&str] = &["planned", "approved", "open", "filled", "frozen", "abolished"];

/// The legal moves. `abolished` has none out. `open` is a seat with a requisition, `filled` a seat with a holder.
pub const MOVES: &[(&str, &str)] = &[
    ("planned", "approved"),
    ("planned", "abolished"),
    ("approved", "open"),
    ("approved", "frozen"),
    ("approved", "abolished"),
    ("open", "filled"),
    ("open", "approved"), // the requisition was withdrawn
    ("open", "frozen"),
    ("open", "abolished"),
    ("filled", "open"), // the holder left and the seat is being filled again
    ("filled", "approved"), // the holder left and nothing is being done yet
    ("filled", "frozen"),
    ("filled", "abolished"),
    ("frozen", "approved"),
    ("frozen", "abolished"),
];

/// What the controller knows about a position when it is asked to move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Facts {
    /// A requisition is linked.
    pub has_requisition: bool,
    /// A worker is linked as the holder.
    pub has_holder: bool,
}

/// Check a move. The table says which states may follow; the facts say whether the position is in the shape the new state means.
///
/// # Errors
///
/// A refusal naming the state the position is in, as every lifecycle does.
pub fn check_move(from: &str, to: &str, facts: Facts) -> Result<(), String> {
    lifecycle::check("position", MOVES, from, to)?;
    match to {
        "open" if !facts.has_requisition => Err(format!(
            "position is {from:?}; it can be open only once a requisition is linked to it"
        )),
        "filled" if !facts.has_holder => Err(format!(
            "position is {from:?}; it can be filled only once a worker is linked as its holder"
        )),
        "open" | "approved" if from == "filled" && facts.has_holder => Err(
            "position is \"filled\"; unlink its holder before it is made vacant".to_string(),
        ),
        "abolished" if facts.has_holder => Err(format!(
            "position is {from:?}; unlink its holder first, so abolishing it settles what happens to the worker (WPM-D104)"
        )),
        _ => Ok(()),
    }
}

/// The funding kinds, the same as post funding.
pub use crate::rules::funding::KINDS as FUNDING_KINDS;

/// Check what a position is made of.
///
/// # Errors
///
/// The problems found, each saying what to change.
pub fn validate(
    cost_centre: &str,
    budget_fte_percent: i32,
    funding_kind: &str,
    funding_ends_on: Option<NaiveDate>,
    programme_ref: Option<&str>,
) -> Result<(), Vec<String>> {
    let mut out = Vec::new();
    if !(1..=80).contains(&cost_centre.trim().chars().count()) {
        out.push("cost_centre is 1 to 80 characters".to_string());
    }
    if !(1..=100).contains(&budget_fte_percent) {
        out.push("budget_fte_percent is 1 to 100: a position is one seat, so at most one full-time equivalent".to_string());
    }
    if let Err(e) = crate::rules::funding::validate_kind(funding_kind) {
        out.push(e);
    } else {
        if matches!(funding_kind, "time_limited" | "external") && funding_ends_on.is_none() {
            out.push(format!("a {funding_kind} funding needs the date it ends"));
        }
        match (funding_kind, programme_ref) {
            ("programme", None) => out.push("a programme funding names the programme (a thing: reference)".to_string()),
            ("programme", Some(p)) if !is_thing(p) => out.push("programme_ref is thing:<uuid>".to_string()),
            (k, Some(_)) if k != "programme" => out.push("programme_ref is only for a programme funding".to_string()),
            _ => {}
        }
    }
    if out.is_empty() { Ok(()) } else { Err(out) }
}

fn is_thing(s: &str) -> bool {
    s.strip_prefix("thing:").is_some_and(|id| uuid::Uuid::parse_str(id).is_ok())
}

/// Who holds a position, as far as the arithmetic needs to know.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Holding {
    /// Nobody holds it.
    Vacant,
    /// A worker holds it, at this FTE percentage.
    Held(i32),
    /// A worker holds it and their FTE is not recorded.
    HeldUnknown,
}

/// A position as the arithmetic sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seat {
    /// The cost centre.
    pub cost_centre: String,
    /// The state.
    pub state: String,
    /// The budget, in FTE percent.
    pub budget_fte_percent: i32,
    /// Who holds it.
    pub holding: Holding,
}

/// A figure in FTE percent, or why there is none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Fte {
    /// A number.
    Known {
        /// FTE percent (100 is one full-time equivalent).
        percent: i64,
    },
    /// A holder's FTE is not recorded for at least one position.
    Unknown,
    /// Under the floor for a caller who may not see small groups.
    Withheld,
}

/// A position's own vacancy and over-establishment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Gap {
    /// Budget minus what is held, when positive. Unknown when the holder's FTE is not recorded.
    pub vacant: Fte,
    /// What is held beyond the budget. Unknown when the holder's FTE is not recorded.
    pub over: Fte,
}

/// The vacancy and over-establishment of one position.
#[must_use]
pub fn gap(s: &Seat) -> Gap {
    let budget = i64::from(s.budget_fte_percent);
    match s.holding {
        Holding::Vacant => Gap { vacant: Fte::Known { percent: budget }, over: Fte::Known { percent: 0 } },
        Holding::Held(held) => {
            let held = i64::from(held);
            Gap {
                vacant: Fte::Known { percent: (budget - held).max(0) },
                over: Fte::Known { percent: (held - budget).max(0) },
            }
        }
        Holding::HeldUnknown => Gap { vacant: Fte::Unknown, over: Fte::Unknown },
    }
}

/// Whether a state counts toward the establishment: approved or later, and not ended. A planned seat is a proposal, an abolished
/// one is gone.
#[must_use]
pub fn in_establishment(state: &str) -> bool {
    matches!(state, "approved" | "open" | "filled" | "frozen")
}

/// The establishment of one cost centre.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CostCentre {
    /// The cost centre.
    pub cost_centre: String,
    /// How many positions are in the establishment.
    pub positions: usize,
    /// Their budget.
    pub budgeted: Fte,
    /// What is held.
    pub held: Fte,
    /// The budget no one holds, added over positions, never netted against an over-establishment.
    pub vacant: Fte,
    /// What is held beyond budget, added over positions.
    pub over: Fte,
    /// Of the vacancy, what sits in frozen positions and is not to be filled for now.
    pub frozen_vacant: Fte,
    /// Positions by state, planned and abolished included.
    pub by_state: BTreeMap<String, usize>,
}

fn add(total: Fte, next: Fte) -> Fte {
    match (total, next) {
        (Fte::Withheld, _) | (_, Fte::Withheld) => Fte::Withheld,
        (Fte::Unknown, _) | (_, Fte::Unknown) => Fte::Unknown,
        (Fte::Known { percent: a }, Fte::Known { percent: b }) => Fte::Known { percent: a + b },
    }
}

/// The establishment per cost centre. `floor` is the smallest number of positions a centre may have before its held, vacant and
/// over figures are shown; `None` shows them all (the caller who may see who holds what). A position count and the budget carry
/// no holder and are always shown.
#[must_use]
pub fn establishment(seats: &[Seat], floor: Option<usize>) -> Vec<CostCentre> {
    let mut centres: BTreeMap<&str, Vec<&Seat>> = BTreeMap::new();
    for s in seats {
        centres.entry(s.cost_centre.as_str()).or_default().push(s);
    }
    centres
        .into_iter()
        .map(|(name, all)| {
            let counted: Vec<&&Seat> = all.iter().filter(|s| in_establishment(&s.state)).collect();
            let mut by_state = BTreeMap::new();
            for s in &all {
                *by_state.entry(s.state.clone()).or_insert(0) += 1;
            }
            let zero = Fte::Known { percent: 0 };
            let (mut budgeted, mut held, mut vacant, mut over, mut frozen_vacant) = (zero, zero, zero, zero, zero);
            for s in &counted {
                let g = gap(s);
                budgeted = add(budgeted, Fte::Known { percent: i64::from(s.budget_fte_percent) });
                held = add(held, match s.holding {
                    Holding::Vacant => zero,
                    Holding::Held(f) => Fte::Known { percent: i64::from(f) },
                    Holding::HeldUnknown => Fte::Unknown,
                });
                vacant = add(vacant, g.vacant);
                over = add(over, g.over);
                if s.state == "frozen" {
                    frozen_vacant = add(frozen_vacant, g.vacant);
                }
            }
            let small = floor.is_some_and(|f| !counted.is_empty() && counted.len() < f);
            let hide = |v: Fte| if small { Fte::Withheld } else { v };
            CostCentre {
                cost_centre: name.to_string(),
                positions: counted.len(),
                budgeted,
                held: hide(held),
                vacant: hide(vacant),
                over: hide(over),
                frozen_vacant: hide(frozen_vacant),
                by_state,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOTHING: Facts = Facts { has_requisition: false, has_holder: false };

    fn seat(cc: &str, state: &str, budget: i32, holding: Holding) -> Seat {
        Seat { cost_centre: cc.into(), state: state.into(), budget_fte_percent: budget, holding }
    }

    #[test]
    fn every_move_uses_the_states_and_abolished_is_final() {
        for (from, to) in MOVES {
            assert!(STATES.contains(from) && STATES.contains(to), "{from} {to}");
            assert_ne!(from, to);
        }
        assert!(MOVES.iter().all(|(from, _)| *from != "abolished"));
        for s in STATES {
            assert!(MOVES.iter().any(|(_, to)| to == s) || *s == "planned", "{s} can be reached");
        }
        let mut seen = std::collections::BTreeSet::new();
        assert!(MOVES.iter().all(|m| seen.insert(*m)), "no move is listed twice");
    }

    #[test]
    fn a_refused_move_names_the_state_it_is_in() {
        let e = check_move("planned", "filled", NOTHING).unwrap_err();
        assert!(e.contains("planned") && e.contains("filled"), "{e}");
        assert!(check_move("abolished", "approved", NOTHING).is_err());
        assert!(check_move("planned", "approved", NOTHING).is_ok());
        assert!(check_move("approved", "frozen", NOTHING).is_ok());
        assert!(check_move("frozen", "approved", NOTHING).is_ok());
    }

    #[test]
    fn open_needs_a_requisition_and_filled_needs_a_holder() {
        let req = Facts { has_requisition: true, has_holder: false };
        let held = Facts { has_requisition: true, has_holder: true };
        assert!(check_move("approved", "open", NOTHING).unwrap_err().contains("requisition"));
        assert!(check_move("approved", "open", req).is_ok());
        assert!(check_move("open", "filled", req).unwrap_err().contains("holder"));
        assert!(check_move("open", "filled", held).is_ok());
    }

    #[test]
    fn a_held_seat_is_not_made_vacant_or_abolished_until_the_holder_is_unlinked() {
        let held = Facts { has_requisition: true, has_holder: true };
        let left = Facts { has_requisition: true, has_holder: false };
        assert!(check_move("filled", "approved", held).is_err());
        assert!(check_move("filled", "open", held).is_err());
        assert!(check_move("filled", "open", left).is_ok());
        assert!(check_move("filled", "abolished", held).unwrap_err().contains("worker"));
        assert!(check_move("filled", "abolished", left).is_ok());
        assert!(check_move("filled", "frozen", held).is_ok(), "a frozen seat may keep its holder");
    }

    #[test]
    fn a_position_is_one_seat_with_a_funding_that_makes_sense() {
        let thing = "thing:11111111-2222-4333-8444-555555555555";
        let end = NaiveDate::from_ymd_opt(2027, 3, 31);
        assert!(validate("CC-100", 100, "core", None, None).is_ok());
        assert!(validate("CC-100", 50, "time_limited", end, None).is_ok());
        assert!(validate("CC-100", 50, "programme", None, Some(thing)).is_ok());
        let bad = |r: Result<(), Vec<String>>| r.unwrap_err().join("; ");
        assert!(bad(validate("", 100, "core", None, None)).contains("cost_centre"));
        assert!(bad(validate("CC", 0, "core", None, None)).contains("budget_fte_percent"));
        assert!(bad(validate("CC", 101, "core", None, None)).contains("one seat"));
        assert!(bad(validate("CC", 50, "time_limited", None, None)).contains("date it ends"));
        assert!(bad(validate("CC", 50, "external", None, None)).contains("date it ends"));
        assert!(bad(validate("CC", 50, "programme", None, None)).contains("names the programme"));
        assert!(bad(validate("CC", 50, "programme", None, Some("programme-7"))).contains("thing:<uuid>"));
        assert!(bad(validate("CC", 50, "core", None, Some(thing))).contains("only for a programme"));
        assert!(bad(validate("CC", 50, "mystery", None, None)).contains("kind"));
    }

    #[test]
    fn vacancy_and_over_are_arithmetic_on_one_position() {
        let g = |s: &Seat| gap(s);
        assert_eq!(g(&seat("A", "open", 80, Holding::Vacant)), Gap { vacant: Fte::Known { percent: 80 }, over: Fte::Known { percent: 0 } });
        assert_eq!(g(&seat("A", "filled", 80, Holding::Held(50))), Gap { vacant: Fte::Known { percent: 30 }, over: Fte::Known { percent: 0 } });
        assert_eq!(g(&seat("A", "filled", 50, Holding::Held(100))), Gap { vacant: Fte::Known { percent: 0 }, over: Fte::Known { percent: 50 } });
        assert_eq!(g(&seat("A", "filled", 100, Holding::Held(100))), Gap { vacant: Fte::Known { percent: 0 }, over: Fte::Known { percent: 0 } });
    }

    #[test]
    fn a_holder_with_no_fte_recorded_is_an_unknown_vacancy_not_the_whole_budget() {
        let g = gap(&seat("A", "filled", 100, Holding::HeldUnknown));
        assert_eq!(g, Gap { vacant: Fte::Unknown, over: Fte::Unknown });
    }

    #[test]
    fn an_over_establishment_is_never_netted_against_a_vacancy_elsewhere() {
        let seats = vec![
            seat("A", "filled", 50, Holding::Held(100)), // 50 over
            seat("A", "open", 100, Holding::Vacant),     // 100 vacant
        ];
        let a = &establishment(&seats, None)[0];
        assert_eq!(a.over, Fte::Known { percent: 50 });
        assert_eq!(a.vacant, Fte::Known { percent: 100 }, "not 50");
        assert_eq!(a.budgeted, Fte::Known { percent: 150 });
        assert_eq!(a.held, Fte::Known { percent: 100 });
    }

    #[test]
    fn only_approved_or_later_positions_make_the_establishment() {
        let seats = vec![
            seat("A", "planned", 100, Holding::Vacant),
            seat("A", "approved", 100, Holding::Vacant),
            seat("A", "abolished", 100, Holding::Vacant),
            seat("A", "frozen", 50, Holding::Vacant),
        ];
        let a = &establishment(&seats, None)[0];
        assert_eq!(a.positions, 2);
        assert_eq!(a.budgeted, Fte::Known { percent: 150 });
        assert_eq!(a.frozen_vacant, Fte::Known { percent: 50 }, "not to be filled for now");
        assert_eq!(a.by_state["planned"], 1);
        assert_eq!(a.by_state["abolished"], 1);
    }

    #[test]
    fn one_unknown_holder_makes_the_centre_unknown_in_what_it_touches() {
        let seats = vec![seat("A", "filled", 100, Holding::HeldUnknown), seat("A", "open", 100, Holding::Vacant)];
        let a = &establishment(&seats, None)[0];
        assert_eq!(a.budgeted, Fte::Known { percent: 200 }, "the budget does not depend on the holder");
        assert_eq!(a.held, Fte::Unknown);
        assert_eq!(a.vacant, Fte::Unknown);
        assert_eq!(a.over, Fte::Unknown);
    }

    #[test]
    fn a_small_centre_shows_no_holder_derived_figure_to_a_caller_under_the_floor() {
        let seats = vec![seat("A", "filled", 100, Holding::Held(60)), seat("B", "open", 100, Holding::Vacant)];
        let shown = establishment(&seats, Some(2));
        for c in &shown {
            assert_eq!(c.positions, 1);
            assert_eq!(c.budgeted, Fte::Known { percent: 100 }, "the budget names no one");
            assert_eq!((c.held, c.vacant, c.over), (Fte::Withheld, Fte::Withheld, Fte::Withheld));
        }
        let hr = establishment(&seats, None);
        assert_eq!(hr[0].held, Fte::Known { percent: 60 });
        let big = establishment(&seats, Some(1));
        assert_eq!(big[0].held, Fte::Known { percent: 60 });
    }

    #[test]
    fn a_centre_with_nothing_in_the_establishment_is_not_withheld_but_empty() {
        let seats = vec![seat("A", "planned", 100, Holding::Vacant)];
        let a = &establishment(&seats, Some(5))[0];
        assert_eq!(a.positions, 0);
        assert_eq!(a.held, Fte::Known { percent: 0 });
    }

    #[test]
    fn centres_come_out_in_order_and_the_output_names_no_one() {
        let seats = vec![seat("B", "open", 100, Holding::Vacant), seat("A", "open", 100, Holding::Vacant)];
        let all = establishment(&seats, None);
        assert_eq!(all.iter().map(|c| c.cost_centre.as_str()).collect::<Vec<_>>(), ["A", "B"]);
        let text = serde_json::to_string(&all).unwrap();
        assert!(!text.contains("worker") && !text.contains("holder"), "{text}");
    }
}
