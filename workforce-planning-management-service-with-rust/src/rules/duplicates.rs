//! Possible duplicate workers (WPM-R136, WPM-D87): given a name, and optionally a worker number,
//! which existing workers might be the same person?
//!
//! A thin, pure layer over `workforce-planning-management-matcher`. WPM holds a single
//! `display_name` and no date of birth, so the evidence is a name and an employer worker number
//! and nothing else. Email and phone are **not** compared: the contact details sit behind a
//! narrower access check than the directory, and "this email matches worker X" would hand a
//! caller the very fact that check withholds.
//!
//! A name alone never says "same person" (two real people share one), so a name-only `match` is
//! reported as `review`. Only a shared worker number, from the same organization, stays `match`.
//! Nothing here writes or merges anything.

use serde::Serialize;
use uuid::Uuid;
use workforce_planning_management_matcher::matcher::{Basis, Breakdown, Decision};
use workforce_planning_management_matcher::{MatchingEngine, WorkerNumber, WorkerRecord};

/// The most candidates one answer lists.
pub const MAX_RESULTS: usize = 10;

/// An existing worker, as much of them as the comparison may see.
#[derive(Debug, Clone)]
pub struct Known {
    /// The worker's public id.
    pub pid: Uuid,
    /// The name as held (one string).
    pub display_name: String,
    /// The organization that issued the number.
    pub organization_ref: String,
    /// The employer's number for them.
    pub worker_number: String,
}

/// Who is being checked.
#[derive(Debug, Clone)]
pub struct Query {
    /// The name being checked.
    pub display_name: String,
    /// The organization the new worker would join.
    pub organization_ref: String,
    /// The number they would be given, when known.
    pub worker_number: Option<String>,
}

/// One possible duplicate, with its working.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Possible {
    /// The existing worker's public id.
    pub worker_pid: Uuid,
    /// Their name, as the directory shows it.
    pub display_name: String,
    /// `match` (a shared worker number) or `review`.
    pub decision: Decision,
    /// What the decision rests on.
    pub basis: Basis,
    /// The weighted score; absent when a number decided it.
    pub score: Option<f64>,
    /// Each compared field's score.
    pub breakdown: Breakdown,
}

/// A display name as (given, family): the first and the last word. One word is a given name only.
#[must_use]
pub fn split_name(display_name: &str) -> (Option<String>, Option<String>) {
    let mut words = display_name.split_whitespace();
    let first = words.next().map(str::to_string);
    let last = words.next_back().map(str::to_string);
    (first, last)
}

fn record(name: &str, organization: &str, number: Option<&str>) -> WorkerRecord {
    let (given, family) = split_name(name);
    let mut r = WorkerRecord::new();
    r.given_name = given;
    r.family_name = family;
    if let Some(n) = number.filter(|n| !n.trim().is_empty()) {
        r.worker_numbers.push(WorkerNumber::new(organization, n));
    }
    r
}

/// The candidates that are a `match` or a `review`, best first, at most [`MAX_RESULTS`].
#[must_use]
pub fn possible_duplicates(query: &Query, known: &[Known]) -> Vec<Possible> {
    let engine = MatchingEngine::default();
    let asked = record(
        &query.display_name,
        &query.organization_ref,
        query.worker_number.as_deref(),
    );
    let candidates: Vec<WorkerRecord> = known
        .iter()
        .map(|k| record(&k.display_name, &k.organization_ref, Some(&k.worker_number)))
        .collect();
    engine
        .rank_one_to_many(&asked, &candidates)
        .into_iter()
        .filter_map(|ranked| {
            let k = &known[ranked.index];
            let r = ranked.result;
            let decision = match (r.decision, r.basis) {
                (Decision::Match, Basis::WorkerNumber) => Decision::Match,
                // a name alone is not evidence of identity
                (Decision::Match | Decision::Review, _) => Decision::Review,
                _ => return None,
            };
            Some(Possible {
                worker_pid: k.pid,
                display_name: k.display_name.clone(),
                decision,
                basis: r.basis,
                score: r.score,
                breakdown: r.breakdown,
            })
        })
        .take(MAX_RESULTS)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn known(name: &str, org: &str, number: &str) -> Known {
        Known {
            pid: Uuid::new_v4(),
            display_name: name.into(),
            organization_ref: org.into(),
            worker_number: number.into(),
        }
    }

    fn query(name: &str, number: Option<&str>) -> Query {
        Query {
            display_name: name.into(),
            organization_ref: "organization:a".into(),
            worker_number: number.map(Into::into),
        }
    }

    #[test]
    fn a_name_splits_into_first_and_last_word() {
        assert_eq!(
            split_name("Ada  Byron King"),
            (Some("Ada".into()), Some("King".into()))
        );
        assert_eq!(split_name("Ada"), (Some("Ada".into()), None));
        assert_eq!(split_name("   "), (None, None));
    }

    #[test]
    fn the_same_name_alone_is_review_never_match() {
        let found = possible_duplicates(
            &query("Ada Lovelace", None),
            &[known("Ada Lovelace", "organization:a", "1001")],
        );
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].decision, Decision::Review);
    }

    #[test]
    fn the_same_worker_number_in_one_organization_is_a_match() {
        let found = possible_duplicates(
            &query("Ada Lovelace", Some("1001")),
            &[known("Ada Lovelace", "organization:a", "1001")],
        );
        assert_eq!(found[0].decision, Decision::Match);
        assert_eq!(found[0].basis, Basis::WorkerNumber);
    }

    #[test]
    fn the_same_number_in_another_organization_is_no_evidence() {
        let found = possible_duplicates(
            &query("Grace Hopper", Some("1001")),
            &[known("Ada Lovelace", "organization:b", "1001")],
        );
        assert_eq!(found, Vec::new());
    }

    #[test]
    fn a_different_name_is_not_listed() {
        let found = possible_duplicates(
            &query("Ada Lovelace", None),
            &[known("Grace Hopper", "organization:a", "2002")],
        );
        assert_eq!(found, Vec::new());
    }

    #[test]
    fn one_word_is_too_little_to_say_anything() {
        let found = possible_duplicates(
            &query("Ada", None),
            &[known("Ada Lovelace", "organization:a", "1001")],
        );
        assert_eq!(found, Vec::new());
    }

    #[test]
    fn at_most_ten_are_listed() {
        let many: Vec<Known> = (0..15)
            .map(|i| known("Ada Lovelace", "organization:a", &format!("{i}")))
            .collect();
        assert_eq!(
            possible_duplicates(&query("Ada Lovelace", None), &many).len(),
            MAX_RESULTS
        );
    }

    #[test]
    fn the_answer_carries_no_email_phone_or_birth_score() {
        let found = possible_duplicates(
            &query("Ada Lovelace", None),
            &[known("Ada Lovelace", "organization:a", "1001")],
        );
        assert!(found[0].breakdown.email.is_none() && found[0].breakdown.phone.is_none());
        assert!(found[0].breakdown.date_of_birth.is_none());
    }
}
