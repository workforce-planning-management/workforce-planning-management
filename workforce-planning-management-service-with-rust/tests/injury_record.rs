//! The injury record under enforcement (WPM-R221, WPM-D179), against the shipped reference policy: HR only, no description in any
//! field, each report a row of its own, audited with no content, and erasure that keeps the proof and drops the person.

use axa_test::prelude::*;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::SigningKey;
use rusty_paseto::core::{Footer, Key, Paseto, PasetoAsymmetricPrivateKey, Payload, Public, V4};
use serde_json::{Value, json};
use serial_test::serial;
use sha2::{Digest, Sha256};
use sqlx::Row;
use workforce_planning_management::app::App;

const ISSUER: &str = "authentication-service";
const AUDIENCE: &str = "main-x-service";
/// Throwaway Ed25519 seed — mints test tokens only, never a secret.
const SEED: [u8; 32] = [13; 32];
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

async fn seat(ctx: &axa::AppContext, person: &str, org: &str, role: &str) {
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "INSERT INTO organization_memberships (pid, person_ref, organization_ref, role, starts_on) \
             VALUES ('{}', 'person:{person}', '{org}', '{role}', '2023-01-05')",
        uuid::Uuid::new_v4()
    )))
    .execute(workforce_planning_management::db::pool(ctx))
    .await
    .unwrap();
}

async fn worker(
    request: &axa_test::TestServer,
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
            "worker_number": number, "display_name": format!("=Person {number}"),
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

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test injury_record -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn an_injury_record_proves_a_report_and_holds_no_description() {
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
    }
    let (me, boss) = (uuid::Uuid::new_v4(), uuid::Uuid::new_v4());
    let as_me = bearer(sign_as(&kid, &me.to_string(), &[]));
    let as_boss = bearer(sign_as(&kid, &boss.to_string(), &[]));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let hr = bearer(sign_as(&kid, "hr-inj", &[("hr", &["true"])]));
    let hr_elsewhere = bearer(sign_as(&kid, "hr-off", &[("hr", &["true"])]));
    let payroll = bearer(sign_as(&kid, "pay-inj", &[("payroll", &["true"])]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        let other_org = format!("organization:{}", uuid::Uuid::new_v4());
        seat(&ctx, "hr-inj", &org, "hr_admin").await;
        seat(&ctx, "pay-inj", &org, "member").await;
        seat(&ctx, &boss.to_string(), &org, "member").await;
        seat(&ctx, &me.to_string(), &org, "member").await;
        let boss_pid = worker(&request, &svc, &org, &boss.to_string(), "I-1", None).await;
        let mine = worker(&request, &svc, &org, &me.to_string(), "I-2", Some(&boss_pid)).await;
        let away = worker(&request, &svc, &other_org, &uuid::Uuid::new_v4().to_string(), "I-3", None).await;
        let incident = json!({
            "organization": org, "jurisdiction": "US-CA", "kind": "accident", "worker_pid": mine,
            "occurred_at": "2026-03-04T09:00:00Z", "learned_at": "2026-03-04T10:30:00Z",
            "class": "absence", "absence_days": 4, "statutory_record_ref": "doc:osha-301/77",
            "keep_until": "2032-01-01",
        });
        let post = |who: String, body: Value| {
            let request = &request;
            async move {
                request.post("/api/injury-records").add_header("authorization", who).json(&body).await
            }
        };

        // Only HR writes: not the manager, payroll, the person or HR elsewhere.
        for (who, want) in [(&as_boss, 403), (&payroll, 403), (&as_me, 403), (&hr_elsewhere, 404)] {
            assert_eq!(post(who.clone(), incident.clone()).await.status_code(), want);
        }
        let created = post(hr.clone(), incident.clone()).await;
        created.assert_status_ok();
        let pid = created.json::<Value>()["pid"].as_str().unwrap().to_string();

        // A description has nowhere to go: no field for it, and a reference cannot hold a sentence.
        for field in ["description", "narrative", "injury", "body_part", "diagnosis", "notes"] {
            let mut body = incident.clone();
            body[field] = json!("fell from a ladder and broke an arm");
            assert_eq!(post(hr.clone(), body).await.status_code(), 422, "{field}");
        }
        let mut sentence = incident.clone();
        sentence["statutory_record_ref"] = json!("fell from a ladder");
        assert_eq!(post(hr.clone(), sentence).await.status_code(), 422);

        // A dangerous occurrence hurts nobody and names nobody; a worker elsewhere cannot be named.
        let occurrence = post(hr.clone(), json!({
            "organization": org, "jurisdiction": "SE", "kind": "dangerous_occurrence",
            "occurred_at": "2026-03-05T09:00:00Z", "class": "no_injury", "people_affected": 0,
        })).await;
        occurrence.assert_status_ok();
        let mut stranger = incident.clone();
        stranger["worker_pid"] = json!(away);
        assert_eq!(post(hr.clone(), stranger).await.status_code(), 422);
        let mut tomorrow = incident.clone();
        tomorrow["occurred_at"] = json!("2999-01-01T00:00:00Z");
        assert_eq!(post(hr.clone(), tomorrow).await.status_code(), 422);

        // One incident owes several reports, each standing on its own times.
        let reports = format!("/api/injury-records/{pid}/reports");
        let add = |body: Value, who: String| {
            let (request, reports) = (&request, reports.clone());
            async move { request.post(&reports).add_header("authorization", who).json(&body).await }
        };
        assert_eq!(add(json!({ "recipient": "insurer" }), as_boss.clone()).await.status_code(), 403);
        assert_eq!(add(json!({ "recipient": "insurer", "note": "he slipped" }), hr.clone()).await.status_code(), 422);
        assert_eq!(add(json!({ "recipient": "the mayor" }), hr.clone()).await.status_code(), 422);
        assert_eq!(
            add(json!({ "recipient": "insurer", "reference": "called on Tuesday" }), hr.clone()).await.status_code(),
            422
        );
        let sent = add(json!({
            "recipient": "safety_authority", "due_at": "2026-03-04T18:30:00Z", "sent_at": "2026-03-04T17:00:00Z",
            "method": "phone", "reference": "CAL-OSHA-991",
        }), hr.clone()).await;
        sent.assert_status_ok();
        let owed = add(json!({ "recipient": "insurer", "due_at": "2026-03-09T10:30:00Z" }), hr.clone()).await;
        owed.assert_status_ok();
        let owed = owed.json::<Value>()["pid"].as_str().unwrap().to_string();
        add(json!({ "recipient": "labour_inspectorate" }), hr.clone()).await.assert_status_ok();

        // The list is HR's, and names the worker to HR.
        let url = format!("/api/injury-records?organization={org}");
        for (who, want) in [(&as_boss, 403), (&payroll, 403), (&as_me, 403), (&hr_elsewhere, 404), (&hr, 200)] {
            assert_eq!(request.get(&url).add_header("authorization", who.clone()).await.status_code(), want);
        }
        let list: Value = request.get(&url).add_header("authorization", hr.clone()).await.json();
        assert_eq!(list.as_array().unwrap().len(), 2);
        let row = list.as_array().unwrap().iter().find(|r| r["pid"] == pid.as_str()).unwrap();
        assert_eq!(row["worker_name"], "=Person I-2");
        let standings: Vec<&str> =
            row["reports"].as_array().unwrap().iter().map(|r| r["standing"].as_str().unwrap()).collect();
        assert_eq!(standings, ["sent_by_due", "past_due", "no_due"]);

        // A report goes, and is said to have gone; a case moves on.
        let mark = format!("/api/injury-reports/{owed}");
        let late = json!({ "sent_at": "2026-03-10T09:00:00Z", "method": "online", "reference": "CLM-1" });
        let refused = request.put(&mark).add_header("authorization", as_boss.clone()).json(&late).await;
        assert_eq!(refused.status_code(), 403);
        request.put(&mark).add_header("authorization", hr.clone()).json(&late).await.assert_status_ok();
        let change = json!({ "class": "serious", "absence_days": 9, "people_affected": 1, "learned_at": "2026-03-04T10:30:00Z" });
        let path = format!("/api/injury-records/{pid}");
        let refused = request.put(&path).add_header("authorization", payroll.clone()).json(&change).await;
        assert_eq!(refused.status_code(), 403);
        request.put(&path).add_header("authorization", hr.clone()).json(&change).await.assert_status_ok();
        let list: Value = request.get(&url).add_header("authorization", hr.clone()).await.json();
        let row = list.as_array().unwrap().iter().find(|r| r["pid"] == pid.as_str()).unwrap();
        assert_eq!(row["class"], "serious");
        assert_eq!(row["reports"][1]["standing"], "sent_after_due");

        // The roll-up names no one, and under the floor says nothing: one serious case is someone.
        let rollup = format!("/api/injury-records/rollup?organization={org}&year=2026");
        for (who, want) in [(&as_boss, 403), (&as_me, 403), (&hr_elsewhere, 404), (&payroll, 200), (&hr, 200)] {
            assert_eq!(request.get(&rollup).add_header("authorization", who.clone()).await.status_code(), want);
        }
        let counts: Value = request.get(&rollup).add_header("authorization", payroll.clone()).await.json();
        assert_eq!(counts["total"], Value::Null);
        assert!(counts["classes"].as_array().unwrap().iter().any(|c| c["class"] == "serious" && c["count"].is_null()));
        assert!(!counts.to_string().contains(&pid) && !counts.to_string().contains("Person"));

        // The year is the calendar year in UTC: the last minutes of 2024 are 2024, and no other year's.
        let floor = workforce_planning_management::rules::equality_monitoring::DEFAULT_FLOOR;
        for _ in 0..floor {
            let at_year_end = json!({
                "organization": org, "jurisdiction": "US-CA", "kind": "accident",
                "occurred_at": "2024-12-31T23:30:00Z", "class": "no_absence", "people_affected": 1,
            });
            request.post("/api/injury-records").add_header("authorization", hr.clone()).json(&at_year_end).await.assert_status_ok();
        }
        let year = |year: i32| {
            let (request, payroll, org) = (&request, payroll.clone(), org.clone());
            async move {
                request
                    .get(&format!("/api/injury-records/rollup?organization={org}&year={year}"))
                    .add_header("authorization", payroll)
                    .await
                    .json::<Value>()
            }
        };
        assert_eq!(year(2024).await["total"], json!(floor));
        assert_eq!(year(2025).await["total"], json!(0));

        // Audited with no content: the action names nothing, and the read holds a count alone.
        let scalar = |sql: String| {
            let ctx = &ctx;
            async move {
                sqlx::query(sqlx::AssertSqlSafe(&*sql)).fetch_optional(workforce_planning_management::db::pool(ctx))
                    .await
                    .unwrap()
                    .unwrap()
                    .try_get::<i64, _>(0)
                    .unwrap()
            }
        };
        assert_eq!(
            scalar(format!("SELECT count(*) FROM audit_logs WHERE entity_pid = '{pid}' AND action = 'injury_recorded' AND snapshot IS NULL")).await,
            1
        );
        assert!(scalar("SELECT count(*) FROM audit_logs WHERE action = 'injury_records_read'".to_string()).await >= 3);

        // In the person's export; erasure takes the person and keeps the proof.
        let export: Value = request
            .get(&format!("/api/workers/{mine}/subject-access"))
            .add_header("authorization", svc.clone())
            .await
            .json();
        assert!(export["injury_records"].to_string().contains(&pid), "{export}");
        for to in ["offboarding", "terminated"] {
            request
                .post(&format!("/api/workers/{mine}/status"))
                .add_header("authorization", svc.clone())
                .json(&json!({ "to": to }))
                .await
                .assert_status_ok();
        }
        request
            .post(&format!("/api/workers/{mine}/erase"))
            .add_header("authorization", svc.clone())
            .await
            .assert_status_ok();
        assert_eq!(
            scalar(format!("SELECT count(*) FROM injury_records WHERE pid = '{pid}' AND worker_pid IS NULL")).await,
            1
        );
        assert_eq!(scalar(format!("SELECT count(*) FROM injury_reports WHERE record_pid = '{pid}'")).await, 3);
    })
    .await;
}
