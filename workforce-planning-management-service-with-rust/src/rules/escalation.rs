//! Escalations of a breached tolerance (WPM-R141, WPM-D94): a person takes a breach to someone who
//! can decide; the measure is stored at each step.
//!
//! Pure: what may be raised, the three levels, the decisions, and the counts the tolerances view shows.

use chrono::NaiveDate;
use serde::Serialize;

use crate::rules::tolerance::Rag;

/// The longest note or decision note, in characters.
pub const MAX_NOTE: usize = 500;

/// How far up an escalation has gone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    /// Level 1: whoever owns the plan.
    PlanOwner,
    /// Level 2: the body the plan owner answers to.
    Board,
    /// Level 3: the top. There is nowhere further.
    Executive,
}

impl Level {
    /// The number stored: 1 to 3.
    #[must_use]
    pub const fn number(self) -> i32 {
        match self {
            Self::PlanOwner => 1,
            Self::Board => 2,
            Self::Executive => 3,
        }
    }

    /// From the stored number.
    #[must_use]
    pub const fn from_number(n: i32) -> Option<Self> {
        match n {
            1 => Some(Self::PlanOwner),
            2 => Some(Self::Board),
            3 => Some(Self::Executive),
            _ => None,
        }
    }

    /// One level up, or `None` at the top.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        Self::from_number(self.number() + 1)
    }
}

/// What the person who decides says happens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    /// The deviation is accepted as it is.
    AcceptDeviation,
    /// The plan is to be changed (by someone, separately).
    ChangePlan,
    /// The plan is to stop.
    Stop,
}

impl Decision {
    /// Every decision.
    pub const ALL: [Self; 3] = [Self::AcceptDeviation, Self::ChangePlan, Self::Stop];

    /// The token used in payloads and storage.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AcceptDeviation => "accept_deviation",
            Self::ChangePlan => "change_plan",
            Self::Stop => "stop",
        }
    }

    /// Parse a token.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|d| d.as_str() == text)
    }
}

/// Whether a dimension in this state may be escalated: amber (forecast to breach) or red (breached).
///
/// # Errors
///
/// A message naming the status when it is anything else.
pub fn may_raise(rag: Rag) -> Result<(), String> {
    match rag {
        Rag::Amber | Rag::Red => Ok(()),
        Rag::Green => Err("this dimension is green: there is nothing to escalate".to_string()),
        Rag::Unknown => Err(
            "this dimension is unknown: it cannot be shown to be near its tolerance".to_string(),
        ),
        Rag::NotSet => Err("this dimension has no tolerance: set one first".to_string()),
    }
}

/// A note trimmed and checked. `required` makes an empty one an error; otherwise an empty one is `None`.
///
/// # Errors
///
/// When it is required and empty, or longer than [`MAX_NOTE`].
pub fn clean_note(note: Option<&str>, required: bool) -> Result<Option<String>, String> {
    let text = note.map(str::trim).filter(|t| !t.is_empty());
    match text {
        None if required => Err("a note is required".to_string()),
        None => Ok(None),
        Some(t) if t.chars().count() > MAX_NOTE => {
            Err(format!("the note is longer than {MAX_NOTE} characters"))
        }
        Some(t) => Ok(Some(t.to_string())),
    }
}

/// Calendar days from `raised_on` to `today`, never negative.
#[must_use]
pub fn days_open(raised_on: NaiveDate, today: NaiveDate) -> i64 {
    (today - raised_on).num_days().max(0)
}

/// Red dimensions with a tolerance that no open escalation covers: breaches nobody has been asked about.
#[must_use]
pub fn breaches_without_escalation(dimensions: &[(Rag, bool)]) -> usize {
    dimensions
        .iter()
        .filter(|(rag, has_open)| *rag == Rag::Red && !has_open)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_go_up_one_at_a_time_and_stop_at_the_top() {
        assert_eq!(Level::PlanOwner.next(), Some(Level::Board));
        assert_eq!(Level::Board.next(), Some(Level::Executive));
        assert_eq!(Level::Executive.next(), None);
        for n in 1..=3 {
            assert_eq!(Level::from_number(n).unwrap().number(), n);
        }
        assert_eq!(Level::from_number(0), None);
        assert_eq!(Level::from_number(4), None);
        assert!(Level::PlanOwner < Level::Executive);
    }

    #[test]
    fn only_amber_and_red_may_be_raised_and_the_refusal_says_why() {
        assert!(may_raise(Rag::Amber).is_ok());
        assert!(may_raise(Rag::Red).is_ok());
        assert!(may_raise(Rag::Green).unwrap_err().contains("green"));
        assert!(may_raise(Rag::Unknown).unwrap_err().contains("unknown"));
        assert!(may_raise(Rag::NotSet).unwrap_err().contains("no tolerance"));
    }

    #[test]
    fn a_decision_round_trips_and_an_unknown_one_is_refused() {
        for d in Decision::ALL {
            assert_eq!(Decision::parse(d.as_str()), Some(d));
        }
        assert_eq!(Decision::parse("ignore"), None);
    }

    #[test]
    fn a_note_is_trimmed_capped_and_required_when_asked() {
        assert_eq!(clean_note(None, false), Ok(None));
        assert_eq!(clean_note(Some("  "), false), Ok(None));
        assert_eq!(clean_note(Some(" why "), true), Ok(Some("why".to_string())));
        assert!(clean_note(None, true).is_err());
        assert!(clean_note(Some("   "), true).is_err());
        let longest = "x".repeat(MAX_NOTE);
        assert!(clean_note(Some(&longest), true).is_ok());
        assert!(clean_note(Some(&format!("{longest}x")), true).is_err());
        // characters, not bytes
        assert!(clean_note(Some(&"é".repeat(MAX_NOTE)), true).is_ok());
    }

    #[test]
    fn days_open_counts_calendar_days_and_is_never_negative() {
        let d = |m, day| NaiveDate::from_ymd_opt(2026, m, day).unwrap();
        assert_eq!(days_open(d(10, 1), d(10, 20)), 19);
        assert_eq!(days_open(d(10, 20), d(10, 20)), 0);
        assert_eq!(days_open(d(10, 21), d(10, 20)), 0);
    }

    #[test]
    fn a_breach_without_escalation_is_a_red_nobody_has_been_asked_about() {
        let dims = [
            (Rag::Red, false),
            (Rag::Red, true),
            (Rag::Amber, false),
            (Rag::Unknown, false),
            (Rag::NotSet, false),
            (Rag::Red, false),
        ];
        assert_eq!(breaches_without_escalation(&dims), 2);
        assert_eq!(breaches_without_escalation(&[]), 0);
    }
}
