//! **Engagements** (WPM-R79, WPM-R81, WPM-R85, WPM-R86, WPM-D56, WPM-D57): pure,
//! database-free and clock-free rules about how a worker is engaged.
//!
//! The engagement **basis** (`permanent`, `fixed_term`, `contractor`, `intern`) is one
//! fact; how much someone works (`fte_percent`) is another (WPM-D56). A basis decides
//! whether an end date is needed. An end date is the **last day** of the engagement:
//! the worker is still engaged on that day and not the day after.
//!
//! Nothing here converts, extends or ends an engagement. It reports where each stands
//! so a person can decide (WPM-D66).

use chrono::{Months, NaiveDate};

/// The default end-of-engagement reminder window, in calendar days.
pub const DEFAULT_WINDOW_CALENDAR_DAYS: i64 = 60;
/// The most calendar days a reminder window may be set to.
pub const MAX_WINDOW_CALENDAR_DAYS: i64 = 366;

/// How a worker is engaged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Basis {
    /// An employee with no planned end.
    Permanent,
    /// An employee on a contract that ends on a known date.
    FixedTerm,
    /// Paid against invoices through a supplier, not through payroll (WPM-D58).
    Contractor,
    /// An intern; an end date is allowed but not required.
    Intern,
}

impl Basis {
    /// Every basis, in a stable order.
    pub const ALL: [Self; 4] = [
        Self::Permanent,
        Self::FixedTerm,
        Self::Contractor,
        Self::Intern,
    ];

    /// Parse the stored `employment_type` token.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "permanent" => Some(Self::Permanent),
            "fixed_term" => Some(Self::FixedTerm),
            "contractor" => Some(Self::Contractor),
            "intern" => Some(Self::Intern),
            _ => None,
        }
    }

    /// The stored token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Permanent => "permanent",
            Self::FixedTerm => "fixed_term",
            Self::Contractor => "contractor",
            Self::Intern => "intern",
        }
    }

    /// Whether this basis is contingent (planned to end), as opposed to permanent.
    #[must_use]
    pub const fn is_contingent(self) -> bool {
        !matches!(self, Self::Permanent)
    }
}

/// What a basis says about an end date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndDate {
    /// Must be given.
    Required,
    /// May be given.
    Allowed,
    /// Must not be given.
    Refused,
}

/// The end-date rule for a basis (WPM-R79).
#[must_use]
pub const fn end_date_rule(basis: Basis) -> EndDate {
    match basis {
        Basis::Permanent => EndDate::Refused,
        Basis::FixedTerm | Basis::Contractor => EndDate::Required,
        Basis::Intern => EndDate::Allowed,
    }
}

/// Check an engagement's end date against its basis and start.
///
/// # Errors
///
/// An unknown basis, an end date that is missing where required or given where refused,
/// or one that is not after the start.
pub fn validate_end_date(
    basis: &str,
    hired_on: NaiveDate,
    ends_on: Option<NaiveDate>,
) -> Result<(), String> {
    let basis = Basis::parse(basis).ok_or_else(|| format!("unknown engagement basis: {basis}"))?;
    match (end_date_rule(basis), ends_on) {
        (EndDate::Required, None) => Err(format!(
            "a {} engagement needs an engagement_ends_on date",
            basis.as_str()
        )),
        (EndDate::Refused, Some(_)) => Err(
            "a permanent engagement has no end date; use fixed_term or contractor for one that ends"
                .to_string(),
        ),
        (_, Some(end)) if end <= hired_on => Err(format!(
            "engagement_ends_on {end} must be after the start {hired_on}"
        )),
        _ => Ok(()),
    }
}

/// An extension must move the end **later**.
///
/// # Errors
///
/// A new end that is not after the previous one.
pub fn validate_extension(previous_end: NaiveDate, new_end: NaiveDate) -> Result<(), String> {
    if new_end > previous_end {
        Ok(())
    } else {
        Err(format!(
            "an extension must move the end later than {previous_end}, not to {new_end}"
        ))
    }
}

/// A reminder window in calendar days: `1..=366`.
///
/// # Errors
///
/// A value outside that range.
pub fn validate_window(calendar_days: i64) -> Result<(), String> {
    if (1..=MAX_WINDOW_CALENDAR_DAYS).contains(&calendar_days) {
        Ok(())
    } else {
        Err(format!(
            "the window is 1 to {MAX_WINDOW_CALENDAR_DAYS} calendar days, not {calendar_days}"
        ))
    }
}

/// Ways a contractor is engaged (WPM-R80).
pub const ROUTES: &[&str] = &["agency", "own_company", "statement_of_work", "direct"];
/// What a contractor's rate is quoted per.
pub const RATE_BASES: &[&str] = &["day", "hour"];
/// Outcomes of an employment-status assessment. Whether one is required, and what the
/// outcome means, is the deployer's jurisdiction; the software records it.
pub const STATUS_OUTCOMES: &[&str] = &["contractor", "employee", "undetermined"];

/// Check a contractor's details: a known route, and a rate that is all of amount, currency
/// and basis or none of them.
///
/// # Errors
///
/// An unknown route or basis, a negative amount, a partial rate, or a currency that is not
/// three capital letters.
pub fn validate_contractor_details(
    route: Option<&str>,
    rate_minor: Option<i64>,
    rate_currency: Option<&str>,
    rate_basis: Option<&str>,
) -> Result<(), String> {
    if let Some(route) = route
        && !ROUTES.contains(&route)
    {
        return Err(format!("route must be one of {}", ROUTES.join(", ")));
    }
    match (rate_minor, rate_currency, rate_basis) {
        (None, None, None) => Ok(()),
        (Some(amount), Some(currency), Some(basis)) => {
            if amount < 0 {
                return Err("rate_minor must not be negative".to_string());
            }
            // A code in the ISO 4217 table, so the rate's minor unit is known (WPM-T400).
            if !super::currency::is_known(currency) {
                return Err("rate_currency must be an ISO 4217 code such as GBP".to_string());
            }
            if !RATE_BASES.contains(&basis) {
                return Err(format!(
                    "rate_basis must be one of {}",
                    RATE_BASES.join(", ")
                ));
            }
            Ok(())
        }
        _ => Err("give rate_minor, rate_currency and rate_basis together, or none".to_string()),
    }
}

/// Where an engagement stands on `as_of`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// A permanent engagement, or an intern with no end date: nothing to watch.
    OpenEnded,
    /// A basis that needs an end date has none recorded. Listed until HR records one;
    /// never given an invented date.
    MissingEndDate,
    /// Ends after the reminder window.
    Running,
    /// Ends inside the window, on or after `as_of`.
    EndingSoon,
    /// The end date has passed and no decision is recorded. Never treated as continuing.
    PastEndUndecided,
    /// The end date has passed and a decision is recorded.
    PastEndDecided,
}

impl Standing {
    /// The token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenEnded => "open_ended",
            Self::MissingEndDate => "missing_end_date",
            Self::Running => "running",
            Self::EndingSoon => "ending_soon",
            Self::PastEndUndecided => "past_end_undecided",
            Self::PastEndDecided => "past_end_decided",
        }
    }
}

/// Where an engagement stands. `decided` is whether a decision (extend, convert, end) is
/// recorded for the current end date.
#[must_use]
pub fn standing(
    basis: Basis,
    ends_on: Option<NaiveDate>,
    as_of: NaiveDate,
    window_calendar_days: i64,
    decided: bool,
) -> Standing {
    let Some(end) = ends_on else {
        return match end_date_rule(basis) {
            EndDate::Required => Standing::MissingEndDate,
            _ => Standing::OpenEnded,
        };
    };
    let until = (end - as_of).num_days();
    if until < 0 {
        if decided {
            Standing::PastEndDecided
        } else {
            Standing::PastEndUndecided
        }
    } else if until <= window_calendar_days {
        Standing::EndingSoon
    } else {
        Standing::Running
    }
}

/// What a person decides about an engagement that is ending (WPM-D66): nothing is decided by length
/// of service or by the software.
pub const DECISIONS: &[&str] = &["extend", "convert", "end"];

/// Check a decision.
///
/// # Errors
///
/// A decision that is not on the list.
pub fn validate_decision(decision: &str) -> Result<(), String> {
    if DECISIONS.contains(&decision) {
        Ok(())
    } else {
        Err(format!("decision must be one of {}", DECISIONS.join(", ")))
    }
}

/// Which reminder, if any, an engagement needs on `as_of`: **ending** inside the window, or
/// **ended undecided** once the end date has passed. Whether a decision is already recorded is the
/// caller's to check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reminder {
    /// Ends inside the window, on or after `as_of`.
    Ending,
    /// The end date has passed.
    EndedUndecided,
}

impl Reminder {
    /// The stored token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ending => "ending",
            Self::EndedUndecided => "ended_undecided",
        }
    }
}

/// The reminder an engagement with this end date needs on `as_of`, if any.
#[must_use]
pub fn reminder_for(
    ends_on: NaiveDate,
    as_of: NaiveDate,
    window_calendar_days: i64,
) -> Option<Reminder> {
    if ends_on < as_of {
        Some(Reminder::EndedUndecided)
    } else if in_end_window(ends_on, as_of, window_calendar_days) {
        Some(Reminder::Ending)
    } else {
        None
    }
}

/// Whether an engagement ends inside the window starting `as_of` (a reminder is due).
#[must_use]
pub fn in_end_window(ends_on: NaiveDate, as_of: NaiveDate, window_calendar_days: i64) -> bool {
    let until = (ends_on - as_of).num_days();
    (0..=window_calendar_days).contains(&until)
}

/// Whether an engagement has been extended more times than `threshold` (WPM-R86).
#[must_use]
pub fn much_extended(extensions: usize, threshold: usize) -> bool {
    extensions > threshold
}

/// Whether a contractor has been engaged **longer than** `months` calendar months on
/// `as_of`: a conversion or status-review candidate (WPM-R86). A start that is not yet
/// reached is not long-running.
#[must_use]
pub fn long_running(hired_on: NaiveDate, as_of: NaiveDate, months: u32) -> bool {
    hired_on
        .checked_add_months(Months::new(months))
        .is_some_and(|limit| as_of > limit)
}

/// Whether an engagement is counted on `date`: it started, and has not passed its end
/// date. An engagement with no end date is counted.
#[must_use]
pub fn is_engaged_on(date: NaiveDate, hired_on: NaiveDate, ends_on: Option<NaiveDate>) -> bool {
    hired_on <= date && ends_on.is_none_or(|end| date <= end)
}

/// Extensions beyond this many make an engagement "much extended" in the contingent view.
pub const EXTENSION_THRESHOLD: usize = 2;
/// A contractor engaged longer than this many calendar months is a conversion or review candidate.
pub const LONG_RUNNING_MONTHS: u32 = 24;

/// One contingent engagement as the aggregate sees it. No name, no pay.
#[derive(Debug, Clone, Copy)]
pub struct ContingentEngagement {
    /// A contractor (rather than fixed-term).
    pub contractor: bool,
    /// The start.
    pub hired_on: NaiveDate,
    /// The end date; `None` is counted nowhere in the ending figures (the missing end is its own list).
    pub ends_on: Option<NaiveDate>,
    /// A decision is recorded for the current end date.
    pub decided: bool,
    /// How many times the end has been extended.
    pub extensions: usize,
}

/// The contingent workforce in numbers (WPM-R86). A figure is `None` when it is above zero but below
/// the floor; zero is shown. The ending figures are **disjoint bands**, never cumulative, and no total
/// is given, so a withheld figure cannot be worked out by subtraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContingentCounts {
    /// Ending 0 to 30 calendar days from today.
    pub ending_within_30: Option<usize>,
    /// Ending 31 to 60 calendar days from today.
    pub ending_31_to_60: Option<usize>,
    /// Ending 61 to 90 calendar days from today.
    pub ending_61_to_90: Option<usize>,
    /// The end has passed and no decision is recorded.
    pub past_end_undecided: Option<usize>,
    /// Extended more than [`EXTENSION_THRESHOLD`] times.
    pub much_extended: Option<usize>,
    /// Contractors engaged longer than [`LONG_RUNNING_MONTHS`] calendar months.
    pub long_running_contractors: Option<usize>,
}

/// The lists of WPM-R86, by token. Each is both a count in the aggregate and a named list for HR.
pub const LISTS: &[&str] = &[
    "ending_within_30",
    "ending_31_to_60",
    "ending_61_to_90",
    "past_end_undecided",
    "much_extended",
    "long_running_contractors",
];

/// Whether an engagement belongs on the list `list`; `None` for a token that is not a list. The one
/// definition behind both the count and the named list, so the two cannot disagree.
#[must_use]
pub fn on_list(list: &str, e: &ContingentEngagement, today: NaiveDate) -> Option<bool> {
    let band = |from: i64, to: i64| {
        e.ends_on
            .is_some_and(|end| (from..=to).contains(&(end - today).num_days()))
    };
    Some(match list {
        "ending_within_30" => band(0, 30),
        "ending_31_to_60" => band(31, 60),
        "ending_61_to_90" => band(61, 90),
        "past_end_undecided" => e.ends_on.is_some_and(|end| end < today) && !e.decided,
        "much_extended" => much_extended(e.extensions, EXTENSION_THRESHOLD),
        "long_running_contractors" => {
            e.contractor && long_running(e.hired_on, today, LONG_RUNNING_MONTHS)
        }
        _ => return None,
    })
}

/// Count the contingent engagements by the questions of WPM-R86.
#[must_use]
pub fn contingent_counts(
    rows: &[ContingentEngagement],
    today: NaiveDate,
    floor: usize,
) -> ContingentCounts {
    let count = |list: &str| {
        let n = rows
            .iter()
            .filter(|r| on_list(list, r, today) == Some(true))
            .count();
        (n == 0 || n >= floor).then_some(n)
    };
    ContingentCounts {
        ending_within_30: count("ending_within_30"),
        ending_31_to_60: count("ending_31_to_60"),
        ending_61_to_90: count("ending_61_to_90"),
        past_end_undecided: count("past_end_undecided"),
        much_extended: count("much_extended"),
        long_running_contractors: count("long_running_contractors"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn contractor_details_validate_the_route_and_the_whole_rate() {
        assert!(validate_contractor_details(None, None, None, None).is_ok());
        assert!(
            validate_contractor_details(Some("agency"), Some(45_000), Some("GBP"), Some("day"))
                .is_ok()
        );
        assert!(
            validate_contractor_details(None, Some(0), Some("EUR"), Some("hour")).is_ok(),
            "zero is a rate"
        );
        for route in ROUTES {
            assert!(validate_contractor_details(Some(route), None, None, None).is_ok());
        }
        assert!(validate_contractor_details(Some("freelance"), None, None, None).is_err());
        // A partial rate is refused: an unknown is never a zero.
        assert!(validate_contractor_details(None, Some(100), None, None).is_err());
        assert!(validate_contractor_details(None, Some(100), Some("GBP"), None).is_err());
        assert!(validate_contractor_details(None, None, Some("GBP"), Some("day")).is_err());
        assert!(validate_contractor_details(None, Some(-1), Some("GBP"), Some("day")).is_err());
        assert!(validate_contractor_details(None, Some(1), Some("gbp"), Some("day")).is_err());
        assert!(validate_contractor_details(None, Some(1), Some("GB"), Some("day")).is_err());
        assert!(validate_contractor_details(None, Some(1), Some("ZZZ"), Some("day")).is_err());
        assert!(validate_contractor_details(None, Some(1), Some("HUF"), Some("day")).is_ok());
        assert!(validate_contractor_details(None, Some(1), Some("GBP"), Some("week")).is_err());
    }

    #[test]
    fn a_reminder_is_due_inside_the_window_and_again_once_it_has_ended() {
        let today = d(2026, 6, 1);
        assert_eq!(
            reminder_for(d(2026, 7, 31), today, 60),
            Some(Reminder::Ending),
            "the last day of the window"
        );
        assert_eq!(
            reminder_for(today, today, 60),
            Some(Reminder::Ending),
            "ends today"
        );
        assert_eq!(
            reminder_for(d(2026, 8, 1), today, 60),
            None,
            "one calendar day beyond the window"
        );
        assert_eq!(
            reminder_for(d(2026, 5, 31), today, 60),
            Some(Reminder::EndedUndecided),
            "yesterday"
        );
        assert_eq!(Reminder::Ending.as_str(), "ending");
        assert_eq!(Reminder::EndedUndecided.as_str(), "ended_undecided");
    }

    #[test]
    fn a_decision_is_extend_convert_or_end() {
        for d in DECISIONS {
            assert!(validate_decision(d).is_ok());
        }
        for bad in ["renew", "", "Extend", "terminate"] {
            assert!(validate_decision(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn bases_round_trip_and_unknown_is_refused() {
        for basis in Basis::ALL {
            assert_eq!(Basis::parse(basis.as_str()), Some(basis));
        }
        assert_eq!(
            Basis::parse("full_time"),
            None,
            "the old spec values are gone"
        );
        assert_eq!(Basis::parse("contract"), None);
        assert!(!Basis::Permanent.is_contingent());
        assert!(Basis::Contractor.is_contingent());
    }

    #[test]
    fn each_basis_needs_allows_or_refuses_an_end_date() {
        let start = d(2026, 1, 5);
        let end = Some(d(2026, 12, 31));
        // Permanent: refused.
        assert!(validate_end_date("permanent", start, None).is_ok());
        assert!(validate_end_date("permanent", start, end).is_err());
        // Fixed-term and contractor: required.
        for basis in ["fixed_term", "contractor"] {
            assert!(validate_end_date(basis, start, end).is_ok(), "{basis}");
            let missing = validate_end_date(basis, start, None).unwrap_err();
            assert!(missing.contains("needs an engagement_ends_on"), "{basis}");
        }
        // Intern: allowed either way.
        assert!(validate_end_date("intern", start, None).is_ok());
        assert!(validate_end_date("intern", start, end).is_ok());
        // An unknown basis is refused with its name.
        assert!(
            validate_end_date("gig", start, end)
                .unwrap_err()
                .contains("gig")
        );
    }

    #[test]
    fn an_end_date_must_be_after_the_start() {
        let start = d(2026, 1, 5);
        assert!(validate_end_date("fixed_term", start, Some(d(2026, 1, 6))).is_ok());
        assert!(validate_end_date("fixed_term", start, Some(start)).is_err());
        assert!(validate_end_date("fixed_term", start, Some(d(2026, 1, 4))).is_err());
    }

    #[test]
    fn an_extension_must_move_the_end_later() {
        let end = d(2026, 6, 30);
        assert!(validate_extension(end, d(2026, 7, 1)).is_ok());
        assert!(
            validate_extension(end, end).is_err(),
            "the same date is no extension"
        );
        assert!(
            validate_extension(end, d(2026, 6, 29)).is_err(),
            "earlier is a shortening"
        );
    }

    #[test]
    fn the_window_is_bounded_in_calendar_days() {
        assert!(validate_window(1).is_ok());
        assert!(validate_window(DEFAULT_WINDOW_CALENDAR_DAYS).is_ok());
        assert!(validate_window(366).is_ok());
        assert!(validate_window(0).is_err());
        assert!(validate_window(367).is_err());
        assert!(validate_window(-5).is_err());
    }

    #[test]
    fn standing_at_each_boundary() {
        let today = d(2026, 6, 1);
        let at = |end: Option<NaiveDate>, decided: bool, basis: Basis| {
            standing(basis, end, today, 60, decided)
        };
        let ft = Basis::FixedTerm;
        // Ends today and on the last day of the window: ending soon.
        assert_eq!(at(Some(today), false, ft), Standing::EndingSoon);
        assert_eq!(at(Some(d(2026, 7, 31)), false, ft), Standing::EndingSoon);
        // One calendar day beyond the window: running.
        assert_eq!(at(Some(d(2026, 8, 1)), false, ft), Standing::Running);
        // Yesterday: past its end, and only a recorded decision settles it.
        assert_eq!(
            at(Some(d(2026, 5, 31)), false, ft),
            Standing::PastEndUndecided
        );
        assert_eq!(at(Some(d(2026, 5, 31)), true, ft), Standing::PastEndDecided);
        // No end date: missing where one is needed, open-ended otherwise.
        assert_eq!(at(None, false, ft), Standing::MissingEndDate);
        assert_eq!(at(None, false, Basis::Contractor), Standing::MissingEndDate);
        assert_eq!(at(None, false, Basis::Permanent), Standing::OpenEnded);
        assert_eq!(at(None, false, Basis::Intern), Standing::OpenEnded);
    }

    #[test]
    fn the_window_test_matches_the_standing() {
        let today = d(2026, 6, 1);
        assert!(in_end_window(today, today, 60));
        assert!(in_end_window(d(2026, 7, 31), today, 60));
        assert!(!in_end_window(d(2026, 8, 1), today, 60));
        assert!(
            !in_end_window(d(2026, 5, 31), today, 60),
            "already ended is not 'in window'"
        );
    }

    #[test]
    fn extended_and_long_running_flags_are_strict() {
        assert!(!much_extended(2, 2), "at the threshold is not above it");
        assert!(much_extended(3, 2));
        assert!(!much_extended(0, 0));
        let start = d(2025, 1, 15);
        // Twelve calendar months from the start is 2026-01-15: longer than twelve months only after.
        assert!(!long_running(start, d(2026, 1, 15), 12));
        assert!(long_running(start, d(2026, 1, 16), 12));
        assert!(
            !long_running(d(2027, 1, 1), d(2026, 1, 1), 12),
            "not started yet"
        );
        // Month-end starts do not overflow (31 January + 1 month is 28 February).
        assert!(!long_running(d(2026, 1, 31), d(2026, 2, 28), 1));
        assert!(long_running(d(2026, 1, 31), d(2026, 3, 1), 1));
    }

    #[test]
    fn engaged_on_the_end_date_but_not_the_day_after() {
        let start = d(2026, 1, 5);
        let end = Some(d(2026, 6, 30));
        assert!(is_engaged_on(d(2026, 1, 5), start, end));
        assert!(!is_engaged_on(d(2026, 1, 4), start, end));
        assert!(is_engaged_on(d(2026, 6, 30), start, end));
        assert!(!is_engaged_on(d(2026, 7, 1), start, end));
        assert!(is_engaged_on(d(2030, 1, 1), start, None));
    }

    fn row(ends: Option<NaiveDate>, decided: bool, extensions: usize) -> ContingentEngagement {
        ContingentEngagement {
            contractor: true,
            hired_on: d(2020, 1, 1),
            ends_on: ends,
            decided,
            extensions,
        }
    }

    #[test]
    fn the_ending_bands_are_disjoint_and_use_calendar_days() {
        let today = d(2026, 6, 1);
        let mut rows = Vec::new();
        for days in [0, 30, 31, 60, 61, 90, 91] {
            rows.push(row(Some(today + chrono::Duration::days(days)), false, 0));
        }
        let c = contingent_counts(&rows, today, 0);
        assert_eq!(c.ending_within_30, Some(2), "day 0 and day 30");
        assert_eq!(c.ending_31_to_60, Some(2));
        assert_eq!(c.ending_61_to_90, Some(2), "day 91 is outside");
    }

    #[test]
    fn past_end_counts_only_the_undecided_and_no_end_is_counted_nowhere() {
        let today = d(2026, 6, 1);
        let rows = [
            row(Some(d(2026, 5, 31)), false, 0),
            row(Some(d(2026, 5, 31)), true, 0),
            row(Some(today), false, 0),
            row(None, false, 0),
        ];
        let c = contingent_counts(&rows, today, 0);
        assert_eq!(
            c.past_end_undecided,
            Some(1),
            "ending today is not yet past"
        );
        assert_eq!(c.ending_within_30, Some(1));
    }

    #[test]
    fn extension_and_length_thresholds_are_strict() {
        let today = d(2026, 6, 1);
        let mut at = row(None, false, EXTENSION_THRESHOLD);
        let mut over = row(None, false, EXTENSION_THRESHOLD + 1);
        at.hired_on = d(2024, 6, 1);
        over.hired_on = d(2024, 5, 31);
        let c = contingent_counts(&[at, over], today, 0);
        assert_eq!(c.much_extended, Some(1));
        assert_eq!(
            c.long_running_contractors,
            Some(1),
            "24 months exactly is not over"
        );
        let mut fixed = over;
        fixed.contractor = false;
        assert_eq!(
            contingent_counts(&[fixed], today, 0).long_running_contractors,
            Some(0)
        );
    }

    #[test]
    fn a_small_count_is_withheld_and_zero_is_shown() {
        let today = d(2026, 6, 1);
        let rows: Vec<_> = (0..3).map(|_| row(Some(today), false, 0)).collect();
        let c = contingent_counts(&rows, today, 10);
        assert_eq!(c.ending_within_30, None);
        assert_eq!(c.ending_31_to_60, Some(0));
        let many: Vec<_> = (0..10).map(|_| row(Some(today), false, 0)).collect();
        assert_eq!(
            contingent_counts(&many, today, 10).ending_within_30,
            Some(10)
        );
    }

    #[test]
    fn every_list_is_known_and_a_stranger_is_not() {
        let today = d(2026, 6, 1);
        let r = row(Some(today), false, 0);
        for list in LISTS {
            assert!(on_list(list, &r, today).is_some(), "{list}");
        }
        assert_eq!(on_list("everyone", &r, today), None);
    }

    #[test]
    fn the_count_and_the_list_agree() {
        let today = d(2026, 6, 1);
        let day = |n: i64| today + chrono::Duration::days(n);
        let mut rows = Vec::new();
        for n in [-5, 0, 10, 31, 45, 61, 90, 200] {
            for ext in [0, 3] {
                rows.push(row(Some(day(n)), n == -5, ext));
            }
        }
        let c = contingent_counts(&rows, today, 0);
        let got = [
            c.ending_within_30,
            c.ending_31_to_60,
            c.ending_61_to_90,
            c.past_end_undecided,
            c.much_extended,
            c.long_running_contractors,
        ];
        for (list, count) in LISTS.iter().zip(got) {
            let listed = rows
                .iter()
                .filter(|r| on_list(list, r, today) == Some(true))
                .count();
            assert_eq!(count, Some(listed), "{list}");
        }
    }
}
