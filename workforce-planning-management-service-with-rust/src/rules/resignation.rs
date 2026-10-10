//! **Resignations** (WPM-R127, WPM-D77): pure rules for a worker logging an intent to resign and a
//! person accepting it.
//!
//! The worker records the date, a proposed last day, and optionally a reason from a closed list.
//! The earliest last day is the date logged plus the notice the deployer configures, counted in
//! **calendar days**. Nothing here ends anyone's employment: accepting a resignation records the
//! agreed last day and opens the leaver process; a person still ends the employment.

use chrono::{Days, NaiveDate};
use std::collections::BTreeMap;

/// The notice period when none is configured, in calendar days.
pub const DEFAULT_NOTICE_CALENDAR_DAYS: i64 = 28;
/// The longest notice that can be configured, in calendar days.
pub const MAX_NOTICE_CALENDAR_DAYS: i64 = 366;

/// Why a worker chose to resign, as they choose to say. Optional, closed, and never free text.
pub const REASONS: &[&str] = &[
    "new_role",
    "career_change",
    "relocation",
    "retirement",
    "study_or_travel",
    "family_or_caring",
    "working_pattern",
    "pay_or_benefits",
    "management_or_culture",
    "other",
    "prefer_not_to_say",
];

/// Where a resignation stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Logged by the worker, awaiting acceptance.
    Logged,
    /// Taken back by the worker before it was accepted.
    Withdrawn,
    /// Accepted by a person, with an agreed last day.
    Accepted,
    /// Accepted, and later set aside by agreement (a conversation, recorded by a person).
    Rescinded,
}

impl Status {
    /// Parse a stored token.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "logged" => Some(Self::Logged),
            "withdrawn" => Some(Self::Withdrawn),
            "accepted" => Some(Self::Accepted),
            "rescinded" => Some(Self::Rescinded),
            _ => None,
        }
    }

    /// The token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Logged => "logged",
            Self::Withdrawn => "withdrawn",
            Self::Accepted => "accepted",
            Self::Rescinded => "rescinded",
        }
    }
}

/// Who does something to a resignation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// The worker takes it back.
    Withdraw,
    /// A person accepts it.
    Accept,
    /// A person sets an accepted resignation aside.
    Rescind,
}

/// The state after an action.
///
/// # Errors
///
/// An action the current state does not allow, saying why. In particular a worker cannot withdraw
/// once it is accepted: that is a conversation with a person, who records it as rescinded.
pub fn next(status: Status, action: Action) -> Result<Status, String> {
    match (status, action) {
        (Status::Logged, Action::Withdraw) => Ok(Status::Withdrawn),
        (Status::Logged, Action::Accept) => Ok(Status::Accepted),
        (Status::Accepted, Action::Rescind) => Ok(Status::Rescinded),
        (Status::Accepted, Action::Withdraw) => {
            Err("it is already accepted; talk to HR, who can record it as rescinded".to_string())
        }
        (s, Action::Withdraw) => Err(format!("a {} resignation cannot be withdrawn", s.as_str())),
        (s, Action::Accept) => Err(format!("a {} resignation cannot be accepted", s.as_str())),
        (s, Action::Rescind) => Err(format!(
            "only an accepted resignation can be rescinded, not a {} one",
            s.as_str()
        )),
    }
}

/// A configured notice period in calendar days: `0..=366`.
///
/// # Errors
///
/// A value outside that range.
pub fn validate_notice(days: i64) -> Result<(), String> {
    if (0..=MAX_NOTICE_CALENDAR_DAYS).contains(&days) {
        Ok(())
    } else {
        Err(format!(
            "the notice period is 0 to {MAX_NOTICE_CALENDAR_DAYS} calendar days, not {days}"
        ))
    }
}

/// The earliest last day: the day it is logged plus the notice, in calendar days.
#[must_use]
pub fn earliest_last_day(logged_on: NaiveDate, notice_calendar_days: i64) -> Option<NaiveDate> {
    logged_on.checked_add_days(Days::new(u64::try_from(notice_calendar_days).ok()?))
}

/// The worker's proposed last day must allow the notice.
///
/// # Errors
///
/// A day earlier than the notice allows (HR can agree an earlier one when accepting).
pub fn validate_proposal(
    logged_on: NaiveDate,
    proposed_last_day: NaiveDate,
    notice_calendar_days: i64,
) -> Result<(), String> {
    let earliest = earliest_last_day(logged_on, notice_calendar_days)
        .ok_or_else(|| "the notice period runs past the supported calendar".to_string())?;
    if proposed_last_day >= earliest {
        Ok(())
    } else {
        Err(format!(
            "the notice is {notice_calendar_days} calendar days, so the earliest last day is {earliest}; \
             ask HR if you want to leave sooner"
        ))
    }
}

/// The last day a person agrees: not before the day it was logged, and within a year of it.
///
/// # Errors
///
/// A day outside that range.
pub fn validate_agreed(logged_on: NaiveDate, agreed: NaiveDate) -> Result<(), String> {
    let latest = logged_on
        .checked_add_days(Days::new(366))
        .ok_or_else(|| "outside the supported calendar".to_string())?;
    if agreed < logged_on {
        Err(format!(
            "the agreed last day cannot be before it was logged ({logged_on})"
        ))
    } else if agreed > latest {
        Err(format!(
            "the agreed last day cannot be more than 366 calendar days after it was logged ({latest})"
        ))
    } else {
        Ok(())
    }
}

/// A reason, if given, must be on the list.
///
/// # Errors
///
/// A reason that is not on the list.
pub fn validate_reason(reason: Option<&str>) -> Result<(), String> {
    match reason {
        None => Ok(()),
        Some(r) if REASONS.contains(&r) => Ok(()),
        Some(_) => Err(format!("reason must be one of {}", REASONS.join(", "))),
    }
}

/// One department's resignations in a period, as an aggregate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepartmentSummary {
    /// The department.
    pub department: String,
    /// How many; `None` when the group is below the floor and so withheld.
    pub count: Option<usize>,
    /// How many gave each reason (`not_given` where none); `None` unless the department and
    /// **every** reason cell reach the floor, so a withheld cell cannot be worked out from the others.
    pub reasons: Option<BTreeMap<String, usize>>,
}

/// Aggregate resignations by department and reason, withholding small groups. `rows` is
/// `(department, reason)` for each resignation counted.
#[must_use]
pub fn summarise(rows: &[(String, Option<String>)], floor: usize) -> Vec<DepartmentSummary> {
    let mut by: BTreeMap<&str, BTreeMap<String, usize>> = BTreeMap::new();
    for (department, reason) in rows {
        *by.entry(department)
            .or_default()
            .entry(reason.clone().unwrap_or_else(|| "not_given".to_string()))
            .or_insert(0) += 1;
    }
    by.into_iter()
        .map(|(department, cells)| {
            let total: usize = cells.values().sum();
            let shown = total >= floor;
            let reasons_ok = shown && cells.values().all(|c| *c >= floor);
            DepartmentSummary {
                department: department.to_string(),
                count: shown.then_some(total),
                reasons: reasons_ok.then_some(cells),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn the_states_move_only_as_allowed() {
        use Action::{Accept, Rescind, Withdraw};
        use Status::{Accepted, Logged, Rescinded, Withdrawn};
        assert_eq!(next(Logged, Withdraw), Ok(Withdrawn));
        assert_eq!(next(Logged, Accept), Ok(Accepted));
        assert_eq!(next(Accepted, Rescind), Ok(Rescinded));
        // The worker cannot take back what is accepted: it is a conversation.
        assert!(next(Accepted, Withdraw).unwrap_err().contains("talk to HR"));
        for (s, a) in [
            (Withdrawn, Withdraw),
            (Withdrawn, Accept),
            (Withdrawn, Rescind),
            (Rescinded, Withdraw),
            (Rescinded, Accept),
            (Rescinded, Rescind),
            (Accepted, Accept),
            (Logged, Rescind),
        ] {
            assert!(next(s, a).is_err(), "{s:?} {a:?}");
        }
    }

    #[test]
    fn tokens_round_trip() {
        for s in [
            Status::Logged,
            Status::Withdrawn,
            Status::Accepted,
            Status::Rescinded,
        ] {
            assert_eq!(Status::parse(s.as_str()), Some(s));
        }
        assert_eq!(Status::parse("declined"), None);
    }

    #[test]
    fn the_notice_is_counted_in_calendar_days() {
        let logged = d(2026, 6, 1);
        assert_eq!(earliest_last_day(logged, 28), Some(d(2026, 6, 29)));
        assert_eq!(earliest_last_day(logged, 0), Some(logged));
        // Across a month and a year end.
        assert_eq!(earliest_last_day(d(2026, 12, 20), 28), Some(d(2027, 1, 17)));
        assert!(validate_notice(0).is_ok() && validate_notice(366).is_ok());
        assert!(validate_notice(-1).is_err() && validate_notice(367).is_err());
    }

    #[test]
    fn a_proposal_must_allow_the_notice() {
        let logged = d(2026, 6, 1);
        assert!(
            validate_proposal(logged, d(2026, 6, 29), 28).is_ok(),
            "exactly the notice"
        );
        assert!(
            validate_proposal(logged, d(2026, 6, 28), 28).is_err(),
            "a day short"
        );
        assert!(validate_proposal(logged, d(2027, 1, 1), 28).is_ok());
        let msg = validate_proposal(logged, d(2026, 6, 10), 28).unwrap_err();
        assert!(msg.contains("2026-06-29") && msg.contains("ask HR"));
    }

    #[test]
    fn the_agreed_day_is_bounded_by_the_day_it_was_logged() {
        let logged = d(2026, 6, 1);
        assert!(
            validate_agreed(logged, logged).is_ok(),
            "may be agreed for the same day"
        );
        assert!(validate_agreed(logged, d(2026, 5, 31)).is_err());
        assert!(
            validate_agreed(logged, d(2027, 6, 2)).is_ok(),
            "366 calendar days on"
        );
        assert!(validate_agreed(logged, d(2027, 6, 3)).is_err());
    }

    #[test]
    fn a_reason_is_optional_and_closed() {
        assert!(validate_reason(None).is_ok());
        for r in REASONS {
            assert!(validate_reason(Some(r)).is_ok(), "{r}");
        }
        assert!(validate_reason(Some("my manager shouts")).is_err());
        assert!(validate_reason(Some("")).is_err());
    }

    #[test]
    fn small_groups_are_withheld_and_a_hidden_cell_cannot_be_worked_out() {
        let row =
            |dept: &str, reason: Option<&str>| (dept.to_string(), reason.map(ToString::to_string));
        let mut rows = Vec::new();
        // Engineering: 6 resignations, three reasons of 2 each: the total shows, the split does not.
        for r in [
            "new_role",
            "new_role",
            "pay_or_benefits",
            "pay_or_benefits",
            "other",
            "other",
        ] {
            rows.push(row("engineering", Some(r)));
        }
        // Finance: 3, below a floor of 5: nothing shows.
        for _ in 0..3 {
            rows.push(row("finance", Some("new_role")));
        }
        // Legal: 5 giving the same reason: total and split both show.
        for _ in 0..5 {
            rows.push(row("legal", Some("relocation")));
        }
        // Support: 5 of which 4 gave a reason and one gave none: the split is withheld.
        for _ in 0..4 {
            rows.push(row("support", Some("new_role")));
        }
        rows.push(row("support", None));
        let out = summarise(&rows, 5);
        let find = |d: &str| out.iter().find(|s| s.department == d).unwrap();
        assert_eq!(find("engineering").count, Some(6));
        assert!(
            find("engineering").reasons.is_none(),
            "cells of 2 are below the floor"
        );
        assert_eq!(find("finance").count, None);
        assert!(find("finance").reasons.is_none());
        assert_eq!(find("legal").count, Some(5));
        assert_eq!(find("legal").reasons.as_ref().unwrap()["relocation"], 5);
        assert_eq!(find("support").count, Some(5));
        assert!(
            find("support").reasons.is_none(),
            "a cell of 1 would let the other be inferred"
        );
        assert_eq!(summarise(&[], 5).len(), 0);
    }
}
