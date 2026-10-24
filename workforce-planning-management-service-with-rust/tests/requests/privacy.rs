//! Subject rights & retention (WPM-R30 / WPM-D22): the subject-access
//! export gathers and names its exclusions, erasure anonymises without
//! touching payroll rows and is refused on open employment, and the
//! retention report/sweep honour the floored horizon.

use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one seeded life, the whole rights surface
async fn subject_rights_round_trip() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "SR-1", Some(3_600_000)).await;
        activate!(&request, &worker).await;

        // Give the worker a footprint: time (with a note), leave,
        // and a wellbeing acknowledgement.
        request
            .post(&format!("/api/workers/{worker}/time-entries"))
            .json(&json!({ "worked_on": "2026-07-20", "minutes": 480,
                           "notes": "client visit in Leeds" }))
            .await
            .assert_status_ok();
        request
            .post(&format!("/api/workers/{worker}/leave-entitlements"))
            .json(&json!({ "kind": "annual", "year": 2026, "entitled_days": 25 }))
            .await
            .assert_status_ok();
        let rule: Value = request
            .post("/api/wellbeing-entitlements")
            .json(&json!({ "name": "SR flu", "description": "x" }))
            .await
            .json();
        request
            .post(&format!("/api/workers/{worker}/wellbeing-acknowledgements"))
            .json(&json!({ "entitlement_pid": rule["pid"], "response": "done" }))
            .await
            .assert_status_ok();

        // ── Subject access: the footprint is present, exclusions named,
        // and the export is audited.
        let export: Value = request
            .get(&format!("/api/workers/{worker}/subject-access"))
            .await
            .json();
        assert_eq!(export["worker"]["worker_number"], "SR-1");
        assert_eq!(export["time_entries"][0]["notes"], "client visit in Leeds");
        assert_eq!(export["leave_entitlements"][0]["entitled_days"], 25);
        assert_eq!(export["wellbeing_acknowledgements"][0]["response"], "done");
        let exclusions = serde_json::to_string(&export["exclusions"]).unwrap();
        assert!(exclusions.contains("pulse"), "structural exclusion named");
        assert!(
            exclusions.contains("identity services"),
            "upstream duty named"
        );
        let audits: Value = request.get("/api/audits/recent").await.json();
        assert!(
            serde_json::to_string(&audits)
                .unwrap()
                .contains("subject_access_exported")
        );

        // The identity provider's link for the worker is held, and erasure removes it.
        {
            use sea_orm::ConnectionTrait;
            ctx.db
                .execute_unprepared(&format!(
                    "INSERT INTO scim_pushes (pid, worker_pid, scim_id) VALUES (gen_random_uuid(), '{worker}', 'idp-1')"
                ))
                .await
                .unwrap();
            ctx.db
                .execute_unprepared(&format!(
                    "WITH s AS (INSERT INTO webhook_subscriptions (pid, name, url, kinds, secret_ref) VALUES (gen_random_uuid(), 'x', 'https://hooks.example/x', ARRAY['worker_terminated'], 'WPM_WEBHOOK_SECRET_X') RETURNING pid)
                     INSERT INTO webhook_deliveries (pid, subscription_pid, event_id, kind, subject, occurred_at) SELECT gen_random_uuid(), pid, gen_random_uuid(), 'worker_terminated', 'worker:{worker}', now() FROM s"
                ))
                .await
                .unwrap();
        }

        {
            use sea_orm::ConnectionTrait;
            ctx.db
                .execute_unprepared(&format!(
                    "INSERT INTO positions (pid, organization_ref, cost_centre, title, budget_fte_percent, funding_kind, state, holder_worker_pid) \
                     SELECT gen_random_uuid(), organization_ref, 'CC-ERASE', 'Seat', 100, 'core', 'filled', pid FROM workers WHERE pid = '{worker}'"
                ))
                .await
                .unwrap();
        }

        // ── Erasure is refused while employment is open.
        assert_eq!(
            request
                .post(&format!("/api/workers/{worker}/erase"))
                .await
                .status_code(),
            422,
            "an active employment is the lawful basis"
        );
        // Terminate (via offboarding — the lifecycle's path), then erase.
        for to in ["offboarding", "terminated"] {
            request
                .post(&format!("/api/workers/{worker}/status"))
                .json(&json!({ "to": to }))
                .await
                .assert_status_ok();
        }
        let erased: Value = request
            .post(&format!("/api/workers/{worker}/erase"))
            .await
            .json();
        assert_eq!(erased["erased"], worker.as_str());
        {
            use sea_orm::ConnectionTrait;
            let left = ctx
                .db
                .query_one_raw(sea_orm::Statement::from_string(
                    sea_orm::DatabaseBackend::Postgres,
                    format!("SELECT count(*) AS n FROM scim_pushes WHERE worker_pid = '{worker}'"),
                ))
                .await
                .unwrap()
                .unwrap();
            let n: i64 = left.try_get("", "n").unwrap();
            assert_eq!(n, 0, "erasure removes the identity provider link");
            let queued = ctx
                .db
                .query_one_raw(sea_orm::Statement::from_string(
                    sea_orm::DatabaseBackend::Postgres,
                    format!("SELECT count(*) AS n FROM webhook_deliveries WHERE subject = 'worker:{worker}'"),
                ))
                .await
                .unwrap()
                .unwrap();
            let n: i64 = queued.try_get("", "n").unwrap();
            assert_eq!(n, 0, "erasure removes the webhook deliveries that name the worker");
            let seat = ctx
                .db
                .query_one_raw(sea_orm::Statement::from_string(
                    sea_orm::DatabaseBackend::Postgres,
                    "SELECT count(*) AS seats, count(holder_worker_pid) AS held FROM positions WHERE cost_centre = 'CC-ERASE'".to_string(),
                ))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(seat.try_get::<i64>("", "seats").unwrap(), 1, "the seat outlives the person (WPM-D104)");
            assert_eq!(seat.try_get::<i64>("", "held").unwrap(), 0, "only the link is cut");
        }
        // The worker is gone from reads (soft-deleted) …
        assert_eq!(
            request
                .get(&format!("/api/workers/{worker}"))
                .await
                .status_code(),
            404
        );
        // … their acknowledgements are deleted, and the audit snapshot
        // carries the counts.
        let audits: Value = request.get("/api/audits/recent").await.json();
        let erase_row = audits
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["action"] == "erased")
            .expect("erasure audited")
            .clone();
        assert_eq!(erase_row["snapshot"]["acknowledgements_deleted"], 1);
        assert_eq!(erase_row["snapshot"]["notes_scrubbed"], 1);

        // ── Retention: the report is readable; a fresh soft-delete is
        // inside the horizon, so nothing is listed or swept.
        let report: Value = request.get("/api/retention").await.json();
        for (kind, days) in report["horizons"].as_object().unwrap() {
            assert!(days.as_i64().unwrap() >= 30, "{kind}: floored horizon");
        }
        assert_eq!(report["horizons"].as_object().unwrap().len(), 8);
        assert!(
            report["soft_deleted_past_horizon"]
                .as_object()
                .unwrap()
                .is_empty(),
            "fresh soft-deletes are inside the horizon"
        );
        let sweep: Value = request.post("/api/retention/sweep").await.json();
        assert_eq!(sweep["rows_deleted"], 0, "nothing past the horizon yet");
        assert_eq!(sweep["candidates_scrubbed"], 0);
        let audits: Value = request.get("/api/audits/recent").await.json();
        assert!(
            serde_json::to_string(&audits)
                .unwrap()
                .contains("retention_swept")
        );
    })
    .await;
}

/// The retention report counts, and the sweep deletes, rows past a horizon in every soft-deleting table
/// (WPM-R142): each has an index on `deleted_at` that holds only the deleted rows, so neither scans the table.
#[tokio::test]
#[serial_test::serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn every_soft_deleting_table_has_a_deleted_at_index() {
    use sea_orm::ConnectionTrait;
    crate::requests::request_open(|_request, ctx| async move {
        let rows = ctx
            .db
            .query_all_raw(sea_orm::Statement::from_string(
                sea_orm::DbBackend::Postgres,
                "SELECT tablename FROM pg_indexes WHERE schemaname = current_schema()
                   AND indexdef LIKE '%(deleted_at) WHERE (deleted_at IS NOT NULL)%'",
            ))
            .await
            .unwrap();
        let indexed: std::collections::BTreeSet<String> = rows
            .iter()
            .map(|r| r.try_get::<String>("", "tablename").unwrap())
            .collect();
        for table in workforce_planning_management::rules::privacy::SOFT_DELETED_TABLES {
            assert!(
                indexed.contains(*table),
                "{table} has no partial index on deleted_at; add it to a migration"
            );
        }
    })
    .await;
}
