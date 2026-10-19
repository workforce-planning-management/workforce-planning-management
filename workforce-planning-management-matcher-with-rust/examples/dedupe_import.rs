//! Screen an incoming feed row against the workers already held.
//!
//! `cargo run --example dedupe_import`. All data here is synthetic.

use workforce_planning_management_matcher::{Decision, MatchConfig, MatchingEngine, WorkerRecord};

fn main() {
    let held = vec![
        WorkerRecord::new()
            .with_given_name("Katherine")
            .with_family_name("Johnson")
            .with_email("k.johnson@example.org")
            .with_worker_number("org:acme", "E-1001"),
        WorkerRecord::new()
            .with_given_name("Kevin")
            .with_family_name("Jonson")
            .with_email("kj@example.org"),
        WorkerRecord::new()
            .with_given_name("Maria")
            .with_family_name("Silva"),
    ];
    let incoming = WorkerRecord::new()
        .with_given_name("Kathy")
        .with_family_name("Johnson")
        .with_email("K.Johnson@Example.org");

    let engine = MatchingEngine::new(MatchConfig {
        min_fields: 2,
        ..MatchConfig::default()
    })
    .expect("a valid configuration");

    for ranked in engine.rank_one_to_many(&incoming, &held) {
        let r = &ranked.result;
        let verdict = match r.decision {
            Decision::Match => "same worker",
            Decision::Review => "a person should look",
            Decision::NoMatch => "different worker",
            Decision::Insufficient => "not enough to compare",
        };
        println!(
            "held #{}: {verdict} (score {}, {} fields, basis {:?})",
            ranked.index,
            r.score.map_or("—".to_string(), |s| format!("{s:.2}")),
            r.fields_compared,
            r.basis,
        );
    }
}
