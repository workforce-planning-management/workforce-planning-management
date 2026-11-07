//! The human capital report (WPM-R147, WPM-D100): `GET /api/reports/human-capital?organization=&as_of=`. One dated,
//! read-only report of figures WPM already works out, each with its definition, source, period and exclusions, laid
//! beside ISO 30414 and ESRS S1 by name. Aggregates only; small groups withheld; what WPM cannot give is `unknown`.
//! Nothing is stored. Spec: `spec/human-capital-reporting.md`.

use axa::prelude::*;
use chrono::{Months, NaiveDate, Utc};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::payroll::check_minimum_wage;
use super::{ensure_org_in_scope, ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db::{self, pool};
use crate::models::_entities::payroll_runs;
use crate::rules::equality_monitoring::DEFAULT_FLOOR;
use crate::rules::human_capital::{self as rules, Inputs, Plan, TypeCount, WageRun};
use crate::rules::metrics::time_to_fill_days;

#[derive(Debug, Deserialize)]
pub(crate) struct ReportQuery {
    organization: String,
    as_of: Option<NaiveDate>,
}

fn n(v: i64) -> usize {
    usize::try_from(v).unwrap_or(0)
}

/// Read what the report is made of, for one organization.
async fn read(ctx: &AppContext, organization: &str, as_of: NaiveDate) -> Result<Inputs> {
    let from = as_of
        .checked_sub_months(Months::new(12))
        .and_then(|d| d.succ_opt())
        .unwrap_or(as_of);
    let opening_date = from.pred_opt().unwrap_or(from);
    // "Employed on the date" is written out in each statement, because each is checked as it stands.
    let by_type = sqlx::query!(
        r#"SELECT w.employment_type,
                  count(*) AS "workers!",
                  COALESCE(sum(w.fte_percent), 0)::bigint AS "fte_percent_total!"
           FROM workers w
           WHERE w.deleted_at IS NULL AND w.organization_ref = $1 AND w.hired_on <= $2
             AND (w.terminated_on IS NULL OR w.terminated_on > $2)
           GROUP BY w.employment_type"#,
        organization,
        as_of,
    )
    .fetch_all(pool(ctx))
    .await?;
    let counts = sqlx::query!(
        r#"SELECT
             count(*) FILTER (WHERE hired_on <= $2 AND (terminated_on IS NULL OR terminated_on > $2)) AS "opening!",
             count(*) FILTER (WHERE hired_on <= $3 AND (terminated_on IS NULL OR terminated_on > $3)) AS "closing!",
             count(*) FILTER (WHERE hired_on >= $4 AND hired_on <= $3) AS "starters!",
             count(*) FILTER (WHERE terminated_on >= $4 AND terminated_on <= $3) AS "leavers!"
           FROM workers WHERE deleted_at IS NULL AND organization_ref = $1"#,
        organization,
        opening_date,
        as_of,
        from,
    )
    .fetch_one(pool(ctx))
    .await?;
    let fill_days = sqlx::query!(
        r#"SELECT opened_on AS "opened_on?", filled_on AS "filled_on?"
           FROM requisitions
           WHERE deleted_at IS NULL AND organization_ref = $1 AND filled_on >= $2 AND filled_on <= $3"#,
        organization,
        from,
        as_of,
    )
    .fetch_all(pool(ctx))
    .await?
    .iter()
    .filter_map(|r| time_to_fill_days(r.opened_on, r.filled_on))
    .collect();
    let training = sqlx::query!(
        r#"SELECT count(*) AS "enrolments!",
                  count(*) FILTER (WHERE e.status = 'completed' AND e.completed_on <= $2) AS "completed!"
           FROM training_enrollments e JOIN workers w ON w.pid = e.worker_pid
           WHERE e.deleted_at IS NULL
             AND w.deleted_at IS NULL AND w.organization_ref = $1 AND w.hired_on <= $2
             AND (w.terminated_on IS NULL OR w.terminated_on > $2)"#,
        organization,
        as_of,
    )
    .fetch_one(pool(ctx))
    .await?;
    let (plans, plans_unplaced) = succession(ctx, organization, as_of).await?;
    let (injury_classes, injuries_ever) = injuries(ctx, organization, from, as_of).await?;
    let wage_runs = wage_runs(ctx, organization, from, as_of).await?;
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
        plans,
        plans_unplaced,
        injury_classes,
        injuries_ever,
        wage_runs,
    })
}

/// The succession plans for people employed here, and how many plans name nobody employed here.
async fn succession(
    ctx: &AppContext,
    organization: &str,
    as_of: NaiveDate,
) -> Result<(Vec<Plan>, usize)> {
    let plans = sqlx::query!(
        r#"SELECT p.criticality,
                  EXISTS (
                      SELECT 1 FROM succession_candidates c
                      WHERE c.plan_pid = p.pid AND c.deleted_at IS NULL AND c.readiness = 'ready_now'
                  ) AS "ready_now!"
           FROM succession_plans p JOIN workers w ON w.pid = p.incumbent_pid
           WHERE p.deleted_at IS NULL
             AND w.deleted_at IS NULL AND w.organization_ref = $1 AND w.hired_on <= $2
             AND (w.terminated_on IS NULL OR w.terminated_on > $2)"#,
        organization,
        as_of,
    )
    .fetch_all(pool(ctx))
    .await?;
    // Plans that name nobody employed here cannot be placed in this organization, whoever they belong to.
    let unplaced = sqlx::query_scalar!(
        r#"SELECT count(*) AS "unplaced!" FROM succession_plans p
           WHERE p.deleted_at IS NULL AND NOT EXISTS (
               SELECT 1 FROM workers w
               WHERE w.pid = p.incumbent_pid
                 AND w.deleted_at IS NULL AND w.organization_ref = $1 AND w.hired_on <= $2
                 AND (w.terminated_on IS NULL OR w.terminated_on > $2)
           )"#,
        organization,
        as_of,
    )
    .fetch_one(pool(ctx))
    .await?;
    let plans = plans
        .into_iter()
        .map(|p| Plan {
            criticality: p.criticality,
            covered_now: p.ready_now,
        })
        .collect();
    Ok((plans, n(unplaced)))
}

/// The class of each incident in the period, and how many this organization has ever recorded.
async fn injuries(
    ctx: &AppContext,
    organization: &str,
    from: NaiveDate,
    as_of: NaiveDate,
) -> Result<(Vec<String>, usize)> {
    let rows = sqlx::query!(
        r#"SELECT class, (occurred_at AT TIME ZONE 'UTC')::date BETWEEN $2 AND $3 AS "in_period!"
           FROM injury_records WHERE organization_ref = $1"#,
        organization,
        from,
        as_of,
    )
    .fetch_all(pool(ctx))
    .await?;
    let ever = rows.len();
    let classes = rows
        .into_iter()
        .filter(|i| i.in_period)
        .map(|i| i.class)
        .collect();
    Ok((classes, ever))
}

/// The minimum-wage outcomes of each approved or paid run ending in the period, as counts. Each person's birth date is read at most
/// once here, only where a profile has age bands, and goes no further than the band (WPM-D114, WPM-D156).
async fn wage_runs(
    ctx: &AppContext,
    organization: &str,
    from: NaiveDate,
    as_of: NaiveDate,
) -> Result<Vec<WageRun>> {
    let runs = sqlx::query_as!(
        payroll_runs::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, organization_ref, period_start, period_end, status, approved_by, notes,
                  deleted_at AS "deleted_at: _"
           FROM payroll_runs
           WHERE organization_ref = $1
             AND deleted_at IS NULL
             AND status IN ('approved', 'paid')
             AND period_end >= $2
             AND period_end <= $3"#,
        organization,
        from,
        as_of,
    )
    .fetch_all(pool(ctx))
    .await?;
    let mut known_births = BTreeMap::new();
    let mut out = Vec::with_capacity(runs.len());
    for run in &runs {
        let check = check_minimum_wage(ctx, run, &mut known_births).await?;
        let of = |outcome: &str| {
            usize::try_from(check.counts.get(outcome).copied().unwrap_or(0)).unwrap_or(usize::MAX)
        };
        out.push(WageRun {
            below: of("below"),
            near: of("near"),
            ok: of("ok"),
            none_by_law: of("none_by_law"),
            unknown: of("unknown"),
        });
    }
    Ok(out)
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
    db::audit(
        &mut *pool(&ctx).acquire().await?,
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
