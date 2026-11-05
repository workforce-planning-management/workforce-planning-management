//! **The separation record** ([`crate::rules::separation`], WPM-R222, WPM-T422).
//!
//! Built from the leaver record: the last day and the reason are read from it, and this holds only the final pay date and a
//! reference to the pay summary. The reason affects a person's eligibility, so HR reads it here and a line manager does not. The
//! check is the write check, which payroll and a plain seat do not pass, plus a seat in the organization. Each read is audited with
//! the pid and no content. WPM submits nothing: the export is for HR and a payroll service to submit.

use std::collections::{BTreeMap, HashMap};

use axa::prelude::*;
use chrono::NaiveDate;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait, TryIntoModel};
use serde::Deserialize;
use uuid::Uuid;

use super::{ensure_org_in_scope, internal, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{movements, separation_records, workers};
use crate::models::audit_logs::Model as Audit;
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

/// A leaver record that still stands, and its worker. Anything else has no separation record.
async fn leaver(ctx: &AppContext, pid: &str) -> Result<(movements::Model, workers::Model)> {
    let movement = movements::Entity::find()
        .filter(movements::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(movements::Column::Kind.eq("leaver"))
        .filter(movements::Column::Status.ne("cancelled"))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let worker = records::find_worker(&ctx.db, movement.worker_pid).await?;
    Ok((movement, worker))
}

async fn stored(ctx: &AppContext, movement: Uuid) -> Result<Option<separation_records::Model>> {
    Ok(separation_records::Entity::find()
        .filter(separation_records::Column::MovementPid.eq(movement))
        .one(&ctx.db)
        .await?)
}

fn view(movement: &movements::Model, row: Option<&separation_records::Model>) -> serde_json::Value {
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
    Audit::record(
        &ctx.db,
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
    let txn = ctx.db.begin().await?;
    let mut active = existing.map_or_else(
        || separation_records::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            movement_pid: ActiveValue::set(movement.pid),
            worker_pid: ActiveValue::set(worker.pid),
            ..Default::default()
        },
        Into::into,
    );
    active.final_pay_on = ActiveValue::set(payload.final_pay_on);
    active.pay_summary_ref = ActiveValue::set(reference);
    active.recorded_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    let row = active.save(&txn).await?.try_into_model()?;
    Audit::record(
        &txn,
        "separation_record",
        movement.pid,
        "separation_recorded",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
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
    axum::extract::Query(query): axum::extract::Query<ExportQuery>,
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
    let staff: HashMap<Uuid, workers::Model> = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&query.organization))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| (w.pid, w))
        .collect();
    let leavers = movements::Entity::find()
        .filter(movements::Column::Kind.eq("leaver"))
        .filter(movements::Column::Status.ne("cancelled"))
        .filter(movements::Column::EffectiveOn.gte(query.from))
        .filter(movements::Column::EffectiveOn.lte(query.to))
        .order_by_asc(movements::Column::EffectiveOn)
        .order_by_asc(movements::Column::Id)
        .all(&ctx.db)
        .await?;
    let pids: Vec<Uuid> = leavers.iter().map(|m| m.pid).collect();
    let added: HashMap<Uuid, separation_records::Model> = super::find_in(
        &ctx,
        separation_records::Entity::find(),
        separation_records::Column::MovementPid,
        &pids,
    )
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
    Audit::record(
        &ctx.db,
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
