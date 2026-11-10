//! A database a `SeaORM` release migrated is adopted, not migrated again (WPM-D193): its `seaql_migrations` history is recorded
//! as applied, and only the newer migrations run.
//!
//! `#[ignore]`d: it needs PostgreSQL (`DATABASE_URL`, a role that may create a database). Run with
//! `cargo test --test migration_adoption -- --ignored --test-threads=1`. It makes and drops its own database.

use std::borrow::Cow;

use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, Executor, PgConnection};
use workforce_planning_management::migrations::{MIGRATOR, up};

/// Where a SeaORM release stopped: the first this many migrations.
const STOPPED_AT: usize = 100;

fn admin_url() -> String {
    std::env::var("DATABASE_URL").expect("DATABASE_URL")
}

/// `admin_url` with its database name replaced.
fn url_for(name: &str) -> String {
    let admin = admin_url();
    let (base, _) = admin
        .rsplit_once('/')
        .expect("a database name in DATABASE_URL");
    format!("{base}/{name}")
}

#[tokio::test]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test migration_adoption -- --ignored`"]
async fn a_seaorm_history_is_adopted_and_only_the_newer_migrations_run() {
    let name = format!(
        "wpm_adoption_{}",
        &uuid::Uuid::new_v4().simple().to_string()[..12]
    );
    let mut admin = PgConnection::connect(&admin_url()).await.expect("connect");
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!("CREATE DATABASE {name}")))
        .execute(&mut admin)
        .await
        .expect("create");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url_for(&name))
        .await
        .expect("pool");

    // The schema a SeaORM release left at migration 100, with SeaORM's history and none of sqlx's.
    let first: Vec<_> = MIGRATOR.iter().take(STOPPED_AT).cloned().collect();
    let last_adopted = first.last().expect("migrations").version;
    let partial = Migrator {
        migrations: Cow::Owned(first.clone()),
        ..Migrator::DEFAULT
    };
    partial.run(&pool).await.expect("the first migrations");
    pool.execute("DROP TABLE _sqlx_migrations")
        .await
        .expect("drop sqlx's history");
    pool.execute(
        "CREATE TABLE seaql_migrations (version VARCHAR PRIMARY KEY, applied_at BIGINT NOT NULL)",
    )
    .await
    .expect("seaql_migrations");
    for m in &first {
        let v = m.version.to_string();
        sqlx::query("INSERT INTO seaql_migrations (version, applied_at) VALUES ($1, 0)")
            .bind(format!(
                "m{}_{}_{}",
                &v[..8],
                &v[8..],
                m.description.replace(' ', "_")
            ))
            .execute(&pool)
            .await
            .expect("history");
    }

    up(&pool).await.expect("adopt and migrate");

    let rows: Vec<(i64, i64, bool)> = sqlx::query_as(
        "SELECT version, execution_time, success FROM _sqlx_migrations ORDER BY version",
    )
    .fetch_all(&pool)
    .await
    .expect("history");
    assert_eq!(
        rows.len(),
        MIGRATOR.iter().count(),
        "every migration is recorded"
    );
    for (version, execution_time, success) in &rows {
        assert!(success, "{version}");
        // `sqlx` records a skipped migration with no execution time and one it ran with how long it took.
        if *version <= last_adopted {
            assert_eq!(*execution_time, -1, "{version} was adopted, not run again");
        } else {
            assert!(*execution_time >= 0, "{version} ran");
        }
    }
    // A second boot changes nothing.
    up(&pool).await.expect("again");
    let again: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(again, i64::try_from(rows.len()).unwrap());

    pool.close().await;
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "DROP DATABASE {name} WITH (FORCE)"
    )))
    .execute(&mut admin)
    .await
    .expect("drop");
}

#[tokio::test]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test migration_adoption -- --ignored`"]
async fn a_seaorm_history_with_a_gap_stops_the_boot() {
    let name = format!(
        "wpm_adoption_{}",
        &uuid::Uuid::new_v4().simple().to_string()[..12]
    );
    let mut admin = PgConnection::connect(&admin_url()).await.expect("connect");
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!("CREATE DATABASE {name}")))
        .execute(&mut admin)
        .await
        .expect("create");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url_for(&name))
        .await
        .expect("pool");
    pool.execute(
        "CREATE TABLE seaql_migrations (version VARCHAR PRIMARY KEY, applied_at BIGINT NOT NULL)",
    )
    .await
    .expect("seaql_migrations");
    pool.execute("INSERT INTO seaql_migrations VALUES ('m20220101_000001_workers', 0), ('m20220101_000003_workforce', 0)")
        .await
        .expect("history");

    let error = up(&pool).await.expect_err("a gap is refused");
    assert!(
        error.to_string().contains("m20220101_000003_workforce"),
        "{error}"
    );
    let ran: bool = sqlx::query_scalar("SELECT to_regclass('workers') IS NOT NULL")
        .fetch_one(&pool)
        .await
        .expect("check");
    assert!(!ran, "nothing was migrated");

    pool.close().await;
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "DROP DATABASE {name} WITH (FORCE)"
    )))
    .execute(&mut admin)
    .await
    .expect("drop");
}
