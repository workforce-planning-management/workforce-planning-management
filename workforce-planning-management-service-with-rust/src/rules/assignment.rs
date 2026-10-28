//! Assignments and their categories (WPM-R161, WPM-R162, WPM-D112, WPM-D116). DB-free and clock-free.
//!
//! An assignment is a **contract**, not a timesheet: its FTE is what the contract says, and a person who holds two jobs holds
//! two assignments. The FTE of the current ones adds up to at most 100, exactly one current assignment is primary, and a change
//! of category, cost centre or FTE ends one assignment and starts another on the effective date. A category that was never
//! recorded reads `unknown`, never a default. Dates are inclusive at both ends.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{Days, NaiveDate};

/// The most FTE a worker's current assignments can add up to in one organization.
pub const FTE_MAX: i32 = 100;

/// The longest job title, department or location.
pub const TEXT_MAX: usize = 120;
/// The longest category code.
pub const CODE_MAX: usize = 60;

/// The word for a category nobody recorded.
pub const UNKNOWN: &str = "unknown";

/// What the rules need to know about one assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    /// What the contract says, 1 to 100.
    pub fte_percent: i32,
    /// The first day in force.
    pub starts_on: NaiveDate,
    /// The last day in force, or none while it continues.
    pub ends_on: Option<NaiveDate>,
    /// Whether this is the worker's primary assignment.
    pub primary: bool,
}

impl Assignment {
    /// Whether the assignment is in force on `on`.
    #[must_use]
    pub fn current_on(&self, on: NaiveDate) -> bool {
        self.starts_on <= on && self.ends_on.is_none_or(|e| e >= on)
    }
}

/// The FTE of the assignments in force on `on`.
#[must_use]
pub fn fte_total(assignments: &[Assignment], on: NaiveDate) -> i32 {
    assignments
        .iter()
        .filter(|a| a.current_on(on))
        .map(|a| a.fte_percent)
        .sum()
}

/// Problems with one assignment on its own: FTE from 1 to 100, and an end that is not before the start.
#[must_use]
pub fn assignment_problems(a: &Assignment) -> Vec<String> {
    let mut out = Vec::new();
    if !(1..=FTE_MAX).contains(&a.fte_percent) {
        out.push(format!("fte_percent must be 1 to {FTE_MAX}"));
    }
    if a.ends_on.is_some_and(|e| e < a.starts_on) {
        out.push("ends_on must not be before starts_on".to_string());
    }
    out
}

/// Problems with the words of an assignment: a job title and department of 1 to [`TEXT_MAX`] characters, and a location and each
/// category code, where given, of 1 to [`TEXT_MAX`] and [`CODE_MAX`]. A category left out is unknown and is not a problem.
#[must_use]
pub fn text_problems(
    job_title: &str,
    department: &str,
    location: Option<&str>,
    codes: &[(&str, Option<&str>)],
) -> Vec<String> {
    let mut out = Vec::new();
    let len = |t: &str| t.trim().chars().count();
    for (name, text) in [("job_title", job_title), ("department", department)] {
        if !(1..=TEXT_MAX).contains(&len(text)) {
            out.push(format!("{name} must be 1 to {TEXT_MAX} characters"));
        }
    }
    if location.is_some_and(|l| !(1..=TEXT_MAX).contains(&len(l))) {
        out.push(format!("location must be 1 to {TEXT_MAX} characters"));
    }
    for (name, code) in codes {
        if code.is_some_and(|c| !(1..=CODE_MAX).contains(&len(c))) {
            out.push(format!("{name} must be 1 to {CODE_MAX} characters"));
        }
    }
    out
}

/// Problems with adding `new` to the worker's `existing` assignments: the total in force on the new start day, and on every day
/// a later-starting assignment begins inside the new one, must not go over [`FTE_MAX`]. The message names the total.
#[must_use]
pub fn fte_problems(existing: &[Assignment], new: &Assignment) -> Vec<String> {
    let mut days = vec![new.starts_on];
    days.extend(
        existing
            .iter()
            .map(|a| a.starts_on)
            .filter(|d| *d > new.starts_on && new.current_on(*d)),
    );
    let worst = days
        .into_iter()
        .map(|d| (d, fte_total(existing, d) + new.fte_percent))
        .max_by_key(|(_, t)| *t);
    match worst {
        Some((d, t)) if t > FTE_MAX => {
            vec![format!("FTE would total {t} on {d}; the most is {FTE_MAX}")]
        }
        _ => Vec::new(),
    }
}

/// Problems with which assignment is primary on `on`: where any are current, exactly one is primary.
#[must_use]
pub fn primary_problems(assignments: &[Assignment], on: NaiveDate) -> Vec<String> {
    let current: Vec<_> = assignments.iter().filter(|a| a.current_on(on)).collect();
    if current.is_empty() {
        return Vec::new();
    }
    match current.iter().filter(|a| a.primary).count() {
        1 => Vec::new(),
        0 => vec!["one current assignment must be primary".to_string()],
        n => vec![format!(
            "only one current assignment can be primary, found {n}"
        )],
    }
}

/// The day the old assignment ends when a change takes effect on `effective_on`: the day before, so the two never overlap.
/// A change on or before the old start, or after the old end, is refused.
///
/// # Errors
/// A sentence saying why the change cannot take effect on that day.
pub fn end_for_change(old: &Assignment, effective_on: NaiveDate) -> Result<NaiveDate, String> {
    if effective_on <= old.starts_on {
        return Err("the change must take effect after the assignment started".to_string());
    }
    if old.ends_on.is_some_and(|e| effective_on > e) {
        return Err("the assignment had already ended".to_string());
    }
    effective_on
        .checked_sub_days(Days::new(1))
        .ok_or_else(|| "the change date is out of range".to_string())
}

/// Whether a job role and a staff group agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agreement {
    /// The vocabulary gives the role this staff group.
    Agrees,
    /// The vocabulary gives the role a different staff group.
    Mismatch,
    /// The vocabulary does not say which staff group the role belongs to, or one of the two is not recorded.
    Unknown,
}

/// Whether the staff group recorded on an assignment is the one the vocabulary gives the job role. `role_staff_group` is the
/// vocabulary's answer for the role (`None` where it does not say). Only a stated disagreement is a mismatch (a `422`).
#[must_use]
pub fn agreement(role_staff_group: Option<&str>, staff_group: Option<&str>) -> Agreement {
    match (role_staff_group, staff_group) {
        (Some(r), Some(s)) if r == s => Agreement::Agrees,
        (Some(_), Some(_)) => Agreement::Mismatch,
        _ => Agreement::Unknown,
    }
}

/// The kinds of category an organization keeps a vocabulary for.
pub const KINDS: &[&str] = &[
    "site",
    "area_of_work",
    "job_role",
    "staff_group",
    "occupation_code",
];

/// Problems with a vocabulary entry: a known kind, a code and label, a standard and its code given together, and a staff group
/// only on a job role.
#[must_use]
pub fn entry_problems(
    kind: &str,
    code: &str,
    label: &str,
    standard: Option<&str>,
    standard_code: Option<&str>,
    staff_group: Option<&str>,
) -> Vec<String> {
    let mut out = Vec::new();
    let len = |t: &str| t.trim().chars().count();
    if !KINDS.contains(&kind) {
        out.push(format!("kind must be one of: {}", KINDS.join(", ")));
    }
    if !(1..=CODE_MAX).contains(&len(code)) {
        out.push(format!("code must be 1 to {CODE_MAX} characters"));
    }
    if !(1..=TEXT_MAX).contains(&len(label)) {
        out.push(format!("label must be 1 to {TEXT_MAX} characters"));
    }
    if standard.is_some() != standard_code.is_some() {
        out.push("a standard and its own code are given together".to_string());
    }
    for (name, v) in [
        ("standard", standard),
        ("standard_code", standard_code),
        ("staff_group", staff_group),
    ] {
        if v.is_some_and(|v| !(1..=CODE_MAX).contains(&len(v))) {
            out.push(format!("{name} must be 1 to {CODE_MAX} characters"));
        }
    }
    if staff_group.is_some() && kind != "job_role" {
        out.push("only a job role belongs to a staff group".to_string());
    }
    out
}

/// A category as it reads: the recorded label, or `unknown`. Blank is not a recorded value.
#[must_use]
pub fn category_or_unknown(value: Option<&str>) -> &str {
    match value.map(str::trim) {
        Some(v) if !v.is_empty() => v,
        _ => UNKNOWN,
    }
}

/// One category's headcount and FTE. Below the floor both are `None`: a small group shows nothing, not a small number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// The category code, or `unknown`.
    pub category: String,
    /// People with at least one assignment in the category.
    pub headcount: Option<usize>,
    /// The FTE of those assignments, in percent points (200 is two whole-time posts).
    pub fte_percent: Option<i64>,
}

/// Headcount and FTE by category. `rows` is `(worker, category, fte_percent)` for each assignment in force; a worker with two
/// assignments in one category counts once there. A category not recorded is its own cell, `unknown`, never merged into another.
/// A cell under `floor` (0 is read as 1) shows nothing. **No total is returned**, so a hidden cell cannot be found by subtraction.
#[must_use]
pub fn breakdown<W: Ord + Clone>(rows: &[(W, Option<String>, i32)], floor: usize) -> Vec<Cell> {
    let mut by: BTreeMap<String, (BTreeSet<W>, i64)> = BTreeMap::new();
    for (worker, category, fte) in rows {
        let cell = by
            .entry(category_or_unknown(category.as_deref()).to_string())
            .or_default();
        cell.0.insert(worker.clone());
        cell.1 += i64::from(*fte);
    }
    by.into_iter()
        .map(|(category, (workers, fte))| {
            let shown = workers.len() >= floor.max(1);
            Cell {
                category,
                headcount: shown.then_some(workers.len()),
                fte_percent: shown.then_some(fte),
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

    fn a(fte: i32, from: NaiveDate, to: Option<NaiveDate>, primary: bool) -> Assignment {
        Assignment {
            fte_percent: fte,
            starts_on: from,
            ends_on: to,
            primary,
        }
    }

    #[test]
    fn total_counts_only_what_is_in_force() {
        let list = [
            a(60, d(2026, 1, 1), None, true),
            a(40, d(2026, 6, 1), Some(d(2026, 8, 31)), false),
            a(50, d(2025, 1, 1), Some(d(2025, 12, 31)), false),
        ];
        assert_eq!(fte_total(&list, d(2026, 5, 31)), 60);
        assert_eq!(fte_total(&list, d(2026, 6, 1)), 100, "start day is in");
        assert_eq!(fte_total(&list, d(2026, 8, 31)), 100, "end day is in");
        assert_eq!(fte_total(&list, d(2026, 9, 1)), 60);
    }

    #[test]
    fn a_write_over_one_hundred_names_the_total() {
        let existing = [a(60, d(2026, 1, 1), None, true)];
        let msg = fte_problems(&existing, &a(50, d(2026, 3, 1), None, false));
        assert_eq!(msg.len(), 1);
        assert!(msg[0].contains("110"), "{msg:?}");
        assert_eq!(
            fte_problems(&existing, &a(40, d(2026, 3, 1), None, false)),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_later_start_inside_the_new_one_counts() {
        let existing = [a(70, d(2026, 6, 1), None, false)];
        let new = a(40, d(2026, 1, 1), None, true);
        assert_eq!(fte_problems(&existing, &new).len(), 1);
        let ends_first = a(40, d(2026, 1, 1), Some(d(2026, 5, 31)), true);
        assert_eq!(fte_problems(&existing, &ends_first), Vec::<String>::new());
    }

    #[test]
    fn one_assignment_is_checked_on_its_own() {
        assert_eq!(
            assignment_problems(&a(100, d(2026, 1, 1), None, true)),
            Vec::<String>::new()
        );
        assert_eq!(
            assignment_problems(&a(0, d(2026, 1, 1), None, true)).len(),
            1
        );
        assert_eq!(
            assignment_problems(&a(101, d(2026, 1, 1), None, true)).len(),
            1
        );
        let backwards = a(50, d(2026, 2, 1), Some(d(2026, 1, 1)), true);
        assert_eq!(assignment_problems(&backwards).len(), 1);
    }

    #[test]
    fn the_words_of_an_assignment_are_bounded_and_a_missing_category_is_fine() {
        let ok = |t: &str, dpt: &str, l: Option<&str>, c: &[(&str, Option<&str>)]| {
            text_problems(t, dpt, l, c)
        };
        assert_eq!(
            ok("Nurse", "Ward 4", None, &[("job_role", None)]),
            Vec::<String>::new()
        );
        assert_eq!(ok(" ", "Ward 4", None, &[]).len(), 1);
        assert_eq!(ok("Nurse", &"x".repeat(121), None, &[]).len(), 1);
        assert_eq!(ok("Nurse", "Ward 4", Some(" "), &[]).len(), 1);
        assert_eq!(
            ok("Nurse", "Ward 4", None, &[("staff_group", Some(" "))]).len(),
            1
        );
        assert_eq!(
            ok("Nurse", "Ward 4", None, &[("site", Some(&"y".repeat(61)))]).len(),
            1
        );
    }

    #[test]
    fn a_vocabulary_entry_needs_a_kind_a_code_and_a_label() {
        let e = |k, c, l, s, sc, g| entry_problems(k, c, l, s, sc, g);
        assert_eq!(
            e("job_role", "RN", "Nurse", None, None, Some("nursing")),
            Vec::<String>::new()
        );
        assert_eq!(
            e("site", "S1", "Main", Some("ESR"), Some("0001"), None),
            Vec::<String>::new()
        );
        assert_eq!(e("ward", "S1", "Main", None, None, None).len(), 1);
        assert_eq!(e("site", " ", "Main", None, None, None).len(), 1);
        assert_eq!(e("site", "S1", "", None, None, None).len(), 1);
        assert_eq!(
            e("site", "S1", "Main", Some("ESR"), None, None).len(),
            1,
            "together"
        );
        assert_eq!(
            e("site", "S1", "Main", None, None, Some("nursing")).len(),
            1,
            "only a role"
        );
    }

    #[test]
    fn a_breakdown_counts_a_person_once_per_category_and_hides_small_cells() {
        let rows = [
            (1, Some("nursing".to_string()), 60),
            (1, Some("nursing".to_string()), 40),
            (2, Some("nursing".to_string()), 100),
            (3, Some("admin".to_string()), 100),
            (4, None, 50),
            (5, Some(" ".to_string()), 50),
        ];
        let cells = breakdown(&rows, 2);
        let get = |c: &str| cells.iter().find(|x| x.category == c).unwrap().clone();
        assert_eq!(
            (get("nursing").headcount, get("nursing").fte_percent),
            (Some(2), Some(200))
        );
        assert_eq!(
            (get("admin").headcount, get("admin").fte_percent),
            (None, None),
            "one person is under a floor of two"
        );
        assert_eq!(
            (get("unknown").headcount, get("unknown").fte_percent),
            (Some(2), Some(100)),
            "blank and missing are one unknown cell"
        );
        assert_eq!(cells.len(), 3);
    }

    #[test]
    fn a_floor_of_zero_is_read_as_one_and_nothing_is_nothing() {
        let rows = [(1, Some("a".to_string()), 10)];
        assert_eq!(breakdown(&rows, 0)[0].headcount, Some(1));
        assert_eq!(breakdown::<i32>(&[], 5), Vec::<Cell>::new());
    }

    #[test]
    fn exactly_one_current_assignment_is_primary() {
        let on = d(2026, 5, 1);
        let ok = [
            a(50, d(2026, 1, 1), None, true),
            a(50, d(2026, 1, 1), None, false),
        ];
        assert_eq!(primary_problems(&ok, on), Vec::<String>::new());
        let none = [a(50, d(2026, 1, 1), None, false)];
        assert_eq!(primary_problems(&none, on).len(), 1);
        let two = [
            a(50, d(2026, 1, 1), None, true),
            a(50, d(2026, 1, 1), None, true),
        ];
        assert!(primary_problems(&two, on)[0].contains('2'));
        assert!(
            primary_problems(&[], on).is_empty(),
            "nobody current, nothing to say"
        );
        let ended_primary = [a(50, d(2025, 1, 1), Some(d(2025, 12, 31)), true)];
        assert!(
            primary_problems(&ended_primary, on).is_empty(),
            "an ended primary is history"
        );
    }

    #[test]
    fn a_change_ends_the_old_assignment_the_day_before() {
        let old = a(50, d(2026, 1, 1), None, true);
        assert_eq!(end_for_change(&old, d(2026, 4, 1)), Ok(d(2026, 3, 31)));
        assert!(
            end_for_change(&old, d(2026, 1, 1)).is_err(),
            "same day as start"
        );
        assert!(end_for_change(&old, d(2025, 12, 1)).is_err());
        let ended = a(50, d(2026, 1, 1), Some(d(2026, 2, 28)), true);
        assert!(end_for_change(&ended, d(2026, 4, 1)).is_err());
        assert_eq!(end_for_change(&ended, d(2026, 2, 28)), Ok(d(2026, 2, 27)));
    }

    #[test]
    fn the_two_halves_of_a_change_never_overlap_or_leave_a_gap() {
        let old = a(50, d(2026, 1, 1), None, true);
        let eff = d(2026, 4, 1);
        let end = end_for_change(&old, eff).unwrap();
        let before = Assignment {
            ends_on: Some(end),
            ..old
        };
        let after = a(80, eff, None, true);
        for day in [end, eff] {
            let both = [before.clone(), after.clone()];
            assert_eq!(both.iter().filter(|x| x.current_on(day)).count(), 1);
            assert_eq!(primary_problems(&both, day), Vec::<String>::new());
        }
    }

    #[test]
    fn only_a_stated_disagreement_is_a_mismatch() {
        assert_eq!(
            agreement(Some("nursing"), Some("nursing")),
            Agreement::Agrees
        );
        assert_eq!(
            agreement(Some("nursing"), Some("admin")),
            Agreement::Mismatch
        );
        assert_eq!(agreement(None, Some("admin")), Agreement::Unknown);
        assert_eq!(agreement(Some("nursing"), None), Agreement::Unknown);
        assert_eq!(agreement(None, None), Agreement::Unknown);
    }

    #[test]
    fn a_category_nobody_recorded_reads_unknown() {
        assert_eq!(category_or_unknown(None), UNKNOWN);
        assert_eq!(category_or_unknown(Some("  ")), UNKNOWN);
        assert_eq!(category_or_unknown(Some(" ward ")), "ward");
    }
}
