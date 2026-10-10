//! `cargo loco task snapshot_headcount [as_of:YYYY-MM-DD]` — record one
//! aggregate headcount snapshot per organization × department.
//!
//! Run it on a schedule (daily or weekly): the history it accrues is what
//! workforce forecasting projects from, and it cannot be backfilled.
//! Idempotent — a (organization, department, date) already recorded is
//! left alone, so a re-run changes nothing.

use loco_rs::prelude::*;
use loco_rs::task::{TaskInfo, Vars};
use sea_orm::{ActiveValue, DatabaseConnection, QueryOrder, QuerySelect, TransactionTrait};
use uuid::Uuid;

use crate::models::_entities::{headcount_snapshots, workers};
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
        let written = run_snapshot(&ctx.db, as_of).await?;
        tracing::info!(%as_of, written, "headcount snapshot recorded");
        Ok(())
    }
}

/// Record the snapshot for `as_of`; returns the number of rows newly
/// written (rows already present are skipped).
///
/// # Errors
/// A database error.
pub async fn run_snapshot(db: &DatabaseConnection, as_of: chrono::NaiveDate) -> Result<usize> {
    let facts: Vec<WorkerFacts> = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .all(db)
        .await?
        .into_iter()
        .map(|w| WorkerFacts {
            organization_ref: w.organization_ref,
            department: w.department,
            fte_percent: w.fte_percent,
            hired_on: w.hired_on,
            terminated_on: w.terminated_on,
        })
        .collect();
    // The window starts after the most recent earlier snapshot.
    let since = headcount_snapshots::Entity::find()
        .filter(headcount_snapshots::Column::AsOf.lt(as_of))
        .order_by_desc(headcount_snapshots::Column::AsOf)
        .limit(1)
        .one(db)
        .await?
        .map(|row| row.as_of);

    let txn = db.begin().await?;
    let mut written = 0;
    for row in metrics::snapshot_rows(as_of, since, &facts) {
        let exists = headcount_snapshots::Entity::find()
            .filter(headcount_snapshots::Column::OrganizationRef.eq(&row.organization_ref))
            .filter(headcount_snapshots::Column::Department.eq(&row.department))
            .filter(headcount_snapshots::Column::AsOf.eq(as_of))
            .one(&txn)
            .await?
            .is_some();
        if exists {
            continue;
        }
        headcount_snapshots::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            organization_ref: ActiveValue::set(row.organization_ref),
            department: ActiveValue::set(row.department),
            as_of: ActiveValue::set(as_of),
            headcount: ActiveValue::set(i32::try_from(row.headcount).unwrap_or(i32::MAX)),
            fte_percent_total: ActiveValue::set(row.fte_percent_total),
            starters: ActiveValue::set(row.starters.and_then(|n| i32::try_from(n).ok())),
            leavers: ActiveValue::set(row.leavers.and_then(|n| i32::try_from(n).ok())),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
        written += 1;
    }
    txn.commit().await?;
    Ok(written)
}
