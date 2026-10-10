//! `/api/me` under enforcement (WPM-R124, WPM-R129, WPM-R130, WPM-D76).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s), against
//! the shipped reference policy, which lets only HR, service and administrator callers write. A
//! plain signed-in worker can still change their own contact details and emergency contacts
//! because those routes are on the explicit self-service list, and only those: every other write
//! stays refused, and nobody can touch another person's record through `/api/me`.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::SigningKey;
use loco_rs::testing::prelude::*;
use rusty_paseto::core::{Footer, Key, Paseto, PasetoAsymmetricPrivateKey, Payload, Public, V4};
use serde_json::{Value, json};
use serial_test::serial;
use sha2::{Digest, Sha256};
use workforce_planning_management_service::app::App;

const ISSUER: &str = "authentication-service";
const AUDIENCE: &str = "main-x-service";
/// Throwaway Ed25519 seed — mints test tokens only, never a secret.
const SEED: [u8; 32] = [9; 32];

/// The published-key-set JSON + `kid` for the test key.
fn keys_and_kid() -> (Value, String) {
    let public = SigningKey::from_bytes(&SEED).verifying_key().to_bytes();
    let kid = URL_SAFE_NO_PAD.encode(Sha256::digest(public));
    let keys = json!({
        "keys": [{ "kty": "OKP", "crv": "Ed25519", "use": "sig",
                   "kid": kid, "x": URL_SAFE_NO_PAD.encode(public) }]
    });
    (keys, kid)
}

/// Mint a PASETO `v4.public` with a given `sub` and ABAC attributes.
fn sign_as(kid: &str, sub: &str, attrs: &[(&str, &[&str])]) -> String {
    let attrs_map: serde_json::Map<String, Value> = attrs
        .iter()
        .map(|(key, values)| ((*key).to_string(), json!(values)))
        .collect();
    let iat: i64 = 1_700_000_000;
    let payload = json!({
        "sub": sub,
        "email": "person@example.com", "name": "Test Person",
        "iss": ISSUER, "aud": AUDIENCE,
        "exp": iat + 10_000_000_000_i64, "iat": iat,
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

/// The matrix runs against **the shipped reference policy file**
/// (`config/abac-policy.reference.json`, WPM-G1) — so what the
/// runbook tells a deployment to mount is exactly what is verified:
/// svc/admin everything; `payroll=true` unmasked read; `hr=true`
/// write + masked read; `$sub` self-read unmasked; masked-read
/// fallback for every other authenticated caller.
const REFERENCE_POLICY_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/config/abac-policy.reference.json"
);

/// One boot, the worker, a neighbour, a stranger with no record, and HR.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test enforcement_me -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn a_worker_manages_only_their_own_details() {
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
    }
    let (p1, p2, p3) = (
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
    );
    let bearer = |t: String| format!("Bearer {t}");
    let me = bearer(sign_as(&kid, &p1.to_string(), &[]));
    let neighbour = bearer(sign_as(&kid, &p2.to_string(), &[]));
    let no_record = bearer(sign_as(&kid, &p3.to_string(), &[]));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let hr = bearer(sign_as(&kid, "hr-user", &[("hr", &["true"])]));

    request::<App, _, _>(|request, _ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        let make = |person: uuid::Uuid, number: &'static str| {
            let request = &request;
            let (t, org) = (svc.clone(), org.clone());
            async move {
                let r = request.post("/api/workers").add_header("authorization", t.clone()).json(&json!({
                    "person_ref": format!("person:{person}"), "organization_ref": org,
                    "worker_number": number, "display_name": format!("Test Worker {number}"),
                    "employment_type": "permanent", "department": "engineering",
                    "job_title": "Engineer", "hired_on": "2023-01-05",
                })).await;
                assert_eq!(r.status_code(), 200, "{number}");
                let pid = r.json::<Value>()["pid"].as_str().unwrap().to_string();
                request.post(&format!("/api/workers/{pid}/status")).add_header("authorization", t)
                    .json(&json!({ "to": "active" })).await.assert_status_ok();
                pid
            }
        };
        let w1 = make(p1, "ME-1").await;
        let w2 = make(p2, "ME-2").await;
        let put = |url: String, token: String, body: Value| {
            let request = &request;
            async move { request.put(&url).add_header("authorization", token).json(&body).await }
        };

        // ── Contact details ─────────────────────────────────────────────────────────────
        let none: Value = request.get("/api/me/contact-details").add_header("authorization", me.clone()).await.json();
        assert_eq!(none["contact_details"], Value::Null);
        let good = json!({ "address_line1": "1 High Street", "city": "Leeds", "postcode": "LS1 4AP",
                           "country": "gb", "phone_mobile": "+44 7700 900123", "personal_email": "me@example.org" });
        // Without a token, or with a junk one, the write is refused as unauthenticated.
        assert_eq!(request.put("/api/me/contact-details").json(&good).await.status_code(), 401);
        assert_eq!(put("/api/me/contact-details".into(), "Bearer v4.public.junk".into(), good.clone()).await.status_code(), 401);
        // A signed-in person with NO worker record has nothing to change.
        assert_eq!(put("/api/me/contact-details".into(), no_record.clone(), good.clone()).await.status_code(), 404);
        // Bad shapes are refused, naming the field.
        for (field, value) in [("phone_mobile", "nope"), ("personal_email", "a@b"), ("country", "GBR"), ("postcode", "LS1;DROP")] {
            let mut bad = good.clone();
            bad[field] = json!(value);
            let r = put("/api/me/contact-details".into(), me.clone(), bad).await;
            assert_eq!(r.status_code(), 422, "{field}");
            assert!(r.text().contains(field), "{field} named");
        }
        // The worker writes their own, past a policy that lets only HR write.
        let saved = put("/api/me/contact-details".into(), me.clone(), good.clone()).await;
        assert_eq!(saved.status_code(), 200);
        assert_eq!(saved.json::<Value>()["contact_details"]["country"], "GB", "country is stored in capitals");
        // It is replaced as a whole: a field left out is cleared.
        let replaced: Value = put("/api/me/contact-details".into(), me.clone(), json!({ "city": "York" })).await.json();
        assert_eq!(replaced["contact_details"]["city"], "York");
        assert_eq!(replaced["contact_details"]["phone_mobile"], Value::Null);
        put("/api/me/contact-details".into(), me.clone(), good.clone()).await.assert_status_ok();

        // Everything else stays refused for the same person: the allow-list is not a loophole.
        assert_eq!(request.post("/api/me/contact-details").add_header("authorization", me.clone()).json(&good).await.status_code(), 403, "a POST is not on the list");
        assert_eq!(put(format!("/api/workers/{w1}"), me.clone(), json!({ "department": "board" })).await.status_code(), 403, "the HR route stays HR's");
        assert_eq!(
            request.post(&format!("/api/workers/{w1}/emergency-contacts")).add_header("authorization", me.clone())
                .json(&json!({ "name": "A", "relationship": "friend", "phone": "01632 960001" })).await.status_code(),
            403,
            "the HR contact route stays HR's"
        );

        // Who reads them: the worker and HR; a neighbour is refused, and HR's read is audited.
        let url = format!("/api/workers/{w1}/contact-details");
        assert_eq!(request.get(&url).add_header("authorization", neighbour.clone()).await.status_code(), 403);
        assert_eq!(request.get(&url).add_header("authorization", me.clone()).await.status_code(), 200);
        let hr_read: Value = request.get(&url).add_header("authorization", hr.clone()).await.json();
        assert_eq!(hr_read["contact_details"]["address_line1"], "1 High Street");
        let audits: Value = request.get(&format!("/api/audits/{w1}")).add_header("authorization", svc.clone()).await.json();
        let text = serde_json::to_string(&audits).unwrap();
        assert!(text.contains("contact_details_set") && text.contains("contact_details_read"));
        assert_eq!(text.matches("contact_details_read").count(), 1, "the worker's own read is not audited, HR's is");
        assert!(!text.contains("High Street") && !text.contains("example.org") && !text.contains("900123"), "no value in the audit");
        // The neighbour's own record is untouched by any of it.
        let theirs: Value = request.get("/api/me/contact-details").add_header("authorization", neighbour.clone()).await.json();
        assert_eq!(theirs["contact_details"], Value::Null);
        // Subject access includes it.
        // (The export is for the subject and unmasked callers, so the worker asks for their own.)
        let export: Value = request.get(&format!("/api/workers/{w1}/subject-access")).add_header("authorization", me.clone()).await.json();
        assert_eq!(export["contact_details"][0]["postcode"], "LS1 4AP");
        // Clear: once, then a 404.
        assert_eq!(request.delete("/api/me/contact-details").add_header("authorization", me.clone()).await.status_code(), 200);
        assert_eq!(request.delete("/api/me/contact-details").add_header("authorization", me.clone()).await.status_code(), 404);

        // ── Emergency contacts ──────────────────────────────────────────────────────────
        let contact = json!({ "name": "Test Contact", "relationship": "sibling", "phone": "01632 960001" });
        let added = request.post("/api/me/emergency-contacts").add_header("authorization", me.clone()).json(&contact).await;
        assert_eq!(added.status_code(), 200);
        let c1 = added.json::<Value>()["pid"].as_str().unwrap().to_string();
        let listed: Vec<Value> = request.get("/api/me/emergency-contacts").add_header("authorization", me.clone()).await.json();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0]["on_behalf"], false, "the worker wrote it themself");
        // A neighbour sees none of it and cannot change or remove it.
        let theirs: Vec<Value> = request.get("/api/me/emergency-contacts").add_header("authorization", neighbour.clone()).await.json();
        assert!(theirs.is_empty());
        assert_eq!(put(format!("/api/me/emergency-contacts/{c1}"), neighbour.clone(), json!({ "phone": "01632 960999" })).await.status_code(), 404);
        assert_eq!(request.delete(&format!("/api/me/emergency-contacts/{c1}")).add_header("authorization", neighbour.clone()).await.status_code(), 404);
        let changed = put(format!("/api/me/emergency-contacts/{c1}"), me.clone(), json!({ "phone": "01632 960002" })).await;
        assert_eq!(changed.status_code(), 200);
        assert_eq!(changed.json::<Value>()["phone"], "01632 960002");
        // The existing limit of five still holds.
        for n in 0..4 {
            let r = request.post("/api/me/emergency-contacts").add_header("authorization", me.clone())
                .json(&json!({ "name": format!("Test Contact {n}"), "relationship": "friend", "phone": "01632 960003" })).await;
            assert_eq!(r.status_code(), 200, "contact {n}");
        }
        let sixth = request.post("/api/me/emergency-contacts").add_header("authorization", me.clone()).json(&contact).await;
        assert_eq!(sixth.status_code(), 422);
        assert_eq!(request.delete(&format!("/api/me/emergency-contacts/{c1}")).add_header("authorization", me.clone()).await.status_code(), 200);
        // HR still reads them through the existing route.
        let hr_list: Vec<Value> = request.get(&format!("/api/workers/{w1}/emergency-contacts")).add_header("authorization", hr.clone()).await.json();
        assert_eq!(hr_list.len(), 4);

        // ── My time-off ─────────────────────────────────────────────────────────────────
        let post = |path: String, body: Value| {
            let request = &request;
            let t = svc.clone();
            async move {
                let r = request.post(&path).add_header("authorization", t).json(&body).await;
                assert_eq!(r.status_code(), 200, "{path}");
                r.json::<Value>()
            }
        };
        for (kind, year, days) in [("annual", 2024, 25), ("sick", 2024, 30), ("annual", 2099, 25)] {
            post(format!("/api/workers/{w1}/leave-entitlements"), json!({ "kind": kind, "year": year, "entitled_days": days })).await;
        }
        let approve = |pid: String| {
            let request = &request;
            let t = svc.clone();
            async move { request.post(&format!("/api/leave-requests/{pid}/approve")).add_header("authorization", t).await.assert_status_ok(); }
        };
        let holiday = post(format!("/api/workers/{w1}/leave-requests"), json!({ "kind": "annual", "start_on": "2024-03-04", "end_on": "2024-03-06", "reason": "family holiday" })).await;
        approve(holiday["pid"].as_str().unwrap().to_string()).await;
        let sick = post(format!("/api/workers/{w1}/leave-requests"), json!({ "kind": "sick", "start_on": "2024-05-06", "end_on": "2024-05-07" })).await;
        approve(sick["pid"].as_str().unwrap().to_string()).await;
        let booked = post(format!("/api/workers/{w1}/leave-requests"), json!({ "kind": "annual", "start_on": "2099-06-01", "end_on": "2099-06-02" })).await;
        approve(booked["pid"].as_str().unwrap().to_string()).await;
        post(format!("/api/workers/{w1}/leave-requests"), json!({ "kind": "annual", "start_on": "2099-07-01", "end_on": "2099-07-01" })).await;

        let past: Value = request.get("/api/me/time-off?year=2024").add_header("authorization", me.clone()).await.json();
        assert_eq!(past["unit"], "calendar_days");
        let annual = past["balances"].as_array().unwrap().iter().find(|b| b["kind"] == "annual").unwrap();
        assert_eq!((annual["entitled"].as_i64(), annual["taken"].as_i64(), annual["remaining"].as_i64()), (Some(25), Some(3), Some(22)));
        let sick_b = past["balances"].as_array().unwrap().iter().find(|b| b["kind"] == "sick").unwrap();
        assert_eq!((sick_b["taken"].as_i64(), sick_b["remaining"].as_i64()), (Some(2), Some(28)));
        // History: the finished absences of 2024, sick leave as days only.
        let h = past["history"].as_array().unwrap().iter().find(|y| y["year"] == 2024).unwrap();
        let sick_h = h["absences"].as_array().unwrap().iter().find(|a| a["kind"] == "sick").unwrap();
        assert_eq!((sick_h["absences"].as_i64(), sick_h["days"].as_i64()), (Some(1), Some(2)));
        let reqs = past["requests"].as_array().unwrap();
        assert!(reqs.iter().any(|r| r["reason"] == "family holiday"), "the worker's own reason on annual leave is shown to them");
        assert!(reqs.iter().filter(|r| r["kind"] == "sick").all(|r| r["reason"].is_null()), "never a reason for sick leave");
        // The future year: booked and requested, with what would remain.
        let future: Value = request.get("/api/me/time-off?year=2099").add_header("authorization", me.clone()).await.json();
        let f = &future["balances"].as_array().unwrap()[0];
        assert_eq!((f["booked"].as_i64(), f["requested"].as_i64(), f["remaining"].as_i64(), f["remaining_if_requested_approved"].as_i64()), (Some(2), Some(1), Some(23), Some(22)));
        // A year with nothing recorded is empty, not zero-filled.
        let empty: Value = request.get("/api/me/time-off?year=1999").add_header("authorization", me.clone()).await.json();
        assert!(empty["balances"].as_array().unwrap().is_empty());
        // The neighbour sees nothing of it, and the response carries no worker id to ask for.
        let theirs: Value = request.get("/api/me/time-off?year=2024").add_header("authorization", neighbour.clone()).await.json();
        assert!(theirs["balances"].as_array().unwrap().is_empty() && theirs["history"].as_array().unwrap().is_empty());
        assert!(!serde_json::to_string(&past).unwrap().contains(&w1));
        // ── Requesting and cancelling leave ─────────────────────────────────────────────
        let mine = |path: String, token: String, body: Value| {
            let request = &request;
            async move { request.post(&path).add_header("authorization", token).json(&body).await }
        };
        // The HR route stays HR's; the worker uses their own.
        assert_eq!(
            mine(format!("/api/workers/{w1}/leave-requests"), me.clone(), json!({ "kind": "annual", "start_on": "2099-08-03", "end_on": "2099-08-04" })).await.status_code(),
            403
        );
        // The same checks apply: no reason on sick leave, a balance, a valid span.
        let refused = mine("/api/me/leave-requests".into(), me.clone(), json!({ "kind": "sick", "start_on": "2099-08-10", "end_on": "2099-08-10", "reason": "flu" })).await;
        assert_eq!(refused.status_code(), 422);
        assert_eq!(mine("/api/me/leave-requests".into(), me.clone(), json!({ "kind": "annual", "start_on": "2099-09-10", "end_on": "2099-12-30" })).await.status_code(), 422, "over the balance");
        assert_eq!(mine("/api/me/leave-requests".into(), no_record.clone(), json!({ "kind": "annual", "start_on": "2099-08-03", "end_on": "2099-08-04" })).await.status_code(), 404, "no worker record");
        let made = mine("/api/me/leave-requests".into(), me.clone(), json!({ "kind": "annual", "start_on": "2099-08-03", "end_on": "2099-08-04" })).await;
        assert_eq!(made.status_code(), 200);
        let r1 = made.json::<Value>()["pid"].as_str().unwrap().to_string();
        // It is the worker's own: a neighbour cannot cancel it, and sees it nowhere.
        assert_eq!(mine(format!("/api/me/leave-requests/{r1}/cancel"), neighbour.clone(), json!({})).await.status_code(), 404);
        // The worker cannot approve their own request: that is still a decision for HR.
        assert_eq!(mine(format!("/api/leave-requests/{r1}/approve"), me.clone(), json!({})).await.status_code(), 403);
        // Cancelling one awaiting a decision works; so does an approved one that has not started,
        // and the days come back.
        assert_eq!(mine(format!("/api/me/leave-requests/{r1}/cancel"), me.clone(), json!({})).await.status_code(), 200);
        assert_eq!(mine(format!("/api/me/leave-requests/{r1}/cancel"), me.clone(), json!({})).await.status_code(), 422, "already cancelled");
        let again = mine("/api/me/leave-requests".into(), me.clone(), json!({ "kind": "annual", "start_on": "2099-08-03", "end_on": "2099-08-04" })).await;
        let r2 = again.json::<Value>()["pid"].as_str().unwrap().to_string();
        approve(r2.clone()).await;
        let before: Value = request.get("/api/me/time-off?year=2099").add_header("authorization", me.clone()).await.json();
        let rem_before = before["balances"].as_array().unwrap()[0]["remaining"].as_i64().unwrap();
        assert_eq!(mine(format!("/api/me/leave-requests/{r2}/cancel"), me.clone(), json!({})).await.status_code(), 200);
        let after: Value = request.get("/api/me/time-off?year=2099").add_header("authorization", me.clone()).await.json();
        assert_eq!(after["balances"].as_array().unwrap()[0]["remaining"].as_i64().unwrap(), rem_before + 2, "the two days came back");
        // Leave that is over is HR's to change.
        assert_eq!(mine(format!("/api/me/leave-requests/{}/cancel", holiday["pid"].as_str().unwrap()), me.clone(), json!({})).await.status_code(), 422);

        // No token, no view.
        assert_eq!(request.get("/api/me/time-off").await.status_code(), 401);
        let _ = w2;
    })
    .await;
}

/// A resignation: logged by the worker, seen by the manager and HR without the reason, accepted by a
/// person into a leaver record, never ending employment by itself.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test enforcement_me -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn a_resignation_is_logged_by_the_worker_and_decided_by_a_person() {
    use sea_orm::ConnectionTrait;
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
    }
    let (worker_p, mgr_p, other_p, hr_p) = (
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
    );
    let bearer = |t: String| format!("Bearer {t}");
    let me = bearer(sign_as(&kid, &worker_p.to_string(), &[]));
    let manager = bearer(sign_as(&kid, &mgr_p.to_string(), &[]));
    let other = bearer(sign_as(&kid, &other_p.to_string(), &[]));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let hr = bearer(sign_as(&kid, "hr-user", &[("hr", &["true"])]));
    // HR who decides: the attribute that lets them write, and a membership in the organization.
    let hr_m = bearer(sign_as(&kid, &hr_p.to_string(), &[("hr", &["true"])]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        let make = |person: uuid::Uuid, number: &'static str, manager_pid: Option<String>| {
            let request = &request;
            let (t, org) = (svc.clone(), org.clone());
            async move {
                let mut body = json!({
                    "person_ref": format!("person:{person}"), "organization_ref": org,
                    "worker_number": number, "display_name": format!("Test Worker {number}"),
                    "employment_type": "permanent", "department": "engineering",
                    "job_title": "Engineer", "hired_on": "2023-01-05",
                });
                if let Some(m) = manager_pid { body["manager_pid"] = json!(m); }
                let r = request.post("/api/workers").add_header("authorization", t.clone()).json(&body).await;
                assert_eq!(r.status_code(), 200, "{number}");
                let pid = r.json::<Value>()["pid"].as_str().unwrap().to_string();
                request.post(&format!("/api/workers/{pid}/status")).add_header("authorization", t)
                    .json(&json!({ "to": "active" })).await.assert_status_ok();
                pid
            }
        };
        let w_mgr = make(mgr_p, "RS-M", None).await;
        let w = make(worker_p, "RS-W", Some(w_mgr.clone())).await;
        let _ = make(other_p, "RS-O", None).await;
        let w_hr = make(hr_p, "RS-H", None).await;
        for (person, wp) in [(worker_p, &w), (mgr_p, &w_mgr), (hr_p, &w_hr)] {
            ctx.db.execute_unprepared(&format!(
                "INSERT INTO organization_memberships (pid, person_ref, organization_ref, worker_pid, role, starts_on) \
                 VALUES ('{}', 'person:{person}', '{org}', '{wp}', '{role}', '2023-01-05')", uuid::Uuid::new_v4(), role = if person == hr_p { "hr_admin" } else { "member" })).await.unwrap();
        }
        let today = chrono::Utc::now().date_naive();
        let day = |n: i64| (today + chrono::Duration::days(n)).to_string();
        let post = |path: String, token: String, body: Value| {
            let request = &request;
            async move { request.post(&path).add_header("authorization", token).json(&body).await }
        };

        // Nothing logged yet.
        let none: Value = request.get("/api/me/resignation").add_header("authorization", me.clone()).await.json();
        assert_eq!(none["resignation"], Value::Null);
        // The notice (28 calendar days by default) is enforced; a reason must be on the list.
        let early = post("/api/me/resignation".into(), me.clone(), json!({ "proposed_last_day": day(10) })).await;
        assert_eq!(early.status_code(), 422);
        assert!(early.text().contains("ask HR"));
        assert_eq!(post("/api/me/resignation".into(), me.clone(), json!({ "proposed_last_day": day(40), "reason": "my manager shouts" })).await.status_code(), 422);
        assert_eq!(request.post("/api/me/resignation").json(&json!({ "proposed_last_day": day(40) })).await.status_code(), 401);
        // A signed-in person with no worker record cannot log one for anyone.
        let nobody = bearer(sign_as(&kid, &uuid::Uuid::new_v4().to_string(), &[]));
        assert_eq!(post("/api/me/resignation".into(), nobody, json!({ "proposed_last_day": day(40) })).await.status_code(), 404);
        // Logged by the worker, past a policy that lets only HR write.
        let logged = post("/api/me/resignation".into(), me.clone(), json!({ "proposed_last_day": day(40), "reason": "pay_or_benefits" })).await;
        assert_eq!(logged.status_code(), 200);
        assert!(logged.json::<Value>()["pid"].is_string());
        assert_eq!(post("/api/me/resignation".into(), me.clone(), json!({ "proposed_last_day": day(41) })).await.status_code(), 422, "one open at a time");

        // The worker sees their own reason. The manager and HR see that it was logged, and not why.
        let own: Value = request.get("/api/me/resignation").add_header("authorization", me.clone()).await.json();
        assert_eq!(own["resignation"]["reason"], "pay_or_benefits");
        assert_eq!(own["resignation"]["notice_calendar_days"], 28);
        for (name, token) in [("manager", manager.clone()), ("HR", hr.clone())] {
            let seen = request.get(&format!("/api/workers/{w}/resignation")).add_header("authorization", token).await;
            assert_eq!(seen.status_code(), 200, "{name}");
            let text = seen.text();
            assert!(text.contains(&day(40)) && !text.contains("pay_or_benefits") && !text.contains("reason"), "{name} must not see the reason: {text}");
        }
        assert_eq!(request.get(&format!("/api/workers/{w}/resignation")).add_header("authorization", other.clone()).await.status_code(), 403, "a stranger sees nothing");
        // The manager is told. The notification names no reason.
        let told: Value = request.get(&format!("/api/workers/{w_mgr}/notifications")).add_header("authorization", manager.clone()).await.json();
        let told_text = serde_json::to_string(&told).unwrap();
        assert!(told_text.contains("resignation_logged") && told_text.contains("has logged a resignation"));
        assert!(!told_text.contains("pay_or_benefits"));
        // The list is for HR, and has no reason.
        assert_eq!(request.get("/api/resignations").add_header("authorization", other.clone()).await.status_code(), 403);
        assert_eq!(request.get("/api/resignations").add_header("authorization", manager.clone()).await.status_code(), 403, "a manager does not list resignations");
        let listed = request.get("/api/resignations").add_header("authorization", hr_m.clone()).await;
        assert_eq!(listed.status_code(), 200);
        assert!(!listed.text().contains("pay_or_benefits"));
        // The audit names the event and holds no reason.
        let audits = request.get(&format!("/api/audits/{w}")).add_header("authorization", svc.clone()).await.text();
        assert!(audits.contains("resignation_logged") && !audits.contains("pay_or_benefits"));

        // Withdrawn by the worker while it is waiting; then logged again.
        assert_eq!(post("/api/me/resignation/withdraw".into(), me.clone(), json!({})).await.status_code(), 200);
        assert_eq!(post("/api/me/resignation/withdraw".into(), me.clone(), json!({})).await.status_code(), 422, "already withdrawn");
        let again = post("/api/me/resignation".into(), me.clone(), json!({ "proposed_last_day": day(35) })).await;
        let r2 = again.json::<Value>()["pid"].as_str().unwrap().to_string();

        // Only a person with authority accepts it; the worker cannot accept their own.
        assert_eq!(post(format!("/api/resignations/{r2}/accept"), me.clone(), json!({})).await.status_code(), 403);
        assert_eq!(post(format!("/api/resignations/{r2}/accept"), manager.clone(), json!({})).await.status_code(), 403);
        assert_eq!(post(format!("/api/resignations/{r2}/accept"), hr_m.clone(), json!({ "agreed_last_day": day(-1) })).await.status_code(), 422, "not before it was logged");
        let accepted = post(format!("/api/resignations/{r2}/accept"), hr_m.clone(), json!({ "agreed_last_day": day(30) })).await;
        assert_eq!(accepted.status_code(), 200);
        let movement = accepted.json::<Value>()["movement_pid"].as_str().unwrap().to_string();
        // The leaver process is open for the agreed day; employment itself has NOT ended.
        let row = ctx.db.query_one_raw(sea_orm::Statement::from_string(sea_orm::DbBackend::Postgres,
            format!("SELECT kind, status, effective_on::text AS day FROM movements WHERE pid = '{movement}'"))).await.unwrap().unwrap();
        assert_eq!(row.try_get::<String>("", "kind").unwrap(), "leaver");
        assert_eq!(row.try_get::<String>("", "status").unwrap(), "open");
        assert_eq!(row.try_get::<String>("", "day").unwrap(), day(30));
        let status = request.get(&format!("/api/workers/{w}")).add_header("authorization", hr_m.clone()).await.json::<Value>();
        assert_eq!(status["status"], "active", "accepting a resignation does not end employment");
        assert!(status["terminated_on"].is_null());
        // The worker is told; they cannot take it back now, only HR can record that it was set aside.
        let told_worker: Value = request.get(&format!("/api/workers/{w}/notifications")).add_header("authorization", me.clone()).await.json();
        assert!(serde_json::to_string(&told_worker).unwrap().contains("resignation_decided"));
        let late = post("/api/me/resignation/withdraw".into(), me.clone(), json!({})).await;
        assert_eq!(late.status_code(), 422);
        assert!(late.text().contains("talk to HR"));
        assert_eq!(post(format!("/api/resignations/{r2}/accept"), hr_m.clone(), json!({})).await.status_code(), 422, "not twice");
        assert_eq!(post(format!("/api/resignations/{r2}/rescind"), me.clone(), json!({})).await.status_code(), 403);
        assert_eq!(post(format!("/api/resignations/{r2}/rescind"), hr_m.clone(), json!({})).await.status_code(), 200);
        let row = ctx.db.query_one_raw(sea_orm::Statement::from_string(sea_orm::DbBackend::Postgres,
            format!("SELECT status FROM movements WHERE pid = '{movement}'"))).await.unwrap().unwrap();
        assert_eq!(row.try_get::<String>("", "status").unwrap(), "cancelled", "setting it aside cancels the open leaver record");

        // The aggregate withholds a group this small: nobody can be picked out.
        let summary: Value = request.get("/api/resignations/summary").add_header("authorization", hr_m.clone()).await.json();
        assert_eq!(summary["floor"], 5);
        assert!(summary["departments"].as_array().unwrap().iter().all(|d| d["count"].is_null() && d["reasons"].is_null()));
        // Subject access includes it; erasure removes it with the rest.
        let export: Value = request.get(&format!("/api/workers/{w}/subject-access")).add_header("authorization", me.clone()).await.json();
        assert_eq!(export["resignations"].as_array().unwrap().len(), 2);
    })
    .await;
}
