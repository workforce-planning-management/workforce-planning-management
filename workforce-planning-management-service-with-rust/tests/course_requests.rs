//! Starting a course (WPM-R236, WPM-D182) on PostgreSQL under the reference policy: a recorded zero enrols at once, a fee or an unknown
//! fee goes to a manager above the person, and only that manager decides.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::SigningKey;
use loco_rs::testing::prelude::*;
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
    request: &loco_rs::TestServer,
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

async fn course(
    request: &loco_rs::TestServer,
    svc: &str,
    skill: &str,
    tag: &str,
    fee: Option<i64>,
) -> String {
    let course_ref = format!("course:{tag}-{}", uuid::Uuid::new_v4());
    let mut body =
        json!({ "course_ref": course_ref, "title": format!("Course {tag}"), "hours": 8 });
    if let Some(fee) = fee {
        body["fee_minor"] = json!(fee);
        body["fee_currency"] = json!("GBP");
    }
    request
        .post(&format!("/api/skills/{skill}/courses"))
        .add_header("authorization", svc.to_string())
        .json(&body)
        .await
        .assert_status_ok();
    course_ref
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test course_requests -- --ignored`"]
async fn a_free_course_starts_at_once_and_any_other_waits_for_a_manager() {
    let (keys, kid) = keys_and_kid();
    let (boss, me, other) = (
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
    );
    let as_boss = bearer(sign_as(&kid, &boss.to_string(), &[]));
    let as_me = bearer(sign_as(&kid, &me.to_string(), &[]));
    let as_other = bearer(sign_as(&kid, &other.to_string(), &[]));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
    }
    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        let manager = worker(&request, &svc, &org, &boss.to_string(), "CR-1", None).await;
        let mine = worker(&request, &svc, &org, &me.to_string(), "CR-2", Some(&manager)).await;
        worker(&request, &svc, &org, &other.to_string(), "CR-3", None).await;
        let skill: Value = request
            .post("/api/skills")
            .add_header("authorization", svc.clone())
            .json(&json!({ "name": format!("CR {}", uuid::Uuid::new_v4()), "category": "technical" }))
            .await
            .json();
        let skill = skill["pid"].as_str().unwrap().to_string();
        let free = course(&request, &svc, &skill, "free", Some(0)).await;
        let paid = course(&request, &svc, &skill, "paid", Some(12_500)).await;
        let unpriced = course(&request, &svc, &skill, "unpriced", None).await;

        let start = |course_ref: String| {
            let (request, as_me) = (&request, as_me.clone());
            async move {
                request
                    .post("/api/me/course-requests")
                    .add_header("authorization", as_me)
                    .json(&json!({ "course_ref": course_ref }))
                    .await
            }
        };
        let courses_of_mine = || {
            let (request, as_me, mine) = (&request, as_me.clone(), mine.clone());
            async move {
                let rows: Value = request
                    .get(&format!("/api/workers/{mine}/training-enrollments"))
                    .add_header("authorization", as_me)
                    .await
                    .json();
                rows.as_array().unwrap().iter().map(|r| r["course_ref"].as_str().unwrap().to_string()).collect::<Vec<_>>()
            }
        };

        assert_eq!(start("course:nowhere".to_string()).await.status_code(), 422, "catalogue courses only");

        let first: Value = start(free.clone()).await.json();
        assert_eq!(first["started"]["outcome"], "enrolled", "a recorded zero starts at once");
        assert_eq!(courses_of_mine().await, vec![free.clone()]);
        assert_eq!(start(free.clone()).await.status_code(), 409, "already enrolled");

        let asked: Value = start(paid.clone()).await.json();
        assert_eq!(asked["started"]["outcome"], "requested", "a fee waits");
        assert_eq!(asked["has_manager"], true);
        let paid_pid = asked["started"]["pid"].as_str().unwrap().to_string();
        assert_eq!(start(paid.clone()).await.status_code(), 409, "one open request");
        let asked: Value = start(unpriced.clone()).await.json();
        assert_eq!(asked["started"]["outcome"], "requested", "unknown is not free");
        let unpriced_pid = asked["started"]["pid"].as_str().unwrap().to_string();
        assert_eq!(courses_of_mine().await.len(), 1, "nothing is enrolled by asking");

        let told = |kind: &'static str, whom: String| {
            let ctx = ctx.clone();
            async move {
                ctx.db
                    .query_all_raw(sea_orm::Statement::from_string(
                        sea_orm::DatabaseBackend::Postgres,
                        format!("SELECT id FROM notifications WHERE kind = '{kind}' AND worker_pid = '{whom}'"),
                    ))
                    .await
                    .unwrap()
                    .len()
            }
        };
        assert_eq!(told("course_requested", manager.clone()).await, 2);

        let act = |pid: String, verb: &'static str, who: String| {
            let request = &request;
            async move {
                request
                    .post(&format!("/api/me/course-requests/{pid}/{verb}"))
                    .add_header("authorization", who)
                    .await
                    .status_code()
            }
        };
        assert_eq!(act(paid_pid.clone(), "approve", as_other.clone()).await, 404, "not their manager");
        assert_eq!(act(paid_pid.clone(), "approve", as_me.clone()).await, 404, "nobody approves their own");
        assert_eq!(act(paid_pid.clone(), "withdraw", as_other.clone()).await, 404, "not theirs");

        let waiting: Value = request
            .get("/api/me/team-course-requests")
            .add_header("authorization", as_boss.clone())
            .await
            .json();
        let waiting = waiting["waiting"].as_array().unwrap();
        assert_eq!(waiting.len(), 2);
        assert_eq!(waiting[0]["fee_minor"], 12_500);
        assert_eq!(waiting[0]["display_name"], "Person CR-2");
        assert!(waiting[1]["fee_minor"].is_null(), "unknown stays unknown");
        let none: Value = request
            .get("/api/me/team-course-requests")
            .add_header("authorization", as_other.clone())
            .await
            .json();
        assert_eq!(none["waiting"], json!([]));

        assert_eq!(act(paid_pid.clone(), "approve", as_boss.clone()).await, 200);
        assert!(courses_of_mine().await.contains(&paid), "approval enrols");
        assert_eq!(act(paid_pid.clone(), "approve", as_boss.clone()).await, 409, "settled once");
        assert_eq!(courses_of_mine().await.len(), 2, "no second enrolment");

        assert_eq!(act(unpriced_pid.clone(), "decline", as_boss.clone()).await, 200);
        assert_eq!(courses_of_mine().await.len(), 2, "a decline enrols nobody");
        assert_eq!(told("course_decided", mine.clone()).await, 2);

        let again: Value = start(unpriced.clone()).await.json();
        let again = again["started"]["pid"].as_str().unwrap().to_string();
        assert_eq!(act(again.clone(), "withdraw", as_me.clone()).await, 200);
        let read: Value = request
            .get("/api/me/course-requests")
            .add_header("authorization", as_me.clone())
            .await
            .json();
        let statuses: Vec<&str> = read["requests"].as_array().unwrap().iter().map(|r| r["status"].as_str().unwrap()).collect();
        assert_eq!(statuses, vec!["withdrawn", "declined", "approved"]);
        assert!(read["requests"][1]["decided_on"].is_string());
        assert_eq!(act(again, "approve", as_boss.clone()).await, 409, "withdrawn is settled");
    })
    .await;
}
