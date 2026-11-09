//! The separation record under enforcement (WPM-R222), against the shipped reference policy: built from the leaver record, HR only,
//! audited with no content, and an export with a row for every leaver in the period.

use axa_test::prelude::*;
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

async fn seat(ctx: &axa::AppContext, person: &str, org: &str) {
    ctx.db
        .execute_unprepared(&format!(
            "INSERT INTO organization_memberships (pid, person_ref, organization_ref, role, starts_on) \
             VALUES ('{}', 'person:{person}', '{org}', 'hr_admin', '2023-01-05')",
            uuid::Uuid::new_v4()
        ))
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

async fn leaver(request: &axa_test::TestServer, token: &str, worker: &str, day: &str) -> String {
    let opened: Value = request
        .post(&format!("/api/workers/{worker}/movements"))
        .add_header("authorization", token.to_string())
        .json(&json!({ "kind": "leaver", "effective_on": day, "reason": "redundancy" }))
        .await
        .json();
    opened["pid"]
        .as_str()
        .or_else(|| opened["movement"]["pid"].as_str())
        .unwrap_or_else(|| panic!("a movement: {opened}"))
        .to_string()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test separation_record -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn a_separation_record_is_built_from_the_leaver_record_for_hr_only() {
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
    let hr = bearer(sign_as(&kid, "hr-sep", &[("hr", &["true"])]));
    let hr_elsewhere = bearer(sign_as(&kid, "hr-away", &[("hr", &["true"])]));
    let payroll = bearer(sign_as(&kid, "pay-sep", &[("payroll", &["true"])]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        seat(&ctx, "hr-sep", &org).await;
        seat(&ctx, "pay-sep", &org).await;
        seat(&ctx, &boss.to_string(), &org).await;
        let boss_pid = worker(&request, &svc, &org, &boss.to_string(), "S-1", None).await;
        let mine = worker(&request, &svc, &org, &me.to_string(), "S-2", Some(&boss_pid)).await;
        let other = worker(&request, &svc, &org, &uuid::Uuid::new_v4().to_string(), "S-3", None).await;
        let movement = leaver(&request, &hr, &mine, "2027-03-31").await;
        let empty = leaver(&request, &hr, &other, "2027-03-15").await;
        let path = format!("/api/movements/{movement}/separation-record");

        // Who reads: HR, not the manager, payroll, the person or HR of another organization.
        for (who, want) in [(&as_boss, 403), (&payroll, 403), (&as_me, 403), (&hr_elsewhere, 404), (&hr, 200)] {
            let read = request.get(&path).add_header("authorization", who.clone()).await;
            assert_eq!(read.status_code(), want);
        }

        // The movement itself says why only to HR, on the one audited read (WPM-D178).
        let detail = format!("/api/movements/{movement}");
        for who in [&as_boss, &payroll, &hr] {
            for url in [
                detail.clone(),
                "/api/movements".to_string(),
                format!("/api/workers/{mine}/movements"),
            ] {
                let read = request.get(&url).add_header("authorization", who.clone()).await;
                read.assert_status_ok();
                let text = read.text();
                let told = who == &hr && url == detail;
                assert_eq!(text.contains("redundancy"), told, "{url}: {text}");
                assert_eq!(text.contains("\"notes\""), told, "{url}: {text}");
            }
        }
        let shown: Value = request.get(&detail).add_header("authorization", as_boss.clone()).await.json();
        assert_eq!(shown["effective_on"], "2027-03-31", "the dates stay for the people who work from them");

        // What the leaver record says comes through, and each gap is named.
        let view: Value = request.get(&path).add_header("authorization", hr.clone()).await.json();
        assert_eq!(view["last_day"], "2027-03-31");
        assert_eq!(view["reason_code"], "redundancy");
        assert_eq!(view["missing"], json!(["final_pay_on", "pay_summary_ref"]));

        let refused = request
            .put(&path)
            .add_header("authorization", as_boss.clone())
            .json(&json!({ "final_pay_on": "2027-04-25" }))
            .await;
        assert_eq!(refused.status_code(), 403);
        let long = request
            .put(&path)
            .add_header("authorization", hr.clone())
            .json(&json!({ "pay_summary_ref": "x".repeat(201) }))
            .await;
        assert_eq!(long.status_code(), 422);
        let done: Value = request
            .put(&path)
            .add_header("authorization", hr.clone())
            .json(&json!({ "final_pay_on": "2027-04-25", "pay_summary_ref": "payroll:run-42" }))
            .await
            .json();
        assert_eq!(done["missing"], json!([]));

        // A joiner has none, and a cancelled leaver has none.
        let cancelled = request
            .post(&format!("/api/movements/{empty}/cancel"))
            .add_header("authorization", hr.clone())
            .await;
        assert_eq!(cancelled.status_code(), 200);
        let gone = request
            .get(&format!("/api/movements/{empty}/separation-record"))
            .add_header("authorization", hr.clone())
            .await;
        assert_eq!(gone.status_code(), 404);

        // The export: everyone in the period, a gap named, a name that cannot run as a formula.
        let third = worker(&request, &svc, &org, &uuid::Uuid::new_v4().to_string(), "S-4", None).await;
        leaver(&request, &hr, &third, "2027-03-20").await;
        let url = format!("/api/separation-records/export?organization={org}&from=2027-03-01&to=2027-03-31");
        for who in [&as_boss, &payroll, &as_me] {
            let refused = request.get(&url).add_header("authorization", who.clone()).await;
            assert_eq!(refused.status_code(), 403);
        }
        let backwards = request
            .get(&format!("/api/separation-records/export?organization={org}&from=2027-04-01&to=2027-03-31"))
            .add_header("authorization", hr.clone())
            .await;
        assert_eq!(backwards.status_code(), 422);
        let csv = request.get(&url).add_header("authorization", hr.clone()).await;
        csv.assert_status_ok();
        let text = csv.text();
        let rows: Vec<&str> = text.trim_end().split("\r\n").collect();
        assert_eq!(rows.len(), 3, "{text}");
        assert!(rows[1].contains("S-4") && rows[1].contains("final_pay_on pay_summary_ref"), "{text}");
        assert!(rows[1].contains("'=Person S-4"), "{text}");
        assert!(rows[2].contains("2027-04-25,payroll:run-42,"), "{text}");
        assert!(!text.contains("S-3"), "a cancelled leaver is not in it");

        // Audited with no content, and in the person's own export.
        let audited = ctx
            .db
            .query_one_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!(
                    "SELECT count(*) FROM audit_logs WHERE entity_pid = '{movement}' AND action = 'separation_read' \
                     AND snapshot IS NULL"
                ),
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get_by_index::<i64>(0)
            .unwrap();
        assert!(audited > 0);
        let why_read = ctx
            .db
            .query_one_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!(
                    "SELECT count(*) FROM audit_logs WHERE entity_pid = '{movement}' AND action = 'movement_why_read' \
                     AND snapshot IS NULL"
                ),
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get_by_index::<i64>(0)
            .unwrap();
        assert_eq!(why_read, 1, "HR's one detail read, and nobody else's");
        let export: Value = request
            .get(&format!("/api/workers/{mine}/subject-access"))
            .add_header("authorization", svc.clone())
            .await
            .json();
        assert!(export.to_string().contains("payroll:run-42"), "{export}");

        // Erasure takes the record and why they left; the dates stay, as for any leaver.
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
        let left = ctx
            .db
            .query_one_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!(
                    "SELECT (SELECT count(*) FROM movements WHERE pid = '{movement}' AND reason IS NULL AND notes IS NULL) \
                     + 10 * (SELECT count(*) FROM separation_records WHERE movement_pid = '{movement}')"
                ),
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get_by_index::<i64>(0)
            .unwrap();
        assert_eq!(left, 1, "the reason cleared and the separation record gone");
    })
    .await;
}
