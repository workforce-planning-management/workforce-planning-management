//! The erasure ledger and its replay after a restore (WPM-R89, WPM-D63): an
//! erasure is recorded, a "restored" database that has the person back is
//! scrubbed again by the replay, the replay is idempotent, an unknown pid is
//! counted rather than an error, and `since` limits what is replayed.

use sea_orm::ConnectionTrait;
use serde_json::json;
use serial_test::serial;
use workforce_planning_management_service::controllers::privacy::{ReplayReport, replay_erasures};

use super::{activate, an_org, seed_worker};

async fn scalar(ctx: &loco_rs::app::AppContext, sql: &str) -> i64 {
    ctx.db
        .query_one_raw(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            sql.to_string(),
        ))
        .await
        .unwrap()
        .and_then(|row| row.try_get::<i64>("", "n").ok())
        .unwrap()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_restore_does_not_undo_an_erasure() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "ER-1", Some(3_600_000)).await;
        activate!(&request, &worker).await;
        request
            .post(&format!("/api/workers/{worker}/time-entries"))
            .json(&json!({ "worked_on": "2026-07-20", "minutes": 480, "notes": "a private note" }))
            .await
            .assert_status_ok();
        for to in ["offboarding", "terminated"] {
            request
                .post(&format!("/api/workers/{worker}/status"))
                .json(&json!({ "to": to }))
                .await
                .assert_status_ok();
        }
        request
            .post(&format!("/api/workers/{worker}/erase"))
            .await
            .assert_status_ok();

        // The erasure is in the ledger: a pid and a time, nothing else.
        assert_eq!(
            scalar(
                &ctx,
                &format!("SELECT COUNT(*) AS n FROM erasure_ledger WHERE worker_pid = '{worker}'")
            )
            .await,
            1
        );

        // ── A "restore": put the person and their note back, as an older dump would.
        ctx.db
            .execute_unprepared(&format!(
                "UPDATE workers SET display_name = 'Restored Person', salary_minor = 3600000, \
                 salary_currency = 'GBP', deleted_at = NULL WHERE pid = '{worker}'"
            ))
            .await
            .unwrap();
        ctx.db
            .execute_unprepared(&format!(
                "UPDATE time_entries SET notes = 'restored private note' WHERE worker_pid = '{worker}'"
            ))
            .await
            .unwrap();
        assert_eq!(
            request
                .get(&format!("/api/workers/{worker}"))
                .await
                .status_code(),
            200,
            "the restore brought the person back"
        );

        // ── The replay erases them again, before traffic.
        let report = replay_erasures(&ctx.db, None).await.unwrap();
        assert_eq!(
            report,
            ReplayReport {
                considered: 1,
                reapplied: 1,
                missing: 0
            }
        );
        assert_eq!(
            request
                .get(&format!("/api/workers/{worker}"))
                .await
                .status_code(),
            404,
            "gone again"
        );
        assert_eq!(
            scalar(
                &ctx,
                &format!(
                    "SELECT COUNT(*) AS n FROM time_entries \
                     WHERE worker_pid = '{worker}' AND notes IS NOT NULL"
                )
            )
            .await,
            0,
            "the note is scrubbed again"
        );
        assert_eq!(
            scalar(
                &ctx,
                &format!(
                    "SELECT COUNT(*) AS n FROM workers WHERE pid = '{worker}' \
                     AND display_name = '[erased]' AND salary_minor IS NULL"
                )
            )
            .await,
            1
        );

        // ── Idempotent: a second replay changes nothing more.
        assert_eq!(replay_erasures(&ctx.db, None).await.unwrap().reapplied, 1);

        // ── A pid the database does not hold is counted, not an error.
        ctx.db
            .execute_unprepared(&format!(
                "INSERT INTO erasure_ledger (worker_pid) VALUES ('{}')",
                uuid::Uuid::new_v4()
            ))
            .await
            .unwrap();
        let report = replay_erasures(&ctx.db, None).await.unwrap();
        assert_eq!((report.considered, report.reapplied, report.missing), (2, 1, 1));

        // ── `since` limits it: nothing was erased tomorrow or later.
        let tomorrow = chrono::Utc::now().date_naive() + chrono::Duration::days(1);
        assert_eq!(
            replay_erasures(&ctx.db, Some(tomorrow)).await.unwrap(),
            ReplayReport::default()
        );

        // ── The replay is audited, and the audit names no one's data.
        assert!(
            scalar(
                &ctx,
                "SELECT COUNT(*) AS n FROM audit_logs WHERE action = 'erasure_replayed'"
            )
            .await
                >= 2
        );
    })
    .await;
}
