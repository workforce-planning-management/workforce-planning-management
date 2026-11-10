//! `cargo run -- task snapshot_headcount [as_of:YYYY-MM-DD]` — record one
//! aggregate headcount snapshot per organization × department.
//!
//! Run it on a schedule (daily or weekly): the history it accrues is what
//! workforce forecasting projects from, and it cannot be backfilled.
//! Idempotent — a (organization, department, date) already recorded is
//! left alone, so a re-run changes nothing.

use axa::prelude::*;
use axa::task::{TaskInfo, Vars};
use uuid::Uuid;

use crate::rules::metrics::{self, WorkerFacts};

/// The headcount-snapshot task.
pub struct SnapshotHeadcount;

#[async_trait]
impl Task for SnapshotHeadcount {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "snapshot_headcount".to_string(),
            detail:
                "Record an aggregate headcount snapshot (optional as_of:YYYY-MM-DD; idempotent)"
                    .to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, vars: &Vars) -> Result<()> {
        let as_of = match vars.cli_arg("as_of") {
            Ok(text) => text
                .parse::<chrono::NaiveDate>()
                .map_err(|e| Error::string(&format!("as_of must be YYYY-MM-DD: {e}")))?,
            Err(_) => chrono::Utc::now().date_naive(),
        };
        let written = run_snapshot(crate::db::pool(ctx), as_of).await?;
        tracing::info!(%as_of, written, "headcount snapshot recorded");
        Ok(())
    }
}

/// Record the snapshot for `as_of`; returns the number of rows newly
/// written (rows already present are skipped).
///
/// # Errors
/// A database error.
pub async fn run_snapshot(pool: &sqlx::PgPool, as_of: chrono::NaiveDate) -> Result<usize> {
    let facts: Vec<WorkerFacts> = sqlx::query_as!(
        WorkerFacts,
        "SELECT organization_ref, department, fte_percent, employment_type, hired_on, terminated_on
           FROM workers WHERE deleted_at IS NULL"
    )
    .fetch_all(pool)
    .await?;
    // The window starts after the most recent earlier snapshot.
    let since = sqlx::query_scalar!(
        "SELECT max(as_of) FROM headcount_snapshots WHERE as_of < $1",
        as_of
    )
    .fetch_one(pool)
    .await?;

    let mut tx = pool.begin().await?;
    let mut written = 0;
    for row in metrics::snapshot_rows(as_of, since, &facts) {
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM headcount_snapshots
                               WHERE organization_ref = $1 AND department = $2 AND as_of = $3) AS "e!""#,
            row.organization_ref,
            row.department,
            as_of,
        )
        .fetch_one(&mut *tx)
        .await?;
        if exists {
            continue;
        }
        sqlx::query!(
            "INSERT INTO headcount_snapshots
                 (pid, organization_ref, department, as_of, headcount, fte_percent_total, contractor_headcount,
                  contractor_fte_percent_total, fixed_term_headcount, fixed_term_fte_percent_total, starters, leavers)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
            Uuid::new_v4(),
            row.organization_ref,
            row.department,
            as_of,
            i32::try_from(row.headcount).unwrap_or(i32::MAX),
            row.fte_percent_total,
            i32::try_from(row.contractor_headcount).ok(),
            Some(row.contractor_fte_percent_total),
            i32::try_from(row.fixed_term_headcount).ok(),
            Some(row.fixed_term_fte_percent_total),
            row.starters.and_then(|n| i32::try_from(n).ok()),
            row.leavers.and_then(|n| i32::try_from(n).ok()),
        )
        .execute(&mut *tx)
        .await?;
        written += 1;
    }
    tx.commit().await?;
    Ok(written)
}
