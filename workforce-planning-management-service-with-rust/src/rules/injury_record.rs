//! The injury and illness notification record (WPM-R221, WPM-D158, WPM-D179, WPM-T421). Pure: no database and no clock.
//!
//! A row proves a report was made and points to the statutory record. It is never that record: every jurisdiction read requires a
//! description or the injury itself, and those stay out of WPM. So there is no text field here. A reference accepts only a reference's
//! characters, with no space, and a sentence cannot be typed into one. The classes are plain and shared; a country's own trigger is for
//! the person filing, and WPM decides no duty. Spec: `spec/injury-and-separation-duties.md`.

use chrono::{DateTime, Utc};

use super::suppression::{self, Suppressed};

/// What happened.
pub const KINDS: [&str; 3] = ["accident", "illness", "dangerous_occurrence"];

/// How bad it was, in words every jurisdiction can use. Never a diagnosis.
pub const CLASSES: [&str; 5] = ["no_injury", "no_absence", "absence", "serious", "fatal"];

/// Who a report goes to.
pub const RECIPIENTS: [&str; 5] = [
    "safety_authority",
    "labour_inspectorate",
    "insurer",
    "social_insurance",
    "health_authority",
];

/// How a report was sent.
pub const METHODS: [&str; 3] = ["online", "phone", "form"];

/// The longest reference.
pub const MAX_REF: usize = 200;

/// The most days of absence a row may state.
pub const MAX_DAYS: i32 = 3_660;

/// The most people a row may state.
pub const MAX_PEOPLE: i32 = 100_000;

/// A reference trimmed, or blank as absent. `Err` when it holds anything a reference does not: a space, above all, because a reference
/// is the only text this record takes, and a sentence must not fit in it.
///
/// # Errors
///
/// A message for a reference that is too long or holds other characters.
pub fn clean_ref(value: Option<&str>) -> Result<Option<String>, String> {
    let Some(text) = value.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    if text.chars().count() > MAX_REF {
        return Err(format!("a reference is at most {MAX_REF} characters"));
    }
    if !text
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "-_./:#".contains(c))
    {
        return Err(
            "a reference takes letters, digits and - _ . / : # only: this record holds no description"
                .to_string(),
        );
    }
    Ok(Some(text.to_string()))
}

/// Whether a jurisdiction is a country code, with an optional subdivision, such as `XA` or `XA-1`.
#[must_use]
pub fn valid_jurisdiction(value: &str) -> bool {
    let mut parts = value.split('-');
    let country = parts.next().unwrap_or_default();
    let upper = |s: &str| {
        s.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    };
    country.len() == 2
        && country.chars().all(|c| c.is_ascii_uppercase())
        && parts
            .next()
            .is_none_or(|p| (1..=3).contains(&p.len()) && upper(p))
        && parts.next().is_none()
}

/// What is wrong with an incident's facts. A dangerous occurrence may have hurt nobody, and nothing else may be `no_injury`.
#[must_use]
#[allow(clippy::too_many_arguments)] // one incident's facts, checked together
pub fn problems(
    jurisdiction: &str,
    kind: &str,
    class: &str,
    occurred_at: DateTime<Utc>,
    learned_at: Option<DateTime<Utc>>,
    absence_days: Option<i32>,
    people_affected: i32,
    now: DateTime<Utc>,
) -> Vec<String> {
    let mut out = Vec::new();
    if !valid_jurisdiction(jurisdiction) {
        out.push("jurisdiction must be a country code, such as XA or XA-1".to_string());
    }
    if !KINDS.contains(&kind) {
        out.push("kind is not one of the kinds".to_string());
    }
    if !CLASSES.contains(&class) {
        out.push("class is not one of the classes".to_string());
    }
    if class == "no_injury" && kind != "dangerous_occurrence" {
        out.push("only a dangerous occurrence can have no injury".to_string());
    }
    if occurred_at > now {
        out.push("occurred_at cannot be in the future".to_string());
    }
    if learned_at.is_some_and(|at| at < occurred_at) {
        out.push("learned_at cannot be before occurred_at".to_string());
    }
    if learned_at.is_some_and(|at| at > now) {
        out.push("learned_at cannot be in the future".to_string());
    }
    if absence_days.is_some_and(|d| !(0..=MAX_DAYS).contains(&d)) {
        out.push(format!("absence_days must be 0 to {MAX_DAYS}"));
    }
    if !(0..=MAX_PEOPLE).contains(&people_affected) {
        out.push(format!("people_affected must be 0 to {MAX_PEOPLE}"));
    }
    out
}

/// What is wrong with a report. A sent report says how it went; an unsent one is a duty still to meet.
#[must_use]
pub fn report_problems(
    recipient: &str,
    method: Option<&str>,
    sent_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Vec<String> {
    let mut out = Vec::new();
    if !RECIPIENTS.contains(&recipient) {
        out.push("recipient is not one of the recipients".to_string());
    }
    if method.is_some_and(|m| !METHODS.contains(&m)) {
        out.push("method is not one of the methods".to_string());
    }
    if sent_at.is_some() != method.is_some() {
        out.push("a sent report gives both when and how".to_string());
    }
    if sent_at.is_some_and(|at| at > now) {
        out.push("sent_at cannot be in the future".to_string());
    }
    out
}

/// Where a report stands. It compares the times HR entered and judges nothing: a report with no due time is not on time, it has none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// Sent, at or before the due time.
    SentByDue,
    /// Sent, after the due time.
    SentAfterDue,
    /// Sent, and no due time was entered.
    SentNoDue,
    /// Not sent, and the due time is still ahead.
    Waiting,
    /// Not sent, and the due time has passed.
    PastDue,
    /// Not sent, and no due time was entered: "immediately" has no time.
    NoDue,
}

impl Standing {
    /// The word the API uses.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SentByDue => "sent_by_due",
            Self::SentAfterDue => "sent_after_due",
            Self::SentNoDue => "sent_no_due",
            Self::Waiting => "waiting",
            Self::PastDue => "past_due",
            Self::NoDue => "no_due",
        }
    }
}

/// Where a report stands at `now`.
#[must_use]
pub fn standing(
    due_at: Option<DateTime<Utc>>,
    sent_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Standing {
    match (due_at, sent_at) {
        (Some(due), Some(sent)) if sent <= due => Standing::SentByDue,
        (Some(_), Some(_)) => Standing::SentAfterDue,
        (None, Some(_)) => Standing::SentNoDue,
        (Some(due), None) if now > due => Standing::PastDue,
        (Some(_), None) => Standing::Waiting,
        (None, None) => Standing::NoDue,
    }
}

/// Counts by class with the floor (WPM-D32, WPM-D35), through [`suppression`]: no cell can be worked out from the total.
#[must_use]
pub fn rollup(classes: &[String], floor: usize) -> Suppressed<&'static str> {
    let cells: Vec<(&'static str, usize)> = CLASSES
        .iter()
        .map(|class| {
            (
                *class,
                classes.iter().filter(|c| c.as_str() == *class).count(),
            )
        })
        .collect();
    suppression::with_total(&cells, floor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 4, hour, 0, 0).unwrap()
    }

    fn check(kind: &str, class: &str) -> Vec<String> {
        problems("XA", kind, class, at(9), Some(at(10)), Some(3), 1, at(12))
    }

    #[test]
    fn a_reference_cannot_hold_a_sentence() {
        assert_eq!(
            clean_ref(Some(" AB-2026/41 ")),
            Ok(Some("AB-2026/41".into()))
        );
        assert_eq!(
            clean_ref(Some("urn:doc:42#3")),
            Ok(Some("urn:doc:42#3".into()))
        );
        assert_eq!(clean_ref(Some("   ")), Ok(None));
        assert_eq!(clean_ref(None), Ok(None));
        for narrative in [
            "fell from a ladder",
            "broken-arm, left",
            "écorchure",
            "a\nb",
        ] {
            assert!(clean_ref(Some(narrative)).is_err(), "{narrative}");
        }
        assert!(clean_ref(Some(&"x".repeat(MAX_REF))).is_ok());
        assert!(clean_ref(Some(&"x".repeat(MAX_REF + 1))).is_err());
    }

    #[test]
    fn a_jurisdiction_is_a_code() {
        for good in ["XA", "XA-1", "XA-31", "XA-BC"] {
            assert!(valid_jurisdiction(good), "{good}");
        }
        for bad in ["", "xa", "X", "XAA", "XA-", "XA-ABCD", "XA-1-2", "Atlantis"] {
            assert!(!valid_jurisdiction(bad), "{bad}");
        }
    }

    #[test]
    fn the_facts_are_checked() {
        assert_eq!(check("accident", "serious"), Vec::<String>::new());
        assert_eq!(
            check("dangerous_occurrence", "no_injury"),
            Vec::<String>::new()
        );
        assert_eq!(
            check("accident", "no_injury").len(),
            1,
            "someone was hurt, or it is not an accident"
        );
        assert_eq!(check("fall", "serious").len(), 1);
        assert_eq!(check("accident", "broken_arm").len(), 1);
        let later = problems("XA", "accident", "absence", at(13), None, None, 1, at(12));
        assert_eq!(later.len(), 1, "{later:?}");
        let backwards = problems(
            "XA",
            "accident",
            "absence",
            at(9),
            Some(at(8)),
            None,
            1,
            at(12),
        );
        assert_eq!(backwards.len(), 1, "{backwards:?}");
        let unknown = problems("XA", "accident", "absence", at(9), None, None, 1, at(12));
        assert!(
            unknown.is_empty(),
            "when it was learned and the days may be unknown"
        );
        let counts = problems(
            "XA",
            "accident",
            "absence",
            at(9),
            None,
            Some(-1),
            -1,
            at(12),
        );
        assert_eq!(counts.len(), 2);
    }

    #[test]
    fn a_report_says_when_and_how_together() {
        assert_eq!(
            report_problems("insurer", Some("online"), Some(at(10)), at(12)),
            Vec::<String>::new()
        );
        assert!(
            report_problems("insurer", None, None, at(12)).is_empty(),
            "a duty still to meet"
        );
        assert_eq!(
            report_problems("insurer", Some("online"), None, at(12)).len(),
            1
        );
        assert_eq!(
            report_problems("insurer", None, Some(at(10)), at(12)).len(),
            1
        );
        assert_eq!(report_problems("the mayor", None, None, at(12)).len(), 1);
        assert_eq!(
            report_problems("insurer", Some("pigeon"), None, at(12)).len(),
            2
        );
        assert_eq!(
            report_problems("insurer", Some("form"), Some(at(13)), at(12)).len(),
            1
        );
    }

    #[test]
    fn a_report_stands_on_the_times_entered() {
        let cases = [
            (Some(at(10)), Some(at(10)), Standing::SentByDue),
            (Some(at(10)), Some(at(11)), Standing::SentAfterDue),
            (None, Some(at(11)), Standing::SentNoDue),
            (Some(at(13)), None, Standing::Waiting),
            (Some(at(12)), None, Standing::Waiting),
            (Some(at(10)), None, Standing::PastDue),
            (None, None, Standing::NoDue),
        ];
        for (due, sent, want) in cases {
            assert_eq!(standing(due, sent, at(12)), want, "{due:?} {sent:?}");
        }
        assert_eq!(Standing::NoDue.as_str(), "no_due");
        assert_eq!(
            standing(Some(at(12) - Duration::seconds(1)), None, at(12)),
            Standing::PastDue
        );
    }

    #[test]
    fn the_rollup_goes_through_the_shared_suppression() {
        let rows = |pairs: &[(&str, usize)]| -> Vec<String> {
            pairs
                .iter()
                .flat_map(|(c, n)| std::iter::repeat_n((*c).to_string(), *n))
                .collect()
        };
        let of = |s: &Suppressed<&'static str>, class: &str| {
            s.cells.iter().find(|(c, _)| *c == class).unwrap().1
        };
        let one_death = rollup(&rows(&[("absence", 12), ("fatal", 1)]), 10);
        assert_eq!(of(&one_death, "fatal"), None, "one death is someone");
        assert_eq!(
            of(&one_death, "absence"),
            None,
            "or it could be worked out from the total"
        );
        assert_eq!(
            of(&one_death, "serious"),
            Some(0),
            "nothing to hide in a zero"
        );
        let plain = rollup(&rows(&[("absence", 12), ("serious", 10)]), 10);
        assert_eq!(plain.total, Some(22));
        assert_eq!(rollup(&[], 10).total, Some(0));
    }
}
