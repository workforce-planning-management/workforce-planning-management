//! Assignments (WPM-R161): a primary one from the worker row, further ones capped at 100 FTE, a change that ends one and starts
//! another and writes through, the worker-row triggers, and the subject export and erasure.

use chrono::{Duration, Utc};
use sea_orm::{ConnectionTrait, Statement};
use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

async fn scalar(ctx: &loco_rs::app::AppContext, sql: String) -> i64 {
    ctx.db
        .query_one_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            sql,
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_worker_has_a_primary_assignment_and_a_change_ends_one_and_starts_another() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let w = seed_worker!(&request, &org, format!("AS-{tag}"), None).await;
        activate!(&request, &w).await;
        let url = format!("/api/workers/{w}/assignments");

        // A new worker already has one primary assignment, from the worker row, with no category recorded.
        let first: Value = request.get(&url).await.json();
        assert_eq!(first["assignments"].as_array().map(Vec::len), Some(1), "{first}");
        let primary = first["assignments"][0].clone();
        assert_eq!(primary["is_primary"], true);
        assert_eq!(primary["job_title"], "Engineer");
        assert_eq!(primary["staff_group"], "unknown", "a category nobody recorded reads unknown");
        assert_eq!(first["fte_total"], 100);
        assert!(primary.get("recorded_by").is_none());
        assert!(primary.get("id").is_none());
        let primary_pid = primary["pid"].as_str().unwrap().to_string();

        // Over 100 is a 422 that names the total; so are bad words and a start before the first day.
        let over = request
            .post(&url)
            .json(&json!({ "job_title": "Locum", "department": "ward", "fte_percent": 50 }))
            .await;
        assert_eq!(over.status_code(), 422);
        assert!(over.text().contains("150"), "{}", over.text());
        for bad in [
            json!({ "job_title": " ", "department": "ward", "fte_percent": 10 }),
            json!({ "job_title": "Locum", "department": "ward", "fte_percent": 0 }),
            json!({ "job_title": "Locum", "department": "ward", "fte_percent": 10, "starts_on": "2000-01-01" }),
            json!({ "job_title": "Locum", "department": "ward", "fte_percent": 10, "staff_group": " " }),
        ] {
            assert_eq!(request.post(&url).json(&bad).await.status_code(), 422, "{bad}");
        }

        // A change: effective in the future, before the start, or over 100 is refused.
        let change = format!("/api/assignments/{primary_pid}/change");
        let future = (Utc::now().date_naive() + Duration::days(3)).to_string();
        assert_eq!(request.post(&change).json(&json!({ "effective_on": future, "fte_percent": 60 })).await.status_code(), 422);
        assert_eq!(request.post(&change).json(&json!({ "effective_on": "2026-01-05", "fte_percent": 60 })).await.status_code(), 422);
        assert_eq!(request.post(&change).json(&json!({ "fte_percent": 101 })).await.status_code(), 422);

        // A real change ends the old one the day before and starts a new one that keeps what was not changed.
        let done = request
            .post(&change)
            .json(&json!({ "fte_percent": 60, "job_title": "Senior Engineer", "staff_group": "clinical" }))
            .await;
        assert_eq!(done.status_code(), 200, "{}", done.text());
        let done: Value = done.json();
        let yesterday = (Utc::now().date_naive() - Duration::days(1)).to_string();
        assert_eq!(done["previous_ends_on"], yesterday);
        let after: Value = request.get(&url).await.json();
        let rows = after["assignments"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        let (new, old) = (&rows[0], &rows[1]);
        assert_eq!((new["job_title"].as_str(), new["fte_percent"].as_i64(), new["staff_group"].as_str()), (Some("Senior Engineer"), Some(60), Some("clinical")));
        assert_eq!(new["department"], "engineering", "kept");
        assert_eq!((new["is_primary"].as_bool(), new["current"].as_bool()), (Some(true), Some(true)));
        assert_eq!((old["ends_on"].as_str(), old["current"].as_bool(), old["fte_percent"].as_i64()), (Some(yesterday.as_str()), Some(false), Some(100)));
        assert_eq!(after["fte_total"], 60, "only what is in force counts");

        // It wrote through to the worker row.
        let worker: Value = request.get(&format!("/api/workers/{w}")).await.json();
        assert_eq!((worker["job_title"].as_str(), worker["fte_percent"].as_i64()), (Some("Senior Engineer"), Some(60)), "{worker}");

        // Now there is room for 40, and not for 41.
        let add = |fte: i64| {
            let request = &request;
            let url = url.clone();
            async move { request.post(&url).json(&json!({ "job_title": "Bank shifts", "department": "ward", "fte_percent": fte })).await }
        };
        assert_eq!(add(41).await.status_code(), 422);
        let extra = add(40).await;
        assert_eq!(extra.status_code(), 200, "{}", extra.text());
        let total: Value = request.get(&url).await.json();
        assert_eq!(total["fte_total"], 100);
        let opens = scalar(&ctx, format!("SELECT count(*) AS n FROM assignments WHERE worker_pid = '{w}' AND is_primary AND ends_on IS NULL")).await;
        assert_eq!(opens, 1, "exactly one open primary");

        // The worker-row trigger: an older writer correcting the department corrects the open primary in place.
        ctx.db.execute_unprepared(&format!("UPDATE workers SET department = 'cardiology' WHERE pid = '{w}'")).await.unwrap();
        let dept = ctx
            .db
            .query_one_raw(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!("SELECT department AS d FROM assignments WHERE worker_pid = '{w}' AND is_primary AND ends_on IS NULL"),
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get::<String>("", "d")
            .unwrap();
        assert_eq!(dept, "cardiology");
        assert_eq!(scalar(&ctx, format!("SELECT count(*) AS n FROM assignments WHERE worker_pid = '{w}'")).await, 3, "no history is made by a correction");

        // A stranger worker pid is a 404; the audit trail names the change.
        assert_eq!(request.get(&format!("/api/workers/{}/assignments", uuid::Uuid::new_v4())).await.status_code(), 404);
        assert!(scalar(&ctx, format!("SELECT count(*) AS n FROM audit_logs WHERE action = 'assignment_changed' AND entity_pid = '{}'", done["pid"].as_str().unwrap())).await >= 1);

        // The subject export lists the assignments; erasure removes them.
        let export: Value = request.get(&format!("/api/workers/{w}/subject-access")).await.json();
        assert_eq!(export["assignments"].as_array().map(Vec::len), Some(3), "{export}");
        for to in ["offboarding", "terminated"] {
            request.post(&format!("/api/workers/{w}/status")).json(&json!({ "to": to })).await.assert_status_ok();
        }
        request.post(&format!("/api/workers/{w}/erase")).await.assert_status_ok();
        assert_eq!(scalar(&ctx, format!("SELECT count(*) AS n FROM assignments WHERE worker_pid = '{w}'")).await, 0);
    })
    .await;
}
