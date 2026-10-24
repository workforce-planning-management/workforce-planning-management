//! The pay gap report under enforcement (WPM-R144, WPM-D99).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s). Proves who can read and
//! change what: a worker nothing, HR (who reads salaries masked) the categories but not a report made of salaries,
//! payroll and the service the report; that the report names no one and carries no head count; and that no audit
//! entry holds a figure.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::SigningKey;
use loco_rs::testing::prelude::*;
use rusty_paseto::core::{Footer, Key, Paseto, PasetoAsymmetricPrivateKey, Payload, Public, V4};
use serde_json::{Value, json};
use serial_test::serial;
use sha2::{Digest, Sha256};
use workforce_planning_management::app::App;

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
    let iat: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_secs()).ok())
        .expect("the clock is after 1970");
    let payload = json!({
        "sub": sub,
        "email": "person@example.com", "name": "Test Person",
        "iss": ISSUER, "aud": AUDIENCE,
        "exp": iat + 3_600, "iat": iat,
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

const CATEGORIES: &str = r#"{"sex":["female","male"]}"#;

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test enforcement_pay_gap -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn only_those_who_may_read_salaries_get_the_report_and_it_names_no_one() {
    use sea_orm::ConnectionTrait;
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
        std::env::set_var(
            "WPM_EQUALITY_MONITORING_BASIS",
            "Test deployment: impact assessment PG-2",
        );
        std::env::set_var("WPM_EQUALITY_CATEGORIES", CATEGORIES);
        std::env::set_var("WPM_EQUALITY_FLOOR", "5");
    }
    let bearer = |t: String| format!("Bearer {t}");
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let hr_p = uuid::Uuid::new_v4();
    let hr = bearer(sign_as(&kid, &hr_p.to_string(), &[("hr", &["true"])]));
    let payroll_p = uuid::Uuid::new_v4();
    let payroll = bearer(sign_as(
        &kid,
        &payroll_p.to_string(),
        &[("payroll", &["true"])],
    ));
    let worker_p = uuid::Uuid::new_v4();
    let worker = bearer(sign_as(&kid, &worker_p.to_string(), &[]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        // Six women on 90,000 and six men on 100,000, all engineers, each with a declared sex.
        let mut pids = Vec::new();
        for i in 0..12 {
            let (sex, salary) = if i < 6 { ("female", 9_000_000) } else { ("male", 10_000_000) };
            let r = request.post("/api/workers").add_header("authorization", svc.clone()).json(&json!({
                "person_ref": format!("person:{}", uuid::Uuid::new_v4()), "organization_ref": org,
                "worker_number": format!("PGE-{i}"), "display_name": format!("Pay Gap Enforcement {i}"),
                "employment_type": "permanent", "department": "engineering", "job_title": "Engineer",
                "salary_minor": salary, "salary_currency": "GBP", "hired_on": "2020-01-05",
            })).await;
            assert_eq!(r.status_code(), 200, "{}", r.text());
            let pid = r.json::<Value>()["pid"].as_str().unwrap().to_string();
            ctx.db.execute_unprepared(&format!(
                "INSERT INTO equality_declarations (worker_pid, category, value) VALUES ('{pid}', 'sex', '{sex}')")).await.unwrap();
            pids.push(pid);
        }
        // The HR and payroll callers are workers of the organization too (so the organization is in their scope).
        for (p, n, role) in [(hr_p, "HR", "hr_admin"), (payroll_p, "PAY", "payroll_admin")] {
            let w = request.post("/api/workers").add_header("authorization", svc.clone()).json(&json!({
                "person_ref": format!("person:{p}"), "organization_ref": org, "worker_number": format!("PGE-{n}"),
                "display_name": format!("Pay Gap Enforcement {n}"), "employment_type": "permanent", "department": "people",
                "job_title": "Administrator", "hired_on": "2020-01-05" })).await.json::<Value>()["pid"].as_str().unwrap().to_string();
            ctx.db.execute_unprepared(&format!(
                "INSERT INTO organization_memberships (pid, person_ref, organization_ref, worker_pid, role, starts_on) \
                 VALUES ('{}', 'person:{p}', '{org}', '{w}', '{role}', '2020-01-05')", uuid::Uuid::new_v4())).await.unwrap();
        }
        let get = |path: String, t: String| {
            let request = &request;
            async move { request.get(&path).add_header("authorization", t).await }
        };
        let body = json!({ "organization_ref": org, "name": "Engineering",
            "basis": "Same skills, effort, responsibility and working conditions.", "job_titles": ["Engineer"] });

        // ── Categories: HR writes them; a worker cannot, and cannot read them ─────────────
        assert_eq!(request.post("/api/pay-categories").json(&body).await.status_code(), 401, "no token");
        assert_eq!(request.post("/api/pay-categories").add_header("authorization", worker.clone()).json(&body).await.status_code(), 403, "a worker");
        let made = request.post("/api/pay-categories").add_header("authorization", hr.clone()).json(&body).await;
        assert_eq!(made.status_code(), 200, "HR sets the categories: {}", made.text());
        let list = format!("/api/pay-categories?organization={org}");
        assert_eq!(get(list.clone(), worker.clone()).await.status_code(), 403, "a worker does not read them");
        assert_eq!(get(list.clone(), hr.clone()).await.status_code(), 200);
        let pc_pid = made.json::<Value>()["pid"].as_str().unwrap().to_string();
        assert_eq!(request.delete(&format!("/api/pay-categories/{pc_pid}")).add_header("authorization", worker.clone()).await.status_code(), 403);

        // ── The report: only a caller who may read salaries ──────────────────────────────
        let report = format!("/api/pay-transparency/gap-report?organization={org}&currency=GBP");
        assert_eq!(request.get(&report).await.status_code(), 401);
        assert_eq!(get(report.clone(), worker.clone()).await.status_code(), 403, "a worker");
        let masked = get(report.clone(), hr.clone()).await;
        assert_eq!(masked.status_code(), 403, "HR reads salaries masked, so not a report made of them: {}", masked.text());
        assert!(masked.text().contains("payroll read"), "{}", masked.text());
        let csv = format!("/api/pay-transparency/gap-report/csv?organization={org}&currency=GBP");
        assert_eq!(get(csv.clone(), worker.clone()).await.status_code(), 403);
        assert_eq!(get(csv.clone(), hr.clone()).await.status_code(), 403);

        let seen = get(report.clone(), payroll.clone()).await;
        assert_eq!(seen.status_code(), 200, "{}", seen.text());
        let text = seen.text();
        let parsed: Value = serde_json::from_str(&text).unwrap();
        let engineering = &parsed["report"]["categories"][0]["basic"];
        assert_eq!(engineering["status"], "shown");
        assert_eq!(engineering["value"]["gap_bp"], 1_000);
        // The service persona holds no membership in this organization, so it is not in its scope.
        assert_eq!(get(report.clone(), svc.clone()).await.status_code(), 404, "outside the organization's scope");
        let csv_text = get(csv.clone(), payroll.clone()).await.text();

        // ── It names no one, and no head count ──────────────────────────────────────────
        for pid in &pids {
            assert!(!text.contains(pid.as_str()) && !csv_text.contains(pid.as_str()), "a worker pid in the report");
        }
        for banned in ["Pay Gap Enforcement", "PGE-", "display_name", "worker_number", "headcount"] {
            assert!(!text.contains(banned) && !csv_text.contains(banned), "{banned} in the report");
        }

        // ── The audit says that it was read, and holds no figure ────────────────────────
        let rows = ctx.db.query_all_raw(sea_orm::Statement::from_string(sea_orm::DatabaseBackend::Postgres,
            "SELECT action, snapshot::text AS snapshot FROM audit_logs WHERE entity IN ('pay_gap_report', 'pay_category') ORDER BY id".to_string()))
            .await.unwrap();
        let actions: Vec<String> = rows.iter().map(|r| r.try_get("", "action").unwrap()).collect();
        for expected in ["pay_category_set", "pay_gap_report_read", "pay_gap_report_exported"] {
            assert!(actions.iter().any(|a| a == expected), "{expected} in {actions:?}");
        }
        for row in &rows {
            let snapshot: Option<String> = row.try_get("", "snapshot").unwrap();
            let snapshot = snapshot.unwrap_or_default();
            for figure in ["9000000", "10000000", "gap_bp", "female", "male"] {
                assert!(!snapshot.contains(figure), "{figure} in an audit snapshot: {snapshot}");
            }
        }

        // ── The joint assessment signal: read like the report, recorded by those who may write ──────
        let signal = format!("/api/pay-transparency/joint-assessment?organization={org}&currency=GBP&reported_on=2020-01-01");
        assert_eq!(request.get(&signal).await.status_code(), 401);
        assert_eq!(get(signal.clone(), worker.clone()).await.status_code(), 403, "a worker");
        assert_eq!(get(signal.clone(), hr.clone()).await.status_code(), 403, "HR reads salaries masked");
        let due: Value = get(signal.clone(), payroll.clone()).await.json();
        assert_eq!(due["categories"][0]["signal"]["status"], "due", "ten percent, reported years ago, nothing recorded");
        let record = json!({ "organization_ref": org, "category": "Engineering", "reported_on": "2020-01-01",
            "justified": true, "justification": "Different grades under the published job evaluation scheme." });
        let records = format!("/api/pay-transparency/assessment-records?organization={org}");
        assert_eq!(request.put("/api/pay-transparency/assessment-records").json(&record).await.status_code(), 401);
        assert_eq!(request.put("/api/pay-transparency/assessment-records").add_header("authorization", worker.clone()).json(&record).await.status_code(), 403, "a worker");
        assert_eq!(request.put("/api/pay-transparency/assessment-records").add_header("authorization", payroll.clone()).json(&record).await.status_code(), 403, "payroll reads, it does not record");
        let put = request.put("/api/pay-transparency/assessment-records").add_header("authorization", hr.clone()).json(&record).await;
        assert_eq!(put.status_code(), 200, "HR records: {}", put.text());
        assert_eq!(get(records.clone(), worker.clone()).await.status_code(), 403, "a worker does not read the records");
        assert_eq!(get(records.clone(), hr.clone()).await.status_code(), 200);
        let after: Value = get(signal.clone(), payroll.clone()).await.json();
        assert_eq!(after["categories"][0]["signal"], json!({ "status": "not_due", "why": "justified" }));
        assert!(!after.to_string().contains("Pay Gap Enforcement"), "no worker named");
        let audits = ctx.db.query_all_raw(sea_orm::Statement::from_string(sea_orm::DatabaseBackend::Postgres,
            "SELECT snapshot::text AS snapshot FROM audit_logs WHERE action IN ('pay_assessment_recorded', 'joint_assessment_read')".to_string()))
            .await.unwrap();
        assert!(audits.len() >= 2);
        for row in &audits {
            let snapshot: Option<String> = row.try_get("", "snapshot").unwrap();
            assert!(!snapshot.unwrap_or_default().contains("job evaluation"), "the words are not in the audit");
        }

        // ── The human capital report: HR, payroll and the service read it; a worker does not; it names no one ──
        let capital = format!("/api/reports/human-capital?organization={org}");
        assert_eq!(request.get(&capital).await.status_code(), 401);
        assert_eq!(get(capital.clone(), worker.clone()).await.status_code(), 403, "a worker");
        let hc = get(capital.clone(), hr.clone()).await;
        assert_eq!(hc.status_code(), 200, "HR reads it, and it holds no salary: {}", hc.text());
        let hc: Value = hc.json();
        assert!(!hc.to_string().contains("Pay Gap Enforcement"), "no worker named");
        assert!(hc["report"]["figures"].as_array().unwrap().iter().any(|f| f["key"] == "headcount"));

        // ── The AI register: any signed-in caller reads it; HR records; nobody else ──────────────────
        assert_eq!(request.get("/api/ai-register").await.status_code(), 401);
        assert_eq!(get("/api/ai-register".to_string(), worker.clone()).await.status_code(), 200, "a worker reads how people are scored");
        let entry = json!({ "overseer_role": "Head of Resourcing" });
        assert_eq!(request.put("/api/ai-register/S3").json(&entry).await.status_code(), 401);
        assert_eq!(request.put("/api/ai-register/S3").add_header("authorization", worker.clone()).json(&entry).await.status_code(), 403, "a worker");
        let recorded = request.put("/api/ai-register/S3").add_header("authorization", hr.clone()).json(&entry).await;
        assert_eq!(recorded.status_code(), 200, "HR records: {}", recorded.text());

        // ── MCP: the endpoint needs a token, and a tool is refused exactly as its route is ──────────────
        let tool = |name: &str, arguments: Value| json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": name, "arguments": arguments } });
        let call = json!(tool("human_capital_report", json!({ "organization": org })));
        assert_eq!(request.post("/mcp").json(&call).await.status_code(), 401, "no token");
        let as_worker: Value = request.post("/mcp").add_header("authorization", worker.clone()).json(&call).await.json();
        assert_eq!(as_worker["result"]["isError"], true, "a worker is refused the report by the API itself");
        assert!(as_worker["result"]["content"][0]["text"].as_str().unwrap().contains("403"), "{as_worker}");
        let as_hr: Value = request.post("/mcp").add_header("authorization", hr.clone()).json(&call).await.json();
        assert_eq!(as_hr["result"]["isError"], false, "HR reads it: {as_hr}");
        assert!(!as_hr.to_string().contains("Pay Gap Enforcement"), "no worker named");
        let salary_free = request.post("/mcp").add_header("authorization", worker.clone()).json(&tool("ai_register", json!({}))).await.json::<Value>();
        assert_eq!(salary_free["result"]["isError"], false, "a worker reads how people are scored");
        // It cannot be steered to a route outside the table.
        let steer = request.post("/mcp").add_header("authorization", worker.clone()).json(&tool("get_workforce_plan", json!({ "pid": "../workers" }))).await.json::<Value>();
        assert_eq!(steer["result"]["isError"], true);

        // ── Take the sex answers away and there is nothing to compare ────────────────────
        ctx.db.execute_unprepared("DELETE FROM equality_declarations").await.unwrap();
        let none: Value = get(report.clone(), payroll.clone()).await.json();
        assert_eq!(none["report"]["categories"][0]["basic"], json!({ "status": "unknown", "value": "no_workers" }));
        assert_eq!(none["report"]["exclusions"]["no_sex_declared"], json!({ "status": "exactly", "value": 14 }));
    })
    .await;
}
