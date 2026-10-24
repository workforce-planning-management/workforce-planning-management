//! Outbound webhooks (WPM-R151, WPM-D106): a subscription is told about an event with a signed reference, a retry is backed
//! off, the log holds no body, and nothing is queued for a change that did not happen.

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use chrono::{Duration, Utc};
use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management::rules::webhook::{PAYLOAD_FIELDS, verify};
use workforce_planning_management::tasks::webhook_deliver::{Summary, run};

use super::{a_person, an_org};

#[derive(Default)]
struct Stub {
    received: Mutex<Vec<(HeaderMap, String)>>,
    answer: Mutex<u16>,
}

async fn hook(State(s): State<Arc<Stub>>, headers: HeaderMap, body: String) -> StatusCode {
    s.received.lock().unwrap().push((headers, body));
    StatusCode::from_u16(*s.answer.lock().unwrap()).unwrap()
}

async fn serve(stub: Arc<Stub>) -> String {
    let app = Router::new().route("/hook", post(hook)).with_state(stub);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/hook", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    url
}

async fn worker(request: &loco_rs::TestServer, number: &str) -> String {
    let created: Value = request
        .post("/api/workers")
        .json(&json!({
            "person_ref": a_person(), "organization_ref": an_org(), "worker_number": number,
            "display_name": "Secret Name Person", "employment_type": "permanent",
            "department": "engineering", "job_title": "Engineer", "hired_on": "2020-01-05",
        }))
        .await
        .json();
    created["pid"].as_str().unwrap().to_string()
}

async fn status(request: &loco_rs::TestServer, pid: &str, to: &str) {
    request
        .post(&format!("/api/workers/{pid}/status"))
        .json(&json!({ "to": to }))
        .await
        .assert_status_ok();
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn a_subscriber_is_told_with_a_signed_reference_and_the_log_holds_no_body() {
    unsafe { std::env::set_var("WPM_WEBHOOK_SECRET_TEST", "whsec-test-1") };
    crate::requests::request_open(|request, ctx| async move {
        let stub: Arc<Stub> = Arc::default();
        *stub.answer.lock().unwrap() = 200;
        let url = serve(stub.clone()).await;

        // Faults are refused with the reason.
        let sub = |extra: Value| {
            let mut b = json!({ "name": "Payroll", "url": url, "kinds": ["worker_terminated"], "secret_ref": "WPM_WEBHOOK_SECRET_TEST" });
            b.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
            b
        };
        for bad in [
            json!({ "url": "http://hooks.example/x" }),
            json!({ "url": "https://169.254.169.254/latest" }),
            json!({ "url": "https://hooks.example/x?token=abc" }),
            json!({ "secret_ref": "DATABASE_URL" }),
            json!({ "kinds": ["updated"] }),
            json!({ "kinds": [] }),
        ] {
            assert_eq!(request.post("/api/webhook-subscriptions").json(&sub(bad.clone())).await.status_code(), 422, "{bad}");
        }
        let made: Value = request.post("/api/webhook-subscriptions").json(&sub(json!({}))).await.json();
        let pid = made["pid"].as_str().unwrap().to_string();
        let off: Value = request.post("/api/webhook-subscriptions").json(&sub(json!({ "name": "Off", "active": false, "kinds": ["worker_activated", "worker_terminated"] }))).await.json();
        assert!(off["pid"].is_string());

        // The secret is never returned; whether it is set is.
        let listed: Value = request.get("/api/webhook-subscriptions").await.json();
        let text = listed.to_string();
        assert!(!text.contains("whsec-test-1"), "{text}");
        assert_eq!(listed["subscriptions"][1]["name"], "Payroll");
        assert_eq!(listed["subscriptions"][1]["secret_set"], true);
        assert_eq!(listed["subscriptions"][1]["secret_ref"], "WPM_WEBHOOK_SECRET_TEST");

        // An activation is not what this subscription asked for; a termination is.
        let w = worker(&request, "WH-SECRET-NUMBER").await;
        status(&request, &w, "active").await;
        let queued = ctx.db.query_one_raw(sea_orm::Statement::from_string(sea_orm::DatabaseBackend::Postgres,
            "SELECT count(*) AS n FROM webhook_deliveries".to_string())).await.unwrap().unwrap();
        assert_eq!(queued.try_get::<i64>("", "n").unwrap(), 0, "nothing asked for, and the inactive one gets nothing");
        status(&request, &w, "offboarding").await;
        status(&request, &w, "terminated").await;
        let log: Value = request.get(&format!("/api/webhook-subscriptions/{pid}/deliveries")).await.json();
        assert_eq!(log.as_array().unwrap().len(), 1);
        assert_eq!(log[0]["status"], "pending");
        assert_eq!(log[0]["subject"], format!("worker:{w}"));

        // Send it: signed, with a reference and a kind only.
        let now = Utc::now();
        assert_eq!(run(&ctx, now).await.unwrap(), Summary { delivered: 1, ..Summary::default() });
        {
            let received = stub.received.lock().unwrap();
            let (headers, body) = &received[0];
            let sent: Value = serde_json::from_str(body).unwrap();
            let mut keys: Vec<&str> = sent.as_object().unwrap().keys().map(String::as_str).collect();
            keys.sort_unstable();
            assert_eq!(keys, PAYLOAD_FIELDS);
            assert_eq!(sent["kind"], "worker_terminated");
            assert_eq!(sent["subject"], format!("worker:{w}"));
            assert!(!body.contains("WH-SECRET-NUMBER") && !body.contains("Secret Name"), "no name or number leaves: {body}");
            assert_eq!(headers["x-wpm-event"], "worker_terminated");
            let signature = headers["x-wpm-signature"].to_str().unwrap();
            assert!(verify(b"whsec-test-1", signature, body.as_bytes(), now.timestamp(), 300), "{signature}");
            assert!(!verify(b"other", signature, body.as_bytes(), now.timestamp(), 300));
        }
        let after: Value = request.get(&format!("/api/webhook-subscriptions/{pid}/deliveries")).await.json();
        assert_eq!(after[0]["status"], "delivered");
        assert_eq!(after[0]["attempts"], 1);
        // The log has these columns and no body.
        let mut columns: Vec<&str> = after[0].as_object().unwrap().keys().map(String::as_str).collect();
        columns.sort_unstable();
        assert_eq!(columns, ["attempts", "created_at", "delivered_at", "event_id", "kind", "last_error", "last_status", "next_attempt_at", "occurred_at", "pid", "status", "subject", "subscription_pid", "updated_at"]);
        assert_eq!(run(&ctx, now).await.unwrap(), Summary::default(), "delivered once, not again");

        // A subscriber that is down is retried after a back-off, then succeeds.
        let w2 = worker(&request, "WH-2").await;
        status(&request, &w2, "active").await;
        status(&request, &w2, "offboarding").await;
        *stub.answer.lock().unwrap() = 503;
        status(&request, &w2, "terminated").await;
        // The database clock and this one may differ by a moment; start the clock a minute on so the delivery is due.
        let t0 = Utc::now() + Duration::seconds(60);
        assert_eq!(run(&ctx, t0).await.unwrap(), Summary { retrying: 1, ..Summary::default() });
        assert_eq!(run(&ctx, t0 + Duration::seconds(30)).await.unwrap(), Summary::default(), "inside the back-off nothing is sent");
        let row: Value = request.get(&format!("/api/webhook-subscriptions/{pid}/deliveries")).await.json();
        assert_eq!(row[0]["last_error"], "HTTP 503");
        assert_eq!(row[0]["last_status"], 503);
        *stub.answer.lock().unwrap() = 200;
        assert_eq!(run(&ctx, t0 + Duration::seconds(90)).await.unwrap(), Summary { delivered: 1, ..Summary::default() });

        // A subscriber that refuses is not asked again.
        let w3 = worker(&request, "WH-3").await;
        status(&request, &w3, "active").await;
        status(&request, &w3, "offboarding").await;
        *stub.answer.lock().unwrap() = 400;
        status(&request, &w3, "terminated").await;
        assert_eq!(run(&ctx, Utc::now() + Duration::seconds(60)).await.unwrap(), Summary { failed: 1, ..Summary::default() });
        let counts: Value = request.get("/api/webhook-subscriptions").await.json();
        let payroll = counts["subscriptions"].as_array().unwrap().iter().find(|s| s["name"] == "Payroll").unwrap();
        assert_eq!(payroll["deliveries"], json!({ "pending": 0, "delivered": 2, "failed": 1 }));

        // No audit, no subscription (WPM-D107).
        ctx.db.execute_unprepared(
            "CREATE FUNCTION refuse_audit() RETURNS trigger AS $$ BEGIN RAISE EXCEPTION 'audit refused'; END $$ LANGUAGE plpgsql;
             CREATE TRIGGER refuse_audit BEFORE INSERT ON audit_logs FOR EACH ROW EXECUTE FUNCTION refuse_audit()").await.unwrap();
        let refused = request.post("/api/webhook-subscriptions").json(&sub(json!({ "name": "Never" }))).await;
        assert!(refused.status_code().is_server_error());
        ctx.db.execute_unprepared("DROP TRIGGER refuse_audit ON audit_logs; DROP FUNCTION refuse_audit()").await.unwrap();
        let names: Value = request.get("/api/webhook-subscriptions").await.json();
        assert!(!names.to_string().contains("Never"), "nothing was kept");

        // A change that rolls back queues nothing: the delivery is written on the change's own transaction.
        let queued = ctx.db.query_one_raw(sea_orm::Statement::from_string(sea_orm::DatabaseBackend::Postgres,
            "SELECT count(*) AS n FROM webhook_deliveries".to_string())).await.unwrap().unwrap();
        let before = queued.try_get::<i64>("", "n").unwrap();
        {
            use sea_orm::TransactionTrait;
            let txn = ctx.db.begin().await.unwrap();
            workforce_planning_management::streaming::emit_on(&txn, "worker", "worker_terminated", &uuid::Uuid::new_v4().to_string(), "x", None, None).await.unwrap();
            txn.rollback().await.unwrap();
        }
        let queued = ctx.db.query_one_raw(sea_orm::Statement::from_string(sea_orm::DatabaseBackend::Postgres,
            "SELECT count(*) AS n FROM webhook_deliveries".to_string())).await.unwrap().unwrap();
        assert_eq!(queued.try_get::<i64>("", "n").unwrap(), before);

        // A subscription can be renamed and switched off, and then it is told nothing.
        let renamed = request
            .put(&format!("/api/webhook-subscriptions/{pid}"))
            .json(&sub(json!({ "name": "Payroll feed", "active": false })))
            .await;
        assert_eq!(renamed.status_code(), 200, "{}", renamed.text());
        let after_rename: Value = request.get("/api/webhook-subscriptions").await.json();
        assert!(after_rename["subscriptions"].as_array().unwrap().iter().any(|s| s["name"] == "Payroll feed" && s["active"] == false));
        assert_eq!(
            request.put("/api/webhook-subscriptions/11111111-2222-4333-8444-555555555555").json(&sub(json!({}))).await.status_code(),
            404
        );

        // Removing the subscription removes its log.
        request.delete(&format!("/api/webhook-subscriptions/{pid}")).await.assert_status_ok();
        let gone = ctx.db.query_one_raw(sea_orm::Statement::from_string(sea_orm::DatabaseBackend::Postgres,
            format!("SELECT count(*) AS n FROM webhook_deliveries WHERE subscription_pid = '{pid}'"))).await.unwrap().unwrap();
        assert_eq!(gone.try_get::<i64>("", "n").unwrap(), 0);
    })
    .await;
    unsafe { std::env::remove_var("WPM_WEBHOOK_SECRET_TEST") };
}
