//! Statutory and mandatory learning (WPM-R170, WPM-D117). DB-free and clock-free: the as-of date is an argument.
//!
//! A requirement has an audience (the organization, or some of staff group, job role, area of work and position), a basis
//! (`statutory` or `mandatory`) and a refresher period in months, or none: once is enough. A person is `compliant`, `due_soon`,
//! `expired`, `never_completed`, `not_applicable`, or `unknown` when their assignments do not record a category the audience is
//! defined by. **Unknown is never compliant and never missing**, and a roll-up counts it apart.

use chrono::{Months, NaiveDate};

/// The two bases a requirement can rest on; a statutory one names its source.
pub const BASES: &[&str] = &["statutory", "mandatory"];

/// The days before a refresher falls due that a person reads `due_soon`, as the existing compliance view.
pub const DUE_SOON_DAYS: i64 = 90;

/// Which people a requirement is aimed at. An empty list does not restrict that category; all four empty is the organization.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Audience {
    /// Staff group codes.
    pub staff_groups: Vec<String>,
    /// Job role codes.
    pub job_roles: Vec<String>,
    /// Area of work codes.
    pub areas_of_work: Vec<String>,
    /// Position numbers.
    pub positions: Vec<String>,
}

/// The categories one current assignment records; `None` is not recorded.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Categories {
    /// Staff group code.
    pub staff_group: Option<String>,
    /// Job role code.
    pub job_role: Option<String>,
    /// Area of work code.
    pub area_of_work: Option<String>,
    /// Position number.
    pub position: Option<String>,
}

/// Whether a requirement is aimed at a person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applies {
    /// Aimed at this person.
    Yes,
    /// Not aimed at this person.
    No,
    /// A category the audience is defined by is not recorded.
    Unknown,
}

#[derive(PartialEq)]
enum Cell {
    Match,
    Miss,
    Unrecorded,
}

fn cell(allowed: &[String], value: Option<&String>) -> Option<Cell> {
    if allowed.is_empty() {
        return None;
    }
    Some(match value {
        None => Cell::Unrecorded,
        Some(v) if allowed.contains(v) => Cell::Match,
        Some(_) => Cell::Miss,
    })
}

fn one_assignment(audience: &Audience, c: &Categories) -> Applies {
    let cells: Vec<Cell> = [
        cell(&audience.staff_groups, c.staff_group.as_ref()),
        cell(&audience.job_roles, c.job_role.as_ref()),
        cell(&audience.areas_of_work, c.area_of_work.as_ref()),
        cell(&audience.positions, c.position.as_ref()),
    ]
    .into_iter()
    .flatten()
    .collect();
    if cells.contains(&Cell::Miss) {
        Applies::No
    } else if cells.contains(&Cell::Unrecorded) {
        Applies::Unknown
    } else {
        Applies::Yes
    }
}

/// Whether the audience takes in a person, from their current assignments. Aimed at them if any assignment is; otherwise
/// unknown if any assignment leaves a needed category blank, or the person has none and the audience is restricted; else no.
#[must_use]
pub fn applies(audience: &Audience, assignments: &[Categories]) -> Applies {
    if *audience == Audience::default() {
        return Applies::Yes;
    }
    let each: Vec<Applies> = assignments
        .iter()
        .map(|c| one_assignment(audience, c))
        .collect();
    if each.contains(&Applies::Yes) {
        Applies::Yes
    } else if each.is_empty() || each.contains(&Applies::Unknown) {
        Applies::Unknown
    } else {
        Applies::No
    }
}

/// A recorded completion of a course that satisfies the requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Completion {
    /// The day it was completed.
    pub on: NaiveDate,
    /// Whether another of the person's organizations recorded it.
    pub transferred: bool,
}

/// The six statuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    /// In date.
    Compliant,
    /// In date, and the refresher falls due inside the window.
    DueSoon,
    /// The refresher date has passed.
    Expired,
    /// Aimed at the person, and no completion is recorded.
    NeverCompleted,
    /// Not aimed at the person.
    NotApplicable,
    /// A category the audience needs is not recorded.
    Unknown,
}

/// The day a completion stops being in date: the refresher period on. `None` when once is enough.
#[must_use]
pub fn expires_on(completed: NaiveDate, refresher_months: Option<u32>) -> Option<NaiveDate> {
    refresher_months.and_then(|m| completed.checked_add_months(Months::new(m)))
}

/// A person's status. `accepts_transferred` is the requirement's own setting (R167); a completion after `as_of` is ignored.
/// A requirement due on `as_of` itself is still in date.
#[must_use]
pub fn status(
    applies: Applies,
    completions: &[Completion],
    accepts_transferred: bool,
    refresher_months: Option<u32>,
    as_of: NaiveDate,
) -> Status {
    match applies {
        Applies::No => return Status::NotApplicable,
        Applies::Unknown => return Status::Unknown,
        Applies::Yes => {}
    }
    let latest = completions
        .iter()
        .filter(|c| c.on <= as_of && (accepts_transferred || !c.transferred))
        .map(|c| c.on)
        .max();
    let Some(latest) = latest else {
        return Status::NeverCompleted;
    };
    match expires_on(latest, refresher_months) {
        None if refresher_months.is_none() => Status::Compliant,
        None => Status::Unknown,
        Some(e) if as_of > e => Status::Expired,
        Some(e) if (e - as_of).num_days() <= DUE_SOON_DAYS => Status::DueSoon,
        Some(_) => Status::Compliant,
    }
}

/// Counts and a rate for a group of people against one requirement.
#[derive(Debug, Clone, PartialEq)]
pub enum Rollup {
    /// The group is smaller than the floor: nothing is shown.
    Hidden,
    /// Counts per status and the rate of those in date among those counted.
    Shown {
        /// Compliant.
        compliant: usize,
        /// Due soon.
        due_soon: usize,
        /// Expired.
        expired: usize,
        /// Never completed.
        never_completed: usize,
        /// Not applicable.
        not_applicable: usize,
        /// Unknown: counted apart, in neither the numerator nor the denominator.
        unknown: usize,
        /// `(compliant + due soon) / (those four)`, or `None` with nothing to divide by.
        rate: Option<f64>,
    },
}

/// Roll a group up. A group smaller than `floor` (0 is read as 1) shows nothing.
#[must_use]
pub fn rollup(statuses: &[Status], floor: usize) -> Rollup {
    if !super::suppression::visible(statuses.len(), floor) || statuses.is_empty() {
        return Rollup::Hidden;
    }
    let n = |s: Status| statuses.iter().filter(|x| **x == s).count();
    let (compliant, due_soon, expired, never_completed) = (
        n(Status::Compliant),
        n(Status::DueSoon),
        n(Status::Expired),
        n(Status::NeverCompleted),
    );
    let counted = compliant + due_soon + expired + never_completed;
    #[allow(clippy::cast_precision_loss)]
    let rate = (counted > 0).then(|| (compliant + due_soon) as f64 / counted as f64);
    Rollup::Shown {
        compliant,
        due_soon,
        expired,
        never_completed,
        not_applicable: n(Status::NotApplicable),
        unknown: n(Status::Unknown),
        rate,
    }
}

/// Problems with a requirement: a known basis, a source for a statutory one, and a refresher of at least one month if any.
#[must_use]
pub fn requirement_problems(
    basis: &str,
    source: Option<&str>,
    refresher_months: Option<u32>,
) -> Vec<String> {
    let mut out = Vec::new();
    if !BASES.contains(&basis) {
        out.push(format!("basis must be one of: {}", BASES.join(", ")));
    } else if basis == "statutory" && source.is_none_or(|s| s.trim().is_empty()) {
        out.push("a statutory requirement must name its source".to_string());
    }
    if refresher_months == Some(0) {
        out.push(
            "refresher_months must be at least 1, or left out when once is enough".to_string(),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn own(on: NaiveDate) -> Completion {
        Completion {
            on,
            transferred: false,
        }
    }

    fn cats(group: Option<&str>, role: Option<&str>) -> Categories {
        Categories {
            staff_group: group.map(String::from),
            job_role: role.map(String::from),
            ..Categories::default()
        }
    }

    fn aud(groups: &[&str], roles: &[&str]) -> Audience {
        Audience {
            staff_groups: groups.iter().map(|s| (*s).to_string()).collect(),
            job_roles: roles.iter().map(|s| (*s).to_string()).collect(),
            ..Audience::default()
        }
    }

    const NOW: fn() -> NaiveDate = || d(2026, 10, 28);

    #[test]
    fn the_organization_audience_takes_everyone_even_with_nothing_recorded() {
        assert_eq!(applies(&Audience::default(), &[]), Applies::Yes);
    }

    #[test]
    fn a_match_a_miss_and_a_blank() {
        let a = aud(&["nursing"], &[]);
        assert_eq!(applies(&a, &[cats(Some("nursing"), None)]), Applies::Yes);
        assert_eq!(applies(&a, &[cats(Some("admin"), None)]), Applies::No);
        assert_eq!(applies(&a, &[cats(None, Some("rn"))]), Applies::Unknown);
        assert_eq!(
            applies(&a, &[]),
            Applies::Unknown,
            "no assignment cannot be shown to be outside"
        );
    }

    #[test]
    fn every_restricted_category_must_match_and_a_miss_beats_a_blank() {
        let a = aud(&["nursing"], &["rn"]);
        assert_eq!(
            applies(&a, &[cats(Some("nursing"), Some("rn"))]),
            Applies::Yes
        );
        assert_eq!(
            applies(&a, &[cats(Some("nursing"), Some("hca"))]),
            Applies::No
        );
        assert_eq!(
            applies(&a, &[cats(Some("nursing"), None)]),
            Applies::Unknown
        );
        assert_eq!(
            applies(&a, &[cats(Some("admin"), None)]),
            Applies::No,
            "a stated miss settles it"
        );
    }

    #[test]
    fn any_assignment_that_matches_is_enough_and_a_blank_one_keeps_it_unknown() {
        let a = aud(&["nursing"], &[]);
        let list = [cats(Some("admin"), None), cats(Some("nursing"), None)];
        assert_eq!(applies(&a, &list), Applies::Yes);
        let list = [cats(Some("admin"), None), cats(None, None)];
        assert_eq!(applies(&a, &list), Applies::Unknown);
    }

    #[test]
    fn never_completed_and_not_applicable_and_unknown() {
        let s = |ap, c: &[Completion]| status(ap, c, false, Some(12), NOW());
        assert_eq!(s(Applies::Yes, &[]), Status::NeverCompleted);
        assert_eq!(s(Applies::No, &[own(d(2026, 1, 1))]), Status::NotApplicable);
        assert_eq!(s(Applies::Unknown, &[own(d(2026, 1, 1))]), Status::Unknown);
    }

    #[test]
    fn in_date_due_soon_and_expired() {
        let s = |on| status(Applies::Yes, &[own(on)], false, Some(12), NOW());
        assert_eq!(s(d(2026, 6, 1)), Status::Compliant);
        assert_eq!(s(d(2025, 12, 1)), Status::DueSoon);
        assert_eq!(
            s(d(2025, 10, 28)),
            Status::DueSoon,
            "due today is still in date"
        );
        assert_eq!(s(d(2025, 10, 27)), Status::Expired);
    }

    #[test]
    fn the_due_soon_window_is_ninety_days() {
        let s = |on| status(Applies::Yes, &[own(on)], false, Some(12), NOW());
        assert_eq!(
            s(d(2026, 1, 26)),
            Status::DueSoon,
            "expires 2027-01-26, exactly 90 days off"
        );
        assert_eq!(s(d(2026, 1, 27)), Status::Compliant, "91 days off");
    }

    #[test]
    fn the_latest_completion_is_the_one_that_counts() {
        let list = [own(d(2020, 1, 1)), own(d(2026, 6, 1))];
        assert_eq!(
            status(Applies::Yes, &list, false, Some(12), NOW()),
            Status::Compliant
        );
    }

    #[test]
    fn once_is_enough_never_expires() {
        let list = [own(d(1999, 1, 1))];
        assert_eq!(
            status(Applies::Yes, &list, false, None, NOW()),
            Status::Compliant
        );
        assert_eq!(
            status(Applies::Yes, &[], false, None, NOW()),
            Status::NeverCompleted
        );
    }

    #[test]
    fn a_completion_after_the_as_of_date_is_ignored() {
        let list = [own(d(2026, 11, 1))];
        assert_eq!(
            status(Applies::Yes, &list, false, Some(12), NOW()),
            Status::NeverCompleted
        );
    }

    #[test]
    fn a_transferred_completion_counts_only_where_the_requirement_accepts_it() {
        let list = [Completion {
            on: d(2026, 6, 1),
            transferred: true,
        }];
        assert_eq!(
            status(Applies::Yes, &list, false, Some(12), NOW()),
            Status::NeverCompleted
        );
        assert_eq!(
            status(Applies::Yes, &list, true, Some(12), NOW()),
            Status::Compliant
        );
    }

    #[test]
    fn a_rollup_counts_unknown_apart_and_never_as_compliant_or_missing() {
        use Status::*;
        let r = rollup(
            &[
                Compliant,
                DueSoon,
                Expired,
                NeverCompleted,
                NotApplicable,
                Unknown,
                Unknown,
            ],
            1,
        );
        assert_eq!(
            r,
            Rollup::Shown {
                compliant: 1,
                due_soon: 1,
                expired: 1,
                never_completed: 1,
                not_applicable: 1,
                unknown: 2,
                rate: Some(0.5),
            }
        );
    }

    #[test]
    fn a_rate_with_nothing_to_divide_by_is_null_not_zero() {
        use Status::*;
        match rollup(&[Unknown, NotApplicable, Unknown], 1) {
            Rollup::Shown { rate, unknown, .. } => {
                assert_eq!(rate, None);
                assert_eq!(unknown, 2);
            }
            Rollup::Hidden => panic!("above the floor"),
        }
    }

    #[test]
    fn a_group_below_the_floor_shows_nothing() {
        use Status::*;
        assert_eq!(rollup(&[Compliant, Expired], 5), Rollup::Hidden);
        assert!(matches!(rollup(&[Compliant; 5], 5), Rollup::Shown { .. }));
        assert!(
            matches!(rollup(&[Compliant], 0), Rollup::Shown { .. }),
            "a floor of 0 is read as 1"
        );
        assert_eq!(rollup(&[], 0), Rollup::Hidden);
    }

    #[test]
    fn a_requirement_needs_a_basis_a_source_if_statutory_and_a_real_refresher() {
        assert_eq!(
            requirement_problems("mandatory", None, Some(12)),
            Vec::<String>::new()
        );
        assert_eq!(
            requirement_problems("statutory", Some("Health and Safety at Work Act"), None),
            Vec::<String>::new()
        );
        assert_eq!(requirement_problems("statutory", None, None).len(), 1);
        assert_eq!(requirement_problems("statutory", Some("  "), None).len(), 1);
        assert_eq!(requirement_problems("optional", None, None).len(), 1);
        assert_eq!(requirement_problems("mandatory", None, Some(0)).len(), 1);
    }
}
