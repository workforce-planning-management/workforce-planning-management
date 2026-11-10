//! The PADR record under enforcement (WPM-R172, WPM-R225, WPM-R232, WPM-D176), against the shipped reference policy: who opens,
//! reads and signs, the objectives as goals and their frozen snapshot, a correction, the due date, the roll-up and what is told.

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

fn set_env(keys: &Value) {
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
    }
}

async fn seat(ctx: &axa::AppContext, person: &str, org: &str) {
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "INSERT INTO organization_memberships (pid, person_ref, organization_ref, role, starts_on) \
             VALUES ('{}', 'person:{person}', '{org}', 'hr_admin', '2023-01-05')",
        uuid::Uuid::new_v4()
    )))
    .execute(workforce_planning_management::db::pool(ctx))
    .await
    .unwrap();
}

async fn count(ctx: &axa::AppContext, sql: String) -> i64 {
    let row = sqlx::query(sqlx::AssertSqlSafe(&*sql))
        .fetch_optional(workforce_planning_management::db::pool(ctx))
        .await
        .unwrap()
        .unwrap();
    row.try_get::<i64, _>(0).unwrap()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test padr -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn a_padr_is_opened_written_signed_and_corrected() {
    let (keys, kid) = keys_and_kid();
    set_env(&keys);
    let (me, boss, top, peer) = (
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
    );
    let as_me = bearer(sign_as(&kid, &me.to_string(), &[]));
    let as_boss = bearer(sign_as(&kid, &boss.to_string(), &[]));
    let as_top = bearer(sign_as(&kid, &top.to_string(), &[]));
    let as_peer = bearer(sign_as(&kid, &peer.to_string(), &[]));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let hr = bearer(sign_as(&kid, "hr-padr", &[("hr", &["true"])]));
    let payroll = bearer(sign_as(&kid, "pay-padr", &[("payroll", &["true"])]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        seat(&ctx, "hr-padr", &org).await;
        seat(&ctx, "pay-padr", &org).await;
        let top_pid = worker(&request, &svc, &org, &top.to_string(), "P-0", None).await;
        let boss_pid = worker(&request, &svc, &org, &boss.to_string(), "P-1", Some(&top_pid)).await;
        let mine = worker(&request, &svc, &org, &me.to_string(), "P-2", Some(&boss_pid)).await;
        let other = worker(&request, &svc, &org, &peer.to_string(), "P-3", Some(&boss_pid)).await;
        let period = json!({ "period_start": "2027-01-01", "period_end": "2027-12-31" });

        // Nothing on record: the due date is unknown, never invented.
        let team: Value = request
            .get("/api/me/padrs")
            .add_header("authorization", as_boss.clone())
            .await
            .json();
        let line = team["team"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["worker_pid"] == mine.as_str())
            .expect("a report is listed by name");
        assert_eq!(line["display_name"], "Person P-2");
        assert_eq!(line["due"]["standing"], "unknown");
        assert!(line["due"]["due_on"].is_null());

        // Only a manager in the chain opens one.
        let own = request
            .post(&format!("/api/me/reports/{mine}/padrs"))
            .add_header("authorization", as_me.clone())
            .json(&period)
            .await;
        assert_eq!(own.status_code(), 404);
        let peers = request
            .post(&format!("/api/me/reports/{other}/padrs"))
            .add_header("authorization", as_me.clone())
            .json(&period)
            .await;
        assert_eq!(peers.status_code(), 404);
        let hr_open = request
            .post(&format!("/api/me/reports/{mine}/padrs"))
            .add_header("authorization", hr.clone())
            .json(&period)
            .await;
        assert_eq!(hr_open.status_code(), 404, "HR does not open one");
        let backwards = request
            .post(&format!("/api/me/reports/{mine}/padrs"))
            .add_header("authorization", as_boss.clone())
            .json(&json!({ "period_start": "2027-12-31", "period_end": "2027-01-01" }))
            .await;
        assert_eq!(backwards.status_code(), 422);
        let opened: Value = request
            .post(&format!("/api/me/reports/{mine}/padrs"))
            .add_header("authorization", as_boss.clone())
            .json(&period)
            .await
            .json();
        let padr = opened["pid"].as_str().expect("padr pid").to_string();
        let path = format!("/api/me/padrs/{padr}");
        let second = request
            .post(&format!("/api/me/reports/{mine}/padrs"))
            .add_header("authorization", as_boss.clone())
            .json(&period)
            .await;
        assert_eq!(second.status_code(), 422, "one open at a time");
        assert_eq!(
            count(&ctx, format!("SELECT count(*) FROM notifications WHERE worker_pid = '{mine}' AND kind = 'padr_opened'")).await,
            1
        );

        // Who reads: the person, the reviewer and the chain; not a peer, HR or payroll.
        for (who, want) in [
            (&as_me, 200),
            (&as_boss, 200),
            (&as_top, 200),
            (&as_peer, 404),
            (&hr, 404),
            (&payroll, 404),
        ] {
            let read = request
                .get(&path)
                .add_header("authorization", who.clone())
                .await;
            assert_eq!(read.status_code(), want);
        }

        // A PADR is not a performance review.
        let elsewhere = request
            .get(&format!("/api/reviews/{padr}"))
            .add_header("authorization", svc.clone())
            .await;
        assert_eq!(elsewhere.status_code(), 404);
        let status = request
            .post(&format!("/api/reviews/{padr}/status"))
            .add_header("authorization", svc.clone())
            .json(&json!({ "to": "shared" }))
            .await;
        assert_eq!(status.status_code(), 404, "its lifecycle is its own");
        let listed: Value = request
            .get(&format!("/api/workers/{mine}/reviews"))
            .add_header("authorization", svc.clone())
            .await
            .json();
        assert_eq!(listed.as_array().map(Vec::len), Some(0));

        // Objectives are the goals of the period, without the withdrawn and without other years.
        let goal = |title: &str, start: &str, end: &str| {
            json!({ "title": title, "period_start": start, "period_end": end,
                    "measures": [{ "name": "Pages", "kind": "number", "target_value": 40 }] })
        };
        let propose = async |body: Value| -> String {
            let made: Value = request
                .post("/api/me/goals")
                .add_header("authorization", as_me.clone())
                .json(&body)
                .await
                .json();
            made["pid"].as_str().expect("goal pid").to_string()
        };
        let kept = propose(goal("Run the on-call rota", "2027-01-01", "2027-06-30")).await;
        let dropped = propose(goal("A goal given up", "2027-01-01", "2027-06-30")).await;
        propose(goal("Last year's goal", "2025-01-01", "2025-12-31")).await;
        request
            .post(&format!("/api/me/goals/{dropped}/status"))
            .add_header("authorization", as_me.clone())
            .json(&json!({ "to": "withdrawn" }))
            .await
            .assert_status_ok();
        let view: Value = request
            .get(&path)
            .add_header("authorization", as_boss.clone())
            .await
            .json();
        assert_eq!(view["role"], "reviewer");
        let live = view["objectives"].as_array().unwrap();
        assert_eq!(live.len(), 1);
        assert_eq!(live[0]["goal"]["pid"], kept.as_str());

        // Signing needs the wellbeing answer.
        let early = request
            .post(&format!("/api/me/padrs/{padr}/sign"))
            .add_header("authorization", as_boss.clone())
            .await;
        assert_eq!(early.status_code(), 422);
        let content = json!({
            "period_start": "2027-01-01", "period_end": "2027-12-31",
            "values_summary": "We talked about how the team works.",
            "wellbeing_offered": true,
            "development_needs": [{ "need": "Coaching" }],
        });
        let not_theirs = request
            .put(&path)
            .add_header("authorization", as_top.clone())
            .json(&content)
            .await;
        assert_eq!(not_theirs.status_code(), 422, "the chain reads and does not write");
        let wrong_step = request
            .put(&path)
            .add_header("authorization", as_boss.clone())
            .json(&json!({
                "period_start": "2027-01-01", "period_end": "2027-12-31",
                "development_needs": [{ "need": "Coaching", "plan_item_pid": uuid::Uuid::new_v4() }],
            }))
            .await;
        assert_eq!(wrong_step.status_code(), 422);
        request
            .put(&path)
            .add_header("authorization", as_boss.clone())
            .json(&content)
            .await
            .assert_status_ok();

        // The first signature freezes the content and the objectives.
        let signed: Value = request
            .post(&format!("/api/me/padrs/{padr}/sign"))
            .add_header("authorization", as_boss.clone())
            .await
            .json();
        assert_eq!(signed["complete"], false);
        assert_eq!(
            count(&ctx, format!("SELECT count(*) FROM notifications WHERE worker_pid = '{mine}' AND kind = 'padr_to_sign'")).await,
            1
        );
        let twice = request
            .post(&format!("/api/me/padrs/{padr}/sign"))
            .add_header("authorization", as_boss.clone())
            .await;
        assert_eq!(twice.status_code(), 422);
        let frozen = request
            .put(&path)
            .add_header("authorization", as_me.clone())
            .json(&content)
            .await;
        assert_eq!(frozen.status_code(), 422, "signed content is frozen");
        propose(goal("Added after signing", "2027-02-01", "2027-03-01")).await;
        let top_sign = request
            .post(&format!("/api/me/padrs/{padr}/sign"))
            .add_header("authorization", as_top.clone())
            .await;
        assert_eq!(top_sign.status_code(), 422);
        let done: Value = request
            .post(&format!("/api/me/padrs/{padr}/sign"))
            .add_header("authorization", as_me.clone())
            .await
            .json();
        assert_eq!(done["complete"], true);

        let view: Value = request
            .get(&path)
            .add_header("authorization", as_me.clone())
            .await
            .json();
        assert_eq!(view["status"], "shared");
        assert_eq!(view["objectives"].as_array().map(Vec::len), Some(0));
        let snapshot = view["versions"][0]["version"]["objectives"].as_array().unwrap();
        assert_eq!(snapshot.len(), 1, "what was signed, and nothing added since");
        assert_eq!(snapshot[0]["title"], "Run the on-call rota");
        assert_eq!(snapshot[0]["measures"][0]["target_value"], 40);

        // The comment stays open, and it is the person's.
        request
            .post(&format!("/api/me/padrs/{padr}/comment"))
            .add_header("authorization", as_me.clone())
            .json(&json!({ "comment": "I agree." }))
            .await
            .assert_status_ok();
        let their_comment = request
            .post(&format!("/api/me/padrs/{padr}/comment"))
            .add_header("authorization", as_boss.clone())
            .json(&json!({ "comment": "So do I." }))
            .await;
        assert_eq!(their_comment.status_code(), 422);

        // Twelve months on, and a correction keeps the signed version and leaves the date alone.
        let team: Value = request
            .get("/api/me/padrs")
            .add_header("authorization", as_me.clone())
            .await
            .json();
        assert_eq!(team["due"]["standing"], "current");
        let due = team["due"]["due_on"].as_str().unwrap().to_string();
        assert_eq!(team["team"].as_array().map(Vec::len), Some(0));
        let corrected: Value = request
            .post(&format!("/api/me/padrs/{padr}/versions"))
            .add_header("authorization", as_me.clone())
            .await
            .json();
        assert_eq!(corrected["version"], 2);
        let unsigned = request
            .post(&format!("/api/me/padrs/{padr}/versions"))
            .add_header("authorization", as_me.clone())
            .await;
        assert_eq!(unsigned.status_code(), 422, "an unsigned version is simply edited");
        let view: Value = request
            .get(&path)
            .add_header("authorization", as_boss.clone())
            .await
            .json();
        let versions = view["versions"].as_array().unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0]["complete"], true);
        assert_eq!(versions[1]["frozen"], false);
        assert_eq!(versions[1]["version"]["values_summary"], "We talked about how the team works.");
        assert!(versions[1]["version"]["objectives"].is_null());
        assert_eq!(view["objectives"].as_array().map(Vec::len), Some(2), "a correction lists the goals afresh");
        let again: Value = request
            .get("/api/me/padrs")
            .add_header("authorization", as_me.clone())
            .await
            .json();
        assert_eq!(again["due"]["due_on"], due.as_str());

        // Audited with no content, and in the person's export.
        assert!(
            count(&ctx, format!("SELECT count(*) FROM audit_logs WHERE entity_pid = '{padr}' AND action = 'padr_read'")).await > 0
        );
        assert_eq!(
            count(&ctx, format!("SELECT count(*) FROM audit_logs WHERE entity_pid = '{padr}' AND snapshot::text LIKE '%talked%'")).await,
            0
        );
        let export: Value = request
            .get(&format!("/api/workers/{mine}/subject-access"))
            .add_header("authorization", svc.clone())
            .await
            .json();
        assert!(export.to_string().contains("padr_versions"), "{export}");

        // HR reads counts and no content; four people is under the floor.
        let refused = request
            .get(&format!("/api/padrs/rollup?organization={org}"))
            .add_header("authorization", as_me.clone())
            .await;
        assert_eq!(refused.status_code(), 403);
        let rollup: Value = request
            .get(&format!("/api/padrs/rollup?organization={org}"))
            .add_header("authorization", hr.clone())
            .await
            .json();
        let line = &rollup["departments"][0];
        assert_eq!(line["department"], "engineering");
        assert!(line["by_status"].is_null(), "{rollup}");
        for n in 4..8 {
            worker(&request, &svc, &org, &uuid::Uuid::new_v4().to_string(), &format!("P-{n}"), None).await;
        }
        let rollup: Value = request
            .get(&format!("/api/padrs/rollup?organization={org}"))
            .add_header("authorization", hr.clone())
            .await
            .json();
        assert_eq!(rollup["departments"][0]["by_status"]["current"], 1);
        assert_eq!(rollup["departments"][0]["by_status"]["unknown"], 7);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test padr -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn a_waiting_goal_tells_the_other_party_and_a_change_goes_back_to_them() {
    let (keys, kid) = keys_and_kid();
    set_env(&keys);
    let (me, boss, peer) = (
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
    );
    let as_me = bearer(sign_as(&kid, &me.to_string(), &[]));
    let as_boss = bearer(sign_as(&kid, &boss.to_string(), &[]));
    let as_peer = bearer(sign_as(&kid, &peer.to_string(), &[]));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        let boss_pid = worker(&request, &svc, &org, &boss.to_string(), "N-1", None).await;
        let mine = worker(&request, &svc, &org, &me.to_string(), "N-2", Some(&boss_pid)).await;
        let lone = worker(&request, &svc, &org, &peer.to_string(), "N-3", None).await;
        let body = json!({ "title": "A secret plan", "period_start": "2027-01-01", "period_end": "2027-12-31" });
        let waiting = async |who: &str| {
            count(&ctx, format!("SELECT count(*) FROM notifications WHERE worker_pid = '{who}' AND kind = 'goal_waiting'")).await
        };

        // A report with no goals is still listed, so a first goal has somewhere to go.
        let team: Value = request
            .get("/api/me/team-goals")
            .add_header("authorization", as_boss.clone())
            .await
            .json();
        assert_eq!(team["team"][0]["worker_pid"], mine.as_str());
        assert_eq!(team["team"][0]["goals"].as_array().map(Vec::len), Some(0));

        // The person proposes, and their manager is told, with no words of the goal.
        let made: Value = request
            .post("/api/me/goals")
            .add_header("authorization", as_me.clone())
            .json(&body)
            .await
            .json();
        let goal = made["pid"].as_str().unwrap().to_string();
        assert_eq!(waiting(&boss_pid).await, 1);
        assert_eq!(waiting(&mine).await, 0);
        assert_eq!(
            count(&ctx, format!("SELECT count(*) FROM notifications WHERE worker_pid = '{boss_pid}' AND (body LIKE '%secret%' OR data::text LIKE '%secret%')")).await,
            0
        );

        // Someone with no manager tells nobody.
        request
            .post("/api/me/goals")
            .add_header("authorization", as_peer.clone())
            .json(&body)
            .await
            .assert_status_ok();
        assert_eq!(waiting(&lone).await, 0);

        // Their own change tells nobody new.
        request
            .put(&format!("/api/me/goals/{goal}"))
            .add_header("authorization", as_me.clone())
            .json(&body)
            .await
            .assert_status_ok();
        assert_eq!(waiting(&boss_pid).await, 1);

        // The manager changes the words, so the manager cannot then agree them alone.
        let changed = json!({ "title": "A different plan", "period_start": "2027-01-01", "period_end": "2027-12-31" });
        request
            .put(&format!("/api/me/goals/{goal}"))
            .add_header("authorization", as_boss.clone())
            .json(&changed)
            .await
            .assert_status_ok();
        assert_eq!(waiting(&mine).await, 1);
        let alone = request
            .post(&format!("/api/me/goals/{goal}/agree"))
            .add_header("authorization", as_boss.clone())
            .await;
        assert_eq!(alone.status_code(), 422);
        request
            .post(&format!("/api/me/goals/{goal}/agree"))
            .add_header("authorization", as_me.clone())
            .await
            .assert_status_ok();

        // A manager's proposal tells the person. The links are checked and named.
        let skill: Value = request
            .post("/api/skills")
            .add_header("authorization", svc.clone())
            .json(&json!({ "name": format!("Facilitation {org}"), "category": "technical" }))
            .await
            .json();
        let skill = skill["pid"].as_str().expect("skill pid").to_string();
        let linked = json!({
            "title": "Lead a retrospective", "period_start": "2027-01-01", "period_end": "2027-12-31",
            "skill_pid": skill, "aligns_to": "objective:reliability",
        });
        let bad_skill = request
            .post(&format!("/api/me/reports/{mine}/goals"))
            .add_header("authorization", as_boss.clone())
            .json(&json!({ "title": "x", "period_start": "2027-01-01", "period_end": "2027-12-31", "skill_pid": uuid::Uuid::new_v4() }))
            .await;
        assert_eq!(bad_skill.status_code(), 422);
        let bad_item = request
            .post(&format!("/api/me/reports/{mine}/goals"))
            .add_header("authorization", as_boss.clone())
            .json(&json!({ "title": "x", "period_start": "2027-01-01", "period_end": "2027-12-31", "plan_item_pid": uuid::Uuid::new_v4() }))
            .await;
        assert_eq!(bad_item.status_code(), 422);
        let plan: Value = request
            .post(&format!("/api/workers/{lone}/development-plans"))
            .add_header("authorization", svc.clone())
            .json(&json!({ "kind": "upskill", "items": [{ "skill_pid": skill, "current_level": 1, "target_level": 3, "method": "course" }] }))
            .await
            .json();
        assert!(plan["pid"].is_string(), "{plan}");
        let plans: Value = request
            .get(&format!("/api/workers/{lone}/development-plans"))
            .add_header("authorization", svc.clone())
            .await
            .json();
        let someone_elses = plans["plans"][0]["items"][0]["pid"].as_str().expect("an item").to_string();
        let foreign = request
            .post(&format!("/api/me/reports/{mine}/goals"))
            .add_header("authorization", as_boss.clone())
            .json(&json!({ "title": "x", "period_start": "2027-01-01", "period_end": "2027-12-31", "plan_item_pid": someone_elses }))
            .await;
        assert_eq!(foreign.status_code(), 422, "another person's plan is not this one's");
        request
            .post(&format!("/api/me/reports/{mine}/goals"))
            .add_header("authorization", as_boss.clone())
            .json(&linked)
            .await
            .assert_status_ok();
        assert_eq!(waiting(&mine).await, 2);
        let goals: Value = request
            .get(&format!("/api/workers/{mine}/goals"))
            .add_header("authorization", as_me.clone())
            .await
            .json();
        let row = goals["goals"]
            .as_array()
            .unwrap()
            .iter()
            .find(|g| g["goal"]["title"] == "Lead a retrospective")
            .expect("the goal");
        assert_eq!(row["skill_name"], format!("Facilitation {org}"));
        assert_eq!(row["goal"]["aligns_to"], "objective:reliability");
    })
    .await;
}
