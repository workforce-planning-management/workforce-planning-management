//! **The separation record** ([`crate::rules::separation`], WPM-R222, WPM-T422).
//!
//! Built from the leaver record: the last day and the reason are read from it, and this holds only the final pay date and a
//! reference to the pay summary. The reason affects a person's eligibility, so HR reads it here and a line manager does not. The
//! check is the write check, which payroll and a plain seat do not pass, plus a seat in the organization. Each read is audited with
//! the pid and no content. WPM submits nothing: the export is for HR and a payroll service to submit.

use std::collections::{BTreeMap, HashMap};

use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use super::{ensure_org_in_scope, internal, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db::{self, pool};
use crate::models::_entities::{separation_records, workers};
use crate::models::records;
use crate::rules::separation as rules;

/// HR, the service or an administrator, in this worker's organization.
async fn ensure_hr(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
) -> Result<()> {
    auth::authorize_record(
        caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(worker),
    )
    .map_err(record_rejection)?;
    ensure_org_in_scope(ctx, caller, &worker.organization_ref).await
}

/// What a separation record reads from the leaver record.
struct Leaver {
    pid: Uuid,
    worker_pid: Uuid,
    effective_on: NaiveDate,
    reason: Option<String>,
}

/// A leaver record that still stands, and its worker. Anything else has no separation record.
async fn leaver(ctx: &AppContext, pid: &str) -> Result<(Leaver, workers::Model)> {
    let movement = sqlx::query_as!(
        Leaver,
        "SELECT pid, worker_pid, effective_on, reason FROM movements
         WHERE pid = $1 AND kind = 'leaver' AND status <> 'cancelled'",
        records::parse_pid(pid)?,
    )
    .fetch_optional(pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let worker = records::worker_by_pid(ctx, movement.worker_pid).await?;
    Ok((movement, worker))
}

async fn stored(ctx: &AppContext, movement: Uuid) -> Result<Option<separation_records::Model>> {
    Ok(sqlx::query_as!(
        separation_records::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, movement_pid, worker_pid, final_pay_on,
                  pay_summary_ref, recorded_by
           FROM separation_records WHERE movement_pid = $1"#,
        movement,
    )
    .fetch_optional(pool(ctx))
    .await?)
}

fn view(movement: &Leaver, row: Option<&separation_records::Model>) -> serde_json::Value {
    let final_pay_on = row.and_then(|r| r.final_pay_on);
    let reference = row.and_then(|r| r.pay_summary_ref.as_deref());
    serde_json::json!({
        "movement_pid": movement.pid,
        "worker_pid": movement.worker_pid,
        "last_day": movement.effective_on,
        "reason_code": movement.reason,
        "final_pay_on": final_pay_on,
        "pay_summary_ref": reference,
        "missing": rules::missing(final_pay_on, reference),
    })
}

/// `GET /api/movements/{pid}/separation-record` — HR. Audited with the pid and no content.
#[debug_handler]
async fn show(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (movement, worker) = leaver(&ctx, &pid).await?;
    ensure_hr(&ctx, &caller, &worker).await?;
    let row = stored(&ctx, movement.pid).await?;
    db::audit(
        &mut *pool(&ctx).acquire().await?,
        "separation_record",
        movement.pid,
        "separation_read",
        caller.actor(),
        None,
    )
    .await?;
    format::json(view(&movement, row.as_ref()))
}

/// `PUT /api/movements/{pid}/separation-record` body.
#[derive(Debug, Deserialize)]
struct RecordPayload {
    #[serde(default)]
    final_pay_on: Option<NaiveDate>,
    #[serde(default)]
    pay_summary_ref: Option<String>,
}

/// `PUT /api/movements/{pid}/separation-record` — HR records the final pay date and the pay summary reference.
#[debug_handler]
async fn record(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<RecordPayload>,
) -> Result<Response> {
    let (movement, worker) = leaver(&ctx, &pid).await?;
    ensure_hr(&ctx, &caller, &worker).await?;
    let reference =
        rules::clean_ref(payload.pay_summary_ref.as_deref()).map_err(|e| unprocessable(&e))?;
    let existing = stored(&ctx, movement.pid).await?;
    let mut tx = pool(&ctx).begin().await?;
    let row = if let Some(row) = existing {
        sqlx::query_as!(
            separation_records::Model,
            r#"UPDATE separation_records SET final_pay_on = $2, pay_summary_ref = $3, recorded_by = $4 WHERE id = $1
               RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, movement_pid, worker_pid,
                         final_pay_on, pay_summary_ref, recorded_by"#,
            row.id,
            payload.final_pay_on,
            reference,
            caller.actor(),
        )
        .fetch_one(&mut *tx)
        .await?
    } else {
        sqlx::query_as!(
            separation_records::Model,
            r#"INSERT INTO separation_records (pid, movement_pid, worker_pid, final_pay_on, pay_summary_ref, recorded_by)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, movement_pid, worker_pid,
                         final_pay_on, pay_summary_ref, recorded_by"#,
            Uuid::new_v4(),
            movement.pid,
            worker.pid,
            payload.final_pay_on,
            reference,
            caller.actor(),
        )
        .fetch_one(&mut *tx)
        .await?
    };
    db::audit(
        &mut tx,
        "separation_record",
        movement.pid,
        "separation_recorded",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(view(&movement, Some(&row)))
}

/// `GET /api/separation-records/export` query.
#[derive(Debug, Deserialize)]
struct ExportQuery {
    organization: String,
    from: NaiveDate,
    to: NaiveDate,
}

/// `GET /api/separation-records/export?organization=&from=&to=` — one row for every leaver whose last day is in the period, with
/// what is missing, so nobody drops out because nothing was recorded. Audited with the organization's count and no names.
#[debug_handler]
async fn export(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(query): axa::extract::Query<ExportQuery>,
) -> Result<Response> {
    rules::valid_period(query.from, query.to).map_err(|e| unprocessable(&e))?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &BTreeMap::from([(
            "organization_ref".to_string(),
            vec![query.organization.clone()],
        )]),
    )
    .map_err(record_rejection)?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let staff: HashMap<Uuid, _> = sqlx::query!(
        "SELECT pid, worker_number, display_name FROM workers WHERE organization_ref = $1",
        query.organization,
    )
    .fetch_all(pool(&ctx))
    .await?
    .into_iter()
    .map(|w| (w.pid, w))
    .collect();
    let leavers = sqlx::query_as!(
        Leaver,
        "SELECT pid, worker_pid, effective_on, reason FROM movements
         WHERE kind = 'leaver' AND status <> 'cancelled' AND effective_on >= $1 AND effective_on <= $2
         ORDER BY effective_on, id",
        query.from,
        query.to,
    )
    .fetch_all(pool(&ctx))
    .await?;
    let pids: Vec<Uuid> = leavers.iter().map(|m| m.pid).collect();
    let added: HashMap<Uuid, _> = sqlx::query!(
        "SELECT movement_pid, final_pay_on, pay_summary_ref FROM separation_records WHERE movement_pid = ANY($1)",
        &pids,
    )
    .fetch_all(pool(&ctx))
    .await?
    .into_iter()
    .map(|r| (r.movement_pid, r))
    .collect();
    let lines: Vec<rules::Line> = leavers
        .into_iter()
        .filter_map(|movement| {
            let worker = staff.get(&movement.worker_pid)?;
            let row = added.get(&movement.pid);
            Some(rules::Line {
                worker_number: worker.worker_number.clone(),
                display_name: worker.display_name.clone(),
                last_day: movement.effective_on,
                reason_code: movement.reason,
                final_pay_on: row.and_then(|r| r.final_pay_on),
                pay_summary_ref: row.and_then(|r| r.pay_summary_ref.clone()),
            })
        })
        .collect();
    db::audit(
        &mut *pool(&ctx).acquire().await?,
        "separation_export",
        Uuid::new_v4(),
        "separation_exported",
        caller.actor(),
        Some(serde_json::json!({ "rows": lines.len() })),
    )
    .await?;
    axum::response::Response::builder()
        .header("content-type", "text/csv; charset=utf-8")
        .header(
            "content-disposition",
            "attachment; filename=\"separation-records.csv\"",
        )
        .body(axum::body::Body::from(rules::to_csv(&lines)))
        .map_err(|e| internal(&e.to_string()))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/movements/{pid}/separation-record", get(show))
        .add("/movements/{pid}/separation-record", put(record))
        .add("/separation-records/export", get(export))
}
