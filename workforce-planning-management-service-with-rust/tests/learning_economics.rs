//! Learning economics under enforcement (WPM-R227 to WPM-R230), against the shipped reference policy: what a plan costs, who may
//! see it, what a masked reader is told, train or hire, a stated benefit, and spend against a budget. Its own test binary,
//! because the auth flag and policy are cached in process-wide `OnceLock`s.

use axa_test::prelude::*;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::SigningKey;
use rusty_paseto::core::{Footer, Key, Paseto, PasetoAsymmetricPrivateKey, Payload, Public, V4};
use serde_json::{Value, json};
use serial_test::serial;
use sha2::{Digest, Sha256};
use workforce_planning_management::app::App;

const ISSUER: &str = "authentication-service";
const AUDIENCE: &str = "main-x-service";
/// Throwaway Ed25519 seed — mints test tokens only, never a secret.
const SEED: [u8; 32] = [11; 32];
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

fn enforce() -> String {
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
    }
    kid
}

fn bearer(token: String) -> String {
    format!("Bearer {token}")
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

/// Create an active worker with a place of work.
async fn worker(
    request: &axa_test::TestServer,
    svc: &str,
    org: &str,
    person: &str,
    number: &str,
    salary: Option<i64>,
) -> String {
    let created: Value = request
        .post("/api/workers")
        .add_header("authorization", svc.to_string())
        .json(&json!({
            "person_ref": format!("person:{person}"), "organization_ref": org,
            "worker_number": number, "display_name": format!("Person {number}"),
            "employment_type": "permanent", "department": "engineering",
            "job_title": "Engineer", "hired_on": "2020-01-05",
            "salary_minor": salary, "salary_currency": salary.map(|_| "GBP"),
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
    request
        .put(&format!("/api/workers/{pid}"))
        .add_header("authorization", svc.to_string())
        .json(&json!({ "work_jurisdiction": "XA" }))
        .await
        .assert_status_ok();
    pid
}

/// One dated row of a jurisdiction profile, through the API so the registry's checks apply.
async fn parameter(request: &axa_test::TestServer, svc: &str, org: &str, key: &str, value: &Value) {
    request
        .post("/api/jurisdiction-parameters")
        .add_header("authorization", svc.to_string())
        .json(&json!({ "organization_ref": org, "code": "XA", "key": key, "value": value,
                       "effective_from": "2000-01-01", "source": "synthetic test profile", "status": "example" }))
        .await
        .assert_status_ok();
}

async fn a_skill(request: &axa_test::TestServer, svc: &str) -> String {
    let skill: Value = request
        .post("/api/skills")
        .add_header("authorization", svc.to_string())
        .json(
            &json!({ "name": format!("skill {}", uuid::Uuid::new_v4()), "category": "technical" }),
        )
        .await
        .json();
    skill["pid"].as_str().expect("skill pid").to_string()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test learning_economics -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn a_plan_costs_fees_and_hours_for_the_person_and_hr_only() {
    let kid = enforce();
    let (p1, p2, boss) = (
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
    );
    let me = bearer(sign_as(&kid, &p1.to_string(), &[]));
    let neighbour = bearer(sign_as(&kid, &p2.to_string(), &[]));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let hr = bearer(sign_as(&kid, "hr-econ", &[("hr", &["true"])]));
    let hr_elsewhere = bearer(sign_as(&kid, "hr-elsewhere", &[("hr", &["true"])]));
    let payroll = bearer(sign_as(&kid, "pay-econ", &[("payroll", &["true"])]));
    let manager = bearer(sign_as(&kid, &boss.to_string(), &[]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        seat(&ctx, "hr-econ", &org).await;
        seat(&ctx, "pay-econ", &org).await;
        let mine = worker(&request, &svc, &org, &p1.to_string(), "EC-1", Some(3_600_000)).await;
        worker(&request, &svc, &org, &p2.to_string(), "EC-2", Some(4_000_000)).await;
        let boss_pid = worker(&request, &svc, &org, &boss.to_string(), "EC-3", Some(5_000_000)).await;
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "UPDATE workers SET manager_pid = '{boss_pid}' WHERE pid = '{mine}'"
            ))).execute(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
        let skill = a_skill(&request, &svc).await;
        let priced: Value = request
            .post(&format!("/api/skills/{skill}/courses"))
            .add_header("authorization", svc.clone())
            .json(&json!({ "course_ref": "RUST-1", "title": "Rust", "hours": 10,
                           "fee_minor": 50_000, "fee_currency": "GBP" }))
            .await
            .json();
        assert_eq!(priced["fee_minor"], 50_000);
        let half = request
            .post(&format!("/api/skills/{skill}/courses"))
            .add_header("authorization", svc.clone())
            .json(&json!({ "course_ref": "RUST-X", "title": "Half", "hours": 1, "fee_minor": 5 }))
            .await;
        assert_eq!(half.status_code(), 422, "a fee with no currency is refused");
        let unpriced_skill = a_skill(&request, &svc).await;
        let unpriced: Value = request
            .post(&format!("/api/skills/{unpriced_skill}/courses"))
            .add_header("authorization", svc.clone())
            .json(&json!({ "course_ref": "RUST-2", "title": "More Rust", "hours": 20 }))
            .await
            .json();
        assert_eq!(unpriced["fee_minor"], Value::Null, "no fee is unknown");

        let plan: Value = request
            .post(&format!("/api/workers/{mine}/development-plans"))
            .add_header("authorization", svc.clone())
            .json(&json!({ "kind": "upskill", "items": [
                { "skill_pid": skill, "current_level": 1, "target_level": 2, "method": "course", "course_ref": "RUST-1" },
                { "skill_pid": unpriced_skill, "current_level": 1, "target_level": 2, "method": "course", "course_ref": "RUST-2" },
            ]}))
            .await
            .json();
        let plan = plan["pid"].as_str().expect("plan pid").to_string();
        let cost_url = format!("/api/development-plans/{plan}/cost");

        // Nothing configured: the fees are known in part, the hours unresolved, and it says why.
        let cost: Value = request
            .get(&cost_url)
            .add_header("authorization", me.clone())
            .await
            .json();
        assert_eq!(cost["cost"]["fees"]["GBP"], 50_000);
        assert_eq!(cost["cost"]["courses_with_fee"], 1);
        assert_eq!(cost["cost"]["courses_without_fee"], 1);
        assert_eq!(cost["cost"]["hours"], 30);
        assert_eq!(cost["hours_cost"]["status"], "unresolved");
        assert_eq!(cost["hours_cost"]["missing"], json!(["on_cost", "full_time_annual_hours"]));

        parameter(&request, &hr, &org, "learning.on_cost_bp", &json!(2500)).await;
        parameter(&request, &hr, &org, "learning.full_time_annual_hours", &json!(1800)).await;

        let cost: Value = request
            .get(&cost_url)
            .add_header("authorization", me.clone())
            .await
            .json();
        assert_eq!(cost["hours_cost"]["status"], "resolved", "{cost}");
        assert_eq!(cost["hours_cost"]["hourly_minor"], 2_500);
        assert_eq!(cost["hours_cost"]["minor"], 75_000);
        assert_eq!(cost["cost"]["fees"]["GBP"], 50_000, "the hours are not added to the fees");

        // HR's read is masked under the reference policy, so the salary cannot be read back out of an hourly rate.
        let as_hr = request.get(&cost_url).add_header("authorization", hr.clone()).await;
        as_hr.assert_status_ok();
        let as_hr: Value = as_hr.json();
        assert_eq!(as_hr["cost"]["fees"]["GBP"], 50_000);
        assert_eq!(as_hr["hours_cost"]["missing"], json!(["pay_withheld"]));
        assert!(as_hr["hours_cost"].get("hourly_minor").is_none());

        for (who, token) in [("a neighbour", &neighbour), ("the line manager", &manager), ("payroll", &payroll)] {
            let refused = request.get(&cost_url).add_header("authorization", token.clone()).await;
            assert_eq!(refused.status_code(), 403, "{who}");
        }
        let elsewhere = request.get(&cost_url).add_header("authorization", hr_elsewhere.clone()).await;
        assert_eq!(elsewhere.status_code(), 404, "HR with no seat here");

        // Viability: no benefit is unmeasured, and a course with no fee leaves the investment unknown.
        parameter(&request, &hr, &org, "learning.discount_rate_bp", &json!(500)).await;
        let viability: Value = request
            .get(&format!("/api/development-plans/{plan}/viability"))
            .add_header("authorization", me.clone())
            .await
            .json();
        assert_eq!(viability["viability"]["on_fees"], Value::Null);
        assert_eq!(viability["viability"]["missing"], json!(["benefit", "fee"]));

        request
            .put(&format!("/api/skill-courses/{}/fee", unpriced["pid"].as_str().unwrap()))
            .add_header("authorization", hr.clone())
            .json(&json!({ "fee_minor": 0, "fee_currency": "GBP" }))
            .await
            .assert_status_ok();

        // The person states a benefit on their own plan, and not on anyone else's.
        let bad = request
            .post(&format!("/api/me/development-plans/{plan}/benefits"))
            .add_header("authorization", me.clone())
            .json(&json!({ "year": 1, "amount_minor": 200_000, "currency": "GBP", "source": "  " }))
            .await;
        assert_eq!(bad.status_code(), 422, "a benefit needs a source");
        let not_theirs = request
            .post(&format!("/api/me/development-plans/{plan}/benefits"))
            .add_header("authorization", neighbour.clone())
            .json(&json!({ "year": 1, "amount_minor": 200_000, "currency": "GBP", "source": "x" }))
            .await;
        assert_eq!(not_theirs.status_code(), 404);
        let benefit: Value = request
            .post(&format!("/api/me/development-plans/{plan}/benefits"))
            .add_header("authorization", me.clone())
            .json(&json!({ "year": 1, "amount_minor": 200_000, "currency": "GBP",
                           "source": "Avoids one external contract" }))
            .await
            .json();
        let again = request
            .post(&format!("/api/development-plans/{plan}/benefits"))
            .add_header("authorization", hr.clone())
            .json(&json!({ "year": 1, "amount_minor": 1, "currency": "GBP", "source": "x" }))
            .await;
        assert_eq!(again.status_code(), 422, "one benefit a year");

        let viability: Value = request
            .get(&format!("/api/development-plans/{plan}/viability"))
            .add_header("authorization", me.clone())
            .await
            .json();
        let figures = &viability["viability"];
        assert_eq!(figures["rate_bp"], 500);
        assert_eq!(figures["currency"], "GBP");
        // 200 000 a year out at 5 percent is 190 476; less 50 000 of fees is 140 476, and less 75 000 of hours 65 476.
        assert_eq!(figures["on_fees"]["npv_minor"], 140_476);
        assert_eq!(figures["with_hours"]["npv_minor"], 65_476);
        assert_eq!(figures["on_fees"]["verdict"], "viable");
        assert_eq!(viability["benefits"][0]["source"], "Avoids one external contract");

        let masked: Value = request
            .get(&format!("/api/development-plans/{plan}/viability"))
            .add_header("authorization", hr.clone())
            .await
            .json();
        assert_eq!(masked["viability"]["with_hours"], Value::Null);
        assert_eq!(masked["viability"]["on_fees"]["npv_minor"], 140_476);
        assert_eq!(masked["viability"]["missing"], json!(["pay_withheld"]));

        // The benefit is in the person's subject access export, and the person can remove it.
        let export: Value = request
            .get(&format!("/api/workers/{mine}/subject-access"))
            .add_header("authorization", svc.clone())
            .await
            .json();
        assert_eq!(export["development_plan_benefits"].as_array().map(Vec::len), Some(1), "{export}");
        let pid = benefit["pid"].as_str().unwrap();
        request
            .delete(&format!("/api/me/development-plan-benefits/{pid}"))
            .add_header("authorization", me.clone())
            .await
            .assert_status_ok();
        // HR states one and cannot delete it: the policy gives HR no delete. An administrator can, once.
        let another: Value = request
            .post(&format!("/api/development-plans/{plan}/benefits"))
            .add_header("authorization", hr.clone())
            .json(&json!({ "year": 2, "amount_minor": 100_000, "currency": "GBP", "source": "A second estimate" }))
            .await
            .json();
        let another = another["pid"].as_str().unwrap();
        let not_hr = request
            .delete(&format!("/api/development-plan-benefits/{another}"))
            .add_header("authorization", hr.clone())
            .await;
        assert_eq!(not_hr.status_code(), 403);
        seat(&ctx, "admin-econ", &org).await;
        let admin = bearer(sign_as(&kid, "admin-econ", &[("access", &["admin"])]));
        request
            .delete(&format!("/api/development-plan-benefits/{another}"))
            .add_header("authorization", admin.clone())
            .await
            .assert_status_ok();
        let gone = request
            .delete(&format!("/api/development-plan-benefits/{another}"))
            .add_header("authorization", admin.clone())
            .await;
        assert_eq!(gone.status_code(), 404);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test learning_economics -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn train_or_hire_shows_every_input_and_a_gap_is_unknown() {
    let kid = enforce();
    let candidate_person = uuid::Uuid::new_v4();
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let hr = bearer(sign_as(&kid, "hr-train", &[("hr", &["true"])]));
    let payroll = bearer(sign_as(&kid, "pay-train", &[("payroll", &["true"])]));
    let plain = bearer(sign_as(&kid, &candidate_person.to_string(), &[]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        seat(&ctx, "hr-train", &org).await;
        seat(&ctx, "pay-train", &org).await;
        let candidate = worker(&request, &svc, &org, &candidate_person.to_string(), "TH-1", Some(3_650_000)).await;
        let title = format!("Seat {}", uuid::Uuid::new_v4());
        let seat_row = request
            .post("/api/positions")
            .add_header("authorization", hr.clone())
            .json(&json!({ "organization_ref": org, "cost_centre": "engineering", "title": title,
                           "budget_fte_percent": 100, "funding_kind": "core" }))
            .await;
        seat_row.assert_status_ok();
        let seat_row: Value = seat_row.json();
        let position = seat_row["pid"].as_str().expect("position pid").to_string();
        let url = format!("/api/positions/{position}/train-or-hire?candidate={candidate}");

        let nothing: Value = request.get(&url).add_header("authorization", payroll.clone()).await.json();
        assert_eq!(nothing["answer"], "unknown", "{nothing}");
        assert_eq!(nothing["estimate"], true);
        let missing = nothing["missing"].as_array().unwrap();
        for name in ["seat_skills", "recruitment_cost", "vacancy_days", "on_cost", "seat_cost"] {
            assert!(missing.iter().any(|m| m == name), "{name} in {missing:?}");
        }

        let refused = request.get(&url).add_header("authorization", plain.clone()).await;
        assert_eq!(refused.status_code(), 403, "a plain worker");

        // Give the seat its skills, the candidate a declared level, and every parameter.
        let skill = a_skill(&request, &svc).await;
        request
            .post(&format!("/api/skills/{skill}/courses"))
            .add_header("authorization", svc.clone())
            .json(&json!({ "course_ref": "SEAT-1", "title": "Seat course", "hours": 1000, "levels": 2,
                           "fee_minor": 100_000, "fee_currency": "GBP" }))
            .await
            .assert_status_ok();
        let (role, line, plan_row) = (uuid::Uuid::new_v4(), uuid::Uuid::new_v4(), uuid::Uuid::new_v4());
        for statement in [
            format!("INSERT INTO role_profiles (pid, job_title) VALUES ('{role}', '{title}')"),
            format!("INSERT INTO role_skill_requirements (pid, role_profile_pid, skill_pid, min_proficiency, importance) VALUES ('{}', '{role}', '{skill}', 3, 'critical')", uuid::Uuid::new_v4()),
            format!("INSERT INTO worker_skills (pid, worker_pid, skill_pid, proficiency, assessed_on) VALUES ('{}', '{candidate}', '{skill}', 1, '2026-01-01')", uuid::Uuid::new_v4()),
            format!("INSERT INTO plan_demand_lines (pid, plan_pid, department, role_profile_pid, target_headcount, target_on, basis) VALUES ('{line}', '{plan_row}', 'engineering', '{role}', 1, '2030-01-01', 'permanent')"),
            format!("UPDATE positions SET demand_line_pid = '{line}' WHERE pid = '{position}'"),
            format!("INSERT INTO benchmarks (pid, job_title, currency, min_minor, median_minor, max_minor, source, as_of) VALUES ('{}', '{title}', 'GBP', 3000000, 3650000, 4000000, 'test', '2026-01-01')", uuid::Uuid::new_v4()),
            format!("INSERT INTO requisitions (pid, organization_ref, job_title, department, status, basis, opened_on, filled_on, headcount) VALUES ('{}', '{org}', 'Filled', 'engineering', 'filled', 'permanent', '2026-01-01', '2026-03-02', 1)", uuid::Uuid::new_v4()),
        ] {
            sqlx::raw_sql(sqlx::AssertSqlSafe(&*statement)).execute(workforce_planning_management::db::pool(&ctx)).await.unwrap();
        }
        for (key, value) in [
            ("learning.on_cost_bp", json!(0)),
            ("learning.full_time_annual_hours", json!(1800)),
            ("learning.ramp_up_days", json!(30)),
            ("recruitment.cost_minor", json!(500_000)),
            ("recruitment.currency", json!(["GBP"])),
            ("recruitment.ramp_up_days", json!(90)),
        ] {
            parameter(&request, &hr, &org, key, &value).await;
        }

        let answer: Value = request.get(&url).add_header("authorization", payroll.clone()).await.json();
        // Train: 100 000 fees, 1000 of 1800 hours of 3 650 000 is 2 027 778, and 30 days at 10 000 is 300 000.
        assert_eq!(answer["train"]["fees_minor"], 100_000, "{answer}");
        assert_eq!(answer["train"]["hours_cost_minor"], 2_027_778);
        assert_eq!(answer["train"]["ramp_up_minor"], 300_000);
        assert_eq!(answer["train"]["total_minor"], 2_427_778);
        // Hire: 500 000, 60 days empty, 90 days of ramp-up, and no pay difference.
        assert_eq!(answer["hire"]["vacancy_days"], 60);
        assert_eq!(answer["hire"]["filled_requisitions"], 1);
        assert_eq!(answer["hire"]["seat_cost_source"], "benchmark");
        assert_eq!(answer["hire"]["total_minor"], 2_000_000);
        assert_eq!(answer["answer"], "hire");
        assert_eq!(answer["missing"], json!([]));

        let masked: Value = request.get(&url).add_header("authorization", hr.clone()).await.json();
        assert_eq!(masked["salary_visible"], false);
        assert_eq!(masked["answer"], "unknown");
        assert_eq!(masked["train"]["total_minor"], Value::Null);
        assert_eq!(masked["hire"]["pay_difference_minor"], Value::Null);
        assert_eq!(masked["train"]["fees_minor"], 100_000);

        let audits: i64 = sqlx::query_scalar(
            "SELECT count(*) AS n FROM audit_logs WHERE action = 'salary_read' AND entity_pid = $1::uuid",
        )
        .bind(candidate.to_string())
        .fetch_one(workforce_planning_management::db::pool(&ctx))
        .await
        .unwrap();
        assert_eq!(audits, 2, "each unmasked answer is a salary read, the masked one is not");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test learning_economics -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn spend_is_recorded_fees_against_a_budget_per_currency() {
    let kid = enforce();
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let hr = bearer(sign_as(&kid, "hr-budget", &[("hr", &["true"])]));
    let plain = bearer(sign_as(&kid, "plain-budget", &[]));
    let person = uuid::Uuid::new_v4();

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        seat(&ctx, "hr-budget", &org).await;
        let one = worker(&request, &svc, &org, &person.to_string(), "LB-1", None).await;
        let skill = a_skill(&request, &svc).await;
        request
            .post(&format!("/api/skills/{skill}/courses"))
            .add_header("authorization", svc.clone())
            .json(&json!({ "course_ref": format!("B-{org}"), "title": "Priced", "hours": 5,
                           "fee_minor": 30_000, "fee_currency": "GBP" }))
            .await
            .assert_status_ok();
        for (course, on) in [
            (format!("B-{org}"), "2026-05-01"),
            ("NOT-IN-CATALOGUE".to_string(), "2026-06-01"),
            (format!("B-{org}"), "2025-06-01"),
        ] {
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                    "INSERT INTO training_enrollments (pid, worker_pid, course_ref, status, completed_on) \
                     VALUES ('{}', '{one}', '{course}', 'completed', '{on}')",
                    uuid::Uuid::new_v4()
                ))).execute(workforce_planning_management::db::pool(&ctx))
                .await
                .unwrap();
        }
        request
            .put("/api/learning-budgets")
            .add_header("authorization", hr.clone())
            .json(&json!({ "organization_ref": org, "department": "engineering", "year": 2026,
                           "amount_minor": 100_000, "currency": "GBP" }))
            .await
            .assert_status_ok();
        request
            .put("/api/learning-budgets")
            .add_header("authorization", hr.clone())
            .json(&json!({ "organization_ref": org, "department": "engineering", "year": 2026,
                           "amount_minor": 120_000, "currency": "GBP" }))
            .await
            .assert_status_ok();
        let url = format!("/api/learning-budgets?organization={}&year=2026", org.replace(':', "%3A"));
        let view: Value = request.get(&url).add_header("authorization", hr.clone()).await.json();
        let department = &view["departments"][0];
        assert_eq!(department["department"], "engineering", "{view}");
        assert_eq!(department["completions_without_fee"], 1);
        assert_eq!(
            department["lines"],
            json!([{ "currency": "GBP", "budget_minor": 120_000, "spent_minor": 30_000, "remaining_minor": 90_000 }])
        );
        let refused = request.get(&url).add_header("authorization", plain.clone()).await;
        assert_eq!(refused.status_code(), 403);
    })
    .await;
}
