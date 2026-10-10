//! The audit trail is append-only and hash-chained (WPM-R115, WPM-D72): the database refuses to
//! change or remove an entry, and a break in the chain is found.

use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_audit_trail_cannot_be_changed_and_a_tamper_is_found() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        // Some real activity, so there are entries written by the application.
        for n in 0..3 {
            let worker = seed_worker!(&request, &org, format!("AC-{n}"), Some(1_000_000)).await;
            activate!(&request, &worker).await;
        }
        let report: Value = request.get("/api/audits/verify").await.json();
        assert_eq!(report["ok"], true, "{report}");
        let entries = report["entries"].as_i64().unwrap();
        assert!(entries >= 6, "entries: {entries}");
        let head = report["head_hash"].as_str().unwrap().to_string();
        assert_eq!(head.len(), 64);

        // The sequence is gapless and each entry names the one before it.
        let rows = ctx
            .db
            .query_all_raw(sea_orm::Statement::from_string(
                sea_orm::DbBackend::Postgres,
                "SELECT chain_seq, prev_hash, entry_hash FROM audit_logs ORDER BY chain_seq",
            ))
            .await
            .unwrap();
        let mut previous = "0".repeat(64);
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(
                row.try_get::<i64>("", "chain_seq").unwrap(),
                i64::try_from(i + 1).unwrap()
            );
            assert_eq!(row.try_get::<String>("", "prev_hash").unwrap(), previous);
            previous = row.try_get::<String>("", "entry_hash").unwrap();
        }
        assert_eq!(previous, head);

        // The database refuses to change or remove an entry, even for a connection that could.
        let update = ctx
            .db
            .execute_unprepared(
                "UPDATE audit_logs SET action = 'nothing happened' WHERE chain_seq = 1",
            )
            .await;
        assert!(update.is_err(), "UPDATE must be refused");
        assert!(update.unwrap_err().to_string().contains("append-only"));
        let delete = ctx
            .db
            .execute_unprepared("DELETE FROM audit_logs WHERE chain_seq = 2")
            .await;
        assert!(delete.is_err(), "DELETE must be refused");

        // A person with the rights to switch the protection off still leaves a trace.
        // 1. Rewrite an entry.
        ctx.db
            .execute_unprepared("ALTER TABLE audit_logs DISABLE TRIGGER audit_logs_immutable")
            .await
            .unwrap();
        ctx.db
            .execute_unprepared(
                "UPDATE audit_logs SET action = 'nothing happened' WHERE chain_seq = 3",
            )
            .await
            .unwrap();
        let tampered: Value = request.get("/api/audits/verify").await.json();
        assert_eq!(tampered["ok"], false);
        assert_eq!(
            tampered["first_break"], 3,
            "the rewritten entry is the first break"
        );
        ctx.db
            .execute_unprepared("ALTER TABLE audit_logs ENABLE TRIGGER audit_logs_immutable")
            .await
            .unwrap();
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn removing_an_entry_from_the_middle_is_found_and_the_tail_is_not() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        for n in 0..3 {
            let worker = seed_worker!(&request, &org, format!("AD-{n}"), None).await;
            activate!(&request, &worker).await;
        }
        let before: Value = request.get("/api/audits/verify").await.json();
        assert_eq!(before["ok"], true);
        let entries = before["entries"].as_i64().unwrap();
        let middle = entries / 2;
        ctx.db
            .execute_unprepared("ALTER TABLE audit_logs DISABLE TRIGGER audit_logs_immutable")
            .await
            .unwrap();
        ctx.db
            .execute_unprepared(&format!(
                "DELETE FROM audit_logs WHERE chain_seq = {middle}"
            ))
            .await
            .unwrap();
        let removed: Value = request.get("/api/audits/verify").await.json();
        assert_eq!(removed["ok"], false);
        assert_eq!(
            removed["first_break"],
            middle + 1,
            "the entry after the gap names a predecessor that is gone"
        );
        // Cutting entries off the end leaves a chain that still holds; only the head hash recorded
        // elsewhere shows it, which is why the report carries one.
        ctx.db
            .execute_unprepared(&format!(
                "DELETE FROM audit_logs WHERE chain_seq >= {middle}"
            ))
            .await
            .unwrap();
        let cut: Value = request.get("/api/audits/verify").await.json();
        assert_eq!(
            cut["ok"], true,
            "a truncated tail is not visible from inside"
        );
        assert_ne!(
            cut["head_hash"], before["head_hash"],
            "but the head hash differs from the recorded one"
        );
        assert!(cut["entries"].as_i64().unwrap() < entries);
        ctx.db
            .execute_unprepared("ALTER TABLE audit_logs ENABLE TRIGGER audit_logs_immutable")
            .await
            .unwrap();
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Entries written at the same moment by different transactions still form one gapless chain.
async fn concurrent_writers_make_one_chain() {
    use sea_orm::TransactionTrait;
    use workforce_planning_management_service::models::audit_logs::Model as Audit;
    crate::requests::request_open(|request, ctx| async move {
        // The application's test pool holds one connection, so the writers would queue; open a
        // pool of their own so they genuinely overlap.
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let mut options = sea_orm::ConnectOptions::new(url);
        options.max_connections(16);
        let pool = sea_orm::Database::connect(options)
            .await
            .expect("a second pool");
        let mut handles = Vec::new();
        for n in 0..16 {
            let db = pool.clone();
            handles.push(tokio::spawn(async move {
                let txn = db.begin().await.unwrap();
                Audit::record(
                    &txn,
                    "probe",
                    uuid::Uuid::new_v4(),
                    "created",
                    Some("tester"),
                    Some(json!({ "n": n })),
                )
                .await
                .unwrap();
                // Hold the transaction open so the writers genuinely overlap.
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                Audit::record(
                    &txn,
                    "probe",
                    uuid::Uuid::new_v4(),
                    "second",
                    Some("tester"),
                    None,
                )
                .await
                .unwrap();
                txn.commit().await.unwrap();
            }));
        }
        for handle in handles {
            handle.await.expect("writer finished");
        }
        let report: Value = request.get("/api/audits/verify").await.json();
        assert_eq!(report["ok"], true, "{report}");
        let distinct: i64 = ctx
            .db
            .query_one_raw(sea_orm::Statement::from_string(
                sea_orm::DbBackend::Postgres,
                "SELECT count(DISTINCT chain_seq)::bigint AS n FROM audit_logs",
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get("", "n")
            .unwrap();
        assert_eq!(distinct, report["entries"].as_i64().unwrap());
        assert!(distinct >= 32);
    })
    .await;
}
