//! Pure rules for **joiners and leavers**: a movement record (someone joining
//! or leaving), its dated checklist, and — for a leaver — what must be handed
//! over before it can be closed.
//!
//! A checklist is built from a standard template with each item dated
//! relative to the effective day (the start day for a joiner, the **last day**
//! for a leaver), so "two days before" is a real date a person can be asked
//! about and be late on. A leaver cannot be completed while checklist items
//! are open or anything they hold is still unassigned.
//!
//! DB-free and clock-free (the day is always supplied).

use chrono::{Duration, NaiveDate};
use serde::Serialize;
use uuid::Uuid;

/// Movement kinds.
pub const KINDS: &[&str] = &["joiner", "leaver"];

/// Movement statuses.
pub const STATUSES: &[&str] = &["open", "completed", "cancelled"];

/// Why someone is leaving.
pub const LEAVER_REASONS: &[&str] = &[
    "resignation",
    "redundancy",
    "retirement",
    "end_of_contract",
    "dismissal",
    "other",
];

/// One template line: days from the effective day (negative = before), the
/// task, and its category.
type Line = (i64, &'static str, &'static str);

const JOINER: &[Line] = &[
    (-7, "Contract signed and right-to-work checked", "admin"),
    (-5, "Equipment ordered", "equipment"),
    (-2, "Accounts and access created", "access"),
    (0, "Welcome and first-day schedule agreed", "people"),
    (0, "Buddy assigned", "people"),
    (5, "First-week check-in with the manager", "people"),
    (30, "30-day review held", "people"),
];

const LEAVER: &[Line] = &[
    (-28, "Notice acknowledged and last day agreed", "admin"),
    (-14, "Handover plan agreed with the manager", "handover"),
    (-7, "Knowledge-transfer sessions held", "handover"),
    (-3, "Exit interview offered", "people"),
    (0, "Equipment returned", "equipment"),
    (0, "Access revoked", "access"),
    (0, "Tasks, bookings and ownerships reassigned", "handover"),
    (5, "Final pay and benefits confirmed", "admin"),
];

/// One checklist item as planned from the template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Planned {
    /// Order in the list.
    pub position: i32,
    /// What to do.
    pub title: String,
    /// `admin`, `equipment`, `access`, `people` or `handover`.
    pub category: String,
    /// The date it is due.
    pub due_on: NaiveDate,
}

/// Whether `kind` is a movement kind.
#[must_use]
pub fn valid_kind(kind: &str) -> bool {
    KINDS.contains(&kind)
}

/// The dated checklist for `kind` around `effective` (start day for a
/// joiner, last day for a leaver).
#[must_use]
pub fn plan(kind: &str, effective: NaiveDate) -> Vec<Planned> {
    let lines = if kind == "leaver" { LEAVER } else { JOINER };
    lines
        .iter()
        .enumerate()
        .map(|(i, (offset, title, category))| Planned {
            position: i32::try_from(i).unwrap_or(i32::MAX),
            title: (*title).to_string(),
            category: (*category).to_string(),
            due_on: effective + Duration::days(*offset),
        })
        .collect()
}

/// Validate a movement's shape.
///
/// # Errors
///
/// A message when the kind is unknown, a leaver's reason is missing or
/// unknown, a joiner has a reason, or a leaver's last day is before the day
/// they were hired.
pub fn validate(
    kind: &str,
    reason: Option<&str>,
    effective: NaiveDate,
    hired_on: NaiveDate,
) -> Result<(), String> {
    if !valid_kind(kind) {
        return Err(format!("kind must be one of {}", KINDS.join(", ")));
    }
    match (kind, reason) {
        ("leaver", Some(r)) if LEAVER_REASONS.contains(&r) => {}
        ("leaver", _) => {
            return Err(format!(
                "a leaver needs a reason: one of {}",
                LEAVER_REASONS.join(", ")
            ));
        }
        (_, Some(_)) => return Err("only a leaver has a reason".to_string()),
        _ => {}
    }
    if kind == "leaver" && effective < hired_on {
        return Err("the last day cannot be before the hire date".to_string());
    }
    Ok(())
}

/// Where a checklist item stands on `today`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemState {
    /// Done.
    Done,
    /// Deliberately skipped, with a reason.
    Skipped,
    /// Not done, and its day has passed.
    Overdue,
    /// Not done, due today.
    DueToday,
    /// Not done, due later.
    Upcoming,
}

/// An item's state: done and skipped win; otherwise by its due date.
#[must_use]
pub fn item_state(due_on: NaiveDate, done: bool, skipped: bool, today: NaiveDate) -> ItemState {
    if done {
        ItemState::Done
    } else if skipped {
        ItemState::Skipped
    } else if due_on < today {
        ItemState::Overdue
    } else if due_on == today {
        ItemState::DueToday
    } else {
        ItemState::Upcoming
    }
}

/// Progress through a checklist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Progress {
    /// Items done or skipped.
    pub closed: usize,
    /// All items.
    pub total: usize,
    /// Items overdue.
    pub overdue: usize,
}

/// Roll item states up.
#[must_use]
pub fn progress(states: &[ItemState]) -> Progress {
    Progress {
        closed: states
            .iter()
            .filter(|s| matches!(s, ItemState::Done | ItemState::Skipped))
            .count(),
        total: states.len(),
        overdue: states.iter().filter(|s| **s == ItemState::Overdue).count(),
    }
}

/// Whether a movement may be completed.
///
/// # Errors
///
/// A message naming what is still open: checklist items, and (for a leaver)
/// things they hold that are not yet reassigned or closed.
pub fn can_complete(open_items: usize, unassigned_handover: usize) -> Result<(), String> {
    let mut problems = Vec::new();
    if open_items > 0 {
        problems.push(format!("{open_items} checklist item(s) still open"));
    }
    if unassigned_handover > 0 {
        problems.push(format!(
            "{unassigned_handover} thing(s) the leaver holds not yet reassigned"
        ));
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

/// What a leaver can hold that must be handed over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoverKind {
    /// Someone's solid-line manager.
    DirectReport,
    /// Someone's dotted-line manager.
    DottedLine,
    /// Lead of a group.
    GroupLead,
    /// A seat in an on-call rota.
    RotaMembership,
    /// An on-call swap (a booking on a rota).
    RotaSwap,
    /// Named as someone's backup.
    Backup,
    /// Mentor in a live mentorship.
    Mentorship,
    /// Booked on a future shift.
    Shift,
    /// A checklist task assigned to them.
    Task,
    /// An organization role (access).
    Access,
}

impl HandoverKind {
    /// Every kind, in the order the inventory lists them.
    pub const ALL: [Self; 10] = [
        Self::DirectReport,
        Self::DottedLine,
        Self::GroupLead,
        Self::RotaMembership,
        Self::RotaSwap,
        Self::Backup,
        Self::Mentorship,
        Self::Shift,
        Self::Task,
        Self::Access,
    ];

    /// The wire name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DirectReport => "direct_report",
            Self::DottedLine => "dotted_line",
            Self::GroupLead => "group_lead",
            Self::RotaMembership => "rota_membership",
            Self::RotaSwap => "rota_swap",
            Self::Backup => "backup",
            Self::Mentorship => "mentorship",
            Self::Shift => "shift",
            Self::Task => "task",
            Self::Access => "access",
        }
    }

    /// Parse a wire name.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == name)
    }

    /// Whether the item needs a new holder. A direct report must have a new
    /// manager (else the org chart is left with an orphan); everything else
    /// can also simply be closed, removed or revoked.
    #[must_use]
    pub fn needs_target(self) -> bool {
        self == Self::DirectReport
    }

    /// Whether the item can be handed to someone else at all. Access is only
    /// ever revoked — a role is not transferred to a person.
    #[must_use]
    pub fn can_reassign(self) -> bool {
        self != Self::Access
    }
}

/// Validate a handover action's target.
///
/// # Errors
///
/// A message when the kind needs a new holder and none is given, when access
/// is handed to someone, or when the leaver is named as their own successor.
pub fn validate_handover(kind: HandoverKind, to: Option<Uuid>, leaver: Uuid) -> Result<(), String> {
    if to == Some(leaver) {
        return Err("the leaver cannot take their own items".to_string());
    }
    match to {
        None if kind.needs_target() => Err("a direct report needs a new manager".to_string()),
        Some(_) if !kind.can_reassign() => {
            Err("access is revoked, not handed to someone".to_string())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, day).unwrap()
    }

    #[test]
    fn a_checklist_is_dated_around_the_effective_day() {
        let leaver = plan("leaver", d(30));
        assert_eq!(leaver.len(), 8);
        assert_eq!(
            leaver[0].due_on,
            NaiveDate::from_ymd_opt(2026, 11, 2).unwrap(),
            "28 days before"
        );
        assert!(
            leaver
                .iter()
                .any(|p| p.title.contains("Access revoked") && p.due_on == d(30))
        );
        assert_eq!(
            leaver.last().unwrap().due_on,
            NaiveDate::from_ymd_opt(2026, 12, 5).unwrap(),
            "5 days after"
        );
        let joiner = plan("joiner", d(3));
        assert_eq!(
            joiner[0].due_on,
            NaiveDate::from_ymd_opt(2026, 10, 27).unwrap()
        );
        assert_eq!(
            joiner.iter().map(|p| p.position).collect::<Vec<_>>(),
            (0..7).collect::<Vec<_>>()
        );
    }

    #[test]
    fn a_movement_is_validated() {
        assert!(validate("leaver", Some("resignation"), d(30), d(1)).is_ok());
        assert!(validate("joiner", None, d(3), d(3)).is_ok());
        assert!(
            validate("leaver", None, d(30), d(1)).is_err(),
            "a leaver needs a reason"
        );
        assert!(validate("leaver", Some("bored"), d(30), d(1)).is_err());
        assert!(validate("joiner", Some("resignation"), d(3), d(3)).is_err());
        assert!(validate("transfer", None, d(3), d(3)).is_err());
        assert!(
            validate("leaver", Some("other"), d(1), d(5)).is_err(),
            "before hire"
        );
    }

    #[test]
    fn items_are_done_skipped_overdue_due_today_or_upcoming() {
        let today = d(10);
        assert_eq!(
            item_state(d(1), true, false, today),
            ItemState::Done,
            "done wins over late"
        );
        assert_eq!(item_state(d(1), false, true, today), ItemState::Skipped);
        assert_eq!(item_state(d(9), false, false, today), ItemState::Overdue);
        assert_eq!(item_state(d(10), false, false, today), ItemState::DueToday);
        assert_eq!(item_state(d(11), false, false, today), ItemState::Upcoming);
        let p = progress(&[
            ItemState::Done,
            ItemState::Skipped,
            ItemState::Overdue,
            ItemState::Upcoming,
        ]);
        assert_eq!((p.closed, p.total, p.overdue), (2, 4, 1));
    }

    #[test]
    fn completion_needs_a_closed_checklist_and_nothing_left_to_hand_over() {
        assert!(can_complete(0, 0).is_ok());
        assert_eq!(
            can_complete(2, 0).unwrap_err(),
            "2 checklist item(s) still open"
        );
        let both = can_complete(1, 3).unwrap_err();
        assert!(both.contains("1 checklist") && both.contains("3 thing(s)"));
    }

    #[test]
    fn handover_kinds_round_trip_and_know_what_they_need() {
        for k in HandoverKind::ALL {
            assert_eq!(HandoverKind::parse(k.as_str()), Some(k));
        }
        assert_eq!(HandoverKind::parse("nope"), None);
        let (leaver, other) = (Uuid::from_u128(1), Uuid::from_u128(2));
        assert!(validate_handover(HandoverKind::DirectReport, Some(other), leaver).is_ok());
        assert!(
            validate_handover(HandoverKind::DirectReport, None, leaver).is_err(),
            "needs a manager"
        );
        assert!(
            validate_handover(HandoverKind::Backup, None, leaver).is_ok(),
            "can just be removed"
        );
        assert!(validate_handover(HandoverKind::Access, None, leaver).is_ok());
        assert!(
            validate_handover(HandoverKind::Access, Some(other), leaver).is_err(),
            "revoked, not handed over"
        );
        assert!(
            validate_handover(HandoverKind::Task, Some(leaver), leaver).is_err(),
            "not to themself"
        );
    }
}
