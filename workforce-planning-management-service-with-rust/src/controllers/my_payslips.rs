//! **My payslips** (WPM-R178, WPM-T324): a person's own payslips, the year to date per tax year, and payslip queries.
//!
//! `/api/me/payslips` resolves the person from the verified token, so there is no worker id to change. It shows approved
//! and paid runs only (a draft or calculated run can still change) and every read is audited. A query is a flag and a
//! comment: it changes no amount. Payroll reads and closes queries at `/api/payslip-queries`, within the organizations
//! the caller holds. A comment is never put in an audit entry.

use std::collections::{BTreeMap, HashMap};

use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;

use super::jurisdictions::Scope;
use super::me::me_worker;
use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{payroll_runs, payslip_queries, payslips, workers};
use crate::models::audit_logs::Model as Audit;
use crate::rules::my_payslips as rules;
use crate::rules::payroll::Deduction;

fn query_view(row: &payslip_queries::Model) -> serde_json::Value {
    json!({
        "pid": row.pid,
        "comment": row.comment,
        "raised_on": row.raised_on,
        "closed_on": row.closed_on,
        "open": row.closed_on.is_none(),
    })
}

/// The deduction lines of a payslip. A stored line that does not read is an error, never a smaller deduction.
fn deduction_lines(slip: &payslips::Model) -> Result<Vec<Deduction>> {
    serde_json::from_value(slip.deductions.clone())
        .map_err(|e| Error::string(&format!("a payslip's deductions do not read: {e}")))
}

/// The tax year start that applies to a worker on a date, if one is configured.
fn tax_year_start(scope: &Scope, today: NaiveDate) -> Option<(u32, u32)> {
    scope
        .resolve("payroll.tax_year_start", today)
        .integer()
        .and_then(rules::tax_year_start)
}

/// `GET /api/me/payslips` — the caller's payslips from approved and paid runs, newest first, and the year to date per tax
/// year and currency. With no tax year start configured the year to date is `null`, never a guess. Audited.
#[debug_handler]
pub(crate) async fn my_payslips(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let rows = payslips::Entity::find()
        .filter(payslips::Column::WorkerPid.eq(worker.pid))
        .filter(payslips::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let run_pids: Vec<Uuid> = rows.iter().map(|s| s.run_pid).collect();
    let runs: HashMap<Uuid, payroll_runs::Model> = payroll_runs::Entity::find()
        .filter(payroll_runs::Column::Pid.is_in(run_pids))
        .filter(payroll_runs::Column::Status.is_in(rules::VISIBLE_RUN_STATUSES))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|r| (r.pid, r))
        .collect();
    let mut queries: BTreeMap<Uuid, payslip_queries::Model> = BTreeMap::new();
    for query in payslip_queries::Entity::find()
        .filter(payslip_queries::Column::WorkerPid.eq(worker.pid))
        .filter(payslip_queries::Column::DeletedAt.is_null())
        .order_by_asc(payslip_queries::Column::Id)
        .all(&ctx.db)
        .await?
    {
        // The latest query on a payslip is the one shown.
        queries.insert(query.payslip_pid, query);
    }
    let mut seen: Vec<(&payslips::Model, &payroll_runs::Model)> = rows
        .iter()
        .filter_map(|slip| runs.get(&slip.run_pid).map(|run| (slip, run)))
        .collect();
    seen.sort_by(|a, b| {
        b.1.period_end
            .cmp(&a.1.period_end)
            .then_with(|| b.0.id.cmp(&a.0.id))
    });
    let mut listed = Vec::with_capacity(seen.len());
    let mut slips = Vec::with_capacity(seen.len());
    for (slip, run) in &seen {
        let deductions = deduction_lines(slip)?;
        listed.push(json!({
            "pid": slip.pid,
            "period_start": run.period_start,
            "period_end": run.period_end,
            "run_status": run.status,
            "currency": slip.currency,
            "gross_minor": slip.gross_minor,
            "statutory_overtime_minor": slip.statutory_overtime_minor,
            "deductions": deductions,
            "net_minor": slip.net_minor,
            "query": queries.get(&slip.pid).map(query_view),
        }));
        slips.push(rules::Slip {
            period_end: run.period_end,
            currency: slip.currency.clone(),
            gross_minor: slip.gross_minor,
            deductions,
            net_minor: slip.net_minor,
        });
    }
    let today = Utc::now().date_naive();
    let scope = Scope::for_worker(&ctx.db, &worker).await?;
    let year_to_date = tax_year_start(&scope, today)
        .map(|start| rules::year_to_date(&slips, start))
        .transpose()
        .map_err(|e| Error::string(&e))?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "my_payslips_read",
        caller.actor(),
        None,
    )
    .await?;
    format::json(json!({ "payslips": listed, "year_to_date": year_to_date }))
}

/// The body of a query.
#[derive(Debug, Deserialize)]
pub(crate) struct QueryPayload {
    /// What the query is about.
    comment: String,
}

/// `POST /api/me/payslips/{pid}/query` — flag your own payslip with a short comment. A payslip that is someone else's, or
/// of a run the person cannot yet see, is a 404. One query is open at a time. Audited, without the comment.
#[debug_handler]
pub(crate) async fn raise_query(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<QueryPayload>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let pid = Uuid::parse_str(&pid).map_err(|_| Error::NotFound)?;
    let slip = payslips::Entity::find()
        .filter(payslips::Column::Pid.eq(pid))
        .filter(payslips::Column::WorkerPid.eq(worker.pid))
        .filter(payslips::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let run = payroll_runs::Entity::find()
        .filter(payroll_runs::Column::Pid.eq(slip.run_pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    if !rules::visible(&run.status) {
        return Err(Error::NotFound);
    }
    let comment = rules::clean_comment(&payload.comment).map_err(|e| unprocessable(&e))?;
    let open = payslip_queries::Entity::find()
        .filter(payslip_queries::Column::PayslipPid.eq(slip.pid))
        .filter(payslip_queries::Column::ClosedOn.is_null())
        .filter(payslip_queries::Column::DeletedAt.is_null())
        .count(&ctx.db)
        .await?
        > 0;
    rules::may_raise(open).map_err(|e| unprocessable(&e))?;
    let query_pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    let saved = payslip_queries::ActiveModel {
        pid: ActiveValue::set(query_pid),
        payslip_pid: ActiveValue::set(slip.pid),
        worker_pid: ActiveValue::set(worker.pid),
        comment: ActiveValue::set(comment),
        raised_on: ActiveValue::set(Utc::now().date_naive()),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "payslip_query",
        query_pid,
        "payslip_query_raised",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(query_view(&saved))
}

/// What the list of queries takes.
#[derive(Debug, Deserialize)]
struct ListQuery {
    /// `open` (the default) or `closed`.
    status: Option<String>,
}

/// `GET /api/payslip-queries` — the queries to deal with (`?status=open|closed`, default open), oldest first, in the
/// organizations the caller holds. HR and payroll. Each names the person, the payslip and its period. Audited, as counts.
#[debug_handler]
async fn list_queries(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    let wanted = query.status.as_deref().unwrap_or("open");
    if !["open", "closed"].contains(&wanted) {
        return Err(unprocessable("status is open or closed"));
    }
    let scope =
        crate::models::memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let filter =
        payslip_queries::Entity::find().filter(payslip_queries::Column::DeletedAt.is_null());
    let filter = if wanted == "open" {
        filter.filter(payslip_queries::Column::ClosedOn.is_null())
    } else {
        filter.filter(payslip_queries::Column::ClosedOn.is_not_null())
    };
    let rows = filter
        .order_by_asc(payslip_queries::Column::RaisedOn)
        .order_by_asc(payslip_queries::Column::Id)
        .all(&ctx.db)
        .await?;
    let worker_pids: Vec<Uuid> = rows.iter().map(|r| r.worker_pid).collect();
    let slip_pids: Vec<Uuid> = rows.iter().map(|r| r.payslip_pid).collect();
    let people: HashMap<Uuid, workers::Model> = workers::Entity::find()
        .filter(workers::Column::Pid.is_in(worker_pids))
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| (w.pid, w))
        .collect();
    let slips: HashMap<Uuid, payslips::Model> = payslips::Entity::find()
        .filter(payslips::Column::Pid.is_in(slip_pids))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|s| (s.pid, s))
        .collect();
    let run_pids: Vec<Uuid> = slips.values().map(|s| s.run_pid).collect();
    let runs: HashMap<Uuid, payroll_runs::Model> = payroll_runs::Entity::find()
        .filter(payroll_runs::Column::Pid.is_in(run_pids))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|r| (r.pid, r))
        .collect();
    let mut out = Vec::new();
    for row in &rows {
        let Some(person) = people.get(&row.worker_pid) else {
            continue;
        };
        if let Some(refs) = &scope
            && !refs.contains(&person.organization_ref)
        {
            continue;
        }
        let run = slips
            .get(&row.payslip_pid)
            .and_then(|slip| runs.get(&slip.run_pid));
        let mut view = query_view(row);
        view["worker_pid"] = json!(person.pid);
        view["display_name"] = json!(person.display_name);
        view["payslip_pid"] = json!(row.payslip_pid);
        view["run_pid"] = json!(run.map(|r| r.pid));
        view["period_start"] = json!(run.map(|r| r.period_start));
        view["period_end"] = json!(run.map(|r| r.period_end));
        out.push(view);
    }
    let shown = out.len();
    Audit::record(
        &ctx.db,
        "payslip_query",
        Uuid::nil(),
        "payslip_queries_read",
        caller.actor(),
        Some(json!({ "shown": shown })),
    )
    .await?;
    format::json(out)
}

/// `POST /api/payslip-queries/{pid}/close` — payroll closes a query. It changes no amount; a correction is made by
/// reopening or adjusting a run, outside the query. Out of scope is a 404.
#[debug_handler]
async fn close_query(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let pid = Uuid::parse_str(&pid).map_err(|_| Error::NotFound)?;
    let row = payslip_queries::Entity::find()
        .filter(payslip_queries::Column::Pid.eq(pid))
        .filter(payslip_queries::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let worker = workers::Entity::find()
        .filter(workers::Column::Pid.eq(row.worker_pid))
        .filter(workers::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    super::ensure_org_in_scope(&ctx, &caller, &worker.organization_ref).await?;
    if row.closed_on.is_some() {
        return Err(unprocessable("this query is already closed"));
    }
    let today = Utc::now().date_naive();
    let txn = ctx.db.begin().await?;
    let mut active: payslip_queries::ActiveModel = row.into();
    active.closed_on = ActiveValue::set(Some(today));
    active.closed_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    let saved = active.update(&txn).await?;
    Audit::record(
        &txn,
        "payslip_query",
        pid,
        "payslip_query_closed",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(query_view(&saved))
}

/// The routes payroll uses; a person's are registered in [`super::me`], with the rest of `/api/me`.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/payslip-queries", get(list_queries))
        .add("/payslip-queries/{pid}/close", post(close_query))
}
