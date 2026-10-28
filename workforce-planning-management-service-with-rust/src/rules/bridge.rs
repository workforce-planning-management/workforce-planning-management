//! Cross-app bridges (WPM-R206 to R211, WPM-D134). DB-free and clock-free.
//!
//! A bridge carries **references and aggregates, never records**. This module holds the two pure pieces of the first bridges: the
//! exact payload a hire or termination event hands to another app, and the overlap question ("which of these people are workers
//! here?"). The point of putting the payload in one function is that a test can pin its keys, so a field cannot be added by accident.

use std::collections::BTreeSet;
use std::str::FromStr;

use chrono::NaiveDate;
use entity_ref::{EntityRef, EntityType};
use serde_json::{Value, json};

/// The most references one overlap call may carry.
pub const OVERLAP_MAX: usize = 500;

/// What a hire or a termination tells another app: who (a `person:` ref), where (an `organization:` ref) and when. Nothing else:
/// no name, no department, no pay, no worker number.
#[must_use]
pub fn handoff_data(person_ref: &str, organization_ref: &str, effective_on: NaiveDate) -> Value {
    json!({
        "person_ref": person_ref,
        "organization_ref": organization_ref,
        "effective_on": effective_on,
    })
}

/// The keys [`handoff_data`] returns, sorted. A test pins this; a consumer relies on it.
pub const HANDOFF_KEYS: [&str; 3] = ["effective_on", "organization_ref", "person_ref"];

/// Problems with an overlap request: 1 to [`OVERLAP_MAX`] references, each a well-formed `person:` URN. A repeat is allowed and answered once.
#[must_use]
pub fn overlap_problems(refs: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    if refs.is_empty() || refs.len() > OVERLAP_MAX {
        out.push(format!("send 1 to {OVERLAP_MAX} person references"));
    }
    let bad = refs
        .iter()
        .filter(|r| !EntityRef::from_str(r).is_ok_and(|e| e.entity_type == EntityType::Person))
        .count();
    if bad > 0 {
        out.push(format!("{bad} of the references are not person: URNs"));
    }
    out
}

/// For each distinct supplied reference, in the order first seen, whether it is among `workers`. Nothing is returned about a reference
/// that was not supplied, and nothing but a yes or no about one that was.
#[must_use]
pub fn overlap(workers: &BTreeSet<String>, refs: &[String]) -> Vec<(String, bool)> {
    let mut seen = BTreeSet::new();
    refs.iter()
        .filter(|r| seen.insert((*r).clone()))
        .map(|r| (r.clone(), workers.contains(r)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "person:0c4f1e2a-0000-4000-8000-000000000001";
    const B: &str = "person:0c4f1e2a-0000-4000-8000-000000000002";
    const ORG: &str = "organization:0c4f1e2a-0000-4000-8000-0000000000aa";

    fn d() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 28).unwrap()
    }

    #[test]
    fn the_handoff_carries_exactly_three_keys_and_no_personal_field() {
        let v = handoff_data(A, ORG, d());
        let mut keys: Vec<_> = v.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        assert_eq!(keys, HANDOFF_KEYS);
        assert_eq!(v["person_ref"], A);
        assert_eq!(v["organization_ref"], ORG);
        assert_eq!(v["effective_on"], "2026-10-28");
        let text = v.to_string();
        for word in [
            "name",
            "salary",
            "department",
            "email",
            "birth",
            "address",
            "rating",
        ] {
            assert!(!text.contains(word), "{word}");
        }
    }

    #[test]
    fn an_overlap_request_is_one_to_five_hundred_person_urns() {
        assert_eq!(overlap_problems(&[A.to_string()]), Vec::<String>::new());
        assert_eq!(overlap_problems(&[]).len(), 1);
        let many: Vec<String> = (0..=OVERLAP_MAX).map(|_| A.to_string()).collect();
        assert_eq!(overlap_problems(&many).len(), 1);
        let full: Vec<String> = (0..OVERLAP_MAX).map(|_| A.to_string()).collect();
        assert_eq!(overlap_problems(&full), Vec::<String>::new());
        assert_eq!(
            overlap_problems(&[ORG.to_string()]).len(),
            1,
            "an organization is not a person"
        );
        assert_eq!(
            overlap_problems(&["not-a-urn".to_string(), A.to_string()]).len(),
            1
        );
    }

    #[test]
    fn the_answer_is_a_yes_or_no_for_each_supplied_reference_once() {
        let workers: BTreeSet<String> = [
            A.to_string(),
            "person:0c4f1e2a-0000-4000-8000-0000000000ff".to_string(),
        ]
        .into();
        let got = overlap(
            &workers,
            &[B.to_string(), A.to_string(), B.to_string(), A.to_string()],
        );
        assert_eq!(got, vec![(B.to_string(), false), (A.to_string(), true)]);
        assert!(
            !got.iter().any(|(r, _)| r.ends_with("ff")),
            "a worker the caller did not name is never mentioned"
        );
        assert_eq!(overlap(&workers, &[]), Vec::<(String, bool)>::new());
    }
}
