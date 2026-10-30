//! Request-level test suites: the hire journey + worker lifecycle
//! in [`hr`], the time / leave / shift flows in [`workforce`], the
//! payroll derivation in [`payroll`], the L&D surface in [`learning`],
//! the aptitude / personality / psychometric / selection tests in
//! [`assessments`], and the talent-strategy surface (development plans,
//! pipelines, early careers, workforce intelligence) in [`talent`].

mod adjustments;
mod ai_register;
mod announcements;
mod appraisals;
mod assessments;
mod assignments;
mod audit_chain;
mod audit_export;
mod bargaining;
mod capacity;
mod change;
mod consultations;
mod contacts;
mod cpd_overview;
mod cpd_requirements;
mod crosswalk;
mod development;
mod directory_paging;
mod duplicates;
mod engagements;
mod equipment;
mod erasure_replay;
mod ergonomics;
mod escalations;
mod expenses;
mod grades;
mod health_detail;
mod hr;
mod human_capital;
mod idempotency;
mod intelligence_overview;
mod job_levels;
mod learning;
mod lists;
mod lms_links;
mod mcp;
mod mobility;
mod movements;
mod org_shape;
mod organizations;
mod pagination;
mod pay_assessment;
mod pay_information;
mod pay_positions;
mod pay_range;
mod pay_scales;
mod pay_transparency;
mod payroll;
mod plan_views;
mod policies;
mod positions;
mod privacy;
mod removals;
mod rotas;
mod scale;
mod scim;
mod sfia_import;
mod shared_cache;
mod skill_gaps;
mod statutory_defaults;
mod talent;
mod tolerances;
mod webhooks;
mod wellbeing;
mod workforce;

/// A fresh synthetic `person:` URN.
pub fn a_person() -> String {
    format!("person:{}", uuid::Uuid::new_v4())
}

/// A fresh synthetic `organization:` URN.
pub fn an_org() -> String {
    format!("organization:{}", uuid::Uuid::new_v4())
}

/// A fresh synthetic `worker:` URN.
pub fn a_worker() -> String {
    format!("worker:{}", uuid::Uuid::new_v4())
}

/// A JSON value as text with its random ids and its timestamps blanked, for a test that checks the text
/// carries no figure: a figure such as "5500" can occur by chance inside a UUID (it did, in CI on 2026-10-30)
/// or a timestamp, and that is not a leak.
pub fn text_without_ids(value: &serde_json::Value) -> String {
    fn blank(v: &serde_json::Value) -> serde_json::Value {
        match v {
            serde_json::Value::String(s)
                if uuid::Uuid::parse_str(s).is_ok()
                    || chrono::DateTime::parse_from_rfc3339(s).is_ok() =>
            {
                serde_json::Value::String(String::new())
            }
            serde_json::Value::Array(a) => serde_json::Value::Array(a.iter().map(blank).collect()),
            serde_json::Value::Object(o) => {
                serde_json::Value::Object(o.iter().map(|(k, v)| (k.clone(), blank(v))).collect())
            }
            other => other.clone(),
        }
    }
    serde_json::to_string(&blank(value)).unwrap_or_default()
}

/// Create one active worker (no onboarding gate: the status stays
/// `onboarding` unless a test activates it). Expands to a value
/// (`String`, the worker pid), await it like a function call.
///
/// A macro, not a function: the `request` handed to each test by
/// loco's `request()` helper is a `loco_rs`-internal-pinned
/// `loco_rs::TestServer` — a type this crate cannot name directly
/// without pulling in that exact same `axum-test` version as a direct
/// dependency (defeating the point of tracking a newer one for the
/// dev-dependency's own sake). Expanding inline sidesteps naming the
/// type at all; the actual value's type is inferred at each call site.
macro_rules! seed_worker {
    ($request:expr, $org:expr, $number:expr, $salary_minor:expr) => {
        async {
            let request = $request;
            let org = $org;
            let number = $number;
            let salary_minor: Option<i64> = $salary_minor;
            let body = serde_json::json!({
                "person_ref": $crate::requests::a_person(),
                "organization_ref": org,
                "worker_number": number,
                "display_name": format!("Test Worker {number}"),
                "employment_type": "permanent",
                "department": "engineering",
                "job_title": "Engineer",
                "salary_minor": salary_minor,
                "salary_currency": salary_minor.map(|_| "GBP"),
                "hired_on": "2026-01-05",
            });
            let created: serde_json::Value =
                request.post("/api/workers").json(&body).await.json();
            created["pid"].as_str().expect("worker pid").to_string()
        }
    };
}
pub(crate) use seed_worker;

/// Activate a worker (no mandatory onboarding items exist in the
/// test flows unless a test adds them). Same macro rationale as
/// [`seed_worker`].
macro_rules! activate {
    ($request:expr, $pid:expr) => {
        async {
            let request = $request;
            let pid = $pid;
            let response = request
                .post(&format!("/api/workers/{pid}/status"))
                .json(&serde_json::json!({ "to": "active" }))
                .await;
            assert_eq!(response.status_code(), 200, "activate {pid}");
        }
    };
}
pub(crate) use activate;

/// Boot the app and run `callback` with sign-in enforcement and rate limits
/// **explicitly off**. Enforcement is on by default (WPM-D52), so every suite that is not
/// about authentication says so here. The flag is read once per process, so
/// it is set before the first boot; the auth behaviour itself is covered by
/// the `enforcement*` and `keycloak` test binaries, each in its own process.
#[allow(clippy::future_not_send)]
pub async fn request_open<F, Fut>(callback: F)
where
    F: FnOnce(loco_rs::TestServer, loco_rs::app::AppContext) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    static OPEN: std::sync::Once = std::sync::Once::new();
    OPEN.call_once(|| {
        // `set_var` is `unsafe` in edition 2024; this runs once, before any
        // boot reads the flag.
        unsafe {
            std::env::set_var("WPM_REQUIRE_AUTH", "0");
            // The suites share one caller bucket; the limiter has its own
            // test binary (`tests/security.rs`).
            std::env::set_var("WPM_RATE_LIMIT_PER_MINUTE", "0");
            std::env::set_var("WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE", "0");
        };
    });
    loco_rs::testing::prelude::request::<workforce_planning_management::app::App, _, _>(callback)
        .await;
}

/// Run set-up DDL for a test (a trigger that refuses audit rows, and its removal). The runtime database role may not change the schema,
/// so the least-privilege drill (`scripts/verify-db-roles.sh`, WPM-R123) sets `WPM_TEST_DDL_URL` to the schema owner's URL and the DDL
/// goes through that connection; everywhere else it goes through the test's own connection.
pub(crate) async fn ddl(ctx: &loco_rs::app::AppContext, sql: &str) {
    use sea_orm::ConnectionTrait;
    match std::env::var("WPM_TEST_DDL_URL")
        .ok()
        .filter(|u| !u.trim().is_empty())
    {
        Some(url) => {
            let owner = sea_orm::Database::connect(&url)
                .await
                .expect("the schema owner's connection");
            owner
                .execute_unprepared(sql)
                .await
                .expect("the DDL runs as the schema owner");
            owner
                .close()
                .await
                .expect("the schema owner's connection closes");
        }
        None => {
            ctx.db.execute_unprepared(sql).await.expect("the DDL runs");
        }
    }
}
