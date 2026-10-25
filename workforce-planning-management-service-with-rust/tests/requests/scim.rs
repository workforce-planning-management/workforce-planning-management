//! SCIM 2.0 outbound provisioning (WPM-R150, WPM-D102), against a stub SCIM server that records every call: managers first,
//! idempotent, retried with back-off, deactivated on leaving, and no contact detail anywhere in a body.

use std::sync::{Arc, Mutex};

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{post, put};
use axum::{Json, Router};
use chrono::{Duration, NaiveDate, Utc};
use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management::tasks::scim_push::{Config, Summary, run};

use super::{a_person, an_org};

#[derive(Default)]
struct Stub {
    calls: Mutex<Vec<(String, String, Value)>>,
    tokens: Mutex<Vec<String>>,
    fail: Mutex<Option<u16>>,
    conflict: Mutex<bool>,
    next: Mutex<u32>,
}

type Shared = Arc<Stub>;

fn note(s: &Stub, headers: &HeaderMap, method: &str, path: &str, body: Value) {
    s.tokens.lock().unwrap().push(
        headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string(),
    );
    s.calls
        .lock()
        .unwrap()
        .push((method.to_string(), path.to_string(), body));
}

async fn post_user(
    State(s): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    note(&s, &headers, "POST", "/Users", body);
    if let Some(code) = *s.fail.lock().unwrap() {
        return (
            StatusCode::from_u16(code).unwrap(),
            Json(json!({ "detail": "SECRET-BODY-FROM-IDP" })),
        );
    }
    if *s.conflict.lock().unwrap() {
        return (StatusCode::CONFLICT, Json(json!({})));
    }
    let mut n = s.next.lock().unwrap();
    *n += 1;
    (
        StatusCode::CREATED,
        Json(json!({ "id": format!("idp-{n}") })),
    )
}

async fn put_user(
    State(s): State<Shared>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    note(&s, &headers, "PUT", &format!("/Users/{id}"), body);
    (StatusCode::OK, Json(json!({ "id": id })))
}

async fn find_user(
    State(s): State<Shared>,
    headers: HeaderMap,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> Json<Value> {
    note(&s, &headers, "GET", "/Users", json!(q));
    Json(json!({ "Resources": [{ "id": "idp-existing" }] }))
}

async fn serve(stub: Shared) -> String {
    let app = Router::new()
        .route("/Users", post(post_user).get(find_user))
        .route("/Users/{id}", put(put_user))
        .with_state(stub);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    url
}

async fn hire(
    request: &loco_rs::TestServer,
    org: &str,
    number: &str,
    manager: Option<&str>,
    hired_on: &str,
) -> String {
    let created: Value = request
        .post("/api/workers")
        .json(&json!({
            "person_ref": a_person(), "organization_ref": org, "worker_number": number,
            "display_name": format!("Scim {number}"), "employment_type": "permanent",
            "department": "engineering", "job_title": "Engineer", "hired_on": hired_on,
            "manager_pid": manager,
        }))
        .await
        .json();
    created["pid"].as_str().expect("pid").to_string()
}

fn today() -> NaiveDate {
    Utc::now().date_naive()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn workers_are_provisioned_once_with_employment_facts_and_retried_when_the_provider_fails() {
    crate::requests::request_open(|request, ctx| async move {
        let stub: Shared = Arc::default();
        let base = serve(stub.clone()).await;
        let org = an_org();
        let manager = hire(&request, &org, "S-1", None, "2020-01-05").await;
        let report = hire(&request, &org, "S-2", Some(&manager), "2020-01-05").await;
        let _later = hire(&request, &org, "S-3", Some(&manager), "2020-01-05").await;
        let future = hire(&request, &org, "S-4", None, &(today() + Duration::days(30)).to_string()).await;
        // Personal contact details exist, and must not leave.
        for pid in [&manager, &report] {
            ctx.db
                .execute_unprepared(&format!(
                    "INSERT INTO worker_contact_details (worker_pid, personal_email, phone_mobile, postcode, on_behalf) VALUES ('{pid}', 'private.person@example.org', '+44 7700 900123', 'ZZ1 1ZZ', false)"
                ))
                .await
                .unwrap();
        }
        let cfg = Config { base_url: base, token: "scim-secret".into(), organization: Some(org.clone()) };
        let now = Utc::now();

        // First run: three created, the not-yet-joined worker skipped, the manager first.
        let first = run(&ctx, &cfg, today(), now).await.unwrap();
        assert_eq!(first, Summary { created: 3, skipped: 1, ..Summary::default() });
        {
            let calls = stub.calls.lock().unwrap();
            let posts: Vec<&Value> = calls.iter().filter(|c| c.0 == "POST").map(|c| &c.2).collect();
            assert_eq!(posts[0]["userName"], "S-1", "the manager is sent first");
            let enterprise = "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User";
            assert!(posts[0][enterprise].get("manager").is_none());
            assert_eq!(posts[1][enterprise]["manager"], json!({ "value": "idp-1" }), "a report carries the provider's id for the manager");
            assert_eq!(posts[1][enterprise]["employeeNumber"], "S-2");
            assert_eq!(posts[1]["externalId"], format!("worker:{report}"));
            assert!(stub.tokens.lock().unwrap().iter().all(|t| t == "Bearer scim-secret"));
            // No contact detail, name of a relative or address left.
            let everything = serde_json::to_string(&*calls).unwrap().to_lowercase();
            for private in ["private.person", "example.org", "7700", "zz1 1zz", "postcode", "phone", "email"] {
                assert!(!everything.contains(private), "{private} left in {everything}");
            }
        }

        // Second run: nothing to do, nothing sent.
        let before = stub.calls.lock().unwrap().len();
        assert_eq!(run(&ctx, &cfg, today(), now).await.unwrap(), Summary { unchanged: 3, skipped: 1, ..Summary::default() });
        assert_eq!(stub.calls.lock().unwrap().len(), before);

        // A change in title replaces one user; a leaver is deactivated.
        ctx.db.execute_unprepared(&format!("UPDATE workers SET job_title = 'Senior Engineer' WHERE pid = '{report}'")).await.unwrap();
        let left = (today() - Duration::days(1)).to_string();
        ctx.db.execute_unprepared(&format!("UPDATE workers SET terminated_on = '{left}' WHERE worker_number = 'S-3'")).await.unwrap();
        let third = run(&ctx, &cfg, today(), now).await.unwrap();
        assert_eq!(third, Summary { replaced: 2, unchanged: 1, skipped: 1, ..Summary::default() });
        {
            let calls = stub.calls.lock().unwrap();
            let puts: Vec<&(String, String, Value)> = calls.iter().filter(|c| c.0 == "PUT").collect();
            assert!(puts.iter().any(|c| c.2["userName"] == "S-3" && c.2["active"] == false), "the leaver is deactivated");
            assert!(puts.iter().any(|c| c.2["userName"] == "S-2" && c.2["title"] == "Senior Engineer"));
        }

        // The future joiner is sent once their first day comes.
        let joined = run(&ctx, &cfg, today() + Duration::days(30), now).await.unwrap();
        assert_eq!(joined.created, 1, "{joined:?}");
        let _ = future;

        // A provider failure is recorded as a status and retried after a back-off; nothing of its body is kept.
        ctx.db.execute_unprepared("DELETE FROM scim_pushes WHERE worker_pid = (SELECT pid FROM workers WHERE worker_number = 'S-4')").await.unwrap();
        *stub.fail.lock().unwrap() = Some(503);
        let failed = run(&ctx, &cfg, today() + Duration::days(30), now).await.unwrap();
        assert_eq!(failed.failed, 1, "{failed:?}");
        let rows = ctx.db.query_all_raw(sea_orm::Statement::from_string(sea_orm::DatabaseBackend::Postgres,
            "SELECT last_error, attempts FROM scim_pushes WHERE last_error IS NOT NULL".to_string())).await.unwrap();
        assert_eq!(rows.len(), 1);
        let error: String = rows[0].try_get("", "last_error").unwrap();
        assert_eq!(error, "HTTP 503");
        assert!(!error.contains("SECRET"));
        let waiting = run(&ctx, &cfg, today() + Duration::days(30), now).await.unwrap();
        assert_eq!(waiting.waiting, 1, "inside the back-off nothing is sent");
        // The first retry is one minute away, as the spec says ("one minute doubling"): not yet at 59 seconds, due at 61.
        let early = run(&ctx, &cfg, today() + Duration::days(30), now + Duration::seconds(59)).await.unwrap();
        assert_eq!(early.waiting, 1, "{early:?}");
        let on_time = run(&ctx, &cfg, today() + Duration::days(30), now + Duration::seconds(61)).await.unwrap();
        assert_eq!(on_time.failed, 1, "one minute after the first failure it is tried again (and fails again, as the provider is still down): {on_time:?}");
        *stub.fail.lock().unwrap() = None;
        let later = run(&ctx, &cfg, today() + Duration::days(30), now + Duration::minutes(5)).await.unwrap();
        assert_eq!(later.created, 1, "after the wait it is tried again and works: {later:?}");

        // A user the provider already has is found by externalId and replaced, not duplicated.
        ctx.db.execute_unprepared("DELETE FROM scim_pushes WHERE worker_pid = (SELECT pid FROM workers WHERE worker_number = 'S-1')").await.unwrap();
        *stub.conflict.lock().unwrap() = true;
        let again = run(&ctx, &cfg, today() + Duration::days(30), now + Duration::minutes(10)).await.unwrap();
        assert_eq!(again.created, 1, "{again:?}");
        assert!(stub.calls.lock().unwrap().iter().any(|c| c.0 == "PUT" && c.1 == "/Users/idp-existing"));
    })
    .await;
}
