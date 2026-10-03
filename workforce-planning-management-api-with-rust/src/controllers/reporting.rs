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

use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::models::_entities::workers;
use crate::models::{memberships, records};
use crate::rules::metrics::is_employed_on;
use crate::rules::org::{self, ReportKind};

/// The employed workers the caller may see, by pid.
async fn visible_staff(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
) -> Result<BTreeMap<Uuid, workers::Model>> {
    let today = chrono::Utc::now().date_naive();
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    Ok(workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| is_employed_on(today, w.hired_on, w.terminated_on))
        .filter(|w| {
            scope
                .as_ref()
                .is_none_or(|refs| refs.contains(&w.organization_ref))
        })
        .map(|w| (w.pid, w))
        .collect())
}

fn person_json(w: &workers::Model) -> serde_json::Value {
    serde_json::json!({
        "pid": w.pid,
        "display_name": w.display_name,
        "job_title": w.job_title,
        "department": w.department,
    })
}

/// The worker, required to be visible to the caller.
async fn visible_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<(workers::Model, BTreeMap<Uuid, workers::Model>)> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(pid)?).await?;
    let staff = visible_staff(ctx, caller).await?;
    // A worker who is not (or no longer) employed is still addressable, but
    // one outside the caller's organizations is not.
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    if scope.is_some_and(|refs| !refs.contains(&worker.organization_ref)) {
        return Err(Error::NotFound);
    }
    Ok((worker, staff))
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
    let (worker, staff) = visible_worker(&ctx, &caller, &pid).await?;
    let manager_of: BTreeMap<Uuid, Uuid> = staff
        .values()
        .filter_map(|w| w.manager_pid.map(|m| (w.pid, m)))
        .collect();
    let chain: Vec<serde_json::Value> = org::upline(worker.pid, &manager_of)
        .iter()
        .enumerate()
        .filter_map(|(i, m)| {
            staff.get(m).map(|boss| {
                let mut json = person_json(boss);
                json["level"] = serde_json::json!(i + 1);
                json["direct_manager"] = serde_json::json!(i == 0);
                json
            })
        })
        .collect();
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
    let (manager, staff) = visible_worker(&ctx, &caller, &pid).await?;
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
    let (manager, staff) = visible_worker(&ctx, &caller, &pid).await?;
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

/// The reporting-line routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/upline", get(upline))
        .add("/workers/{pid}/downline", get(downline))
        .add("/workers/{pid}/reports", get(reports))
}
