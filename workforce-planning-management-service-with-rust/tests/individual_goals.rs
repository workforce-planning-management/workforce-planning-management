//! Individual goals under enforcement (WPM-R224 to WPM-R226), against the shipped reference policy: who proposes and agrees, who
//! reads, what is audited, a measure and a check-in, the review tie-in and its frozen snapshot, and the roll-up's floor.

use axa::testing::prelude::*;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::SigningKey;
use rusty_paseto::core::{Footer, Key, Paseto, PasetoAsymmetricPrivateKey, Payload, Public, V4};
use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;
use sha2::{Digest, Sha256};
use workforce_planning_management::app::App;

const ISSUER: &str = "authentication-service";
const AUDIENCE: &str = "main-x-service";
/// Throwaway Ed25519 seed — mints test tokens only, never a secret.
const SEED: [u8; 32] = [12; 32];
const REFERENCE_POLICY_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/config/abac-policy.reference.json"
);

fn keys_and_kid() -> (Value, String) {
    let public = SigningKey::from_bytes(&SEED).verifying_key().to_bytes();
    let kid = URL_SAFE_NO_PAD.encode(Sha256::digest(public));
    let keys = json!({
        "keys": [{ "kty": "OKP", "crv": "Ed25519", "use": "sig",
                   "kid": kid, "x": URL_SAFE_NO_PAD.encode(public) }]
    });
    (keys, kid)
}

fn sign_as(kid: &str, sub: &str, attrs: &[(&str, &[&str])]) -> String {
    let attrs_map: serde_json::Map<String, Value> = attrs
        .iter()
        .map(|(key, values)| ((*key).to_string(), json!(values)))
        .collect();
    let iat: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_secs()).ok())
        .expect("the clock is after 1970");
    let payload = json!({
        "sub": sub, "email": "person@example.com", "name": "Test Person",
        "iss": ISSUER, "aud": AUDIENCE, "exp": iat + 3_600, "iat": iat,
        "sid": "test-sid", "attrs": attrs_map,
    })
    .to_string();
    let keypair = SigningKey::from_bytes(&SEED).to_keypair_bytes();
    let key = Key::<64>::from(keypair);
    let private = PasetoAsymmetricPrivateKey::<V4, Public>::from(&key);
    let footer = format!(r#"{{"kid":"{kid}"}}"#);
    let mut builder = Paseto::<V4, Public>::builder();
    builder.set_payload(Payload::from(payload.as_str()));
    builder.set_footer(Footer::from(footer.as_str()));
    builder.try_sign(&private).expect("sign")
}

fn bearer(token: String) -> String {
    format!("Bearer {token}")
}

async fn worker(
    request: &axa::TestServer,
    svc: &str,
    org: &str,
    person: &str,
    number: &str,
    manager: Option<&str>,
) -> String {
    let created: Value = request
        .post("/api/workers")
        .add_header("authorization", svc.to_string())
        .json(&json!({
            "person_ref": format!("person:{person}"), "organization_ref": org,
            "worker_number": number, "display_name": format!("Person {number}"),
            "employment_type": "permanent", "department": "engineering",
            "job_title": "Engineer", "hired_on": "2020-01-05", "manager_pid": manager,
        }))
        .await
        .json();
    let pid = created["pid"].as_str().expect("worker pid").to_string();
    request
        .post(&format!("/api/workers/{pid}/status"))
        .add_header("authorization", svc.to_string())
        .json(&json!({ "to": "active" }))
        .await
        .assert_status_ok();
    pid
}

fn a_goal() -> Value {
    json!({
        "title": "Publish the runbook", "period_start": "2026-01-01", "period_end": "2026-12-31",
        "aligns_to": "objective:reliability",
        "measures": [
            { "name": "Pages", "kind": "number", "unit": "pages", "start_value": 0, "target_value": 40 },
            { "name": "Signed off", "kind": "milestone" },
        ],
    })
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test individual_goals -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn a_goal_is_agreed_followed_reviewed_and_frozen() {
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
    }
    let (me, boss, peer) = (
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
    );
    let as_me = bearer(sign_as(&kid, &me.to_string(), &[]));
    let as_boss = bearer(sign_as(&kid, &boss.to_string(), &[]));
    let as_peer = bearer(sign_as(&kid, &peer.to_string(), &[]));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let hr = bearer(sign_as(&kid, "hr-goals", &[("hr", &["true"])]));
    let payroll = bearer(sign_as(&kid, "pay-goals", &[("payroll", &["true"])]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        for person in ["hr-goals", "pay-goals"] {
            ctx.db
                .execute_unprepared(&format!(
                    "INSERT INTO organization_memberships (pid, person_ref, organization_ref, role, starts_on) \
                     VALUES ('{}', 'person:{person}', '{org}', 'hr_admin', '2023-01-05')",
                    uuid::Uuid::new_v4()
                ))
                .await
                .unwrap();
        }
        let boss_pid = worker(&request, &svc, &org, &boss.to_string(), "G-1", None).await;
        let mine = worker(&request, &svc, &org, &me.to_string(), "G-2", Some(&boss_pid)).await;
        worker(&request, &svc, &org, &peer.to_string(), "G-3", None).await;

        let bad = request
            .post("/api/me/goals")
            .add_header("authorization", as_me.clone())
            .json(&json!({ "title": "x", "period_start": "2026-02-01", "period_end": "2026-01-01" }))
            .await;
        assert_eq!(bad.status_code(), 422, "a period that ends first");
        let score = request
            .post("/api/me/goals")
            .add_header("authorization", as_me.clone())
            .json(&json!({ "title": "x", "period_start": "2026-01-01", "period_end": "2026-02-01",
                           "measures": [{ "name": "Mood", "kind": "score" }] }))
            .await;
        assert_eq!(score.status_code(), 422, "there is no free-form score");

        let goal: Value = request
            .post("/api/me/goals")
            .add_header("authorization", as_me.clone())
            .json(&a_goal())
            .await
            .json();
        let goal = goal["pid"].as_str().expect("goal pid").to_string();
        let agree = format!("/api/me/goals/{goal}/agree");

        let hr_write = request
            .post(&agree)
            .add_header("authorization", hr.clone())
            .await;
        assert_eq!(hr_write.status_code(), 404, "HR sees and does not agree");
        let own = request.post(&agree).add_header("authorization", as_me.clone()).await;
        assert_eq!(own.status_code(), 422, "the proposer cannot agree their own goal");
        let stranger = request.post(&agree).add_header("authorization", as_peer.clone()).await;
        assert_eq!(stranger.status_code(), 404);
        let early = request
            .post(&format!("/api/me/goals/{goal}/check-ins"))
            .add_header("authorization", as_me.clone())
            .json(&json!({ "status": "on_track" }))
            .await;
        assert_eq!(early.status_code(), 422, "agree it first");
        request.post(&agree).add_header("authorization", as_boss.clone()).await.assert_status_ok();

        let edit = request
            .put(&format!("/api/me/goals/{goal}"))
            .add_header("authorization", as_me.clone())
            .json(&a_goal())
            .await;
        assert_eq!(edit.status_code(), 422, "an agreed goal stays as agreed");

        // A manager proposes too, and then the person agrees.
        let theirs: Value = request
            .post(&format!("/api/me/reports/{mine}/goals"))
            .add_header("authorization", as_boss.clone())
            .json(&json!({ "title": "Mentor a joiner", "period_start": "2027-01-01", "period_end": "2027-06-30" }))
            .await
            .json();
        let theirs = theirs["pid"].as_str().unwrap();
        request
            .post(&format!("/api/me/goals/{theirs}/agree"))
            .add_header("authorization", as_me.clone())
            .await
            .assert_status_ok();
        let not_mine = request
            .post(&format!("/api/me/reports/{boss_pid}/goals"))
            .add_header("authorization", as_me.clone())
            .json(&a_goal())
            .await;
        assert_eq!(not_mine.status_code(), 404, "a report cannot propose upward");

        // Reads.
        let url = format!("/api/workers/{mine}/goals");
        let seen: Value = request.get(&url).add_header("authorization", as_me.clone()).await.json();
        let found = seen["goals"].as_array().unwrap();
        assert_eq!(found.len(), 2);
        let measure = found
            .iter()
            .find(|g| g["goal"]["pid"] == goal.as_str())
            .map(|g| g["measures"][0].clone())
            .unwrap();
        assert_eq!(measure["current_value"], Value::Null, "unknown stays unknown");
        assert!(measure.get("percent").is_none());
        for token in [&as_boss, &hr] {
            request.get(&url).add_header("authorization", token.clone()).await.assert_status_ok();
        }
        for (who, token) in [("a colleague", &as_peer), ("payroll", &payroll)] {
            let refused = request.get(&url).add_header("authorization", token.clone()).await;
            assert_eq!(refused.status_code(), 403, "{who}");
        }
        let team: Value = request
            .get("/api/me/team-goals")
            .add_header("authorization", as_boss.clone())
            .await
            .json();
        assert_eq!(team["team"][0]["display_name"], "Person G-2", "{team}");

        // A check-in updates its measure.
        request
            .post(&format!("/api/me/goals/{goal}/check-ins"))
            .add_header("authorization", as_me.clone())
            .json(&json!({ "status": "at_risk", "note": "Waiting on review",
                           "measure_pid": measure["pid"], "current_value": 12 }))
            .await
            .assert_status_ok();
        request
            .post(&format!("/api/me/goals/{goal}/status"))
            .add_header("authorization", as_me.clone())
            .json(&json!({ "to": "in_progress" }))
            .await
            .assert_status_ok();

        // The review tie-in.
        let cycle: Value = request
            .post("/api/review-cycles")
            .add_header("authorization", hr.clone())
            .json(&json!({ "name": "Mid year", "period_start": "2026-04-01", "period_end": "2026-09-30",
                           "organization_ref": org }))
            .await
            .json();
        let review: Value = request
            .post(&format!("/api/review-cycles/{}/reviews", cycle["pid"].as_str().expect("cycle pid")))
            .add_header("authorization", hr.clone())
            .json(&json!({ "worker_pid": mine, "reviewer_ref": format!("worker:{boss_pid}") }))
            .await
            .json();
        let review = review["pid"].as_str().expect("review pid").to_string();
        let goals_url = format!("/api/reviews/{review}/individual-goals");
        let listed: Value = request.get(&goals_url).add_header("authorization", as_boss.clone()).await.json();
        assert_eq!(listed["goals"].as_array().map(Vec::len), Some(1), "only the goal in the period: {listed}");
        let refused = request.get(&goals_url).add_header("authorization", payroll.clone()).await;
        assert_eq!(refused.status_code(), 403, "payroll");

        let outcome = format!("/api/me/reviews/{review}/goals/{goal}/outcome");
        request
            .put(&outcome)
            .add_header("authorization", as_boss.clone())
            .json(&json!({ "outcome": "partly_achieved" }))
            .await
            .assert_status_ok();
        request
            .put(&outcome)
            .add_header("authorization", as_me.clone())
            .json(&json!({ "outcome": "achieved" }))
            .await
            .assert_status_ok();
        let outsider = request
            .put(&outcome)
            .add_header("authorization", as_peer.clone())
            .json(&json!({ "outcome": "achieved" }))
            .await;
        assert_eq!(outsider.status_code(), 404);
        let listed: Value = request.get(&goals_url).add_header("authorization", as_me.clone()).await.json();
        assert_eq!(listed["goals"][0]["reviewer_outcome"], "partly_achieved");
        assert_eq!(listed["goals"][0]["reviewee_outcome"], "achieved");

        for to in ["submitted", "calibrated", "shared"] {
            request
                .post(&format!("/api/reviews/{review}/status"))
                .add_header("authorization", hr.clone())
                .json(&json!({ "to": to }))
                .await
                .assert_status_ok();
        }
        request
            .post(&format!("/api/me/goals/{goal}/check-ins"))
            .add_header("authorization", as_me.clone())
            .json(&json!({ "status": "on_track", "measure_pid": measure["pid"], "current_value": 40 }))
            .await
            .assert_status_ok();
        let frozen: Value = request.get(&goals_url).add_header("authorization", as_me.clone()).await.json();
        assert_eq!(frozen["frozen"], true);
        let snapshot = &frozen["goals"][0]["snapshot"];
        assert_eq!(snapshot["status"], "in_progress", "{frozen}");
        assert_eq!(snapshot["measures"][0]["current_value"], 12, "later edits do not reach it");
        assert_eq!(frozen["goals"][0]["reviewee_outcome"], "achieved");
        let late = request
            .put(&outcome)
            .add_header("authorization", as_me.clone())
            .json(&json!({ "outcome": "not_achieved" }))
            .await;
        assert_eq!(late.status_code(), 422, "a shared review is frozen");

        // Rights.
        let export: Value = request
            .get(&format!("/api/workers/{mine}/subject-access"))
            .add_header("authorization", svc.clone())
            .await
            .json();
        assert_eq!(export["worker_goals"].as_array().map(Vec::len), Some(2));
        assert_eq!(export["review_goal_outcomes"].as_array().map(Vec::len), Some(1));
        ctx.db
            .execute_unprepared(&format!(
                "UPDATE workers SET status = 'terminated', terminated_on = '2026-11-01' WHERE pid = '{mine}'"
            ))
            .await
            .unwrap();
        request
            .post(&format!("/api/workers/{mine}/erase"))
            .add_header("authorization", svc.clone())
            .json(&json!({}))
            .await
            .assert_status_ok();
        let left = ctx
            .db
            .query_one_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!(
                    "SELECT 1 FROM goal_measures WHERE worker_pid = '{mine}' UNION ALL \
                     SELECT 1 FROM goal_check_ins WHERE worker_pid = '{mine}' UNION ALL \
                     SELECT 1 FROM review_goal_outcomes WHERE worker_pid = '{mine}' UNION ALL \
                     SELECT 1 FROM worker_goals WHERE worker_pid = '{mine}'"
                ),
            ))
            .await
            .unwrap();
        assert!(left.is_none(), "erasure leaves nothing");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test individual_goals -- --ignored`"]
async fn the_rollup_counts_by_department_with_a_floor() {
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
    }
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let hr = bearer(sign_as(&kid, "hr-rollup", &[("hr", &["true"])]));
    let plain = bearer(sign_as(&kid, "plain-goals", &[]));
    let people: Vec<uuid::Uuid> = (0..6).map(|_| uuid::Uuid::new_v4()).collect();

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        ctx.db
            .execute_unprepared(&format!(
                "INSERT INTO organization_memberships (pid, person_ref, organization_ref, role, starts_on) \
                 VALUES ('{}', 'person:hr-rollup', '{org}', 'hr_admin', '2023-01-05')",
                uuid::Uuid::new_v4()
            ))
            .await
            .unwrap();
        for (n, person) in people.iter().enumerate() {
            worker(
                &request,
                &svc,
                &org,
                &person.to_string(),
                &format!("R-{n}"),
                None,
            )
            .await;
            request
                .post("/api/me/goals")
                .add_header(
                    "authorization",
                    bearer(sign_as(&kid, &person.to_string(), &[])),
                )
                .json(&a_goal())
                .await
                .assert_status_ok();
        }
        let url = format!("/api/goals/rollup?organization={}", org.replace(':', "%3A"));
        let view: Value = request
            .get(&url)
            .add_header("authorization", hr.clone())
            .await
            .json();
        assert_eq!(view["departments"][0]["by_status"]["proposed"], 6, "{view}");
        let refused = request
            .get(&url)
            .add_header("authorization", plain.clone())
            .await;
        assert_eq!(refused.status_code(), 403);
    })
    .await;
}

// WPM-R233, WPM-R234: a goal names a skill and a target level, shows the declared level beside the target, keeps the target through an edit
// that does not send it, and refuses a target with no skill. Evidence never moves the goal (WPM-D180).
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_skill_goal_shows_the_declared_level_beside_its_target() {
    let (keys, kid) = keys_and_kid();
    let me = uuid::Uuid::new_v4();
    let as_me = bearer(sign_as(&kid, &me.to_string(), &[]));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
    }
    {
        request::<App, _, _>(|request, _ctx| async move {
            let org = format!("organization:{}", uuid::Uuid::new_v4());
            let mine = worker(&request, &svc, &org, &me.to_string(), "GS-1", None).await;
            let skill: Value = request
                .post("/api/skills")
                .add_header("authorization", svc.clone())
                .json(&json!({ "name": format!("GS {}", uuid::Uuid::new_v4()), "category": "technical" }))
                .await
                .json();
            let skill = skill["pid"].as_str().unwrap().to_string();
            let body = |level: Value| {
                json!({ "title": "Reach it", "period_start": "2026-01-01", "period_end": "2026-12-31",
                        "skill_pid": skill, "skill_target_level": level })
            };
            let post = |payload: Value| {
                let (request, as_me) = (&request, as_me.clone());
                async move {
                    request.post("/api/me/goals").add_header("authorization", as_me).json(&payload).await
                }
            };
            assert_eq!(post(body(json!(6))).await.status_code(), 422, "the scale stops at five");
            let no_skill = json!({ "title": "x", "period_start": "2026-01-01", "period_end": "2026-02-01", "skill_target_level": 3 });
            assert_eq!(post(no_skill).await.status_code(), 422, "a target needs a skill");

            let goal: Value = post(body(json!(3))).await.json();
            let pid = goal["pid"].as_str().unwrap().to_string();
            let evidence = || {
                let (request, as_me, mine) = (&request, as_me.clone(), mine.clone());
                async move {
                    let read: Value = request
                        .get(&format!("/api/workers/{mine}/goals"))
                        .add_header("authorization", as_me)
                        .await
                        .json();
                    read["goals"][0]["skill_evidence"].clone()
                }
            };
            let before = evidence().await;
            assert_eq!(before["state"], "undeclared", "unknown is not short");
            assert!(before["declared"].is_null());

            request
                .put(&format!("/api/workers/{mine}/skills"))
                .add_header("authorization", svc.clone())
                .json(&json!({ "skill_pid": skill, "proficiency": 2 }))
                .await
                .assert_status_ok();
            assert_eq!(evidence().await, json!({ "target": 3, "declared": 2, "state": "short" }));

            // An edit from a form that does not show the target keeps it.
            request
                .put(&format!("/api/me/goals/{pid}"))
                .add_header("authorization", as_me.clone())
                .json(&json!({ "title": "Reach it soon", "period_start": "2026-01-01", "period_end": "2026-12-31", "skill_pid": skill }))
                .await
                .assert_status_ok();
            assert_eq!(evidence().await["target"], 3);

            request
                .put(&format!("/api/workers/{mine}/skills"))
                .add_header("authorization", svc.clone())
                .json(&json!({ "skill_pid": skill, "proficiency": 3 }))
                .await
                .assert_status_ok();
            assert_eq!(evidence().await["state"], "reached");
            let read: Value = request
                .get(&format!("/api/workers/{mine}/goals"))
                .add_header("authorization", as_me.clone())
                .await
                .json();
            assert_eq!(read["goals"][0]["goal"]["status"], "proposed", "evidence does not move a goal");
        })
        .await;
    }
}
