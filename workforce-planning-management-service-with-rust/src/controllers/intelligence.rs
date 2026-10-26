//! **Workforce intelligence** (WPM-R24) — the read-only analytical
//! layer over everything WPM records: headcount and shape, capability
//! and capability gaps, succession bench strength, and the talent
//! pipeline / early-career funnel.
//!
//! Four principles hold across every view here, because an analytics
//! surface is exactly where numbers stop being checkable:
//!
//! 1. **Every rate carries its terms.** A ratio is always
//!    `{numerator, denominator, value}` ([`crate::rules::talent::ratio`]),
//!    and is `null` — never `0` — when the denominator is zero.
//! 2. **Nothing is imputed.** A missing declaration is reported as
//!    missing (`not_assessed`, `undeclared`), never as a zero that
//!    reads like a measurement.
//! 3. **Every payload names its derivation** in a `derivation` field,
//!    so a consumer cannot mistake a proxy for the thing itself.
//! 4. **No individual's sensitive data appears.** These are aggregate
//!    counts: no salary, no assessment score, no review content. The
//!    one place an individual is named is the succession
//!    single-point-of-failure list, which names *roles* and their
//!    incumbent worker pid — the same information the succession
//!    endpoints already return under their own audit.
//!
//! Aggregates were computed in-process from the entity rows so the derivations stay in reviewable Rust
//! next to their tests. Where the load test showed that reading every worker fails at scale (the overview
//! and the period metrics, WPM-D90), the same counts are made by the database, with the definitions
//! (`is_employed_on`, starters, leavers) written once in SQL and a request test that pins the answers.

use super::ratio_json;
use loco_rs::prelude::*;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, DatabaseBackend, EntityTrait, PaginatorTrait,
    QueryFilter, QuerySelect, Select, Statement,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    early_career_programs, headcount_snapshots, pipeline_members, program_placements, requisitions,
    skills, succession_candidates, succession_plans, talent_pipelines, workers,
};
use crate::models::memberships;
use crate::rules::assessment as assessment_rules;
use crate::rules::capability as capability_rules;
use crate::rules::insights as insight_rules;
use crate::rules::metrics as metric_rules;
use crate::rules::talent as rules;

/// Query accepted by every intelligence view.
#[derive(Debug, Deserialize)]
struct AsOfQuery {
    /// The date the view is computed against (default today).
    as_of: Option<chrono::NaiveDate>,
}

/// `GET /api/workforce-intelligence/overview?as_of=` — the shape of the
/// workforce: headcount by department, status, and employment type;
/// total FTE; tenure distribution; and manager spans of control.
///
/// FTE is summed from the declared `fte_percent`, and reported as
/// hundredths so no float rounds a headcount.
#[debug_handler]
async fn overview(
    axum::extract::Query(query): axum::extract::Query<AsOfQuery>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let as_of = query
        .as_of
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    // Counts are worked out by the database, one small grouped query each: the dashboard asks for this
    // on every load, and reading every worker into memory made it fail at 50,000 (WPM-D90).
    let headcount = employed_on(as_of).count(&ctx.db).await?;
    let by_department = grouped(&ctx, as_of, workers::Column::Department).await?;
    let by_status = grouped(&ctx, as_of, workers::Column::Status).await?;
    let by_employment_type = grouped(&ctx, as_of, workers::Column::EmploymentType).await?;
    let hire_dates: Vec<(chrono::NaiveDate, i64)> = employed_on(as_of)
        .select_only()
        .column(workers::Column::HiredOn)
        .column_as(Expr::cust("count(*)"), "n")
        .group_by(workers::Column::HiredOn)
        .into_tuple()
        .all(&ctx.db)
        .await?;
    let mut by_tenure: BTreeMap<&str, usize> = BTreeMap::new();
    for (hired_on, n) in hire_dates {
        let months = rules::months_of_service(hired_on, as_of);
        *by_tenure.entry(rules::tenure_bucket(months)).or_default() +=
            usize::try_from(n).unwrap_or(0);
    }
    let (fte_percent_total,): (i64,) = employed_on(as_of)
        .select_only()
        .column_as(Expr::cust("COALESCE(SUM(fte_percent), 0)::bigint"), "total")
        .into_tuple()
        .one(&ctx.db)
        .await?
        .unwrap_or((0,));
    let managers: Vec<(Uuid, i64)> = employed_on(as_of)
        .select_only()
        .column(workers::Column::ManagerPid)
        .column_as(Expr::cust("count(*)"), "n")
        .filter(workers::Column::ManagerPid.is_not_null())
        .group_by(workers::Column::ManagerPid)
        .into_tuple()
        .all(&ctx.db)
        .await?;
    // a manager is named only if they are employed on the date too
    let names: BTreeMap<Uuid, String> = employed_on(as_of)
        .select_only()
        .column(workers::Column::Pid)
        .column(workers::Column::DisplayName)
        .filter(workers::Column::Pid.is_in(managers.iter().map(|(m, _)| *m)))
        .into_tuple::<(Uuid, String)>()
        .all(&ctx.db)
        .await?
        .into_iter()
        .collect();
    let mut spans: Vec<serde_json::Value> = managers
        .iter()
        .map(|(manager, reports)| {
            serde_json::json!({
                "manager_pid": manager,
                "display_name": names.get(manager),
                "direct_reports": reports,
            })
        })
        .collect();
    spans.sort_by_key(|s| std::cmp::Reverse(s["direct_reports"].as_u64().unwrap_or(0)));

    format::json(serde_json::json!({
        "as_of": as_of,
        "derivation": "headcount counts workers EMPLOYED on as_of (hired on or before it, not \
                       terminated on or before it; soft-deleted records excluded) — the same \
                       definition as /metrics; FTE is the \
                       sum of declared fte_percent (in hundredths of a person); tenure is whole \
                       completed months since hired_on",
        "headcount": headcount,
        "fte_percent_total": fte_percent_total,
        "by_department": by_department,
        "by_status": by_status,
        "by_employment_type": by_employment_type,
        "by_tenure": by_tenure,
        "spans_of_control": spans,
    }))
}

/// `GET /api/workforce-intelligence/capability` — what the workforce
/// can do and where it cannot: declared skill coverage, declared gaps
/// (proficiency below a declared target), the development plans in
/// flight against those gaps, and assessment coverage.
///
/// "Coverage" is deliberately narrow: it counts **declarations**, not
/// ability. A skill nobody has declared is reported as undeclared, not
/// as absent.
#[debug_handler]
#[allow(clippy::too_many_lines)] // one pass over skills, plans, and sittings
async fn capability(State(ctx): State<AppContext>) -> Result<Response> {
    let today = chrono::Utc::now().date_naive();
    let headcount = employed_count(&ctx, today).await?;

    let skill_rows = skills::Entity::find()
        .filter(skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let skill_name: BTreeMap<Uuid, &str> = skill_rows
        .iter()
        .map(|s| (s.pid, s.name.as_str()))
        .collect();

    // skill → (holders, at-or-above-target, below-target), counted by the database over employed people
    // (a declaration from a departed worker is not counted).
    let mut per_skill: BTreeMap<Uuid, (usize, usize, usize)> = BTreeMap::new();
    for row in rows_on(
        &ctx,
        &format!(
            "SELECT ws.skill_pid AS skill_pid, COUNT(*)::bigint AS holders,
                    (COUNT(*) FILTER (WHERE ws.target IS NOT NULL AND ws.proficiency >= ws.target))::bigint AS at_target,
                    (COUNT(*) FILTER (WHERE ws.target IS NOT NULL AND ws.proficiency < ws.target))::bigint AS below
               FROM worker_skills ws JOIN ({EMPLOYED_SQL}) e ON e.pid = ws.worker_pid
              WHERE ws.deleted_at IS NULL GROUP BY ws.skill_pid"
        ),
        today,
    )
    .await?
    {
        let count = |name: &str| -> Result<usize> {
            Ok(usize::try_from(row.try_get::<i64>("", name)?).unwrap_or(0))
        };
        per_skill.insert(
            row.try_get("", "skill_pid")?,
            (count("holders")?, count("at_target")?, count("below")?),
        );
    }
    // Employed people with a declared gap, and how many of them have an active development plan.
    let gap_counts = rows_on(
        &ctx,
        &format!(
            "SELECT COUNT(DISTINCT ws.worker_pid)::bigint AS with_gap,
                    (COUNT(DISTINCT ws.worker_pid) FILTER (WHERE EXISTS (
                        SELECT 1 FROM development_plans dp
                         WHERE dp.worker_pid = ws.worker_pid AND dp.deleted_at IS NULL
                           AND dp.status = 'active')))::bigint AS covered
               FROM worker_skills ws JOIN ({EMPLOYED_SQL}) e ON e.pid = ws.worker_pid
              WHERE ws.deleted_at IS NULL AND ws.target IS NOT NULL AND ws.proficiency < ws.target"
        ),
        today,
    )
    .await?;
    let count_of = |name: &str| -> Result<usize> {
        let n: i64 = gap_counts.first().map_or(Ok(0), |r| r.try_get("", name))?;
        Ok(usize::try_from(n).unwrap_or(0))
    };
    let (with_gap, gap_covered) = (count_of("with_gap")?, count_of("covered")?);

    let coverage: Vec<serde_json::Value> = skill_rows
        .iter()
        .map(|skill| {
            let (holders, at_target, below_target) =
                per_skill.get(&skill.pid).copied().unwrap_or((0, 0, 0));
            serde_json::json!({
                "skill": skill.name,
                "category": skill.category,
                "declared_by": holders,
                "coverage": ratio_json(rules::ratio(holders, headcount)),
                "at_or_above_target": at_target,
                "below_target": below_target,
                "undeclared": holders == 0,
            })
        })
        .collect();

    // Development plans in flight, by kind and status.
    let mut plans_by_kind: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    for row in rows_on(
        &ctx,
        "SELECT kind, status, COUNT(*)::bigint AS total FROM development_plans
          WHERE deleted_at IS NULL AND $1::date IS NOT NULL GROUP BY kind, status",
        today,
    )
    .await?
    {
        plans_by_kind
            .entry(row.try_get("", "kind")?)
            .or_default()
            .insert(
                row.try_get("", "status")?,
                usize::try_from(row.try_get::<i64>("", "total")?).unwrap_or(0),
            );
    }

    // Assessment coverage: workers with at least one completed sitting, per instrument category.
    let mut assessed_by_category: BTreeMap<String, usize> = BTreeMap::new();
    for row in rows_on(
        &ctx,
        "SELECT i.category AS category, COUNT(DISTINCT a.subject_pid)::bigint AS assessed
           FROM assessments a JOIN assessment_instruments i ON i.pid = a.instrument_pid
          WHERE a.deleted_at IS NULL AND a.subject_kind = 'worker' AND a.status = 'completed'
            AND $1::date IS NOT NULL
          GROUP BY i.category",
        today,
    )
    .await?
    {
        assessed_by_category.insert(
            row.try_get("", "category")?,
            usize::try_from(row.try_get::<i64>("", "assessed")?).unwrap_or(0),
        );
    }
    let assessment_coverage: Vec<serde_json::Value> = assessment_rules::ASSESSMENT_CATEGORIES
        .iter()
        .map(|category| {
            let assessed = assessed_by_category.get(*category).copied().unwrap_or(0);
            serde_json::json!({
                "category": category,
                "workers_assessed": assessed,
                "coverage": ratio_json(rules::ratio(assessed, headcount)),
            })
        })
        .collect();

    format::json(serde_json::json!({
        "derivation": "coverage counts DECLARED skills over live headcount — a skill nobody \
                       declared is `undeclared`, not absent; a gap is a declared proficiency \
                       below a declared target; assessment coverage counts workers with at \
                       least one COMPLETED sitting in that category",
        "headcount": headcount,
        "skills": coverage,
        "people_with_a_declared_gap": with_gap,
        "gaps_with_an_active_plan": ratio_json(rules::ratio(gap_covered, with_gap)),
        "development_plans_by_kind": plans_by_kind,
        "assessment_coverage": assessment_coverage,
        "skill_catalog_size": skill_name.len(),
    }))
}

/// Query for the capability-analysis view.
#[derive(Debug, Deserialize)]
struct CapabilityAnalysisQuery {
    /// Proficiency (1–5) at which a declaration counts as working
    /// capability. Default 3.
    min_proficiency: Option<i32>,
    /// Workers at that proficiency a skill needs to not depend on one
    /// person. Default 2.
    min_depth: Option<usize>,
}

/// `GET /api/workforce-intelligence/capability-analysis?min_proficiency=&min_depth=`
/// — the strategic read of declared skills: for every catalogue skill,
/// how many workers hold it at working proficiency, in how many
/// departments, and whether that depth is `adequate`, `thin`,
/// `no_proficient`, or `undeclared`; rolled up by category.
///
/// Read-only and derived — no new stored state — over the same
/// declarations as `/capability`. The thresholds are the caller's to
/// set, and are echoed back so a figure is never read without them.
#[debug_handler]
#[allow(clippy::too_many_lines)] // one pass over skills and declarations
async fn capability_analysis(
    axum::extract::Query(query): axum::extract::Query<CapabilityAnalysisQuery>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let min_proficiency = query
        .min_proficiency
        .unwrap_or(capability_rules::DEFAULT_MIN_PROFICIENCY);
    let min_depth = query
        .min_depth
        .unwrap_or(capability_rules::DEFAULT_MIN_DEPTH);
    capability_rules::validate_thresholds(min_proficiency, min_depth).map_err(|e| match e {
        capability_rules::ThresholdError::Proficiency => {
            super::unprocessable("min_proficiency must be 1–5")
        }
        capability_rules::ThresholdError::Depth => {
            super::unprocessable("min_depth must be at least 1")
        }
    })?;

    let today = chrono::Utc::now().date_naive();
    let headcount = employed_count(&ctx, today).await?;
    let skill_rows = skills::Entity::find()
        .filter(skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;

    // skill → (declared, proficient, departments of the proficient), counted by the database over employed people
    // (a declaration from a departed worker is not counted).
    let mut per_skill: BTreeMap<Uuid, (usize, usize, usize)> = BTreeMap::new();
    for row in ctx
        .db
        .query_all_raw(sea_orm::Statement::from_sql_and_values(
            sea_orm::DbBackend::Postgres,
            format!(
                "SELECT ws.skill_pid AS skill_pid, COUNT(*)::bigint AS declared,
                        (COUNT(*) FILTER (WHERE ws.proficiency >= $2))::bigint AS proficient,
                        COUNT(DISTINCT e.department) FILTER (WHERE ws.proficiency >= $2) AS departments
                   FROM worker_skills ws JOIN ({EMPLOYED_SQL}) e ON e.pid = ws.worker_pid
                  WHERE ws.deleted_at IS NULL GROUP BY ws.skill_pid"
            ),
            [today.into(), min_proficiency.into()],
        ))
        .await?
    {
        let count = |name: &str| -> Result<usize> {
            Ok(usize::try_from(row.try_get::<i64>("", name)?).unwrap_or(0))
        };
        per_skill.insert(
            row.try_get("", "skill_pid")?,
            (count("declared")?, count("proficient")?, count("departments")?),
        );
    }

    // category → status → count
    let mut by_category: BTreeMap<String, BTreeMap<&str, usize>> = BTreeMap::new();
    let mut skills_out = Vec::with_capacity(skill_rows.len());
    for skill in &skill_rows {
        let (declared_by, proficient, departments) =
            per_skill.get(&skill.pid).copied().unwrap_or((0, 0, 0));
        let status = capability_rules::depth_status(declared_by, proficient, min_depth);
        *by_category
            .entry(skill.category.clone())
            .or_default()
            .entry(status)
            .or_default() += 1;
        skills_out.push(serde_json::json!({
            "skill": skill.name,
            "category": skill.category,
            "declared_by": declared_by,
            "proficient": proficient,
            "proficient_departments": departments,
            "proficient_share": ratio_json(rules::ratio(proficient, headcount)),
            "status": status,
        }));
    }
    let adequate = skills_out
        .iter()
        .filter(|s| s["status"] == "adequate")
        .count();

    format::json(serde_json::json!({
        "derivation": "counts DECLARED proficiency at or above min_proficiency over live \
                       headcount; `undeclared` means nobody declared the skill (unknown, not \
                       absent), `no_proficient` means declared but nobody at the bar, `thin` \
                       means fewer than min_depth proficient workers",
        "thresholds": { "min_proficiency": min_proficiency, "min_depth": min_depth },
        "headcount": headcount,
        "skills_in_catalog": skills_out.len(),
        "adequately_covered": ratio_json(rules::ratio(adequate, skills_out.len())),
        "by_category": by_category,
        "skills": skills_out,
    }))
}

/// Query for the metrics view.
#[derive(Debug, Deserialize)]
struct MetricsQuery {
    /// Period start (default: one year before `to`).
    from: Option<chrono::NaiveDate>,
    /// Period end, inclusive (default today).
    to: Option<chrono::NaiveDate>,
}

/// The shared metrics for one period, computed once for both the
/// `metrics` and `insights` views.
struct PeriodMetrics {
    from: chrono::NaiveDate,
    to: chrono::NaiveDate,
    opening_date: chrono::NaiveDate,
    opening: usize,
    closing: usize,
    starters: usize,
    leavers: usize,
    turnover_rate: Option<f64>,
    span: Option<metric_rules::Span>,
    fill_time: Option<metric_rules::FillTime>,
}

/// Resolve the period (default: the year ending today) and compute the
/// shared metrics over it.
async fn period_metrics(query: MetricsQuery, ctx: &AppContext) -> Result<PeriodMetrics> {
    let to = query.to.unwrap_or_else(|| chrono::Utc::now().date_naive());
    let from = query
        .from
        .unwrap_or_else(|| to.checked_sub_months(chrono::Months::new(12)).unwrap_or(to));
    if from > to {
        return Err(super::unprocessable("from must not be after to"));
    }

    // Opening and closing headcount, starters and leavers: one pass in the database over live workers.
    let opening_date = from.pred_opt().unwrap_or(from);
    let counts_row = ctx
        .db
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT count(*) FILTER (WHERE hired_on <= $1 AND (terminated_on IS NULL OR terminated_on > $1)) AS opening,
                    count(*) FILTER (WHERE hired_on <= $2 AND (terminated_on IS NULL OR terminated_on > $2)) AS closing,
                    count(*) FILTER (WHERE hired_on >= $3 AND hired_on <= $2) AS starters,
                    count(*) FILTER (WHERE terminated_on >= $3 AND terminated_on <= $2) AS leavers
               FROM workers WHERE deleted_at IS NULL",
            [opening_date.into(), to.into(), from.into()],
        ))
        .await?
        .ok_or_else(|| super::internal("the headcount query returned no row"))?;
    let count = |name: &str| -> Result<usize> {
        let n: i64 = counts_row.try_get("", name)?;
        Ok(usize::try_from(n).unwrap_or(0))
    };
    let (opening, closing, starters, leavers) = (
        count("opening")?,
        count("closing")?,
        count("starters")?,
        count("leavers")?,
    );

    // Span of control over managers of workers employed on `to`: one row per manager.
    let reports: Vec<(i64,)> = employed_on(to)
        .select_only()
        .column_as(Expr::cust("count(*)"), "n")
        .filter(workers::Column::ManagerPid.is_not_null())
        .group_by(workers::Column::ManagerPid)
        .into_tuple()
        .all(&ctx.db)
        .await?;
    let counts: Vec<usize> = reports
        .iter()
        .map(|(n,)| usize::try_from(*n).unwrap_or(0))
        .collect();

    // Time-to-fill over requisitions filled within the period.
    let filled_days: Vec<i64> = requisitions::Entity::find()
        .filter(requisitions::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .iter()
        .filter(|r| r.filled_on.is_some_and(|d| d >= from && d <= to))
        .filter_map(|r| metric_rules::time_to_fill_days(r.opened_on, r.filled_on))
        .collect();

    Ok(PeriodMetrics {
        from,
        to,
        opening_date,
        opening,
        closing,
        starters,
        leavers,
        turnover_rate: metric_rules::turnover_rate(leavers, opening, closing),
        span: metric_rules::span_of_control(&counts),
        fill_time: metric_rules::fill_time_summary(&filled_days),
    })
}

/// `GET /api/workforce-intelligence/metrics?from=&to=` — the shared
/// metric vocabulary: headcount (opening and closing), starters,
/// leavers, turnover rate, and span of control, each defined once in
/// [`metric_rules::DEFINITIONS`] and returned beside the numbers.
///
/// Headcount here counts workers *employed on the date* (hire and
/// termination dates both respected); the `overview` view's headcount
/// counts every live record regardless of status — see its derivation.
#[debug_handler]
async fn metrics(
    axum::extract::Query(query): axum::extract::Query<MetricsQuery>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let m = period_metrics(query, &ctx).await?;
    let definitions: BTreeMap<&str, &str> = metric_rules::DEFINITIONS.iter().copied().collect();
    format::json(serde_json::json!({
        "period": { "from": m.from, "to": m.to },
        "definitions": definitions,
        "headcount": { "opening": m.opening, "closing": m.closing, "opening_date": m.opening_date },
        "starters": m.starters,
        "leavers": m.leavers,
        "turnover_rate": m.turnover_rate,
        "span_of_control": m.span.map(|s| serde_json::json!({
            "managers": s.managers, "mean": s.mean, "max": s.max,
        })),
        "time_to_fill": m.fill_time.map(|f| serde_json::json!({
            "requisitions": f.count, "mean_days": f.mean, "median_days": f.median,
        })),
    }))
}

/// `GET /api/workforce-intelligence/insights?from=&to=` — findings
/// derived from the same metrics (WPM-T44) by
/// [`insight_rules::derive`], each with an observation and a suggested
/// next step, plus the heuristic thresholds that triggered them. An
/// empty list means nothing crossed a threshold, not that nothing is
/// known.
#[debug_handler]
async fn insights(
    axum::extract::Query(query): axum::extract::Query<MetricsQuery>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let m = period_metrics(query, &ctx).await?;
    let found = insight_rules::derive(&insight_rules::Inputs {
        opening: m.opening,
        closing: m.closing,
        turnover_rate: m.turnover_rate,
        span_mean: m.span.map(|s| s.mean),
        fill_median_days: m.fill_time.map(|f| f.median),
    });
    let thresholds: BTreeMap<&str, serde_json::Value> = insight_rules::THRESHOLDS
        .iter()
        .map(|(n, v, d)| (*n, serde_json::json!({ "value": v, "meaning": d })))
        .collect();
    format::json(serde_json::json!({
        "period": { "from": m.from, "to": m.to },
        "insights": found,
        "thresholds": thresholds,
    }))
}

/// Query for the headcount-history view.
#[derive(Debug, Deserialize)]
struct HistoryQuery {
    /// Restrict to one organization URN (must be within the caller's scope).
    organization: Option<String>,
    /// First snapshot date (inclusive).
    from: Option<chrono::NaiveDate>,
    /// Last snapshot date (inclusive).
    to: Option<chrono::NaiveDate>,
}

/// `GET /api/workforce-intelligence/headcount-history?organization=&from=&to=`
/// — the recorded headcount snapshots (WPM-R35), oldest first. Aggregate
/// only; scoped to the caller's organizations. An empty list means no
/// snapshot has been taken yet (`cargo loco task snapshot_headcount`),
/// not that headcount was zero.
#[debug_handler]
async fn headcount_history(
    axum::extract::Query(query): axum::extract::Query<HistoryQuery>,
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
) -> Result<Response> {
    use sea_orm::QueryOrder;
    let mut select = headcount_snapshots::Entity::find();
    if let Some(org) = &query.organization {
        select = select.filter(headcount_snapshots::Column::OrganizationRef.eq(org));
    }
    if let Some(from) = query.from {
        select = select.filter(headcount_snapshots::Column::AsOf.gte(from));
    }
    if let Some(to) = query.to {
        select = select.filter(headcount_snapshots::Column::AsOf.lte(to));
    }
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await? {
        select = select.filter(headcount_snapshots::Column::OrganizationRef.is_in(refs));
    }
    let rows = select
        .order_by_asc(headcount_snapshots::Column::AsOf)
        .order_by_asc(headcount_snapshots::Column::OrganizationRef)
        .order_by_asc(headcount_snapshots::Column::Department)
        .all(&ctx.db)
        .await?;
    let snapshots: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "as_of": r.as_of,
                "organization_ref": r.organization_ref,
                "department": r.department,
                "headcount": r.headcount,
                "fte_percent_total": r.fte_percent_total,
                "starters": r.starters,
                "leavers": r.leavers,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "derivation": "one row per organization × department × date, recorded by the \
                       snapshot_headcount task via the shared employed-on-date definition; \
                       starters/leavers cover the window since the previous snapshot and are \
                       null for the first one; FTE is in hundredths of a person",
        "snapshots": snapshots,
    }))
}

/// `GET /api/workforce-intelligence/succession` — bench strength across
/// the succession plans, and the **single points of failure**: critical
/// roles with nobody ready now (a high risk of loss lowers the
/// criticality threshold, since exposure is criticality × risk).
#[debug_handler]
async fn succession(State(ctx): State<AppContext>) -> Result<Response> {
    let plans = succession_plans::Entity::find()
        .filter(succession_plans::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    // Only the plan and the rating are needed, so only those two columns are read, grouped once.
    let mut readiness_of: BTreeMap<Uuid, Vec<String>> = BTreeMap::new();
    for (plan_pid, readiness) in succession_candidates::Entity::find()
        .select_only()
        .column(succession_candidates::Column::PlanPid)
        .column(succession_candidates::Column::Readiness)
        .filter(succession_candidates::Column::DeletedAt.is_null())
        .into_tuple::<(Uuid, String)>()
        .all(&ctx.db)
        .await?
    {
        readiness_of.entry(plan_pid).or_default().push(readiness);
    }

    let mut by_coverage: BTreeMap<&str, usize> = BTreeMap::new();
    let mut per_plan = Vec::with_capacity(plans.len());
    let mut single_points = Vec::new();
    let mut covered_now = 0usize;

    for plan in &plans {
        let readiness: Vec<String> = readiness_of.remove(&plan.pid).unwrap_or_default();
        let coverage = rules::bench_coverage(&readiness);
        *by_coverage.entry(coverage).or_default() += 1;
        if coverage == "covered_now" {
            covered_now += 1;
        }
        let mut by_readiness: BTreeMap<String, usize> = BTreeMap::new();
        for rating in &readiness {
            *by_readiness.entry(rating.clone()).or_default() += 1;
        }
        let spof = rules::is_single_point_of_failure(
            plan.criticality,
            coverage,
            plan.risk_of_loss.as_deref(),
        );
        if spof {
            single_points.push(serde_json::json!({
                "plan_pid": plan.pid,
                "role_title": plan.role_title,
                "department": plan.department,
                "criticality": plan.criticality,
                "risk_of_loss": plan.risk_of_loss,
                "incumbent_pid": plan.incumbent_pid,
                "coverage": coverage,
                "vacancy_expected_on": plan.vacancy_expected_on,
            }));
        }
        per_plan.push(serde_json::json!({
            "plan_pid": plan.pid,
            "role_title": plan.role_title,
            "department": plan.department,
            "criticality": plan.criticality,
            "risk_of_loss": plan.risk_of_loss,
            "bench": by_readiness,
            "bench_size": readiness.len(),
            "coverage": coverage,
            "single_point_of_failure": spof,
        }));
    }

    format::json(serde_json::json!({
        "derivation": "coverage is conservative — `covered_now` requires a ready_now successor; \
                       a single point of failure is an uncovered role with criticality ≥ 4, or \
                       ≥ 3 when the incumbent's risk_of_loss is high",
        "plans": plans.len(),
        "covered_now": ratio_json(rules::ratio(covered_now, plans.len())),
        "by_coverage": by_coverage,
        "single_points_of_failure": single_points,
        "per_plan": per_plan,
    }))
}

/// `GET /api/workforce-intelligence/pipelines` — the talent funnel:
/// each pipeline's health, plus the early-career programmes' placement
/// counts and conversion rates.
#[debug_handler]
async fn pipelines(State(ctx): State<AppContext>) -> Result<Response> {
    let pipelines = talent_pipelines::Entity::find()
        .filter(talent_pipelines::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let mut stages_of: BTreeMap<Uuid, Vec<String>> = BTreeMap::new();
    for (pipeline_pid, stage) in pipeline_members::Entity::find()
        .select_only()
        .column(pipeline_members::Column::PipelinePid)
        .column(pipeline_members::Column::Stage)
        .filter(pipeline_members::Column::DeletedAt.is_null())
        .into_tuple::<(Uuid, String)>()
        .all(&ctx.db)
        .await?
    {
        stages_of.entry(pipeline_pid).or_default().push(stage);
    }

    let mut by_purpose: BTreeMap<String, usize> = BTreeMap::new();
    let mut per_pipeline = Vec::with_capacity(pipelines.len());
    let mut ready_total = 0usize;
    let mut live_total = 0usize;
    for pipeline in &pipelines {
        *by_purpose.entry(pipeline.purpose.clone()).or_default() += 1;
        let stages: Vec<String> = stages_of.remove(&pipeline.pid).unwrap_or_default();
        let health = rules::pipeline_health(&stages);
        ready_total += health.ready;
        live_total += health.live;
        per_pipeline.push(serde_json::json!({
            "pid": pipeline.pid,
            "name": pipeline.name,
            "purpose": pipeline.purpose,
            "target_job_title": pipeline.target_job_title,
            "target_department": pipeline.target_department,
            "health": health,
        }));
    }

    let programs = early_career_programs::Entity::find()
        .filter(early_career_programs::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let mut placements_of: BTreeMap<Uuid, Vec<(String, String)>> = BTreeMap::new();
    for (program_pid, status, outcome) in program_placements::Entity::find()
        .select_only()
        .column(program_placements::Column::ProgramPid)
        .column(program_placements::Column::Status)
        .column(program_placements::Column::Outcome)
        .filter(program_placements::Column::DeletedAt.is_null())
        .into_tuple::<(Uuid, String, String)>()
        .all(&ctx.db)
        .await?
    {
        placements_of
            .entry(program_pid)
            .or_default()
            .push((status, outcome));
    }

    let mut per_kind: BTreeMap<&str, (usize, Vec<String>, usize)> = BTreeMap::new();
    for program in &programs {
        let mine: Vec<(String, String)> = placements_of.remove(&program.pid).unwrap_or_default();
        let entry = per_kind.entry(program.kind.as_str()).or_default();
        entry.0 += 1;
        entry.1.extend(
            mine.iter()
                .filter(|(status, _)| status == "completed")
                .map(|(_, outcome)| outcome.clone()),
        );
        entry.2 += mine.iter().filter(|(status, _)| status == "active").count();
    }
    let early_careers: Vec<serde_json::Value> = per_kind
        .iter()
        .map(|(kind, (programs, completed_outcomes, active))| {
            serde_json::json!({
                "kind": kind,
                "programs": programs,
                "active_placements": active,
                "completed_placements": completed_outcomes.len(),
                "conversion_rate": ratio_json(rules::conversion_rate(completed_outcomes)),
            })
        })
        .collect();

    format::json(serde_json::json!({
        "derivation": "a pipeline's live pool excludes placed and exited members; the \
                       conversion rate divides converted outcomes by COMPLETED placements only \
                       (a running placement has not had the chance to convert)",
        "pipelines": per_pipeline,
        "by_purpose": by_purpose,
        "ready_across_pipelines": ready_total,
        "live_across_pipelines": live_total,
        "ready_share": ratio_json(rules::ratio(ready_total, live_total)),
        "early_careers": early_careers,
    }))
}

/// Live workers employed on `date` ([`metric_rules::is_employed_on`] in SQL): the one population every
/// headcount-style view counts.
fn employed_on(date: chrono::NaiveDate) -> Select<workers::Entity> {
    workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::HiredOn.lte(date))
        .filter(
            Condition::any()
                .add(workers::Column::TerminatedOn.is_null())
                .add(workers::Column::TerminatedOn.gt(date)),
        )
}

/// Workers employed on `date` counted by one text column (department, status, employment type).
async fn grouped(
    ctx: &AppContext,
    date: chrono::NaiveDate,
    column: workers::Column,
) -> Result<BTreeMap<String, usize>> {
    Ok(employed_on(date)
        .select_only()
        .column(column)
        .column_as(Expr::cust("count(*)"), "n")
        .group_by(column)
        .into_tuple::<(String, i64)>()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|(key, n)| (key, usize::try_from(n).unwrap_or(0)))
        .collect())
}

/// The people employed on `$1` ([`metric_rules::is_employed_on`], in SQL), as a subquery.
const EMPLOYED_SQL: &str = "SELECT pid, department FROM workers
     WHERE deleted_at IS NULL AND hired_on <= $1 AND (terminated_on IS NULL OR terminated_on > $1)";

/// Run `sql` (which binds `$1` as the date) and return its rows.
async fn rows_on(
    ctx: &AppContext,
    sql: &str,
    date: chrono::NaiveDate,
) -> Result<Vec<sea_orm::QueryResult>> {
    Ok(ctx
        .db
        .query_all_raw(sea_orm::Statement::from_sql_and_values(
            sea_orm::DbBackend::Postgres,
            sql,
            [date.into()],
        ))
        .await?)
}

/// How many people are employed on `date`.
async fn employed_count(ctx: &AppContext, date: chrono::NaiveDate) -> Result<usize> {
    let rows = rows_on(
        ctx,
        &format!("SELECT COUNT(*)::bigint AS total FROM ({EMPLOYED_SQL}) e"),
        date,
    )
    .await?;
    let total: i64 = rows.first().map_or(Ok(0), |r| r.try_get("", "total"))?;
    Ok(usize::try_from(total).unwrap_or(0))
}

/// The workforce-intelligence routes (read-only).
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/workforce-intelligence")
        .add("/overview", get(overview))
        .add("/capability", get(capability))
        .add("/capability-analysis", get(capability_analysis))
        .add("/metrics", get(metrics))
        .add("/insights", get(insights))
        .add("/headcount-history", get(headcount_history))
        .add("/succession", get(succession))
        .add("/pipelines", get(pipelines))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ratio is rendered with its terms, and a zero denominator is
    /// `null` rather than a misleading `0`.
    #[test]
    fn ratios_render_their_terms_or_null() {
        let value = ratio_json(rules::ratio(3, 4));
        assert_eq!(value["numerator"], 3);
        assert_eq!(value["denominator"], 4);
        assert!((value["value"].as_f64().expect("f64") - 0.75).abs() < f64::EPSILON);

        assert!(
            ratio_json(rules::ratio(0, 0)).is_null(),
            "nothing to divide ⇒ null, never 0"
        );
        let real_zero = ratio_json(rules::ratio(0, 7));
        assert_eq!(real_zero["numerator"], 0);
        assert_eq!(real_zero["denominator"], 7, "0 of 7 is a measurement");
    }
}
