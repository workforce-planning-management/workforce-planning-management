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
        std::env::set_var("WPM_CREDENTIAL_ISSUER", "http://localhost:5150");
        std::env::set_var("WPM_CREDENTIAL_SIGNING_KEY", "nWGxne_9WmC6hEr0kuwsxERJxWl7MmkZcDusAxyuf2A");
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
    let admin = bearer(sign_as(&kid, "admin-user", &[("access", &["admin"])]));

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

        // ── Pay position analysis: made of salaries, so payroll reads it; HR (masked) and a worker do not ──
        let range = format!("/api/reports/pay-range?organization={org}");
        assert_eq!(request.get(&range).await.status_code(), 401);
        assert_eq!(get(range.clone(), worker.clone()).await.status_code(), 403, "a worker");
        let masked = get(range.clone(), hr.clone()).await;
        assert_eq!(masked.status_code(), 403, "HR reads salaries masked: {}", masked.text());
        assert!(masked.text().contains("payroll read"), "{}", masked.text());
        let placed = get(range.clone(), payroll.clone()).await;
        assert_eq!(placed.status_code(), 200, "payroll reads it: {}", placed.text());
        assert!(!placed.text().contains("Pay Gap Enforcement"), "no worker named");

        // ── Equipment: HR keeps the asset list and issues; a worker may not ──
        let assets = format!("/api/assets?organization={org}");
        assert_eq!(request.get(&assets).await.status_code(), 401);
        assert_eq!(get(assets.clone(), worker.clone()).await.status_code(), 403, "a worker");
        assert_eq!(get(assets.clone(), hr.clone()).await.status_code(), 200, "HR");
        let new_asset = json!({ "organization_ref": org, "kind": "laptop", "tag": "ENF-1", "owner_role": "IT" });
        assert_eq!(request.post("/api/assets").add_header("authorization", worker.clone()).json(&new_asset).await.status_code(), 403, "a worker adds none");
        assert_eq!(request.post("/api/assets").add_header("authorization", hr.clone()).json(&new_asset).await.status_code(), 200, "HR adds one");

        // ── Policies: HR keeps them; the acknowledgement is the worker's own; the names are HR's ──
        let me = request.post("/api/workers").add_header("authorization", svc.clone()).json(&json!({
            "person_ref": format!("person:{worker_p}"), "organization_ref": org, "worker_number": "PGE-ME",
            "display_name": "Pay Gap Enforcement Me", "employment_type": "permanent", "department": "engineering",
            "job_title": "Engineer", "hired_on": "2020-01-05" })).await.json::<Value>()["pid"].as_str().unwrap().to_string();
        let policy_body = json!({ "organization_ref": org, "name": "Conduct" });
        assert_eq!(request.post("/api/policies").json(&policy_body).await.status_code(), 401);
        assert_eq!(request.post("/api/policies").add_header("authorization", worker.clone()).json(&policy_body).await.status_code(), 403, "a worker adds no policy");
        let policy = request.post("/api/policies").add_header("authorization", hr.clone()).json(&policy_body).await;
        assert_eq!(policy.status_code(), 200, "{}", policy.text());
        let policy = policy.json::<Value>()["pid"].as_str().unwrap().to_string();
        let version = request.post(&format!("/api/policies/{policy}/versions")).add_header("authorization", hr.clone())
            .json(&json!({ "title": "Conduct", "document_ref": "doc:conduct", "effective_on": "2026-01-01", "audience": "all" })).await;
        assert_eq!(version.status_code(), 200, "{}", version.text());
        let version = version.json::<Value>()["pid"].as_str().unwrap().to_string();
        let ack = "/api/me/policy-acknowledgements";
        let ack_body = json!({ "version_pid": version });
        assert_eq!(request.post(ack).json(&ack_body).await.status_code(), 401);
        // It is the caller's own act: HR has no worker record of the audience here, and no route acknowledges for another.
        assert_eq!(request.post(&format!("/api/workers/{me}/policy-acknowledgements")).add_header("authorization", hr.clone()).json(&ack_body).await.status_code(), 404, "there is no route to acknowledge for someone");
        let own = request.post(ack).add_header("authorization", worker.clone()).json(&ack_body).await;
        assert_eq!(own.status_code(), 200, "the worker's own: {}", own.text());
        let again = request.post(ack).add_header("authorization", worker.clone()).json(&ack_body).await;
        assert_eq!(again.json::<Value>()["acknowledged_on"], own.json::<Value>()["acknowledged_on"], "the first day is kept");
        let mine = get("/api/me/policies".to_string(), worker.clone()).await;
        assert_eq!(mine.json::<Value>()["waiting"], 0);
        // A version that is not the newest in force cannot be acknowledged.
        let newer = request.post(&format!("/api/policies/{policy}/versions")).add_header("authorization", hr.clone())
            .json(&json!({ "title": "Conduct", "document_ref": "doc:conduct-2", "effective_on": "2026-02-01", "audience": "all" })).await;
        assert_eq!(newer.status_code(), 200);
        assert_eq!(request.post(ack).add_header("authorization", worker.clone()).json(&ack_body).await.status_code(), 422, "version 1 is superseded");
        assert_eq!(get("/api/me/policies".to_string(), worker.clone()).await.json::<Value>()["waiting"], 1, "version 2 waits");
        let named = format!("/api/policy-versions/{version}/acknowledgements");
        assert_eq!(get(named.clone(), worker.clone()).await.status_code(), 403, "a worker does not read the names");
        let record = get(named.clone(), hr.clone()).await;
        assert_eq!(record.status_code(), 200, "{}", record.text());
        assert!(record.text().contains("PGE-ME"), "HR sees who");
        let counts = format!("/api/workers/{me}/team-policy-counts?version_pid={version}");
        assert_eq!(get(counts.clone(), worker.clone()).await.status_code(), 200, "a manager (here, any worker as themself) reads counts, which are nothing under the floor");
        assert_eq!(get(counts, hr.clone()).await.status_code(), 200);

        // ── Credentials: the worker issues from their own verified entry; the issuer key and the list are public ──
        use workforce_planning_management::rules::credential as cred;
        let public = request.get("/api/credentials/issuer").await;
        assert_eq!(public.status_code(), 200, "a verifier needs no account: {}", public.text());
        let public: Value = public.json();
        assert_eq!(public["jwk"]["x"], "11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo");
        assert!(public["jwk"].get("d").is_none(), "the private half never leaves");
        let x: [u8; 32] = base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, public["jwk"]["x"].as_str().unwrap()).unwrap().try_into().unwrap();
        assert_eq!(request.post("/api/me/credentials").json(&json!({ "cpd_entry_pid": uuid::Uuid::new_v4() })).await.status_code(), 401);
        let entry = uuid::Uuid::new_v4();
        ctx.db.execute_unprepared(&format!(
            "INSERT INTO cpd_entries (pid, worker_pid, entry_date, activity, category, unit, amount_hundredths, source) \
             VALUES ('{entry}', '{me}', '2026-09-01', 'Safeguarding refresher', 'course', 'hours', 600, 'self')")).await.unwrap();
        let issue = |entry: uuid::Uuid| {
            let (request, worker) = (&request, worker.clone());
            async move { request.post("/api/me/credentials").add_header("authorization", worker).json(&json!({ "cpd_entry_pid": entry })).await }
        };
        assert_eq!(issue(entry).await.status_code(), 422, "an unverified entry is not issued");
        ctx.db.execute_unprepared(&format!("UPDATE cpd_entries SET verified_on = '2026-09-02', verified_by = 'hr' WHERE pid = '{entry}'")).await.unwrap();
        assert_eq!(issue(uuid::Uuid::new_v4()).await.status_code(), 404, "an entry that is not there");
        let issued = issue(entry).await;
        assert_eq!(issued.status_code(), 200, "{}", issued.text());
        let issued: Value = issued.json();
        let payload = cred::verify(&x, issued["credential"].as_str().unwrap()).expect("the credential verifies against the published key");
        assert_eq!(payload["type"][1], "OpenBadgeCredential");
        assert_eq!(payload["credentialSubject"]["achievement"]["name"], "Safeguarding refresher");
        assert!(!payload.to_string().contains("Pay Gap Enforcement"), "no person is named in it");
        assert!(!payload.to_string().contains(&me), "nor is a worker pid");
        assert_eq!(issue(entry).await.status_code(), 422, "one live credential per entry");
        let index = issued["status_index"].as_u64().unwrap() as usize;
        let list: Value = request.get("/api/credentials/status-list").await.json();
        let signed = cred::verify(&x, list["jwt"].as_str().unwrap()).expect("the list verifies too");
        assert_eq!(cred::is_revoked(signed["credentialSubject"]["encodedList"].as_str().unwrap(), index), Ok(false));
        let mine: Value = get("/api/me/credentials".to_string(), worker.clone()).await.json();
        assert!(!mine.to_string().contains(issued["credential"].as_str().unwrap()), "the credential is not kept");
        // A worker cannot revoke through the HR route; HR can; and then the list says so and the entry can be issued again.
        let pid = issued["pid"].as_str().unwrap().to_string();
        assert_eq!(request.post(&format!("/api/credentials/{pid}/revoke")).add_header("authorization", worker.clone()).await.status_code(), 403);
        assert_eq!(request.post(&format!("/api/credentials/{pid}/revoke")).add_header("authorization", hr.clone()).await.status_code(), 200);
        assert_eq!(request.post(&format!("/api/me/credentials/{pid}/revoke")).add_header("authorization", worker.clone()).await.status_code(), 422, "already revoked");
        let list: Value = request.get("/api/credentials/status-list").await.json();
        let signed = cred::verify(&x, list["jwt"].as_str().unwrap()).unwrap();
        assert_eq!(cred::is_revoked(signed["credentialSubject"]["encodedList"].as_str().unwrap(), index), Ok(true));
        let again = issue(entry).await;
        assert_eq!(again.status_code(), 200, "a revoked credential can be replaced");
        assert_ne!(again.json::<Value>()["status_index"], issued["status_index"], "with a place of its own on the list");
        let second = again.json::<Value>()["pid"].as_str().unwrap().to_string();
        assert_eq!(request.post(&format!("/api/me/credentials/{second}/revoke")).add_header("authorization", worker.clone()).await.status_code(), 200, "the worker withdraws their own");

        // ── The crosswalk: anyone signed in reads it; HR states it; a worker does not ──
        let mapping = json!({ "from_skill_pid": uuid::Uuid::new_v4(), "to_skill_pid": uuid::Uuid::new_v4(), "confidence": "exact", "basis": "authority", "source": "x" });
        assert_eq!(request.get("/api/skill-mappings").await.status_code(), 401);
        assert_eq!(get("/api/skill-mappings".to_string(), worker.clone()).await.status_code(), 200);
        assert_eq!(request.post("/api/skill-mappings").add_header("authorization", worker.clone()).json(&mapping).await.status_code(), 403, "a worker states no mapping");
        assert_eq!(request.post("/api/skill-mappings").add_header("authorization", hr.clone()).json(&mapping).await.status_code(), 422, "HR may; these skills do not exist");

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

        // ── Webhooks: administrators only, and the secret is a name, not a value ─────────────────────
        let subs = "/api/webhook-subscriptions";
        let new = json!({ "name": "Payroll", "url": "https://hooks.example/wpm", "kinds": ["worker_terminated"], "secret_ref": "WPM_WEBHOOK_SECRET_PAYROLL" });
        assert_eq!(request.get(subs).await.status_code(), 401);
        assert_eq!(get(subs.to_string(), worker.clone()).await.status_code(), 403, "a worker");
        assert_eq!(get(subs.to_string(), hr.clone()).await.status_code(), 403, "HR is not an administrator");
        assert_eq!(request.post(subs).add_header("authorization", hr.clone()).json(&new).await.status_code(), 403, "HR may not subscribe");
        assert_eq!(request.post(subs).add_header("authorization", worker.clone()).json(&new).await.status_code(), 403);
        let made = request.post(subs).add_header("authorization", admin.clone()).json(&new).await;
        assert_eq!(made.status_code(), 200, "an administrator subscribes: {}", made.text());
        let hook: Value = made.json();
        assert_eq!(get(subs.to_string(), admin.clone()).await.status_code(), 200);
        let log = format!("{subs}/{}/deliveries", hook["pid"].as_str().unwrap());
        assert_eq!(get(log.clone(), hr.clone()).await.status_code(), 403);
        assert_eq!(get(log, admin.clone()).await.status_code(), 200);
        assert_eq!(request.delete(&format!("{subs}/{}", hook["pid"].as_str().unwrap())).add_header("authorization", hr.clone()).await.status_code(), 403);
        assert_eq!(request.delete(&format!("{subs}/{}", hook["pid"].as_str().unwrap())).add_header("authorization", admin.clone()).await.status_code(), 200);

        // ── Positions: HR plans and links; a reader sees that a seat is held, not by whom ───────────────
        let seat = json!({ "organization_ref": org, "cost_centre": "CC-9", "title": "Engineer", "budget_fte_percent": 100, "funding_kind": "core" });
        assert_eq!(request.post("/api/positions").json(&seat).await.status_code(), 401);
        assert_eq!(request.post("/api/positions").add_header("authorization", worker.clone()).json(&seat).await.status_code(), 403, "a worker");
        let made = request.post("/api/positions").add_header("authorization", hr.clone()).json(&seat).await;
        assert_eq!(made.status_code(), 200, "HR plans a seat: {}", made.text());
        let seat_pid = made.json::<Value>()["pid"].as_str().unwrap().to_string();
        let someone = ctx.db.query_one_raw(sea_orm::Statement::from_string(sea_orm::DatabaseBackend::Postgres,
            format!("SELECT pid::text AS pid FROM workers WHERE organization_ref = '{org}' LIMIT 1"))).await.unwrap().unwrap();
        let someone: String = someone.try_get("", "pid").unwrap();
        for step in ["approved"] {
            let moved = request.post(&format!("/api/positions/{seat_pid}/status")).add_header("authorization", hr.clone()).json(&json!({ "to": step })).await;
            assert_eq!(moved.status_code(), 200, "{}", moved.text());
        }
        assert_eq!(request.put(&format!("/api/positions/{seat_pid}/holder")).add_header("authorization", worker.clone()).json(&json!({ "worker_pid": someone })).await.status_code(), 403);
        let linked = request.put(&format!("/api/positions/{seat_pid}/holder")).add_header("authorization", hr.clone()).json(&json!({ "worker_pid": someone })).await;
        assert_eq!(linked.status_code(), 200, "{}", linked.text());
        let listing = format!("/api/positions?organization={org}");
        let as_hr: Value = get(listing.clone(), hr.clone()).await.json();
        assert_eq!(as_hr["positions"][0]["holder_worker_pid"], someone, "HR sees who holds it");
        // Outside the organization the worker sees nothing; as a plain member of it they see the seat and not the holder.
        assert_eq!(get(listing.clone(), worker.clone()).await.status_code(), 404);
        ctx.db.execute_unprepared(&format!(
            "INSERT INTO organization_memberships (pid, person_ref, organization_ref, role, starts_on) VALUES ('{}', 'person:{worker_p}', '{org}', 'member', '2020-01-01')",
            uuid::Uuid::new_v4())).await.unwrap();
        let as_worker = get(listing.clone(), worker.clone()).await;
        assert_eq!(as_worker.status_code(), 200, "{}", as_worker.text());
        let as_worker: Value = as_worker.json();
        assert!(!as_worker.to_string().contains(&someone), "a reader does not see the holder: {as_worker}");
        assert_eq!(as_worker["positions"][0]["held"], true);
        let est_worker: Value = get(format!("/api/positions/establishment?organization={org}"), worker.clone()).await.json();
        assert_eq!(est_worker["small_group_floor"], 10, "a plain member gets the floor");
        assert_eq!(est_worker["cost_centres"][0]["held"], json!({ "status": "withheld" }), "one seat is under it");
        assert_eq!(est_worker["cost_centres"][0]["budgeted"]["status"], "known");
        let est = format!("/api/positions/establishment?organization={org}");
        let est_hr: Value = get(est.clone(), hr.clone()).await.json();
        assert_eq!(est_hr["small_group_floor"], Value::Null, "HR sees the unfloored figures");

        // ── Take the sex answers away and there is nothing to compare ────────────────────
        ctx.db.execute_unprepared("DELETE FROM equality_declarations").await.unwrap();
        let none: Value = get(report.clone(), payroll.clone()).await.json();
        assert_eq!(none["report"]["categories"][0]["basic"], json!({ "status": "unknown", "value": "no_workers" }));
        assert_eq!(none["report"]["exclusions"]["no_sex_declared"], json!({ "status": "exactly", "value": 15 })); // twelve engineers, HR, payroll and the policy test's worker
    })
    .await;
}
