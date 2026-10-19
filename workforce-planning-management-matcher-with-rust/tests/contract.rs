//! The public contract: JSON shapes a caller can store, and what must never leak.

use workforce_planning_management_matcher::{
    Decision, MatchConfig, MatchingEngine, NicknameTable, WorkerRecord,
};

#[test]
fn config_round_trips_and_partial_json_takes_defaults() {
    let c = MatchConfig::strict();
    let back: MatchConfig = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    assert_eq!(c, back);

    let partial: MatchConfig = serde_json::from_str(r#"{"match_threshold": 0.9}"#).unwrap();
    assert_eq!(partial.match_threshold, 0.9);
    assert_eq!(
        partial.review_threshold,
        MatchConfig::default().review_threshold
    );
    assert_eq!(partial.weights, MatchConfig::default().weights);
}

#[test]
fn a_config_from_a_file_is_still_validated() {
    let c: MatchConfig = serde_json::from_str(r#"{"review_threshold": 0.95}"#).unwrap();
    assert!(
        MatchingEngine::new(c).is_err(),
        "review 0.95 is above the default match 0.88"
    );
}

#[test]
fn record_deserialises_with_every_field_optional() {
    let r: WorkerRecord = serde_json::from_str(r#"{"given_name": "Ada"}"#).unwrap();
    assert_eq!(r.given_name.as_deref(), Some("Ada"));
    assert!(r.emails.is_empty() && r.date_of_birth.is_none());
}

#[test]
fn result_serialises_unknowns_as_null_and_decisions_as_words() {
    let a = WorkerRecord::new()
        .with_given_name("Ada")
        .with_family_name("Lovelace");
    let r = MatchingEngine::default().match_workers(&a, &a);
    let v: serde_json::Value = serde_json::to_value(&r).unwrap();
    assert_eq!(v["decision"], "match");
    assert_eq!(v["basis"], "fields");
    assert!(v["breakdown"]["email"].is_null());
    assert_eq!(v["breakdown"]["family_name"], 1.0);
}

#[test]
fn a_custom_nickname_class_is_used() {
    let a = WorkerRecord::new()
        .with_given_name("Siân")
        .with_family_name("Rees")
        .with_email("s@x.org");
    let b = WorkerRecord::new()
        .with_given_name("Shan")
        .with_family_name("Rees")
        .with_email("s@x.org");
    let e = MatchingEngine::default()
        .with_nicknames(NicknameTable::empty().with_class(["Siân", "Shan"]));
    assert_eq!(e.match_workers(&a, &b).breakdown.given_name, Some(0.95));
}

#[test]
fn two_workers_who_share_only_a_surname_are_not_a_match() {
    let a = WorkerRecord::new()
        .with_given_name("Priya")
        .with_family_name("Patel")
        .with_date_of_birth(1985, 3, 2)
        .unwrap();
    let b = WorkerRecord::new()
        .with_given_name("Rahul")
        .with_family_name("Patel")
        .with_date_of_birth(1971, 9, 30)
        .unwrap();
    assert_eq!(
        MatchingEngine::default().match_workers(&a, &b).decision,
        Decision::NoMatch
    );
}

#[test]
fn the_engine_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MatchingEngine>();
    assert_send_sync::<WorkerRecord>();
}
