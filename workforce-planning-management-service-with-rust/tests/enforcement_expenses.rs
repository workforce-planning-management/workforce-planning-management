//! Who may do what with an expense claim under enforcement (WPM-R55).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide
//! `OnceLock`s). The blanket policy here lets every signed-in caller through, so
//! what is verified is the controller's own rule: the claimant and HR write a
//! claim, the manager and HR decide it, a stranger sees nothing — and **nobody,
//! not even HR, decides their own claim**.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::SigningKey;
use loco_rs::testing::prelude::*;
use rusty_paseto::core::{Footer, Key, Paseto, PasetoAsymmetricPrivateKey, Payload, Public, V4};
use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;
use sha2::{Digest, Sha256};
use workforce_planning_management_service::app::App;

const SEED: [u8; 32] = [7; 32];

fn keys_and_kid() -> (Value, String) {
    let public = SigningKey::from_bytes(&SEED).verifying_key().to_bytes();
    let kid = URL_SAFE_NO_PAD.encode(Sha256::digest(public));
    let keys = json!({ "keys": [{ "kty": "OKP", "crv": "Ed25519", "use": "sig",
                                   "kid": kid, "x": URL_SAFE_NO_PAD.encode(public) }] });
    (keys, kid)
}

fn sign_as(kid: &str, sub: &str) -> String {
    let iat: i64 = 1_700_000_000;
    let payload = json!({
        "sub": sub, "email": "person@example.com", "name": "Test Person",
        "iss": "authentication-service", "aud": "main-x-service",
        "exp": iat + 10_000_000_000_i64, "iat": iat, "sid": "test-sid", "attrs": {},
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

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test enforcement_expenses -- --ignored`"]
async fn nobody_decides_their_own_claim() {
    let (keys, kid) = keys_and_kid();
    // Every signed-in caller passes the blanket guard; the controller decides.
    let open_policy = r#"{"rules":[{"effect":"allow","actions":["read","write","delete","destructive"],"when":{}}]}"#;
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY", open_policy);
        std::env::remove_var("WPM_ABAC_POLICY_FILE");
    }
    let people: Vec<uuid::Uuid> = (0..5).map(|_| uuid::Uuid::new_v4()).collect();
    let [emp, mgr, hr, stranger, peer] = [0, 1, 2, 3, 4].map(|i| people[i]);
    let token = |p: uuid::Uuid| format!("Bearer {}", sign_as(&kid, &p.to_string()));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        let other_org = format!("organization:{}", uuid::Uuid::new_v4());
        let make = |person: uuid::Uuid, number: &'static str, org: String| {
            let request = &request;
            let t = token(mgr);
            async move {
                let r = request
                    .post("/api/workers")
                    .add_header("authorization", t)
                    .json(&json!({
                        "person_ref": format!("person:{person}"), "organization_ref": org,
                        "worker_number": number, "display_name": format!("Person {number}"),
                        "employment_type": "permanent", "department": "engineering",
                        "job_title": "Engineer", "hired_on": "2026-01-05",
                    }))
                    .await;
                assert_eq!(r.status_code(), 200, "{number}");
                r.json::<Value>()["pid"].as_str().unwrap().to_string()
            }
        };
        let w_mgr = make(mgr, "XE-M", org.clone()).await;
        let w_emp = make(emp, "XE-E", org.clone()).await;
        let w_hr = make(hr, "XE-H", org.clone()).await;
        let w_stranger = make(stranger, "XE-S", other_org.clone()).await;
        let w_peer = make(peer, "XE-P", org.clone()).await;
        for w in [&w_emp, &w_hr] {
            request
                .put(&format!("/api/workers/{w}"))
                .add_header("authorization", token(mgr))
                .json(&json!({ "manager_pid": w_mgr }))
                .await
                .assert_status_ok();
        }
        // HR: a membership role in the claimants' organization, not a policy attribute.
        ctx.db
            .execute_unprepared(&format!(
                "INSERT INTO organization_memberships (pid, person_ref, organization_ref, worker_pid, role, starts_on) \
                 VALUES ('{}', 'person:{hr}', '{org}', '{w_hr}', 'hr_admin', '2026-01-05')",
                uuid::Uuid::new_v4()
            ))
            .await
            .unwrap();
        let _ = (&w_stranger, &w_peer);

        let day = (chrono::Utc::now().date_naive() - chrono::Duration::days(2)).to_string();
        let as_ = |who: uuid::Uuid| token(who);

        // Writing a claim: the claimant and HR; not the manager, a peer or a stranger.
        let url = format!("/api/workers/{w_emp}/expense-claims");
        let body = json!({ "title": "Train fare", "currency": "GBP" });
        for (who, name) in [(mgr, "manager"), (peer, "peer"), (stranger, "stranger")] {
            assert_eq!(
                request.post(&url).add_header("authorization", as_(who)).json(&body).await.status_code(),
                403,
                "{name} cannot write someone's claim"
            );
        }
        let claim: Value = request
            .post(&url)
            .add_header("authorization", as_(emp))
            .json(&body)
            .await
            .json();
        let pid = claim["pid"].as_str().unwrap().to_string();
        // HR may write on the claimant's behalf (recorded as such).
        let on_behalf = request.post(&url).add_header("authorization", as_(hr)).json(&body).await;
        assert_eq!(on_behalf.status_code(), 200);
        assert_eq!(on_behalf.json::<Value>()["on_behalf"], true);

        let items = format!("/api/expense-claims/{pid}/items");
        let item = json!({ "incurred_on": day, "category": "travel", "amount_minor": 4_250 });
        assert_eq!(
            request.post(&items).add_header("authorization", as_(mgr)).json(&item).await.status_code(),
            403,
            "a manager reviews, they do not write the claim"
        );
        request.post(&items).add_header("authorization", as_(emp)).json(&item).await.assert_status_ok();
        let submit = format!("/api/expense-claims/{pid}/submit");
        assert_eq!(request.post(&submit).add_header("authorization", as_(peer)).await.status_code(), 404,
            "a peer cannot even tell it exists");
        request.post(&submit).add_header("authorization", as_(emp)).await.assert_status_ok();

        // Seeing it: claimant, manager and HR yes; a peer or stranger not.
        for (who, status) in [(emp, 200), (mgr, 200), (hr, 200), (peer, 404), (stranger, 404)] {
            assert_eq!(
                request.get(&format!("/api/expense-claims/{pid}")).add_header("authorization", as_(who)).await.status_code(),
                status
            );
        }
        for (who, status) in [(emp, 200), (hr, 200), (peer, 403), (stranger, 403)] {
            assert_eq!(request.get(&url).add_header("authorization", as_(who)).await.status_code(), status);
        }

        // The decision queue: the manager and HR see it, the claimant, a peer and a stranger do not.
        let queue = |who: uuid::Uuid| {
            let request = &request;
            let t = as_(who);
            async move {
                let v: Value = request.get("/api/expense-claims?status=submitted").add_header("authorization", t).await.json();
                v.as_array().unwrap().iter().map(|c| c["pid"].as_str().unwrap().to_string()).collect::<Vec<_>>()
            }
        };
        assert!(queue(mgr).await.contains(&pid));
        assert!(queue(hr).await.contains(&pid));
        assert!(!queue(emp).await.contains(&pid), "never your own claim in your queue");
        assert!(queue(peer).await.is_empty());
        assert!(queue(stranger).await.is_empty());

        // Deciding: not the claimant, not a peer; the manager can.
        let approve = format!("/api/expense-claims/{pid}/approve");
        assert_eq!(request.post(&approve).add_header("authorization", as_(emp)).await.status_code(), 403, "the claimant cannot approve their own claim");
        assert_eq!(request.post(&approve).add_header("authorization", as_(peer)).await.status_code(), 404);
        assert_eq!(request.post(&approve).add_header("authorization", as_(mgr)).await.status_code(), 200);
        // Reimbursing is also a decision: not the claimant.
        let pay = format!("/api/expense-claims/{pid}/reimburse");
        assert_eq!(request.post(&pay).add_header("authorization", as_(emp)).await.status_code(), 403);
        assert_eq!(request.post(&pay).add_header("authorization", as_(hr)).await.status_code(), 200);

        // HR cannot decide their *own* claim — their manager does.
        let hr_claim: Value = request
            .post(&format!("/api/workers/{w_hr}/expense-claims"))
            .add_header("authorization", as_(hr))
            .json(&body)
            .await
            .json();
        let hp = hr_claim["pid"].as_str().unwrap().to_string();
        request.post(&format!("/api/expense-claims/{hp}/items")).add_header("authorization", as_(hr)).json(&item).await.assert_status_ok();
        request.post(&format!("/api/expense-claims/{hp}/submit")).add_header("authorization", as_(hr)).await.assert_status_ok();
        let hr_approve = format!("/api/expense-claims/{hp}/approve");
        assert_eq!(request.post(&hr_approve).add_header("authorization", as_(hr)).await.status_code(), 403,
            "HR cannot approve their own claim");
        assert!(!queue(hr).await.contains(&hp), "HR's own claim is not in HR's queue");
        assert!(queue(mgr).await.contains(&hp), "their manager decides it");
        assert_eq!(request.post(&hr_approve).add_header("authorization", as_(mgr)).await.status_code(), 200);

        // `can` tells each viewer what they may do.
        let can = |who: uuid::Uuid, claim: String| {
            let request = &request;
            let t = as_(who);
            async move {
                request.get(&format!("/api/expense-claims/{claim}")).add_header("authorization", t).await.json::<Value>()["can"].clone()
            }
        };
        let second: Value = request.post(&url).add_header("authorization", as_(emp)).json(&body).await.json();
        let s2 = second["pid"].as_str().unwrap().to_string();
        request.post(&format!("/api/expense-claims/{s2}/items")).add_header("authorization", as_(emp)).json(&item).await.assert_status_ok();
        request.post(&format!("/api/expense-claims/{s2}/submit")).add_header("authorization", as_(emp)).await.assert_status_ok();
        assert_eq!(can(emp, s2.clone()).await["decide"], false);
        assert_eq!(can(emp, s2.clone()).await["withdraw"], true);
        assert_eq!(can(mgr, s2.clone()).await["decide"], true);
        assert_eq!(can(mgr, s2).await["withdraw"], false, "a manager cannot withdraw a claim they did not make");
    })
    .await;
}
