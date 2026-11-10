//! The audit trail is append-only and hash-chained (WPM-R115, WPM-D72): the database refuses to
//! change or remove an entry, and a break in the chain is found.

use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};
use sqlx::Row;

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
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
        let rows = sqlx::query(
            "SELECT chain_seq, prev_hash, entry_hash FROM audit_logs ORDER BY chain_seq",
        )
        .fetch_all(workforce_planning_management::db::pool(&ctx))
        .await
        .unwrap();
        let mut previous = "0".repeat(64);
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(
                row.try_get::<i64, _>("chain_seq").unwrap(),
                i64::try_from(i + 1).unwrap()
            );
            assert_eq!(row.try_get::<String, _>("prev_hash").unwrap(), previous);
            previous = row.try_get::<String, _>("entry_hash").unwrap();
        }
        assert_eq!(previous, head);

        // The database refuses to change or remove an entry, even for a connection that could.
        let update =
            sqlx::raw_sql("UPDATE audit_logs SET action = 'nothing happened' WHERE chain_seq = 1")
                .execute(workforce_planning_management::db::pool(&ctx))
                .await;
        assert!(update.is_err(), "UPDATE must be refused");
        assert!(update.unwrap_err().to_string().contains("append-only"));
        let delete = sqlx::raw_sql("DELETE FROM audit_logs WHERE chain_seq = 2")
            .execute(workforce_planning_management::db::pool(&ctx))
            .await;
        assert!(delete.is_err(), "DELETE must be refused");

        // A person with the rights to switch the protection off still leaves a trace.
        // 1. Rewrite an entry.
        sqlx::raw_sql("ALTER TABLE audit_logs DISABLE TRIGGER audit_logs_immutable")
            .execute(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
        sqlx::raw_sql("UPDATE audit_logs SET action = 'nothing happened' WHERE chain_seq = 3")
            .execute(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
        let tampered: Value = request.get("/api/audits/verify").await.json();
        assert_eq!(tampered["ok"], false);
        assert_eq!(
            tampered["first_break"], 3,
            "the rewritten entry is the first break"
        );
        sqlx::raw_sql("ALTER TABLE audit_logs ENABLE TRIGGER audit_logs_immutable")
            .execute(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
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
        sqlx::raw_sql("ALTER TABLE audit_logs DISABLE TRIGGER audit_logs_immutable")
            .execute(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "DELETE FROM audit_logs WHERE chain_seq = {middle}"
        )))
        .execute(workforce_planning_management::db::pool(&ctx))
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
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "DELETE FROM audit_logs WHERE chain_seq >= {middle}"
        )))
        .execute(workforce_planning_management::db::pool(&ctx))
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
        sqlx::raw_sql("ALTER TABLE audit_logs ENABLE TRIGGER audit_logs_immutable")
            .execute(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
// Entries written at the same moment by different transactions still form one gapless chain.
async fn concurrent_writers_make_one_chain() {
    use workforce_planning_management::db;
    crate::requests::request_open(|request, ctx| async move {
        // The application's test pool holds one connection, so the writers would queue; open a
        // pool of their own so they genuinely overlap.
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(16)
            .connect(&url)
            .await
            .expect("a second pool");
        let mut handles = Vec::new();
        for n in 0..16 {
            let pool = pool.clone();
            handles.push(tokio::spawn(async move {
                let mut tx = pool.begin().await.unwrap();
                db::audit(
                    &mut tx,
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
                db::audit(
                    &mut tx,
                    "probe",
                    uuid::Uuid::new_v4(),
                    "second",
                    Some("tester"),
                    None,
                )
                .await
                .unwrap();
                tx.commit().await.unwrap();
            }));
        }
        for handle in handles {
            handle.await.expect("writer finished");
        }
        let report: Value = request.get("/api/audits/verify").await.json();
        assert_eq!(report["ok"], true, "{report}");
        let distinct: i64 =
            sqlx::query("SELECT count(DISTINCT chain_seq)::bigint AS n FROM audit_logs")
                .fetch_optional(workforce_planning_management::db::pool(&ctx))
                .await
                .unwrap()
                .unwrap()
                .try_get("n")
                .unwrap();
        assert_eq!(distinct, report["entries"].as_i64().unwrap());
        assert!(distinct >= 32);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_change_whose_audit_row_fails_is_not_kept() {
    crate::requests::request_open(|request, ctx| async move {
        let tag = uuid::Uuid::new_v4().simple().to_string();
        // The audit insert fails for anything named with this test's tag, and only that (WPM-T250).
        crate::requests::ddl(&ctx, &format!(
                "CREATE FUNCTION refuse_atom_{tag}() RETURNS trigger LANGUAGE plpgsql AS $$
                 BEGIN
                   IF NEW.entity = 'skill' AND EXISTS (SELECT 1 FROM skills WHERE pid = NEW.entity_pid AND name LIKE 'ATOM{tag}%')
                      OR NEW.entity = 'skill_course' AND EXISTS (SELECT 1 FROM skill_courses WHERE pid = NEW.entity_pid AND title LIKE 'ATOM{tag}%')
                   THEN RAISE EXCEPTION 'audit refused for the test';
                   END IF;
                   RETURN NEW;
                 END $$;
                 CREATE TRIGGER refuse_atom_{tag} BEFORE INSERT ON audit_logs
                   FOR EACH ROW EXECUTE FUNCTION refuse_atom_{tag}();"
            )).await;

        // A new skill whose audit row fails is not created.
        let refused = request
            .post("/api/skills")
            .json(&json!({ "name": format!("ATOM{tag}-skill"), "category": "technical" }))
            .await;
        assert!(refused.status_code().is_server_error(), "{}", refused.status_code());
        let skills: Value = request.get("/api/skills").await.json();
        assert!(
            !skills.to_string().contains(&format!("ATOM{tag}-skill")),
            "the skill was kept without its audit row"
        );

        // A catalogue course whose audit row fails is not added.
        let skill: Value = request
            .post("/api/skills")
            .json(&json!({ "name": format!("Plain{tag}"), "category": "technical" }))
            .await
            .json();
        let courses = format!("/api/skills/{}/courses", skill["pid"].as_str().unwrap());
        let refused = request
            .post(&courses)
            .json(&json!({
                "course_ref": "course:11111111-1111-4111-8111-1111111111c1",
                "title": format!("ATOM{tag}-course"),
                "hours": 5,
            }))
            .await;
        assert!(refused.status_code().is_server_error(), "{}", refused.status_code());
        let listed: Value = request.get(&courses).await.json();
        assert_eq!(listed["courses"].as_array().unwrap().len(), 0, "the course was kept without its audit row");

        crate::requests::ddl(&ctx, &format!(
                "DROP TRIGGER refuse_atom_{tag} ON audit_logs; DROP FUNCTION refuse_atom_{tag}();"
            )).await;
    })
    .await;
}
