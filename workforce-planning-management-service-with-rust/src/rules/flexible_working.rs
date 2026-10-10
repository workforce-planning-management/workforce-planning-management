//! **Flexible working requests** (WPM-R126, WPM-D77): pure rules for a worker asking for a
//! different working arrangement and a person deciding.
//!
//! The software computes the dates and keeps the record; it never decides, and it never changes a
//! contract. An approved change of hours is a *proposal* for HR to apply. Whether a request is a
//! statutory right, how long a decision may take and how many requests a worker may make are the
//! **deployer's** to set (a jurisdiction may fix them); the defaults here are starting points a
//! deployer confirms. Periods are calendar months or calendar days, as each says.

use chrono::{Days, Months, NaiveDate};

/// The default time a decision may take, in calendar months, unless configured.
pub const DEFAULT_DECISION_MONTHS: u32 = 2;
/// The default number of requests a worker may make in any 12 calendar months, unless configured.
pub const DEFAULT_REQUESTS_PER_YEAR: u32 = 2;
/// The default time a worker has to appeal a refusal, in calendar days.
pub const DEFAULT_APPEAL_CALENDAR_DAYS: u32 = 14;

/// What is being asked for.
pub const KINDS: &[&str] = &[
    "hours",
    "times",
    "days",
    "place_of_work",
    "job_share",
    "compressed_hours",
    "other",
];

/// Why a request may be refused: a closed list, never only free text. A deployer replaces it to
/// match the grounds their jurisdiction allows.
pub const DEFAULT_REFUSAL_REASONS: &[&str] = &[
    "additional_cost",
    "cannot_meet_demand",
    "cannot_reorganise_work",
    "cannot_recruit",
    "quality_impact",
    "performance_impact",
    "insufficient_work_in_periods",
    "planned_changes",
    "other_recorded",
];

/// Where a request stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Waiting for a decision.
    Requested,
    /// Approved as asked.
    Approved,
    /// The decider offered something different; the worker answers.
    CounterProposed,
    /// The worker accepted the counter-proposal.
    Accepted,
    /// The worker declined the counter-proposal.
    Declined,
    /// Refused, with a reason.
    Refused,
    /// The worker appealed a refusal.
    Appealed,
    /// The appeal was dismissed.
    RefusedOnAppeal,
    /// The worker took it back.
    Withdrawn,
}

impl Status {
    /// Every status.
    pub const ALL: [Self; 9] = [
        Self::Requested,
        Self::Approved,
        Self::CounterProposed,
        Self::Accepted,
        Self::Declined,
        Self::Refused,
        Self::Appealed,
        Self::RefusedOnAppeal,
        Self::Withdrawn,
    ];

    /// The stored token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Approved => "approved",
            Self::CounterProposed => "counter_proposed",
            Self::Accepted => "accepted",
            Self::Declined => "declined",
            Self::Refused => "refused",
            Self::Appealed => "appealed",
            Self::RefusedOnAppeal => "refused_on_appeal",
            Self::Withdrawn => "withdrawn",
        }
    }

    /// Parse a stored token.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.as_str() == text)
    }

    /// Whether this is an arrangement now in force (to be applied by HR where hours change).
    #[must_use]
    pub const fn is_agreed(self) -> bool {
        matches!(self, Self::Approved | Self::Accepted)
    }
}

/// Something done to a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// The decider approves as asked.
    Approve,
    /// The decider refuses.
    Refuse,
    /// The decider offers something different.
    Counter,
    /// The worker accepts the counter-proposal.
    Accept,
    /// The worker declines the counter-proposal.
    Decline,
    /// The worker appeals a refusal.
    Appeal,
    /// The decider upholds the appeal, so the request is approved.
    Uphold,
    /// The decider dismisses the appeal.
    Dismiss,
    /// The worker withdraws.
    Withdraw,
}

/// The state after an action.
///
/// # Errors
///
/// An action the current state does not allow, saying which states do.
pub fn next(status: Status, action: Action) -> Result<Status, String> {
    use Action::{Accept, Appeal, Approve, Counter, Decline, Dismiss, Refuse, Uphold, Withdraw};
    use Status::{
        Accepted, Appealed, Approved, CounterProposed, Declined, Refused, RefusedOnAppeal,
        Requested, Withdrawn,
    };
    match (status, action) {
        (Requested, Approve) | (Appealed, Uphold) => Ok(Approved),
        (Requested, Refuse) => Ok(Refused),
        (Requested, Counter) => Ok(CounterProposed),
        (Requested | CounterProposed, Withdraw) => Ok(Withdrawn),
        (CounterProposed, Accept) => Ok(Accepted),
        (CounterProposed, Decline) => Ok(Declined),
        (Refused, Appeal) => Ok(Appealed),
        (Appealed, Dismiss) => Ok(RefusedOnAppeal),
        (s, a) => Err(format!(
            "a request that is {} cannot be {}",
            s.as_str(),
            action_word(a)
        )),
    }
}

const fn action_word(action: Action) -> &'static str {
    match action {
        Action::Approve => "approved",
        Action::Refuse => "refused",
        Action::Counter => "counter-proposed",
        Action::Accept => "accepted",
        Action::Decline => "declined",
        Action::Appeal => "appealed",
        Action::Uphold => "upheld",
        Action::Dismiss => "dismissed",
        Action::Withdraw => "withdrawn",
    }
}

/// The date by which a decision is due: the day the request was received plus the configured
/// calendar months (a month-end clamps to the last day of the shorter month).
#[must_use]
pub fn decide_by(received_on: NaiveDate, months: u32) -> Option<NaiveDate> {
    received_on.checked_add_months(Months::new(months))
}

/// Whether a request still waiting for a decision is past its decide-by date.
#[must_use]
pub fn is_overdue(status: Status, decide_by: NaiveDate, today: NaiveDate) -> bool {
    matches!(status, Status::Requested | Status::Appealed) && today > decide_by
}

/// Whether a worker may make another request: fewer than `limit` requests (not counting
/// withdrawn ones) in the 12 calendar months before `today`. A `limit` of `0` means no limit.
#[must_use]
pub fn within_limit(previous: &[(NaiveDate, Status)], today: NaiveDate, limit: u32) -> bool {
    if limit == 0 {
        return true;
    }
    let Some(since) = today.checked_sub_months(Months::new(12)) else {
        return true;
    };
    let made = previous
        .iter()
        .filter(|(on, status)| *on > since && *status != Status::Withdrawn)
        .count();
    u32::try_from(made).is_ok_and(|n| n < limit)
}

/// Whether a refusal can still be appealed: within `appeal_calendar_days` of the decision.
#[must_use]
pub fn can_appeal(decided_on: NaiveDate, today: NaiveDate, appeal_calendar_days: u32) -> bool {
    decided_on
        .checked_add_days(Days::new(u64::from(appeal_calendar_days)))
        .is_some_and(|last| today <= last)
}

/// Check what a worker asks for.
///
/// # Errors
///
/// An unknown kind, a start before today, or an FTE outside `1..=100`.
pub fn validate_request(
    kind: &str,
    proposed_start: NaiveDate,
    proposed_fte_percent: Option<i32>,
    today: NaiveDate,
) -> Result<(), String> {
    if !KINDS.contains(&kind) {
        return Err(format!("kind must be one of {}", KINDS.join(", ")));
    }
    if proposed_start < today {
        return Err("the proposed start cannot be in the past".to_string());
    }
    if proposed_fte_percent.is_some_and(|p| !(1..=100).contains(&p)) {
        return Err("proposed_fte_percent must be between 1 and 100".to_string());
    }
    Ok(())
}

/// Check a refusal reason against the allowed list.
///
/// # Errors
///
/// A reason not on the list.
pub fn validate_refusal_reason(reason: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.contains(&reason) {
        Ok(())
    } else {
        Err(format!("reason must be one of {}", allowed.join(", ")))
    }
}

/// Parse a deployer's list of refusal reasons: comma-separated lowercase tokens, none blank.
///
/// # Errors
///
/// An empty list or a token that is not lowercase letters, digits and underscores.
pub fn parse_reasons(text: &str) -> Result<Vec<String>, String> {
    let list: Vec<String> = text
        .split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();
    if list.is_empty() {
        return Err("the list of refusal reasons is empty".to_string());
    }
    if let Some(bad) = list.iter().find(|t| {
        !t.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    }) {
        return Err(format!("`{bad}` is not a lowercase token"));
    }
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn the_state_machine_allows_exactly_these_moves() {
        use Action::*;
        use Status::*;
        let allowed = [
            (Requested, Approve, Approved),
            (Requested, Refuse, Refused),
            (Requested, Counter, CounterProposed),
            (Requested, Withdraw, Withdrawn),
            (CounterProposed, Withdraw, Withdrawn),
            (CounterProposed, Accept, Accepted),
            (CounterProposed, Decline, Declined),
            (Refused, Appeal, Appealed),
            (Appealed, Uphold, Approved),
            (Appealed, Dismiss, RefusedOnAppeal),
        ];
        let actions = [
            Approve, Refuse, Counter, Accept, Decline, Appeal, Uphold, Dismiss, Withdraw,
        ];
        for status in Status::ALL {
            for action in actions {
                let expected = allowed
                    .iter()
                    .find(|(s, a, _)| *s == status && *a == action)
                    .map(|t| t.2);
                match (next(status, action), expected) {
                    (Ok(got), Some(want)) => assert_eq!(got, want, "{status:?} {action:?}"),
                    (Err(_), None) => {}
                    (got, want) => panic!("{status:?} {action:?}: got {got:?}, expected {want:?}"),
                }
            }
        }
    }

    #[test]
    fn terminal_states_move_nowhere() {
        for s in [
            Status::Approved,
            Status::Accepted,
            Status::Declined,
            Status::RefusedOnAppeal,
            Status::Withdrawn,
        ] {
            for a in [
                Action::Approve,
                Action::Refuse,
                Action::Counter,
                Action::Accept,
                Action::Decline,
                Action::Appeal,
                Action::Uphold,
                Action::Dismiss,
                Action::Withdraw,
            ] {
                assert!(next(s, a).is_err(), "{s:?} {a:?}");
            }
        }
        // A refusal can be appealed once: an appeal that is dismissed cannot be appealed again.
        assert!(next(Status::RefusedOnAppeal, Action::Appeal).is_err());
        assert!(
            next(Status::Requested, Action::Appeal).is_err(),
            "only a refusal is appealed"
        );
    }

    #[test]
    fn tokens_round_trip_and_agreed_states_are_named() {
        for s in Status::ALL {
            assert_eq!(Status::parse(s.as_str()), Some(s));
        }
        assert_eq!(Status::parse("pending"), None);
        let agreed: Vec<_> = Status::ALL.into_iter().filter(|s| s.is_agreed()).collect();
        assert_eq!(agreed, vec![Status::Approved, Status::Accepted]);
    }

    #[test]
    fn the_decide_by_date_is_in_calendar_months() {
        assert_eq!(decide_by(d(2026, 6, 15), 2), Some(d(2026, 8, 15)));
        assert_eq!(decide_by(d(2026, 12, 15), 2), Some(d(2027, 2, 15)));
        // A month-end clamps: 31 December plus two months is 28 February.
        assert_eq!(decide_by(d(2026, 12, 31), 2), Some(d(2027, 2, 28)));
        assert_eq!(decide_by(d(2026, 6, 15), 0), Some(d(2026, 6, 15)));
    }

    #[test]
    fn only_a_request_still_waiting_can_be_overdue() {
        let due = d(2026, 8, 15);
        assert!(
            !is_overdue(Status::Requested, due, d(2026, 8, 15)),
            "the due day itself is not late"
        );
        assert!(is_overdue(Status::Requested, due, d(2026, 8, 16)));
        assert!(is_overdue(Status::Appealed, due, d(2026, 8, 16)));
        for s in [
            Status::Approved,
            Status::Refused,
            Status::CounterProposed,
            Status::Withdrawn,
            Status::Declined,
        ] {
            assert!(!is_overdue(s, due, d(2027, 1, 1)), "{s:?}");
        }
    }

    #[test]
    fn the_request_limit_counts_twelve_calendar_months_and_ignores_withdrawn() {
        let today = d(2026, 6, 15);
        let two = [
            (d(2026, 1, 1), Status::Refused),
            (d(2025, 12, 1), Status::Approved),
        ];
        assert!(!within_limit(&two, today, 2), "two in the year: no more");
        assert!(within_limit(&two, today, 3));
        // One is older than 12 months, so only one counts.
        let old = [
            (d(2025, 6, 15), Status::Approved),
            (d(2026, 1, 1), Status::Refused),
        ];
        assert!(
            within_limit(&old, today, 2),
            "a request exactly 12 months ago has aged out"
        );
        // A withdrawn request does not count against the limit.
        let withdrawn = [
            (d(2026, 1, 1), Status::Withdrawn),
            (d(2026, 2, 1), Status::Refused),
        ];
        assert!(within_limit(&withdrawn, today, 2));
        assert!(within_limit(&[], today, 1));
        // Zero is no limit.
        assert!(within_limit(&two, today, 0));
    }

    #[test]
    fn an_appeal_must_come_within_the_window() {
        assert!(
            can_appeal(d(2026, 6, 1), d(2026, 6, 15), 14),
            "the last day"
        );
        assert!(!can_appeal(d(2026, 6, 1), d(2026, 6, 16), 14));
        assert!(can_appeal(d(2026, 6, 1), d(2026, 6, 1), 0));
        assert!(!can_appeal(d(2026, 6, 1), d(2026, 6, 2), 0));
    }

    #[test]
    fn a_request_is_validated() {
        let today = d(2026, 6, 15);
        assert!(validate_request("hours", d(2026, 7, 1), Some(60), today).is_ok());
        assert!(
            validate_request("hours", today, None, today).is_ok(),
            "today is allowed"
        );
        assert!(validate_request("hours", d(2026, 6, 14), None, today).is_err());
        assert!(validate_request("sabbatical", d(2026, 7, 1), None, today).is_err());
        for fte in [0, 101, -5] {
            assert!(
                validate_request("hours", d(2026, 7, 1), Some(fte), today).is_err(),
                "{fte}"
            );
        }
        assert!(validate_request("hours", d(2026, 7, 1), Some(1), today).is_ok());
        assert!(validate_request("hours", d(2026, 7, 1), Some(100), today).is_ok());
    }

    #[test]
    fn refusal_reasons_are_a_closed_list_a_deployer_can_replace() {
        assert!(validate_refusal_reason("additional_cost", DEFAULT_REFUSAL_REASONS).is_ok());
        assert!(validate_refusal_reason("dislike", DEFAULT_REFUSAL_REASONS).is_err());
        assert_eq!(parse_reasons("a_b, c1 ,d").unwrap(), vec!["a_b", "c1", "d"]);
        assert!(parse_reasons("").is_err());
        assert!(parse_reasons(" , ").is_err());
        assert!(parse_reasons("Upper").is_err());
        assert!(parse_reasons("has space").is_err());
        assert!(validate_refusal_reason("x", &["x"]).is_ok());
    }
}
