//! Collective agreements as reference data (WPM-R103, WPM-D68). DB-free and clock-free.
//!
//! An agreement is **transcribed and cited**, like a pay scale: a title, a version, the dates it is in force, where it came from, and the few
//! terms WPM can use. It is tied to a bargaining unit, so who it applies to is *derived* from the unit's rules and is never recorded. That a
//! worker is covered by an agreement is a fact about their job, not a statement that they belong to anything.

use chrono::NaiveDate;

/// The longest title.
pub const TITLE_MAX: usize = 120;
/// The longest source (a citation, a document reference or a link).
pub const SOURCE_MAX: usize = 300;

/// The terms WPM can use; each is optional because an agreement may say nothing about it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Terms<'a> {
    /// A pay scale the agreement sets pay by (an id the service knows).
    pub pay_scale_id: Option<&'a str>,
    /// The most hours a week the agreement allows.
    pub max_weekly_hours: Option<i32>,
    /// The notice, in weeks.
    pub notice_weeks: Option<i32>,
    /// A consultation procedure longer than the law's, in calendar days (it feeds `rules::consultation`).
    pub consultation_days: Option<i32>,
}

/// Problems with an agreement, one sentence each. `known_scale` says whether the service knows a pay scale id.
#[must_use]
pub fn agreement_problems(
    title: &str,
    source: &str,
    effective_from: NaiveDate,
    effective_to: Option<NaiveDate>,
    terms: &Terms<'_>,
    known_scale: impl Fn(&str) -> bool,
) -> Vec<String> {
    let mut out = Vec::new();
    for (name, text, max) in [("title", title, TITLE_MAX), ("source", source, SOURCE_MAX)] {
        let n = text.trim().chars().count();
        if n == 0 || n > max {
            out.push(format!("{name} must be 1 to {max} characters"));
        }
    }
    if effective_to.is_some_and(|to| to < effective_from) {
        out.push("effective_to must not be before effective_from".to_string());
    }
    if let Some(id) = terms.pay_scale_id
        && !known_scale(id.trim())
    {
        out.push("pay_scale_id is not a pay scale this service knows".to_string());
    }
    for (name, value, low, high) in [
        ("max_weekly_hours", terms.max_weekly_hours, 1, 168),
        ("notice_weeks", terms.notice_weeks, 0, 104),
        ("consultation_days", terms.consultation_days, 1, 365),
    ] {
        if value.is_some_and(|v| v < low || v > high) {
            out.push(format!("{name} must be {low} to {high}"));
        }
    }
    out
}

/// Problems with adding a version: it must take effect after the newest version already held, so the history reads in order.
#[must_use]
pub fn version_problems(effective_from: NaiveDate, newest_from: Option<NaiveDate>) -> Vec<String> {
    match newest_from {
        Some(newest) if effective_from <= newest => {
            vec![
                "a new version must take effect after the newest version already recorded"
                    .to_string(),
            ]
        }
        _ => Vec::new(),
    }
}

/// One version of an agreement, for choosing which is in force.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    /// The version number.
    pub version: i32,
    /// First day in force.
    pub from: NaiveDate,
    /// Last day in force, or open.
    pub to: Option<NaiveDate>,
}

/// The version in force on `on`: the highest-numbered one whose dates include the day; `None` when none is (not yet, or lapsed).
#[must_use]
pub fn in_force(versions: &[Version], on: NaiveDate) -> Option<i32> {
    versions
        .iter()
        .filter(|v| v.from <= on && v.to.is_none_or(|to| to >= on))
        .map(|v| v.version)
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    fn known(id: &str) -> bool {
        id == "national-2026-27"
    }

    #[test]
    fn an_agreement_needs_a_title_a_source_and_dates_in_order() {
        let ok = Terms {
            pay_scale_id: Some("national-2026-27"),
            max_weekly_hours: Some(40),
            notice_weeks: Some(4),
            consultation_days: Some(60),
        };
        assert_eq!(
            agreement_problems(
                "Pay and conditions",
                "Agreement of 2026-04-01, clause 4",
                d("2026-04-01"),
                None,
                &ok,
                known
            ),
            Vec::<String>::new()
        );
        assert_eq!(
            agreement_problems(" ", "s", d("2026-04-01"), None, &Terms::default(), known).len(),
            1
        );
        assert_eq!(
            agreement_problems(
                "t",
                &"s".repeat(301),
                d("2026-04-01"),
                None,
                &Terms::default(),
                known
            )
            .len(),
            1
        );
        assert_eq!(
            agreement_problems(
                "t",
                "s",
                d("2026-04-01"),
                Some(d("2026-03-31")),
                &Terms::default(),
                known
            )
            .len(),
            1
        );
        assert_eq!(
            agreement_problems(
                "t",
                "s",
                d("2026-04-01"),
                Some(d("2026-04-01")),
                &Terms::default(),
                known
            ),
            Vec::<String>::new(),
            "one day is fine"
        );
    }

    #[test]
    fn terms_are_checked_against_their_ranges_and_the_pay_scale_must_be_known() {
        let bad = Terms {
            pay_scale_id: Some("made-up"),
            max_weekly_hours: Some(0),
            notice_weeks: Some(105),
            consultation_days: Some(366),
        };
        assert_eq!(
            agreement_problems("t", "s", d("2026-04-01"), None, &bad, known).len(),
            4
        );
        for edge in [(1, 0, 1), (168, 104, 365)] {
            let t = Terms {
                pay_scale_id: None,
                max_weekly_hours: Some(edge.0),
                notice_weeks: Some(edge.1),
                consultation_days: Some(edge.2),
            };
            assert_eq!(
                agreement_problems("t", "s", d("2026-04-01"), None, &t, known),
                Vec::<String>::new(),
                "{edge:?}"
            );
        }
    }

    #[test]
    fn a_new_version_comes_after_the_newest() {
        assert_eq!(
            version_problems(d("2026-04-01"), None),
            Vec::<String>::new()
        );
        assert_eq!(
            version_problems(d("2027-04-01"), Some(d("2026-04-01"))),
            Vec::<String>::new()
        );
        assert_eq!(
            version_problems(d("2026-04-01"), Some(d("2026-04-01"))).len(),
            1
        );
        assert_eq!(
            version_problems(d("2025-01-01"), Some(d("2026-04-01"))).len(),
            1
        );
    }

    #[test]
    fn the_version_in_force_is_the_newest_whose_dates_include_the_day() {
        let v = [
            Version {
                version: 1,
                from: d("2024-04-01"),
                to: Some(d("2025-03-31")),
            },
            Version {
                version: 2,
                from: d("2025-04-01"),
                to: None,
            },
            Version {
                version: 3,
                from: d("2027-04-01"),
                to: None,
            },
        ];
        assert_eq!(in_force(&v, d("2024-03-31")), None, "not yet");
        assert_eq!(in_force(&v, d("2024-04-01")), Some(1));
        assert_eq!(
            in_force(&v, d("2025-03-31")),
            Some(1),
            "the last day counts"
        );
        assert_eq!(in_force(&v, d("2025-04-01")), Some(2));
        assert_eq!(in_force(&v, d("2027-03-31")), Some(2));
        assert_eq!(
            in_force(&v, d("2027-04-01")),
            Some(3),
            "the newest wins once it starts"
        );
        let lapsed = [Version {
            version: 1,
            from: d("2024-04-01"),
            to: Some(d("2025-03-31")),
        }];
        assert_eq!(
            in_force(&lapsed, d("2026-01-01")),
            None,
            "a lapsed agreement is not in force"
        );
        assert_eq!(in_force(&[], d("2026-01-01")), None);
    }
}
