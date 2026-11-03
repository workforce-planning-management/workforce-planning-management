//! Coverage evidence: the standing of a workers' compensation or employer's liability record, and what a record must hold
//! (WPM-R220, WPM-D154, WPM-T420). Pure: no database and no clock; `today` is passed in.
//!
//! A record states a fact and its dates. WPM never decides that an organization is covered or compliant: a record with no end date
//! is `no_end_date`, never "valid", and a kind that needs no policy (a statutory scheme, an elective choice of none) is said as such.

use chrono::{Duration, NaiveDate};

/// The default reminder window in calendar days.
pub const DEFAULT_WINDOW_DAYS: i64 = 60;

/// The kinds of cover a record may state.
pub const KINDS: [&str; 4] = [
    "statutory_scheme",
    "private_insurance",
    "self_insured",
    "elective_none",
];

/// Where a record stands on `today`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// Its start date is after today.
    NotYetValid,
    /// In force, and it ends later than the window.
    InForce,
    /// In force, and it ends within the window, on this many calendar days from today (0 is today).
    EndingSoon(i64),
    /// Its end date has passed.
    Expired,
    /// It has no end date, so nothing here says when it stops. Never read as "valid for ever".
    NoEndDate,
}

impl Standing {
    /// The word the API uses.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotYetValid => "not_yet_valid",
            Self::InForce => "in_force",
            Self::EndingSoon(_) => "ending_soon",
            Self::Expired => "expired",
            Self::NoEndDate => "no_end_date",
        }
    }
}

/// The standing of a record on `today`, with a reminder window of `window_days` calendar days.
#[must_use]
pub fn standing(
    valid_from: Option<NaiveDate>,
    valid_to: Option<NaiveDate>,
    today: NaiveDate,
    window_days: i64,
) -> Standing {
    if valid_from.is_some_and(|from| from > today) {
        return Standing::NotYetValid;
    }
    let Some(to) = valid_to else {
        return Standing::NoEndDate;
    };
    if to < today {
        return Standing::Expired;
    }
    if to <= today + Duration::days(window_days.max(0)) {
        return Standing::EndingSoon((to - today).num_days());
    }
    Standing::InForce
}

/// What a record must hold, as a list of problems (empty when it is acceptable). A private policy names its insurer and reference; a
/// kind that is not a policy holds neither; a minimum cover has a currency; the dates run forwards.
#[must_use]
#[allow(clippy::too_many_arguments)] // one record's fields, checked together
pub fn problems(
    jurisdiction: &str,
    kind: &str,
    insurer: Option<&str>,
    policy_reference: Option<&str>,
    minimum_cover_minor: Option<i64>,
    currency: Option<&str>,
    valid_from: Option<NaiveDate>,
    valid_to: Option<NaiveDate>,
) -> Vec<String> {
    let mut out = Vec::new();
    let code_ok = (2..=20).contains(&jurisdiction.len())
        && jurisdiction
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
        && !jurisdiction.starts_with('-');
    if !code_ok {
        out.push("jurisdiction must be a code of 2 to 20 capitals, digits or hyphens".to_string());
    }
    if !KINDS.contains(&kind) {
        out.push(format!("kind must be one of {}", KINDS.join(", ")));
    }
    if kind == "private_insurance" && (insurer.is_none() || policy_reference.is_none()) {
        out.push("a private insurance record names its insurer and policy reference".to_string());
    }
    if matches!(kind, "elective_none" | "statutory_scheme")
        && (insurer.is_some() || policy_reference.is_some())
    {
        out.push(format!("{kind} has no insurer or policy reference"));
    }
    if minimum_cover_minor.is_some_and(|m| m < 0) {
        out.push("minimum_cover_minor must not be negative".to_string());
    }
    if minimum_cover_minor.is_some() && currency.is_none() {
        out.push("a minimum cover has a currency".to_string());
    }
    if currency.is_some_and(|c| c.len() != 3 || !c.chars().all(|ch| ch.is_ascii_uppercase())) {
        out.push("currency must be three capital letters".to_string());
    }
    if let (Some(from), Some(to)) = (valid_from, valid_to)
        && from > to
    {
        out.push("valid_from must not be after valid_to".to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn a_record_is_not_yet_valid_in_force_ending_soon_or_expired() {
        let today = d(2026, 11, 3);
        let s = |from, to| standing(from, to, today, 60);
        assert_eq!(
            s(Some(d(2026, 11, 4)), Some(d(2027, 11, 3))),
            Standing::NotYetValid
        );
        assert_eq!(
            s(Some(d(2026, 1, 1)), Some(d(2027, 11, 3))),
            Standing::InForce
        );
        assert_eq!(s(None, Some(d(2026, 12, 3))), Standing::EndingSoon(30));
        assert_eq!(
            s(None, Some(d(2027, 1, 2))),
            Standing::EndingSoon(60),
            "the window includes its last day"
        );
        assert_eq!(s(None, Some(d(2027, 1, 3))), Standing::InForce);
        assert_eq!(
            s(None, Some(today)),
            Standing::EndingSoon(0),
            "ends today is still in force"
        );
        assert_eq!(s(None, Some(d(2026, 11, 2))), Standing::Expired);
    }

    #[test]
    fn no_end_date_is_said_and_never_read_as_valid() {
        let today = d(2026, 11, 3);
        assert_eq!(standing(None, None, today, 60), Standing::NoEndDate);
        assert_eq!(
            standing(Some(d(2020, 1, 1)), None, today, 60),
            Standing::NoEndDate
        );
        assert_eq!(Standing::NoEndDate.as_str(), "no_end_date");
    }

    #[test]
    fn a_record_is_checked_for_what_its_kind_needs() {
        let ok = |kind, insurer, policy, cover, cur| {
            problems("XA", kind, insurer, policy, cover, cur, None, None)
        };
        assert_eq!(
            ok(
                "private_insurance",
                Some("Acme"),
                Some("P-1"),
                Some(500_000_000),
                Some("GBP")
            ),
            Vec::<String>::new()
        );
        assert_eq!(
            ok("private_insurance", None, Some("P-1"), None, None).len(),
            1
        );
        assert_eq!(ok("elective_none", Some("Acme"), None, None, None).len(), 1);
        assert_eq!(
            ok("statutory_scheme", None, None, None, None),
            Vec::<String>::new()
        );
        assert_eq!(
            ok("self_insured", None, None, Some(1), None).len(),
            1,
            "a cover needs a currency"
        );
        assert_eq!(ok("other", None, None, None, None).len(), 1);
        assert_eq!(
            problems("xa", "self_insured", None, None, None, None, None, None).len(),
            1
        );
        assert_eq!(
            problems(
                "XA",
                "self_insured",
                None,
                None,
                None,
                None,
                Some(d(2027, 1, 1)),
                Some(d(2026, 1, 1))
            )
            .len(),
            1
        );
    }
}
