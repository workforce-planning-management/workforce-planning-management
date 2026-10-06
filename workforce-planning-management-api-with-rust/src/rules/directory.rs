//! Pure rules for the **employee directory**: a read-only, searchable
//! listing of who works where, deliberately narrow — name, title,
//! department, location, organization, manager. It carries nothing
//! sensitive (no salary, no employment dates, no person reference; being
//! away is shown, never why), so it can be shown to anyone who can read the organization at all.
//!
//! DB-free and clock-free: the controller decides who is listed (employed
//! workers in the caller's scope); this module only filters and orders.

use serde::Serialize;
use uuid::Uuid;

/// One directory row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    /// The worker's public id.
    pub pid: Uuid,
    /// Display name.
    pub display_name: String,
    /// Job title.
    pub job_title: String,
    /// Department.
    pub department: String,
    /// Work location, when recorded.
    pub location: Option<String>,
    /// Owning organization URN.
    pub organization_ref: String,
    /// The solid-line manager's display name, when there is one.
    pub manager_name: Option<String>,
    /// Whether the worker is away today (on approved leave). Only *that*
    /// — never the kind of leave or any reason, which can be health data.
    pub away_today: bool,
    /// While away: the display name of who is covering for them (their
    /// best-ranked available backup), when someone is.
    pub covered_by: Option<String>,
    /// The on-call rotas the worker is on call for today (names), if any.
    pub on_call: Vec<String>,
}

/// Whether `entry` matches `query`: every whitespace-separated term must
/// appear (case-insensitively) in the name, title, department, location
/// manager's name, or the name of a rota they are on call for. An empty or blank query matches everyone.
#[must_use]
pub fn matches(entry: &Entry, query: &str) -> bool {
    let haystack = [
        Some(entry.display_name.as_str()),
        Some(entry.job_title.as_str()),
        Some(entry.department.as_str()),
        entry.location.as_deref(),
        entry.manager_name.as_deref(),
    ]
    .into_iter()
    .flatten()
    .chain(entry.on_call.iter().map(String::as_str))
    .collect::<Vec<_>>()
    .join("\n")
    .to_lowercase();
    query
        .split_whitespace()
        .all(|term| haystack.contains(&term.to_lowercase()))
}

/// Filter `entries` by `query` and an optional exact (case-insensitive)
/// `department`, ordered by display name (then pid, so the order is
/// total and stable across pages).
#[must_use]
pub fn search(entries: Vec<Entry>, query: &str, department: Option<&str>) -> Vec<Entry> {
    let mut found: Vec<Entry> = entries
        .into_iter()
        .filter(|e| department.is_none_or(|d| e.department.eq_ignore_ascii_case(d)))
        .filter(|e| matches(e, query))
        .collect();
    found.sort_by(|a, b| {
        a.display_name
            .to_lowercase()
            .cmp(&b.display_name.to_lowercase())
            .then(a.pid.cmp(&b.pid))
    });
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(n: u128, name: &str, title: &str, dept: &str, loc: Option<&str>) -> Entry {
        Entry {
            pid: Uuid::from_u128(n),
            display_name: name.to_string(),
            job_title: title.to_string(),
            department: dept.to_string(),
            location: loc.map(str::to_string),
            organization_ref: "organization:x".to_string(),
            manager_name: None,
            away_today: false,
            covered_by: None,
            on_call: Vec::new(),
        }
    }

    fn names(found: &[Entry]) -> Vec<&str> {
        found.iter().map(|e| e.display_name.as_str()).collect()
    }

    fn sample() -> Vec<Entry> {
        vec![
            entry(3, "Zoe Adams", "Payroll Analyst", "Finance", Some("Leeds")),
            entry(1, "alice Brown", "Engineer", "Engineering", Some("London")),
            entry(2, "Bob Clarke", "Engineering Manager", "Engineering", None),
        ]
    }

    #[test]
    fn a_blank_query_lists_everyone_by_name_ignoring_case() {
        assert_eq!(
            names(&search(sample(), "  ", None)),
            ["alice Brown", "Bob Clarke", "Zoe Adams"]
        );
    }

    #[test]
    fn every_term_must_match_somewhere_case_insensitively() {
        assert_eq!(
            names(&search(sample(), "ENGINEER lon", None)),
            ["alice Brown"]
        );
        // "engineering" is in two titles/departments; "manager" narrows.
        assert_eq!(
            names(&search(sample(), "engineering manager", None)),
            ["Bob Clarke"]
        );
        assert_eq!(names(&search(sample(), "nobody", None)), Vec::<&str>::new());
    }

    #[test]
    fn location_and_manager_name_are_searchable_and_absent_ones_are_skipped() {
        let mut with_manager = entry(4, "Dee Evans", "Designer", "Product", None);
        with_manager.manager_name = Some("Alice Brown".to_string());
        let all = vec![with_manager, entry(2, "Bob Clarke", "Dev", "Eng", None)];
        assert_eq!(
            names(&search(all.clone(), "leeds", None)),
            Vec::<&str>::new()
        );
        assert_eq!(names(&search(all, "alice", None)), ["Dee Evans"]);
    }

    #[test]
    fn the_rota_a_person_is_on_call_for_is_searchable() {
        let mut on_call = entry(7, "Eve Park", "SRE", "Ops", None);
        on_call.on_call = vec!["Platform on-call".to_string()];
        let all = vec![on_call, entry(8, "Fay Quinn", "SRE", "Ops", None)];
        assert_eq!(names(&search(all, "platform on-call", None)), ["Eve Park"]);
    }

    #[test]
    fn department_filter_is_exact_and_case_insensitive() {
        assert_eq!(
            names(&search(sample(), "", Some("engineering"))),
            ["alice Brown", "Bob Clarke"]
        );
        assert_eq!(
            names(&search(sample(), "adams", Some("Engineering"))),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn equal_names_order_by_pid_so_pages_are_stable() {
        let twins = vec![
            entry(9, "Sam Lee", "A", "X", None),
            entry(5, "Sam Lee", "B", "X", None),
        ];
        let found = search(twins, "", None);
        assert_eq!(found[0].pid, Uuid::from_u128(5));
    }
}
