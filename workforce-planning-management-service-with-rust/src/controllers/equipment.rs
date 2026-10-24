//! Equipment issued and returned (WPM-R155): assets, issues to a worker, and returns or write-offs. An asset is not located: no
//! position, cost or reading is held. Equipment still out keeps a leaver's record from being completed (`controllers::movements`).
//! HR writes; a worker reads their own. Spec: `spec/joiners-and-leavers.md`.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, row_json, unprocessable, writable_worker};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{asset_issues, assets, workers};
use crate::models::audit_logs::Model as Audit;
use crate::rules::equipment as rules;
use crate::rules::metrics::is_employed_on;

/// How many pieces of equipment `worker` still holds.
pub(crate) async fn out_count(ctx: &AppContext, worker: Uuid) -> Result<usize> {
    let n = asset_issues::Entity::find()
        .filter(asset_issues::Column::WorkerPid.eq(worker))
        .filter(asset_issues::Column::ReturnedOn.is_null())
        .count(&ctx.db)
        .await?;
    Ok(usize::try_from(n).unwrap_or(usize::MAX))
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListQuery {
    organization: String,
}

/// `GET /api/assets?organization=` — the assets, and whether each is out. HR; who holds one is read per worker.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let rows = assets::Entity::find()
        .filter(assets::Column::OrganizationRef.eq(&query.organization))
        .order_by_asc(assets::Column::Tag)
        .all(&ctx.db)
        .await?;
    let open: Vec<Uuid> = asset_issues::Entity::find()
        .filter(asset_issues::Column::ReturnedOn.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|i| i.asset_pid)
        .collect();
    let mut out = Vec::new();
    for row in &rows {
        let mut v = row_json(row)?;
        if let Some(o) = v.as_object_mut() {
            o.remove("recorded_by");
            o.insert("out".into(), json!(open.contains(&row.pid)));
        }
        out.push(v);
    }
    format::json(json!({ "assets": out }))
}

#[derive(Debug, Deserialize)]
struct AssetPayload {
    organization_ref: String,
    kind: String,
    tag: String,
    owner_role: String,
}

/// `POST /api/assets` — add an asset. A tag is unique within the organization.
#[debug_handler]
async fn create(State(ctx): State<AppContext>, caller: MaybeAuthUser, Json(payload): Json<AssetPayload>) -> Result<Response> {
    ensure_valid(&organization_problems(&payload.organization_ref))?;
    ensure_org_in_scope(&ctx, &caller, &payload.organization_ref).await?;
    ensure_valid(&rules::asset_problems(&payload.kind, &payload.tag, &payload.owner_role))?;
    let tag = payload.tag.trim().to_string();
    let clash = assets::Entity::find()
        .filter(assets::Column::OrganizationRef.eq(&payload.organization_ref))
        .filter(assets::Column::Tag.eq(&tag))
        .count(&ctx.db)
        .await?;
    if clash > 0 {
        return Err(unprocessable("another asset in this organization has that tag"));
    }
    let pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    assets::ActiveModel {
        pid: ActiveValue::set(pid),
        organization_ref: ActiveValue::set(payload.organization_ref),
        kind: ActiveValue::set(payload.kind.clone()),
        tag: ActiveValue::set(tag),
        owner_role: ActiveValue::set(payload.owner_role.trim().to_string()),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(&txn, "asset", pid, "asset_created", caller.actor(), Some(json!({ "kind": payload.kind }))).await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

async fn find_asset(ctx: &AppContext, caller: &MaybeAuthUser, pid: Uuid) -> Result<assets::Model> {
    let row = assets::Entity::find()
        .filter(assets::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &row.organization_ref).await?;
    Ok(row)
}

#[derive(Debug, Deserialize)]
struct IssuePayload {
    worker_pid: Uuid,
    issued_on: Option<NaiveDate>,
}

/// `POST /api/assets/{pid}/issue` — give an asset to a worker of its organization who is still employed. One holder at a time.
#[debug_handler]
async fn issue(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(payload): Json<IssuePayload>,
) -> Result<Response> {
    let asset = find_asset(&ctx, &caller, pid).await?;
    let worker = workers::Entity::find()
        .filter(workers::Column::Pid.eq(payload.worker_pid))
        .filter(workers::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or_else(|| unprocessable("the worker is not in this organization"))?;
    if worker.organization_ref != asset.organization_ref {
        return Err(unprocessable("the worker is not in this organization"));
    }
    let today = Utc::now().date_naive();
    if !is_employed_on(today, worker.hired_on, worker.terminated_on) {
        return Err(unprocessable("the worker is not employed today"));
    }
    let issued_on = payload.issued_on.unwrap_or(today);
    ensure_valid(&rules::issue_problems(issued_on, today, worker.hired_on))?;
    let held = asset_issues::Entity::find()
        .filter(asset_issues::Column::AssetPid.eq(pid))
        .filter(asset_issues::Column::ReturnedOn.is_null())
        .count(&ctx.db)
        .await?;
    if held > 0 {
        return Err(unprocessable("the asset is already out; record its return first"));
    }
    let issue_pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    asset_issues::ActiveModel {
        pid: ActiveValue::set(issue_pid),
        asset_pid: ActiveValue::set(pid),
        worker_pid: ActiveValue::set(worker.pid),
        issued_on: ActiveValue::set(issued_on),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(&txn, "asset_issue", issue_pid, "asset_issued", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(json!({ "pid": issue_pid }))
}

#[derive(Debug, Deserialize)]
struct ReturnPayload {
    outcome: String,
    returned_on: Option<NaiveDate>,
    note: Option<String>,
}

/// `POST /api/asset-issues/{pid}/return` — close an open issue as returned, or written off with a reason.
#[debug_handler]
async fn close(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(payload): Json<ReturnPayload>,
) -> Result<Response> {
    let issue = asset_issues::Entity::find()
        .filter(asset_issues::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    find_asset(&ctx, &caller, issue.asset_pid).await?;
    if issue.returned_on.is_some() {
        return Err(unprocessable("this issue is already closed"));
    }
    let today = Utc::now().date_naive();
    let returned_on = payload.returned_on.unwrap_or(today);
    ensure_valid(&rules::return_problems(
        &payload.outcome,
        returned_on,
        issue.issued_on,
        today,
        payload.note.as_deref(),
    ))?;
    let action = if payload.outcome == "returned" { "asset_returned" } else { "asset_written_off" };
    let txn = ctx.db.begin().await?;
    let mut active: asset_issues::ActiveModel = issue.into();
    active.returned_on = ActiveValue::set(Some(returned_on));
    active.outcome = ActiveValue::set(Some(payload.outcome.clone()));
    active.note = ActiveValue::set(payload.note.as_ref().map(|n| n.trim().to_string()).filter(|n| !n.is_empty()));
    active.recorded_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    active.update(&txn).await?;
    Audit::record(&txn, "asset_issue", pid, action, caller.actor(), None).await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid, "outcome": payload.outcome }))
}

/// `GET /api/workers/{pid}/equipment` — what the worker holds and has held. The worker themself and HR.
#[debug_handler]
async fn of_worker(State(ctx): State<AppContext>, caller: MaybeAuthUser, Path(pid): Path<String>) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let issues = asset_issues::Entity::find()
        .filter(asset_issues::Column::WorkerPid.eq(worker.pid))
        .order_by_desc(asset_issues::Column::IssuedOn)
        .all(&ctx.db)
        .await?;
    let mut out = Vec::new();
    for i in &issues {
        let asset = assets::Entity::find()
            .filter(assets::Column::Pid.eq(i.asset_pid))
            .one(&ctx.db)
            .await?;
        out.push(json!({
            "issue_pid": i.pid,
            "asset_pid": i.asset_pid,
            "kind": asset.as_ref().map(|a| a.kind.clone()),
            "tag": asset.as_ref().map(|a| a.tag.clone()),
            "issued_on": i.issued_on,
            "returned_on": i.returned_on,
            "outcome": i.outcome,
            "note": i.note,
        }));
    }
    let still_out = issues.iter().filter(|i| i.returned_on.is_none()).count();
    format::json(json!({ "worker_pid": worker.pid, "out": still_out, "issues": out }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/assets", get(list))
        .add("/assets", post(create))
        .add("/assets/{pid}/issue", post(issue))
        .add("/asset-issues/{pid}/return", post(close))
        .add("/workers/{pid}/equipment", get(of_worker))
}
