//! **My payslips** (WPM-R178, WPM-T324): a person's own payslips, the year to date per tax year, and payslip queries.
//!
//! `/api/me/payslips` resolves the person from the verified token, so there is no worker id to change. It shows approved
//! and paid runs only (a draft or calculated run can still change) and every read is audited. A query is a flag and a
//! comment: it changes no amount. Payroll reads and closes queries at `/api/payslip-queries`, within the organizations
//! the caller holds. A comment is never put in an audit entry.

use std::collections::{BTreeMap, HashMap};

use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::json;

use super::jurisdictions::Scope;
use super::me::me_worker;
use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::db::{self, pool};
use crate::models::_entities::{payroll_runs, payslip_queries, payslips};
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
    let rows = sqlx::query_as!(
        payslips::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, run_pid, worker_pid, currency, gross_minor, deductions, net_minor,
                  statutory_overtime_minor, employer_cost_minor, employer_costs, deleted_at AS "deleted_at: _"
           FROM payslips WHERE worker_pid = $1 AND deleted_at IS NULL"#,
        worker.pid,
    )
    .fetch_all(pool(&ctx))
    .await?;
    let run_pids: Vec<Uuid> = rows.iter().map(|s| s.run_pid).collect();
    let visible = rules::VISIBLE_RUN_STATUSES.map(String::from);
    let runs: HashMap<Uuid, payroll_runs::Model> = sqlx::query_as!(
        payroll_runs::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, organization_ref, period_start, period_end, status, approved_by, notes,
                  deleted_at AS "deleted_at: _"
           FROM payroll_runs WHERE pid = ANY($1) AND status = ANY($2)"#,
        &run_pids,
        &visible[..],
    )
    .fetch_all(pool(&ctx))
    .await?
    .into_iter()
    .map(|r| (r.pid, r))
    .collect();
    let mut queries: BTreeMap<Uuid, payslip_queries::Model> = BTreeMap::new();
    for query in sqlx::query_as!(
        payslip_queries::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, payslip_pid, worker_pid, comment, raised_on, closed_on, closed_by, deleted_at AS "deleted_at: _"
           FROM payslip_queries
           WHERE worker_pid = $1 AND deleted_at IS NULL
           ORDER BY id"#,
        worker.pid,
    )
    .fetch_all(pool(&ctx))
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
    let scope = Scope::for_worker(crate::db::pool(&ctx), &worker).await?;
    let year_to_date = tax_year_start(&scope, today)
        .map(|start| rules::year_to_date(&slips, start))
        .transpose()
        .map_err(|e| Error::string(&e))?;
    db::audit(
        &mut *pool(&ctx).acquire().await?,
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
    let slip = sqlx::query!(
        "SELECT pid, run_pid FROM payslips
         WHERE pid = $1 AND worker_pid = $2 AND deleted_at IS NULL",
        pid,
        worker.pid,
    )
    .fetch_optional(pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let status = sqlx::query_scalar!(
        "SELECT status FROM payroll_runs WHERE pid = $1",
        slip.run_pid
    )
    .fetch_optional(pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    if !rules::visible(&status) {
        return Err(Error::NotFound);
    }
    let comment = rules::clean_comment(&payload.comment).map_err(|e| unprocessable(&e))?;
    let open = sqlx::query_scalar!(
        r#"SELECT EXISTS (
               SELECT 1 FROM payslip_queries
               WHERE payslip_pid = $1 AND closed_on IS NULL AND deleted_at IS NULL
           ) AS "open!""#,
        slip.pid,
    )
    .fetch_one(pool(&ctx))
    .await?;
    rules::may_raise(open).map_err(|e| unprocessable(&e))?;
    let query_pid = Uuid::new_v4();
    let mut tx = pool(&ctx).begin().await?;
    let saved = sqlx::query_as!(
        payslip_queries::Model,
        r#"INSERT INTO payslip_queries (pid, payslip_pid, worker_pid, comment, raised_on)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, payslip_pid, worker_pid, comment, raised_on, closed_on, closed_by, deleted_at AS "deleted_at: _""#,
        query_pid,
        slip.pid,
        worker.pid,
        comment,
        Utc::now().date_naive(),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "payslip_query",
        query_pid,
        "payslip_query_raised",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    axa::extract::Query(query): axa::extract::Query<ListQuery>,
) -> Result<Response> {
    let wanted = query.status.as_deref().unwrap_or("open");
    if !["open", "closed"].contains(&wanted) {
        return Err(unprocessable("status is open or closed"));
    }
    let scope =
        crate::models::memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let rows = sqlx::query_as!(
        payslip_queries::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, payslip_pid, worker_pid, comment, raised_on, closed_on, closed_by, deleted_at AS "deleted_at: _"
           FROM payslip_queries
           WHERE deleted_at IS NULL AND (closed_on IS NULL) = $1
           ORDER BY raised_on, id"#,
        wanted == "open",
    )
    .fetch_all(pool(&ctx))
    .await?;
    let worker_pids: Vec<Uuid> = rows.iter().map(|r| r.worker_pid).collect();
    let slip_pids: Vec<Uuid> = rows.iter().map(|r| r.payslip_pid).collect();
    let people: HashMap<Uuid, _> = sqlx::query!(
        "SELECT pid, display_name, organization_ref FROM workers
         WHERE pid = ANY($1) AND deleted_at IS NULL",
        &worker_pids,
    )
    .fetch_all(pool(&ctx))
    .await?
    .into_iter()
    .map(|w| (w.pid, w))
    .collect();
    let slips: HashMap<Uuid, Uuid> = sqlx::query!(
        "SELECT pid, run_pid FROM payslips WHERE pid = ANY($1)",
        &slip_pids,
    )
    .fetch_all(pool(&ctx))
    .await?
    .into_iter()
    .map(|s| (s.pid, s.run_pid))
    .collect();
    let run_pids: Vec<Uuid> = slips.values().copied().collect();
    let runs: HashMap<Uuid, _> = sqlx::query!(
        "SELECT pid, period_start, period_end FROM payroll_runs WHERE pid = ANY($1)",
        &run_pids,
    )
    .fetch_all(pool(&ctx))
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
            .and_then(|run_pid| runs.get(run_pid));
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
    db::audit(
        &mut *pool(&ctx).acquire().await?,
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
    let row = sqlx::query!(
        "SELECT worker_pid, closed_on FROM payslip_queries WHERE pid = $1 AND deleted_at IS NULL",
        pid,
    )
    .fetch_optional(pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let organization = sqlx::query_scalar!(
        "SELECT organization_ref FROM workers WHERE pid = $1 AND deleted_at IS NULL",
        row.worker_pid,
    )
    .fetch_optional(pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    super::ensure_org_in_scope(&ctx, &caller, &organization).await?;
    if row.closed_on.is_some() {
        return Err(unprocessable("this query is already closed"));
    }
    let today = Utc::now().date_naive();
    let mut tx = pool(&ctx).begin().await?;
    let saved = sqlx::query_as!(
        payslip_queries::Model,
        r#"UPDATE payslip_queries SET closed_on = $2, closed_by = $3 WHERE pid = $1
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, payslip_pid, worker_pid, comment, raised_on, closed_on, closed_by, deleted_at AS "deleted_at: _""#,
        pid,
        today,
        caller.actor(),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "payslip_query",
        pid,
        "payslip_query_closed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(query_view(&saved))
}

/// The routes payroll uses; a person's are registered in [`super::me`], with the rest of `/api/me`.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/payslip-queries", get(list_queries))
        .add("/payslip-queries/{pid}/close", post(close_query))
}
