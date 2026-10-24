//! The human capital report (WPM-R147, WPM-D100): `GET /api/reports/human-capital?organization=&as_of=`. One dated,
//! read-only report of figures WPM already works out, each with its definition, source, period and exclusions, laid
//! beside ISO 30414 and ESRS S1 by name. Aggregates only; small groups withheld; what WPM cannot give is `unknown`.
//! Nothing is stored. Spec: `spec/human-capital-reporting.md`.

use chrono::{Months, NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{DatabaseBackend, FromQueryResult, Statement};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::audit_logs::Model as Audit;
use crate::rules::equality_monitoring::DEFAULT_FLOOR;
use crate::rules::human_capital::{self as rules, Inputs, Plan, TypeCount};
use crate::rules::metrics::time_to_fill_days;

#[derive(Debug, Deserialize)]
pub(crate) struct ReportQuery {
    organization: String,
    as_of: Option<NaiveDate>,
}

#[derive(Debug, FromQueryResult)]
struct TypeRow {
    employment_type: String,
    workers: i64,
    fte_percent_total: i64,
}

#[derive(Debug, FromQueryResult)]
struct Counts {
    opening: i64,
    closing: i64,
    starters: i64,
    leavers: i64,
}

#[derive(Debug, FromQueryResult)]
struct Training {
    enrolments: i64,
    completed: i64,
}

#[derive(Debug, FromQueryResult)]
struct PlanRow {
    criticality: i32,
    ready_now: bool,
}

#[derive(Debug, FromQueryResult)]
struct Unplaced {
    unplaced: i64,
}

#[derive(Debug, FromQueryResult)]
struct FillRow {
    opened_on: Option<NaiveDate>,
    filled_on: Option<NaiveDate>,
}

fn n(v: i64) -> usize {
    usize::try_from(v).unwrap_or(0)
}

fn stmt(sql: &str, values: Vec<sea_orm::Value>) -> Statement {
    Statement::from_sql_and_values(DatabaseBackend::Postgres, sql, values)
}

/// Read what the report is made of, for one organization.
async fn read(ctx: &AppContext, organization: &str, as_of: NaiveDate) -> Result<Inputs> {
    let from = as_of
        .checked_sub_months(Months::new(12))
        .and_then(|d| d.succ_opt())
        .unwrap_or(as_of);
    let opening_date = from.pred_opt().unwrap_or(from);
    let employed = "w.deleted_at IS NULL AND w.organization_ref = $1 AND w.hired_on <= $2 AND (w.terminated_on IS NULL OR w.terminated_on > $2)";
    let by_type = TypeRow::find_by_statement(stmt(
        &format!("SELECT w.employment_type, count(*) AS workers, COALESCE(sum(w.fte_percent), 0)::bigint AS fte_percent_total FROM workers w WHERE {employed} GROUP BY w.employment_type"),
        vec![organization.into(), as_of.into()],
    ))
    .all(&ctx.db)
    .await?;
    let counts = Counts::find_by_statement(stmt(
        "SELECT count(*) FILTER (WHERE hired_on <= $2 AND (terminated_on IS NULL OR terminated_on > $2)) AS opening,
                count(*) FILTER (WHERE hired_on <= $3 AND (terminated_on IS NULL OR terminated_on > $3)) AS closing,
                count(*) FILTER (WHERE hired_on >= $4 AND hired_on <= $3) AS starters,
                count(*) FILTER (WHERE terminated_on >= $4 AND terminated_on <= $3) AS leavers
           FROM workers WHERE deleted_at IS NULL AND organization_ref = $1",
        vec![
            organization.into(),
            opening_date.into(),
            as_of.into(),
            from.into(),
        ],
    ))
    .one(&ctx.db)
    .await?
    .ok_or_else(|| super::internal("the headcount query returned no row"))?;
    let fill_days = FillRow::find_by_statement(stmt(
        "SELECT opened_on, filled_on FROM requisitions WHERE deleted_at IS NULL AND organization_ref = $1 AND filled_on >= $2 AND filled_on <= $3",
        vec![organization.into(), from.into(), as_of.into()],
    ))
    .all(&ctx.db)
    .await?
    .iter()
    .filter_map(|r| time_to_fill_days(r.opened_on, r.filled_on))
    .collect();
    let training = Training::find_by_statement(stmt(
        &format!("SELECT count(*) AS enrolments, count(*) FILTER (WHERE e.status = 'completed' AND e.completed_on <= $2) AS completed
                    FROM training_enrollments e JOIN workers w ON w.pid = e.worker_pid
                   WHERE e.deleted_at IS NULL AND {employed}"),
        vec![organization.into(), as_of.into()],
    ))
    .one(&ctx.db)
    .await?
    .ok_or_else(|| super::internal("the training query returned no row"))?;
    let plans = PlanRow::find_by_statement(stmt(
        &format!("SELECT p.criticality,
                         EXISTS (SELECT 1 FROM succession_candidates c WHERE c.plan_pid = p.pid AND c.deleted_at IS NULL AND c.readiness = 'ready_now') AS ready_now
                    FROM succession_plans p JOIN workers w ON w.pid = p.incumbent_pid
                   WHERE p.deleted_at IS NULL AND {employed}"),
        vec![organization.into(), as_of.into()],
    ))
    .all(&ctx.db)
    .await?;
    // Plans that name nobody employed here cannot be placed in this organization, whoever they belong to.
    let unplaced = Unplaced::find_by_statement(stmt(
        &format!("SELECT count(*) AS unplaced FROM succession_plans p
                   WHERE p.deleted_at IS NULL AND NOT EXISTS (SELECT 1 FROM workers w WHERE w.pid = p.incumbent_pid AND {employed})"),
        vec![organization.into(), as_of.into()],
    ))
    .one(&ctx.db)
    .await?
    .ok_or_else(|| super::internal("the succession query returned no row"))?;
    Ok(Inputs {
        as_of,
        from,
        by_type: by_type
            .into_iter()
            .map(|t| TypeCount {
                employment_type: t.employment_type,
                workers: n(t.workers),
                fte_percent_total: t.fte_percent_total,
            })
            .collect(),
        opening: n(counts.opening),
        closing: n(counts.closing),
        starters: n(counts.starters),
        leavers: n(counts.leavers),
        fill_days,
        enrolments: n(training.enrolments),
        completed: n(training.completed),
        plans: plans
            .into_iter()
            .map(|p| Plan {
                criticality: p.criticality,
                covered_now: p.ready_now,
            })
            .collect(),
        plans_unplaced: n(unplaced.unplaced),
    })
}

/// `GET /api/reports/human-capital?organization=&as_of=` — the report. HR only; audited.
#[debug_handler]
async fn human_capital(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ReportQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let as_of = query.as_of.unwrap_or_else(|| Utc::now().date_naive());
    if as_of > Utc::now().date_naive() {
        return Err(unprocessable("as_of must not be in the future"));
    }
    let inputs = read(&ctx, &query.organization, as_of).await?;
    let report = rules::report(&inputs, DEFAULT_FLOOR);
    Audit::record(
        &ctx.db,
        "human_capital_report",
        Uuid::nil(),
        "human_capital_report_read",
        caller.actor(),
        Some(json!({ "as_of": as_of })),
    )
    .await?;
    format::json(json!({
        "organization": query.organization,
        "report": report,
        "mapping_note": "A mapping by name, not a disclosure. ISO 30414 areas are the 2018 edition's wording and were not compared with the 2025 edition. ESRS requirement numbers are given for both the 2023 set and the revised 2026/1563 set. WPM holds no text of either standard.",
    }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/reports/human-capital", get(human_capital))
}
