//! Pure rules for **cutover readiness** (WPM-R70, WPM-T117): the people side of a go-live at a
//! site. Three checks (trained staff per required role, hypercare cover from the on-call rota, and
//! key-person risk in the pools the cutover depends on), each `ready`, `at_risk` or `unknown` with
//! reasons, and the worst of them overall.
//!
//! DB-free and clock-free. Nothing here names a person. **The go/no-go stays a human decision outside
//! WPM**: this reports the people evidence for it and never decides.

use chrono::NaiveDate;

use crate::rules::key_person::Status as KeyStatus;

/// The longest window a cutover may cover, in calendar days (the on-call schedule limit).
pub const MAX_WINDOW_CALENDAR_DAYS: i64 = 92;

/// The result of a check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Readiness {
    /// Nothing found that puts the go-live at risk, and everything could be checked.
    Ready,
    /// The answer cannot be given from the data on file.
    Unknown,
    /// Something found puts the go-live at risk.
    AtRisk,
}

impl Readiness {
    /// The token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Unknown => "unknown",
            Self::AtRisk => "at_risk",
        }
    }
}

/// One reason as data: a code and its figures, so a client can say it in its own language. The same
/// reason is also given as English text in [`Check::reasons`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detail {
    /// What kind of reason.
    pub code: &'static str,
    /// The names and figures in it.
    pub params: Vec<(&'static str, String)>,
}

/// A reason: English text for a reader of the API, and the same thing as data for a client.
fn why(code: &'static str, params: Vec<(&'static str, String)>, text: String) -> (Detail, String) {
    (Detail { code, params }, text)
}

/// A check's result with its reasons, in words and without a person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// The result.
    pub status: Readiness,
    /// Why, in English.
    pub reasons: Vec<String>,
    /// Why, as data; one entry for each reason, in the same order.
    pub details: Vec<Detail>,
}

impl Check {
    fn new(status: Readiness, whys: Vec<(Detail, String)>) -> Self {
        let (details, reasons) = whys.into_iter().unzip();
        Self {
            status,
            reasons,
            details,
        }
    }
}

/// Overall: a known risk outweighs an unknown, which outweighs ready. An unknown is never a pass.
#[must_use]
pub fn overall(checks: &[&Check]) -> Readiness {
    checks
        .iter()
        .map(|c| c.status)
        .max()
        .unwrap_or(Readiness::Unknown)
}

/// Validate a cutover window.
///
/// # Errors
///
/// When the end is before the start, or the window is longer than [`MAX_WINDOW_CALENDAR_DAYS`].
pub fn validate_window(start: NaiveDate, end: NaiveDate) -> Result<(), String> {
    if end < start {
        return Err("the window must not end before it starts".to_string());
    }
    let days = (end - start).num_days() + 1;
    if days > MAX_WINDOW_CALENDAR_DAYS {
        return Err(format!(
            "the window may cover at most {MAX_WINDOW_CALENDAR_DAYS} calendar days, not {days}"
        ));
    }
    Ok(())
}

/// One required role at the site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleCount {
    /// People required.
    pub required: u32,
    /// People of the role at the site who completed the course and hold it through the window.
    pub trained: u32,
    /// People of the role, employed in the organization, with no site recorded. They might be at
    /// the site; the data does not say.
    pub unplaced: u32,
    /// Whether the course is known to the learning records at all (any enrollment, or a catalogue
    /// entry). A course nobody has an enrollment in may have been renamed, so training cannot be
    /// checked against it.
    pub course_known: bool,
}

/// Trained staff per required role. A role short of its headcount is at risk, unless people of that
/// role with no site on file could close the gap, in which case it cannot be said. With no role listed
/// the check is unknown.
#[must_use]
pub fn trained_staff(roles: &[(String, RoleCount)]) -> Check {
    if roles.is_empty() {
        return Check::new(
            Readiness::Unknown,
            vec![why(
                "no_role",
                vec![],
                "no required role is listed for this cutover".to_string(),
            )],
        );
    }
    let mut status = Readiness::Ready;
    let mut reasons = Vec::new();
    for (name, r) in roles {
        if !r.course_known {
            status = status.max(Readiness::Unknown);
            reasons.push(why(
                "course_unknown",
                vec![("name", name.clone())],
                format!(
                    "{name}: the course is not in the learning records (renamed or never enrolled), so training cannot be checked"
                ),
            ));
            continue;
        }
        if r.trained >= r.required {
            continue;
        }
        let short = r.required - r.trained;
        if r.unplaced >= short {
            status = status.max(Readiness::Unknown);
            reasons.push(why(
                "trained_unplaced",
                vec![
                    ("name", name.clone()),
                    ("trained", r.trained.to_string()),
                    ("required", r.required.to_string()),
                    ("unplaced", r.unplaced.to_string()),
                ],
                format!(
                    "{name}: {} of {} trained at the site, and {} people of the role have no site recorded",
                    r.trained, r.required, r.unplaced
                ),
            ));
        } else {
            status = Readiness::AtRisk;
            reasons.push(why(
                "trained_short",
                vec![
                    ("name", name.clone()),
                    ("trained", r.trained.to_string()),
                    ("required", r.required.to_string()),
                    ("short", short.to_string()),
                ],
                format!(
                    "{name}: {} of {} trained at the site, short by {short}",
                    r.trained, r.required
                ),
            ));
        }
    }
    Check::new(status, reasons)
}

/// Hypercare cover from the on-call rota. `rota` is `false` when none is linked; `uncovered` is the
/// days of the window with nobody on call, oldest first.
#[must_use]
pub fn hypercare(rota: bool, uncovered: &[NaiveDate]) -> Check {
    if !rota {
        return Check::new(
            Readiness::Unknown,
            vec![why(
                "no_rota",
                vec![],
                "no on-call rota is linked for the cutover window".to_string(),
            )],
        );
    }
    match uncovered.first() {
        None => Check::new(Readiness::Ready, Vec::new()),
        Some(first) => Check::new(
            Readiness::AtRisk,
            vec![why(
                "uncovered_days",
                vec![
                    ("count", uncovered.len().to_string()),
                    ("first", first.to_string()),
                ],
                format!(
                    "{} calendar days of the window have nobody on call, the first on {first}",
                    uncovered.len()
                ),
            )],
        ),
    }
}

/// Key-person risk in the skills of the pools the cutover depends on, read **at the site**. `skills` is
/// each skill's name, its status among the people recorded at the site, and how many employed holders
/// of it have no site recorded (they might be at the site). A skill with no critical minimum on a live
/// role profile is not assessed and does not appear. A risk that unplaced holders could close is
/// unknown, not at risk. With none assessable the check is unknown.
#[must_use]
pub fn key_person(pools_listed: bool, skills: &[(String, KeyStatus, usize)]) -> Check {
    if !pools_listed {
        return Check::new(
            Readiness::Unknown,
            vec![why(
                "no_pool",
                vec![],
                "no pool is listed as a dependency of this cutover".to_string(),
            )],
        );
    }
    if skills.is_empty() {
        return Check::new(
            Readiness::Unknown,
            vec![why(
                "no_critical_skill",
                vec![],
                "none of the skills in the listed pools is marked critical on a role profile"
                    .to_string(),
            )],
        );
    }
    let mut status = Readiness::Ready;
    let mut reasons = Vec::new();
    for (name, held, unplaced) in skills {
        let (code, what) = match held {
            KeyStatus::NoHolder => (
                "key_no_holder",
                format!("{name}: nobody at the site holds it"),
            ),
            KeyStatus::SinglePointOfFailure => (
                "key_single_holder",
                format!("{name}: one holder at the site, with no backup or ready successor"),
            ),
            KeyStatus::CoveredSingleHolder | KeyStatus::Shared => continue,
        };
        if *unplaced > 0 {
            status = status.max(Readiness::Unknown);
            let code = if code == "key_no_holder" {
                "key_no_holder_unplaced"
            } else {
                "key_single_holder_unplaced"
            };
            reasons.push(why(
                code,
                vec![("name", name.clone()), ("unplaced", unplaced.to_string())],
                format!(
                    "{what}; {unplaced} holders elsewhere in the organization have no site recorded"
                ),
            ));
        } else {
            status = Readiness::AtRisk;
            reasons.push(why(code, vec![("name", name.clone())], what));
        }
    }
    Check::new(status, reasons)
}

/// A site name as compared: lower case, apostrophes dropped, other punctuation turned into spaces,
/// runs of spaces collapsed.
/// "St. Mary's  Hospital" and "st marys hospital" differ only in punctuation and spacing, and compare
/// equal; a site spelt another way needs an alias on the cutover.
#[must_use]
pub fn normalise_site(name: &str) -> String {
    name.chars()
        .filter(|c| !matches!(c, '\'' | '\u{2019}'))
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_lowercase().collect::<String>()
            } else {
                " ".to_string()
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether a worker's recorded location is the site, under any of its names (the site and its aliases).
#[must_use]
pub fn at_site(recorded: Option<&str>, names: &[String]) -> bool {
    let Some(recorded) = recorded else {
        return false;
    };
    let here = normalise_site(recorded);
    !here.is_empty() && names.iter().any(|n| normalise_site(n) == here)
}

/// Whether a worker has no site on file (blank counts as none).
#[must_use]
pub fn unplaced(recorded: Option<&str>) -> bool {
    recorded.is_none_or(|l| normalise_site(l).is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn role(required: u32, trained: u32, unplaced: u32) -> (String, RoleCount) {
        (
            "Operator".to_string(),
            RoleCount {
                required,
                trained,
                unplaced,
                course_known: true,
            },
        )
    }
    fn day(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2027, 3, d).unwrap()
    }

    #[test]
    fn a_role_at_or_over_its_headcount_is_ready() {
        assert_eq!(trained_staff(&[role(3, 3, 0)]).status, Readiness::Ready);
        assert_eq!(trained_staff(&[role(3, 5, 0)]).status, Readiness::Ready);
    }

    #[test]
    fn a_role_short_is_at_risk_and_says_by_how_many() {
        let c = trained_staff(&[role(3, 1, 1)]);
        assert_eq!(c.status, Readiness::AtRisk);
        assert!(c.reasons[0].contains("short by 2"));
    }

    #[test]
    fn people_with_no_site_could_close_the_gap_so_it_cannot_be_said() {
        let c = trained_staff(&[role(3, 1, 2)]);
        assert_eq!(c.status, Readiness::Unknown);
        assert!(c.reasons[0].contains("no site recorded"));
        // Not enough of them to close it: a known shortfall.
        assert_eq!(trained_staff(&[role(3, 1, 1)]).status, Readiness::AtRisk);
    }

    #[test]
    fn no_listed_role_is_unknown_never_ready() {
        assert_eq!(trained_staff(&[]).status, Readiness::Unknown);
    }

    #[test]
    fn one_short_role_makes_the_check_at_risk_even_if_another_is_unknown() {
        let roles = [role(3, 1, 2), role(2, 0, 0)];
        assert_eq!(trained_staff(&roles).status, Readiness::AtRisk);
    }

    #[test]
    fn hypercare_needs_a_rota_and_every_day_covered() {
        assert_eq!(hypercare(false, &[]).status, Readiness::Unknown);
        assert_eq!(hypercare(true, &[]).status, Readiness::Ready);
        let c = hypercare(true, &[day(5), day(6)]);
        assert_eq!(c.status, Readiness::AtRisk);
        assert!(c.reasons[0].contains("2 calendar days") && c.reasons[0].contains("2027-03-05"));
    }

    #[test]
    fn key_person_risk_is_at_risk_for_no_holder_or_one_uncovered_holder() {
        use KeyStatus::{CoveredSingleHolder, NoHolder, Shared, SinglePointOfFailure};
        let ok = [
            ("A".to_string(), Shared, 0),
            ("B".to_string(), CoveredSingleHolder, 0),
        ];
        assert_eq!(key_person(true, &ok).status, Readiness::Ready);
        let spof = [
            ("A".to_string(), Shared, 0),
            ("B".to_string(), SinglePointOfFailure, 0),
        ];
        assert_eq!(key_person(true, &spof).status, Readiness::AtRisk);
        let none = [("A".to_string(), NoHolder, 0)];
        assert_eq!(key_person(true, &none).status, Readiness::AtRisk);
    }

    #[test]
    fn a_site_risk_that_unplaced_holders_might_close_is_unknown() {
        use KeyStatus::{NoHolder, SinglePointOfFailure};
        let c = key_person(true, &[("A".to_string(), SinglePointOfFailure, 1)]);
        assert_eq!(c.status, Readiness::Unknown);
        assert!(c.reasons[0].contains("no site recorded"));
        assert_eq!(
            key_person(true, &[("A".to_string(), NoHolder, 2)]).status,
            Readiness::Unknown
        );
        // A known risk elsewhere in the list still makes the check at risk.
        let mixed = [
            ("A".to_string(), NoHolder, 0),
            ("B".to_string(), NoHolder, 2),
        ];
        assert_eq!(key_person(true, &mixed).status, Readiness::AtRisk);
    }

    #[test]
    fn key_person_is_unknown_with_no_pool_or_nothing_assessable() {
        assert_eq!(key_person(false, &[]).status, Readiness::Unknown);
        assert_eq!(key_person(true, &[]).status, Readiness::Unknown);
    }

    #[test]
    fn a_course_nobody_is_enrolled_in_cannot_be_checked() {
        let known = RoleCount {
            required: 1,
            trained: 0,
            unplaced: 0,
            course_known: false,
        };
        let c = trained_staff(&[("Operator".to_string(), known)]);
        assert_eq!(c.status, Readiness::Unknown);
        assert!(c.reasons[0].contains("not in the learning records"));
    }

    #[test]
    fn site_names_compare_without_case_punctuation_or_spacing() {
        assert_eq!(
            normalise_site("  St. Mary's   Hospital "),
            "st marys hospital"
        );
        let names = vec!["St Marys Hospital".to_string(), "Annex".to_string()];
        assert!(at_site(Some("ST MARYS  HOSPITAL"), &names));
        assert!(at_site(Some("annex."), &names));
        assert!(!at_site(Some("Main Street"), &names));
        assert!(!at_site(Some("  "), &names), "a blank is no site");
        assert!(!at_site(None, &names));
        assert!(unplaced(None) && unplaced(Some(" . ")) && !unplaced(Some("Annex")));
    }

    #[test]
    fn overall_is_the_worst_and_an_unknown_is_not_a_pass() {
        let ready = Check {
            status: Readiness::Ready,
            reasons: vec![],
            details: vec![],
        };
        let unknown = Check {
            status: Readiness::Unknown,
            reasons: vec![],
            details: vec![],
        };
        let risk = Check {
            status: Readiness::AtRisk,
            reasons: vec![],
            details: vec![],
        };
        assert_eq!(overall(&[&ready, &ready]), Readiness::Ready);
        assert_eq!(overall(&[&ready, &unknown]), Readiness::Unknown);
        assert_eq!(overall(&[&unknown, &risk, &ready]), Readiness::AtRisk);
        assert_eq!(overall(&[]), Readiness::Unknown);
    }

    #[test]
    fn the_window_is_ordered_and_bounded() {
        assert!(validate_window(day(1), day(1)).is_ok());
        assert!(validate_window(day(2), day(1)).is_err());
        let far = day(1) + chrono::Duration::days(92);
        assert!(validate_window(day(1), far).is_err());
        assert!(validate_window(day(1), far - chrono::Duration::days(1)).is_ok());
    }
}
