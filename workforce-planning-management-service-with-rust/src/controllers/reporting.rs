//! **Reporting lines**: who a worker reports to (their *upline*), who reports to
//! them (their *downline*), and the distinction between a **direct report**
//! (straight to the manager) and an **indirect report** (through someone
//! else). Derived from `manager_pid` over the workers **employed today**;
//! pure rules in [`crate::rules::org`].
//!
//! Scoped to the caller's organizations like the org chart: a worker outside
//! the caller's scope is `404`, and people outside it never appear.

use loco_rs::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{dotted_line_reports, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::rules::metrics::is_employed_on;
use crate::rules::org::{self, ReportKind};
use sea_orm::{ActiveValue, QueryOrder};

/// The workers employed today that the caller may see (none of them read yet).
async fn staff_select(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
) -> Result<sea_orm::Select<workers::Entity>> {
    let today = chrono::Utc::now().date_naive();
    let mut select = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::HiredOn.lte(today))
        .filter(
            sea_orm::Condition::any()
                .add(workers::Column::TerminatedOn.is_null())
                .add(workers::Column::TerminatedOn.gt(today)),
        );
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await? {
        select = select.filter(workers::Column::OrganizationRef.is_in(refs));
    }
    Ok(select)
}

/// Those of `pids` who are employed and visible, by pid. Reads only them (WPM-R142): the staff of an
/// organization of 200,000 are never loaded to answer for a handful.
async fn staff_among(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pids: &[Uuid],
) -> Result<BTreeMap<Uuid, workers::Model>> {
    let select = staff_select(ctx, caller).await?;
    Ok(super::find_in(ctx, select, workers::Column::Pid, pids)
        .await?
        .into_iter()
        .map(|w| (w.pid, w))
        .collect())
}

/// Everyone below `manager` who is employed and visible, level by level (a cycle revisits no one).
async fn staff_below(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    manager: Uuid,
) -> Result<BTreeMap<Uuid, workers::Model>> {
    let select = staff_select(ctx, caller).await?;
    let mut found: BTreeMap<Uuid, workers::Model> = BTreeMap::new();
    let mut frontier = vec![manager];
    while !frontier.is_empty() {
        let level =
            super::find_in(ctx, select.clone(), workers::Column::ManagerPid, &frontier).await?;
        frontier = Vec::new();
        for w in level {
            if w.pid != manager && !found.contains_key(&w.pid) {
                frontier.push(w.pid);
                found.insert(w.pid, w);
            }
        }
    }
    Ok(found)
}

/// The worker, required to be visible to the caller (a worker who is not, or no longer, employed is
/// still addressable; one outside the caller's organizations is not).
async fn visible_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<workers::Model> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(pid)?).await?;
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    if scope.is_some_and(|refs| !refs.contains(&worker.organization_ref)) {
        return Err(Error::NotFound);
    }
    Ok(worker)
}

fn person_json(w: &workers::Model) -> serde_json::Value {
    serde_json::json!({
        "pid": w.pid,
        "display_name": w.display_name,
        "job_title": w.job_title,
        "department": w.department,
    })
}

/// `GET /api/workers/{pid}/upline` — the management chain above a worker,
/// nearest first: `level` 1 is their direct manager, 2 their manager's
/// manager, and so on up to the top.
#[debug_handler]
async fn upline(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = visible_worker(&ctx, &caller, &pid).await?;
    // One manager at a time, up from the worker, while each is employed and visible; a cycle stops.
    let today = chrono::Utc::now().date_naive();
    let mut chain: Vec<serde_json::Value> = Vec::new();
    let mut seen = std::collections::BTreeSet::from([worker.pid]);
    let mut manager = is_employed_on(today, worker.hired_on, worker.terminated_on)
        .then_some(worker.manager_pid)
        .flatten();
    while let Some(pid) = manager.filter(|m| seen.insert(*m)) {
        let Some(boss) = staff_among(&ctx, &caller, &[pid]).await?.remove(&pid) else {
            break;
        };
        let mut json = person_json(&boss);
        json["level"] = serde_json::json!(chain.len() + 1);
        json["direct_manager"] = serde_json::json!(chain.is_empty());
        chain.push(json);
        manager = boss.manager_pid;
    }
    format::json(serde_json::json!({
        "worker": person_json(&worker),
        "upline": chain,
    }))
}

/// Query for the reports listing.
#[derive(Debug, Deserialize)]
struct KindQuery {
    /// `direct`, `indirect`, or absent for both.
    kind: Option<String>,
}

/// The downline of a manager: `(worker, depth)` for everyone below.
fn below(manager: Uuid, staff: &BTreeMap<Uuid, workers::Model>) -> Vec<(&workers::Model, usize)> {
    let mut reports_of: BTreeMap<Uuid, Vec<Uuid>> = BTreeMap::new();
    for w in staff.values() {
        if let Some(boss) = w.manager_pid {
            reports_of.entry(boss).or_default().push(w.pid);
        }
    }
    org::downline(manager, &reports_of)
        .into_iter()
        .filter_map(|(pid, depth)| staff.get(&pid).map(|w| (w, depth)))
        .collect()
}

fn report_json(w: &workers::Model, depth: usize) -> serde_json::Value {
    let mut json = person_json(w);
    json["depth"] = serde_json::json!(depth);
    json["report_kind"] = serde_json::json!(org::report_kind(depth).map(ReportKind::as_str));
    json
}

/// `GET /api/workers/{pid}/downline` — everyone below a manager: their
/// **direct reports** (depth 1) and **indirect reports** (reports of
/// reports, any depth), each marked with `report_kind` and `depth`, plus the
/// counts.
#[debug_handler]
async fn downline(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let manager = visible_worker(&ctx, &caller, &pid).await?;
    let staff = staff_below(&ctx, &caller, manager.pid).await?;
    let team = below(manager.pid, &staff);
    let direct = team.iter().filter(|(_, d)| *d == 1).count();
    format::json(serde_json::json!({
        "manager": person_json(&manager),
        "summary": { "direct": direct, "indirect": team.len() - direct, "total": team.len() },
        "downline": team.iter().map(|(w, d)| report_json(w, *d)).collect::<Vec<_>>(),
    }))
}

/// `GET /api/workers/{pid}/reports?kind=direct|indirect` — just the direct or
/// just the indirect reports (both when `kind` is absent).
#[debug_handler]
async fn reports(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axum::extract::Query(query): axum::extract::Query<KindQuery>,
) -> Result<Response> {
    let wanted = match query.kind.as_deref() {
        None => None,
        Some("direct") => Some(ReportKind::Direct),
        Some("indirect") => Some(ReportKind::Indirect),
        Some(_) => return Err(unprocessable("kind must be direct or indirect")),
    };
    let manager = visible_worker(&ctx, &caller, &pid).await?;
    let staff = staff_below(&ctx, &caller, manager.pid).await?;
    let team: Vec<serde_json::Value> = below(manager.pid, &staff)
        .iter()
        .filter(|(_, d)| wanted.is_none_or(|k| org::report_kind(*d) == Some(k)))
        .map(|(w, d)| report_json(w, *d))
        .collect();
    format::json(serde_json::json!({
        "manager": person_json(&manager),
        "kind": query.kind,
        "reports": team,
    }))
}

fn dotted_json(
    d: &dotted_line_reports::Model,
    person: Option<&workers::Model>,
) -> serde_json::Value {
    let mut json = person.map_or_else(|| serde_json::json!({}), person_json);
    json["note"] = serde_json::json!(d.note);
    json["started_at"] = serde_json::json!(d.started_at);
    json["ended_at"] = serde_json::json!(d.ended_at);
    json["current"] = serde_json::json!(d.ended_at.is_none());
    json["on_behalf"] = serde_json::json!(d.on_behalf);
    json
}

/// `GET /api/workers/{pid}/dotted-line?include_past=` — a worker's
/// **dotted-line managers** (who they report to on a secondary line) and
/// **dotted-line reports** (who reports to them that way). A dotted line is
/// separate from the solid line: it changes no org chart and, by itself,
/// grants no access to anyone's private or manager-visible aspirations.
#[debug_handler]
async fn dotted_line(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axum::extract::Query(query): axum::extract::Query<PastQuery>,
) -> Result<Response> {
    let worker = visible_worker(&ctx, &caller, &pid).await?;
    let mut select = dotted_line_reports::Entity::find();
    if !query.include_past {
        select = select.filter(dotted_line_reports::Column::EndedAt.is_null());
    }
    let rows = select
        .filter(
            sea_orm::Condition::any()
                .add(dotted_line_reports::Column::ReportPid.eq(worker.pid))
                .add(dotted_line_reports::Column::ManagerPid.eq(worker.pid)),
        )
        .order_by_desc(dotted_line_reports::Column::StartedAt)
        .all(&ctx.db)
        .await?;
    let others: Vec<Uuid> = rows
        .iter()
        .map(|d| if d.report_pid == worker.pid { d.manager_pid } else { d.report_pid })
        .collect();
    let staff = staff_among(&ctx, &caller, &others).await?;
    let managers: Vec<serde_json::Value> = rows
        .iter()
        .filter(|d| d.report_pid == worker.pid)
        .filter_map(|d| staff.get(&d.manager_pid).map(|m| dotted_json(d, Some(m))))
        .collect();
    let reports: Vec<serde_json::Value> = rows
        .iter()
        .filter(|d| d.manager_pid == worker.pid)
        .filter_map(|d| staff.get(&d.report_pid).map(|r| dotted_json(d, Some(r))))
        .collect();
    format::json(serde_json::json!({
        "worker": person_json(&worker),
        "dotted_line_managers": managers,
        "dotted_line_reports": reports,
    }))
}

/// Query for past entries.
#[derive(Debug, Deserialize)]
struct PastQuery {
    #[serde(default)]
    include_past: bool,
}

/// `POST /api/workers/{pid}/dotted-line-managers` body.
#[derive(Debug, Deserialize)]
struct DottedPayload {
    /// The dotted-line manager — anyone employed, not only the solid-line chain.
    manager_pid: Uuid,
    #[serde(default)]
    note: Option<String>,
}

/// `POST /api/workers/{pid}/dotted-line-managers` — give a worker a
/// dotted-line manager (they may have several). Choosing one they already have
/// only updates the note. A write to the *report's* record: they manage their
/// own, HR on their behalf.
#[debug_handler]
async fn add_dotted_manager(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<DottedPayload>,
) -> Result<Response> {
    let report = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&report),
    )
    .map_err(record_rejection)?;
    org::validate_dotted_line(&report.pid, &payload.manager_pid).map_err(|e| unprocessable(&e))?;
    if payload
        .note
        .as_ref()
        .is_some_and(|n| n.chars().count() > 500)
    {
        return Err(unprocessable("note is longer than 500 characters"));
    }
    let staff = staff_among(&ctx, &caller, &[payload.manager_pid]).await?;
    if !staff.contains_key(&payload.manager_pid) {
        return Err(unprocessable(
            "the dotted-line manager must be an employed worker you can see",
        ));
    }
    let current = dotted_line_reports::Entity::find()
        .filter(dotted_line_reports::Column::ReportPid.eq(report.pid))
        .filter(dotted_line_reports::Column::ManagerPid.eq(payload.manager_pid))
        .filter(dotted_line_reports::Column::EndedAt.is_null())
        .one(&ctx.db)
        .await?;
    let row = if let Some(existing) = current {
        let mut active: dotted_line_reports::ActiveModel = existing.into();
        active.note = ActiveValue::set(payload.note);
        active.update(&ctx.db).await?
    } else {
        dotted_line_reports::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            report_pid: ActiveValue::set(report.pid),
            manager_pid: ActiveValue::set(payload.manager_pid),
            note: ActiveValue::set(payload.note),
            started_at: ActiveValue::set(chrono::Utc::now().into()),
            ended_at: ActiveValue::set(None),
            recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
            on_behalf: ActiveValue::set(auth::acting_for_other(&caller, &report.person_ref)),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?
    };
    Audit::record(
        &ctx.db,
        "worker",
        report.pid,
        "dotted_line_added",
        caller.actor(),
        None,
    )
    .await?;
    format::json(dotted_json(&row, staff.get(&row.manager_pid)))
}

/// `DELETE /api/workers/{pid}/dotted-line-managers/{manager_pid}` — end a
/// dotted-line relationship; it is closed, not deleted.
#[debug_handler]
async fn end_dotted_manager(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, manager_pid)): Path<(String, String)>,
) -> Result<Response> {
    let report = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&report),
    )
    .map_err(record_rejection)?;
    let current = dotted_line_reports::Entity::find()
        .filter(dotted_line_reports::Column::ReportPid.eq(report.pid))
        .filter(dotted_line_reports::Column::ManagerPid.eq(records::parse_pid(&manager_pid)?))
        .filter(dotted_line_reports::Column::EndedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let mut active: dotted_line_reports::ActiveModel = current.into();
    active.ended_at = ActiveValue::set(Some(chrono::Utc::now().into()));
    active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        report.pid,
        "dotted_line_ended",
        caller.actor(),
        None,
    )
    .await?;
    format::empty_json()
}

/// The reporting-line routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/upline", get(upline))
        .add("/workers/{pid}/downline", get(downline))
        .add("/workers/{pid}/reports", get(reports))
        .add("/workers/{pid}/dotted-line", get(dotted_line))
        .add(
            "/workers/{pid}/dotted-line-managers",
            post(add_dotted_manager),
        )
        .add(
            "/workers/{pid}/dotted-line-managers/{manager_pid}",
            delete(end_dotted_manager),
        )
}
