//! Equipment issued and returned (WPM-R155): assets, issues to a worker, and returns or write-offs. An asset is not located: no
//! position, cost or reading is held. Equipment still out keeps a leaver's record from being completed (`controllers::movements`).
//! HR writes; a worker reads their own. Spec: `spec/joiners-and-leavers.md`.

use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{
    ensure_org_in_scope, ensure_valid, forbidden, row_json, unprocessable, writable_worker,
};
use crate::auth::{self, MaybeAuthUser};
use crate::db::{self, pool};
use crate::models::_entities::assets;
use crate::models::records;
use crate::rules::equality_monitoring::DEFAULT_FLOOR;
use crate::rules::equipment as rules;
use crate::rules::metrics::is_employed_on;

/// How many pieces of equipment `worker` still holds.
pub(crate) async fn out_count(ctx: &AppContext, worker: Uuid) -> Result<usize> {
    let n = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM asset_issues WHERE worker_pid = $1 AND returned_on IS NULL"#,
        worker,
    )
    .fetch_one(pool(ctx))
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
    let rows = sqlx::query_as!(
        assets::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, organization_ref, kind, tag, owner_role, recorded_by
           FROM assets WHERE organization_ref = $1 ORDER BY tag"#,
        query.organization,
    )
    .fetch_all(pool(&ctx))
    .await?;
    let open = sqlx::query_scalar!("SELECT asset_pid FROM asset_issues WHERE returned_on IS NULL")
        .fetch_all(pool(&ctx))
        .await?;
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
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<AssetPayload>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&payload.organization_ref))?;
    ensure_org_in_scope(&ctx, &caller, &payload.organization_ref).await?;
    ensure_valid(&rules::asset_problems(
        &payload.kind,
        &payload.tag,
        &payload.owner_role,
    ))?;
    let tag = payload.tag.trim().to_string();
    let clash = sqlx::query_scalar!(
        r#"SELECT EXISTS (
               SELECT 1 FROM assets WHERE organization_ref = $1 AND tag = $2
           ) AS "clash!""#,
        payload.organization_ref,
        tag,
    )
    .fetch_one(pool(&ctx))
    .await?;
    if clash {
        return Err(unprocessable(
            "another asset in this organization has that tag",
        ));
    }
    let pid = Uuid::new_v4();
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO assets (pid, organization_ref, kind, tag, owner_role, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6)",
        pid,
        payload.organization_ref,
        payload.kind,
        tag,
        payload.owner_role.trim(),
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "asset",
        pid,
        "asset_created",
        caller.actor(),
        Some(json!({ "kind": payload.kind })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

async fn find_asset(ctx: &AppContext, caller: &MaybeAuthUser, pid: Uuid) -> Result<assets::Model> {
    let row = sqlx::query_as!(
        assets::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, organization_ref, kind, tag, owner_role, recorded_by
           FROM assets WHERE pid = $1"#,
        pid,
    )
    .fetch_optional(pool(ctx))
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
    let worker = sqlx::query!(
        "SELECT pid, organization_ref, hired_on, terminated_on FROM workers
         WHERE pid = $1 AND deleted_at IS NULL",
        payload.worker_pid,
    )
    .fetch_optional(pool(&ctx))
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
    let held = sqlx::query_scalar!(
        r#"SELECT EXISTS (
               SELECT 1 FROM asset_issues WHERE asset_pid = $1 AND returned_on IS NULL
           ) AS "held!""#,
        pid,
    )
    .fetch_one(pool(&ctx))
    .await?;
    if held {
        return Err(unprocessable(
            "the asset is already out; record its return first",
        ));
    }
    let issue_pid = Uuid::new_v4();
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO asset_issues (pid, asset_pid, worker_pid, issued_on, recorded_by)
         VALUES ($1, $2, $3, $4, $5)",
        issue_pid,
        pid,
        worker.pid,
        issued_on,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "asset_issue",
        issue_pid,
        "asset_issued",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let issue = sqlx::query!(
        "SELECT asset_pid, issued_on, returned_on FROM asset_issues WHERE pid = $1",
        pid,
    )
    .fetch_optional(pool(&ctx))
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
    let action = if payload.outcome == "returned" {
        "asset_returned"
    } else {
        "asset_written_off"
    };
    let note = payload
        .note
        .as_ref()
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE asset_issues SET returned_on = $2, outcome = $3, note = $4, recorded_by = $5
         WHERE pid = $1",
        pid,
        returned_on,
        payload.outcome,
        note,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "asset_issue", pid, action, caller.actor(), None).await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid, "outcome": payload.outcome }))
}

/// `GET /api/workers/{pid}/equipment` — what the worker holds and has held. The worker themself and HR.
#[debug_handler]
async fn of_worker(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let issues = sqlx::query!(
        r#"SELECT i.pid, i.asset_pid, i.issued_on, i.returned_on, i.outcome, i.note,
                  a.kind AS "kind?", a.tag AS "tag?"
           FROM asset_issues i
           LEFT JOIN assets a ON a.pid = i.asset_pid
           WHERE i.worker_pid = $1
           ORDER BY i.issued_on DESC"#,
        worker.pid,
    )
    .fetch_all(pool(&ctx))
    .await?;
    let still_out = issues.iter().filter(|i| i.returned_on.is_none()).count();
    let out: Vec<serde_json::Value> = issues
        .into_iter()
        .map(|i| {
            json!({
                "issue_pid": i.pid,
                "asset_pid": i.asset_pid,
                "kind": i.kind,
                "tag": i.tag,
                "issued_on": i.issued_on,
                "returned_on": i.returned_on,
                "outcome": i.outcome,
                "note": i.note,
            })
        })
        .collect();
    format::json(json!({ "worker_pid": worker.pid, "out": still_out, "issues": out }))
}

/// `GET /api/workers/{pid}/team-equipment-count` — for a manager: how many of their direct reports in work hold equipment, from the floor;
/// under it, nothing. No item, kind or name. Read as the manager `{pid}`; anyone else needs HR's access.
#[debug_handler]
async fn team_count(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let manager = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    if auth::acting_for_other(&caller, &manager.person_ref)
        && !super::policies::privileged(&ctx, &caller, &manager.organization_ref).await?
    {
        return Err(forbidden("a manager's team count is for that manager"));
    }
    let today = Utc::now().date_naive();
    let team: Vec<Uuid> = sqlx::query!(
        "SELECT pid, hired_on, terminated_on FROM workers
         WHERE manager_pid = $1 AND organization_ref = $2 AND deleted_at IS NULL",
        manager.pid,
        manager.organization_ref,
    )
    .fetch_all(pool(&ctx))
    .await?
    .into_iter()
    .filter(|w| is_employed_on(today, w.hired_on, w.terminated_on))
    .map(|w| w.pid)
    .collect();
    let holders = sqlx::query_scalar!(
        "SELECT DISTINCT worker_pid FROM asset_issues
         WHERE worker_pid = ANY($1) AND returned_on IS NULL",
        &team,
    )
    .fetch_all(pool(&ctx))
    .await?;
    let count = rules::team_holding(team.len(), holders.len(), DEFAULT_FLOOR);
    format::json(json!({ "manager_pid": manager.pid, "floor": DEFAULT_FLOOR, "count": count }))
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
        .add("/workers/{pid}/team-equipment-count", get(team_count))
}
