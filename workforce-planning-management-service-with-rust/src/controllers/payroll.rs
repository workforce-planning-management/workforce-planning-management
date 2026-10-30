//! Payroll & compensation (WPM-R13, WPM-R14): runs (draft →
//! calculated → approved → paid), derived payslips (WPM-D5: payroll
//! is a derivation, not an editor), and salary benchmarking.
//! Payslip reads honour the `mask` obligation via the owning
//! worker's record-level ABAC pass (WPM-R15).

use super::row_json;
use loco_rs::prelude::*;
use sea_orm::{QueryOrder, QuerySelect, TransactionTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ensure_valid, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::metrics::Metrics;
use crate::models::_entities::{
    benchmarks, benefit_enrollments, benefit_plans, payroll_runs, payslips, time_entries, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::rules::{benchmark, lifecycle, payroll as rules, workforce};
use crate::streaming;
use crate::validation::Problems;

// ─── The stub's parameters (the deployer's, with example defaults; WPM-T383) ──

fn setting(name: &str) -> Option<String> {
    crate::compat::env_var(name).filter(|v| !v.trim().is_empty())
}

fn parsed<T: std::str::FromStr>(name: &str) -> Option<T> {
    let text = setting(name)?;
    let value = text.trim().parse::<T>().ok();
    if value.is_none() {
        tracing::warn!("{name} is not a whole number; using the example");
    }
    value
}

/// The full-time contracted day in minutes: `WPM_CONTRACTED_DAY_MINUTES`, 1 to 1,440, else the
/// example's 450 (7.5 hours). Overtime is the recorded time beyond it (FTE-scaled).
#[must_use]
pub fn contracted_day() -> i32 {
    parsed::<i32>("WPM_CONTRACTED_DAY_MINUTES")
        .filter(|&m| {
            let ok = workforce::validate_contracted_day(m).is_ok();
            if !ok {
                tracing::warn!("WPM_CONTRACTED_DAY_MINUTES is not 1 to 1440; using the example");
            }
            ok
        })
        .unwrap_or(workforce::CONTRACTED_DAY_MINUTES)
}

/// The payroll stub in force and whether the deployer configured it. `WPM_PAYROLL_STUB_CURRENCY`,
/// `WPM_PAYROLL_STUB_ALLOWANCE_MINOR` (minor units of that currency, a month) and
/// `WPM_PAYROLL_STUB_RATE_PERCENT` replace the example; `WPM_PAYROLL_MONTH_MINUTES` the working
/// minutes in a month. A currency other than the example's is taken only with its own allowance and
/// rate, so the example's pounds are never read as another currency's units; a setting that does not
/// check is ignored with a warning.
#[must_use]
pub fn tax_stub() -> (rules::TaxStub, bool) {
    let example = rules::TaxStub::example();
    let currency = setting("WPM_PAYROLL_STUB_CURRENCY").map(|c| c.trim().to_string());
    let allowance = parsed::<i64>("WPM_PAYROLL_STUB_ALLOWANCE_MINOR");
    let rate = parsed::<i64>("WPM_PAYROLL_STUB_RATE_PERCENT");
    let month = parsed::<i64>("WPM_PAYROLL_MONTH_MINUTES");
    let configured = currency.is_some() || allowance.is_some() || rate.is_some() || month.is_some();
    let mut stub = example.clone();
    if let Some(m) = month {
        stub.month_minutes = m;
    }
    match currency {
        Some(code) if code != example.currency && (allowance.is_none() || rate.is_none()) => {
            tracing::warn!(
                "WPM_PAYROLL_STUB_CURRENCY needs WPM_PAYROLL_STUB_ALLOWANCE_MINOR and \
                 WPM_PAYROLL_STUB_RATE_PERCENT for that currency; using the example"
            );
        }
        other => {
            if let Some(code) = other {
                stub.currency = code;
            }
            if let Some(a) = allowance {
                stub.allowance_minor = a;
            }
            if let Some(r) = rate {
                stub.rate_percent = r;
            }
        }
    }
    if let Err(why) = stub.validate() {
        tracing::warn!("payroll stub settings ignored: {why}");
        return (example, false);
    }
    (stub, configured)
}

/// `POST /api/payroll-runs` body.
#[derive(Debug, Deserialize)]
struct RunPayload {
    organization_ref: String,
    period_start: chrono::NaiveDate,
    period_end: chrono::NaiveDate,
    #[serde(default)]
    notes: Option<String>,
}

/// `POST /api/benchmarks` body.
#[derive(Debug, Deserialize)]
struct BenchmarkPayload {
    job_title: String,
    currency: String,
    min_minor: i64,
    median_minor: i64,
    max_minor: i64,
    source: String,
    as_of: chrono::NaiveDate,
}

/// A `{pid}` reference response.
#[derive(Debug, Serialize)]
struct PidRef {
    pid: String,
}

/// `POST /api/payroll-runs` — open a draft run.
#[debug_handler]
async fn create_run(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<RunPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        &payload.organization_ref,
    );
    if payload.period_end < payload.period_start {
        problems.push("period_end is before period_start".to_string());
    }
    problems.cap_opt("notes", payload.notes.as_deref());
    ensure_valid(&problems.into_vec())?;
    let txn = ctx.db.begin().await?;
    let row = payroll_runs::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        organization_ref: ActiveValue::set(payload.organization_ref.clone()),
        period_start: ActiveValue::set(payload.period_start),
        period_end: ActiveValue::set(payload.period_end),
        status: ActiveValue::set("draft".to_string()),
        approved_by: ActiveValue::set(None),
        notes: ActiveValue::set(payload.notes.clone()),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "payroll_run",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    streaming::emit_on(
        &txn,
        "payroll_run",
        "created",
        &row.pid.to_string(),
        "",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}

/// `GET /api/payroll-runs`.
#[debug_handler]
async fn list_runs(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(page): axum::extract::Query<super::Page>,
) -> Result<Response> {
    let window = super::window(page)?;
    let mut query = payroll_runs::Entity::find().filter(payroll_runs::Column::DeletedAt.is_null());
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await? {
        query = query.filter(payroll_runs::Column::OrganizationRef.is_in(refs));
    }
    super::paged_select(
        &ctx,
        query.order_by_asc(payroll_runs::Column::Id),
        window,
        row_json,
    )
    .await
}

/// `GET /api/payroll-runs/{pid}`.
#[debug_handler]
async fn get_run(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    format::json(records::find_payroll_run(&ctx.db, records::parse_pid(&pid)?).await?)
}

/// `POST /api/payroll-runs/{pid}/calculate` — derive one payslip per
/// salaried in-scope worker (never a contractor, WPM-D58) from salary × FTE, **approved**
/// overtime in the period, and benefit worker-costs; stub tax
/// (WPM-R13, WPM-D5). Re-calculation replaces the run's payslips
/// (drafts only — the lifecycle gate enforces it).
#[allow(clippy::too_many_lines)] // one linear derivation walk per worker
#[debug_handler]
async fn calculate_run(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let run = records::find_payroll_run(&ctx.db, records::parse_pid(&pid)?).await?;
    lifecycle::check("payroll run", lifecycle::PAYROLL, &run.status, "calculated")
        .map_err(|e| unprocessable(&e))?;
    let staff = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::OrganizationRef.eq(&run.organization_ref))
        .filter(workers::Column::Status.is_in(["active", "on_leave"]))
        .filter(workers::Column::SalaryMinor.is_not_null())
        // Contractors are paid against invoices, not through payroll (WPM-D58).
        .filter(workers::Column::EmploymentType.is_not_in(rules::OFF_PAYROLL_TYPES.iter().copied()))
        .all(&ctx.db)
        .await?;
    let txn = ctx.db.begin().await?;
    // Replace any prior calculation for this run.
    payslips::Entity::delete_many()
        .filter(payslips::Column::RunPid.eq(run.pid))
        .exec(&txn)
        .await?;
    let (stub, stub_configured) = tax_stub();
    let full_day = contracted_day();
    let mut count: u64 = 0;
    for worker in &staff {
        let Some(salary) = worker.salary_minor else {
            continue;
        };
        // A salary stored before a currency was required is taken as the stub's currency.
        let currency = worker
            .salary_currency
            .clone()
            .unwrap_or_else(|| stub.currency.clone());
        // Approved time in the period → overtime minutes.
        let entries = time_entries::Entity::find()
            .filter(time_entries::Column::WorkerPid.eq(worker.pid))
            .filter(time_entries::Column::Status.eq("approved"))
            .filter(time_entries::Column::WorkedOn.gte(run.period_start))
            .filter(time_entries::Column::WorkedOn.lte(run.period_end))
            .filter(time_entries::Column::DeletedAt.is_null())
            .all(&txn)
            .await?;
        let mut days: std::collections::BTreeMap<chrono::NaiveDate, (i32, i32)> =
            std::collections::BTreeMap::new();
        for entry in &entries {
            let slot = days.entry(entry.worked_on).or_default();
            if entry.kind == "overtime" {
                slot.1 += entry.minutes;
            } else {
                slot.0 += entry.minutes;
            }
        }
        let overtime: i64 = days
            .values()
            .map(|(regular, explicit)| {
                i64::from(workforce::overtime_minutes(
                    *regular,
                    *explicit,
                    worker.fte_percent,
                    full_day,
                ))
            })
            .sum();
        // Benefit worker-costs (same currency only).
        let enrollments = benefit_enrollments::Entity::find()
            .filter(benefit_enrollments::Column::WorkerPid.eq(worker.pid))
            .filter(benefit_enrollments::Column::DeletedAt.is_null())
            .all(&txn)
            .await?;
        let mut benefit_costs = Vec::new();
        for enrollment in &enrollments {
            if let Some(plan) = benefit_plans::Entity::find()
                .filter(benefit_plans::Column::Pid.eq(enrollment.plan_pid))
                .filter(benefit_plans::Column::DeletedAt.is_null())
                .one(&txn)
                .await?
                && plan.currency.eq_ignore_ascii_case(&currency)
                && plan.worker_cost_minor > 0
            {
                benefit_costs.push((plan.name.clone(), plan.worker_cost_minor));
            }
        }
        let slip = rules::compute_payslip(
            &stub,
            &currency,
            salary,
            worker.fte_percent,
            overtime,
            &benefit_costs,
        )
        .map_err(|e| unprocessable(&format!("payslip for {}: {e}", worker.worker_number)))?;
        // The persist gate re-checks the invariant (WPM-R13).
        rules::reconcile(&slip).map_err(|e| unprocessable(&e))?;
        payslips::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            run_pid: ActiveValue::set(run.pid),
            worker_pid: ActiveValue::set(worker.pid),
            currency: ActiveValue::set(currency),
            gross_minor: ActiveValue::set(slip.gross_minor),
            deductions: ActiveValue::set(
                serde_json::to_value(&slip.deductions).unwrap_or_default(),
            ),
            net_minor: ActiveValue::set(slip.net_minor),
            deleted_at: ActiveValue::set(None),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
        count += 1;
    }
    let mut active: payroll_runs::ActiveModel = run.clone().into();
    active.status = ActiveValue::set("calculated".to_string());
    let row = active.update(&txn).await?;
    Audit::record(
        &txn,
        "payroll_run",
        row.pid,
        "payroll_run_calculated",
        caller.actor(),
        Some(serde_json::json!({ "payslips": count })),
    )
    .await?;
    streaming::emit_on(
        &txn,
        "payroll_run",
        "payroll_run_calculated",
        &row.pid.to_string(),
        "",
        caller.actor(),
        Some(serde_json::json!({ "payslips": count })),
    )
    .await?;
    txn.commit().await?;
    Metrics::global().payroll_calculated_total.inc();
    format::json(serde_json::json!({
        "pid": row.pid,
        "status": row.status,
        "payslips": count,
        // Whose figures the tax line used: the deployer's, or the example written for GBP (WPM-T383).
        "tax_stub": {
            "currency": stub.currency,
            "allowance_minor": stub.allowance_minor,
            "rate_percent": stub.rate_percent,
            "month_minutes": stub.month_minutes,
            "contracted_day_minutes": full_day,
            "source": if stub_configured { "configured" } else { "example" },
            "note": "a stub, not any country's tax; the example is the shape of the UK's 2025-26 personal \
                     allowance and basic rate, written for GBP, and a payslip in another currency is refused",
        },
    }))
}

/// One non-calculate run transition (`draft` reopen / `approved` /
/// `paid`), via the lifecycle table.
async fn run_transition(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
    to: &str,
) -> Result<Response> {
    let run = records::find_payroll_run(&ctx.db, records::parse_pid(pid)?).await?;
    lifecycle::check("payroll run", lifecycle::PAYROLL, &run.status, to)
        .map_err(|e| unprocessable(&e))?;
    let txn = ctx.db.begin().await?;
    let from = run.status.clone();
    let mut active: payroll_runs::ActiveModel = run.into();
    active.status = ActiveValue::set(to.to_string());
    if to == "approved" {
        active.approved_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    }
    let row = active.update(&txn).await?;
    let kind = format!("payroll_run_{to}");
    Audit::record(
        &txn,
        "payroll_run",
        row.pid,
        &kind,
        caller.actor(),
        Some(serde_json::json!({ "from": from })),
    )
    .await?;
    streaming::emit_on(
        &txn,
        "payroll_run",
        &kind,
        &row.pid.to_string(),
        "",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(row_json(&row)?)
}

/// `POST /api/payroll-runs/{pid}/approve`.
#[debug_handler]
async fn approve_run(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    run_transition(&ctx, &caller, &pid, "approved").await
}

/// `POST /api/payroll-runs/{pid}/pay`.
#[debug_handler]
async fn pay_run(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    run_transition(&ctx, &caller, &pid, "paid").await
}

/// `POST /api/payroll-runs/{pid}/reopen` — calculated → draft.
#[debug_handler]
async fn reopen_run(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    run_transition(&ctx, &caller, &pid, "draft").await
}

/// Payslip read + mask helper: the record-level pass runs against the
/// **owning worker's** attributes, so the same policy that masks an
/// worker's salary masks their payslips.
async fn masked_payslips(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    rows: Vec<payslips::Model>,
) -> Result<Vec<payslips::Model>> {
    let mut out = Vec::with_capacity(rows.len());
    for slip in rows {
        let worker = records::find_worker(&ctx.db, slip.worker_pid).await?;
        let obligations = auth::authorize_record(
            caller,
            authentication_verifier::Action::Read,
            &auth::worker_resource_attrs(&worker),
        )
        .map_err(record_rejection)?;
        out.push(if obligations.iter().any(|o| o == "mask") {
            auth::mask_payslip(slip)
        } else {
            slip
        });
    }
    Ok(out)
}

/// `GET /api/payroll-runs/{pid}/payslips` — the run's payslips; the
/// read is audited (WPM-D7).
#[debug_handler]
async fn run_payslips(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let run = records::find_payroll_run(&ctx.db, records::parse_pid(&pid)?).await?;
    let rows = payslips::Entity::find()
        .filter(payslips::Column::RunPid.eq(run.pid))
        .filter(payslips::Column::DeletedAt.is_null())
        .order_by_asc(payslips::Column::Id)
        .all(&ctx.db)
        .await?;
    Audit::record(
        &ctx.db,
        "payroll_run",
        run.pid,
        "payslips_read",
        caller.actor(),
        None,
    )
    .await?;
    format::json(masked_payslips(&ctx, &caller, rows).await?)
}

/// `GET /api/workers/{pid}/payslips` — one worker's payslips
/// (self-service surface, WPM-R8); audited.
#[debug_handler]
async fn worker_payslips(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let rows = payslips::Entity::find()
        .filter(payslips::Column::WorkerPid.eq(worker.pid))
        .filter(payslips::Column::DeletedAt.is_null())
        .order_by_asc(payslips::Column::Id)
        .all(&ctx.db)
        .await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "payslips_read",
        caller.actor(),
        None,
    )
    .await?;
    format::json(masked_payslips(&ctx, &caller, rows).await?)
}

/// `POST /api/benchmarks`.
#[debug_handler]
async fn create_benchmark(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<BenchmarkPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("job_title", &payload.job_title);
    problems.require_currency("currency", &payload.currency);
    problems.require_text("source", &payload.source);
    if payload.min_minor < 0
        || payload.min_minor > payload.median_minor
        || payload.median_minor > payload.max_minor
    {
        problems.push("band must satisfy 0 <= min <= median <= max".to_string());
    }
    ensure_valid(&problems.into_vec())?;
    let txn = ctx.db.begin().await?;
    let row = benchmarks::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        job_title: ActiveValue::set(payload.job_title.clone()),
        currency: ActiveValue::set(payload.currency.clone()),
        min_minor: ActiveValue::set(payload.min_minor),
        median_minor: ActiveValue::set(payload.median_minor),
        max_minor: ActiveValue::set(payload.max_minor),
        source: ActiveValue::set(payload.source.clone()),
        as_of: ActiveValue::set(payload.as_of),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(&txn, "benchmark", row.pid, "created", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}

/// `GET /api/benchmarks`.
#[debug_handler]
async fn list_benchmarks(State(ctx): State<AppContext>) -> Result<Response> {
    let rows = benchmarks::Entity::find()
        .filter(benchmarks::Column::DeletedAt.is_null())
        .order_by_asc(benchmarks::Column::Id)
        .limit(500)
        .all(&ctx.db)
        .await?;
    format::json(rows)
}

/// `GET /api/benchmarks/comparison?organization=<ref>` — every
/// salaried worker vs the newest benchmark for their job title:
/// `below_min` / `within` / `above_max` (WPM-R14). Salary values are
/// **not** echoed — only the flags — so the view is compensation-
/// persona data without leaking amounts; the read is audited.
#[derive(Debug, Deserialize)]
struct ComparisonParams {
    organization: String,
}

#[debug_handler]
async fn benchmark_comparison(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Query(params): Query<ComparisonParams>,
) -> Result<Response> {
    let mut staff_query = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::OrganizationRef.eq(&params.organization))
        .filter(workers::Column::SalaryMinor.is_not_null());
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await? {
        // `params.organization` outside the caller's scope ANDs down to
        // zero rows here — no separate 403 needed for this read.
        staff_query = staff_query.filter(workers::Column::OrganizationRef.is_in(refs));
    }
    let staff = staff_query.all(&ctx.db).await?;
    let bands = benchmarks::Entity::find()
        .filter(benchmarks::Column::DeletedAt.is_null())
        .order_by_desc(benchmarks::Column::AsOf)
        .all(&ctx.db)
        .await?;
    let mut rows = Vec::new();
    for worker in &staff {
        let (Some(salary), Some(currency)) =
            (worker.salary_minor, worker.salary_currency.as_deref())
        else {
            continue;
        };
        let band = bands
            .iter()
            .find(|b| b.job_title.eq_ignore_ascii_case(&worker.job_title));
        let flag = band.and_then(|b| {
            benchmark::compare(salary, currency, b.min_minor, b.max_minor, &b.currency)
        });
        rows.push(serde_json::json!({
            "worker_pid": worker.pid,
            "job_title": worker.job_title,
            "department": worker.department,
            "benchmark_pid": band.map(|b| b.pid),
            "flag": flag,
        }));
    }
    Audit::record(
        &ctx.db,
        "benchmark",
        Uuid::nil(),
        "comparison_read",
        caller.actor(),
        None,
    )
    .await?;
    format::json(serde_json::json!({ "organization": params.organization, "rows": rows }))
}

/// The payroll routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/payroll-runs", post(create_run))
        .add("/payroll-runs", get(list_runs))
        .add("/payroll-runs/{pid}", get(get_run))
        .add("/payroll-runs/{pid}/calculate", post(calculate_run))
        .add("/payroll-runs/{pid}/approve", post(approve_run))
        .add("/payroll-runs/{pid}/pay", post(pay_run))
        .add("/payroll-runs/{pid}/reopen", post(reopen_run))
        .add("/payroll-runs/{pid}/payslips", get(run_payslips))
        .add("/workers/{pid}/payslips", get(worker_payslips))
        .add("/benchmarks", post(create_benchmark))
        .add("/benchmarks", get(list_benchmarks))
        .add("/benchmarks/comparison", get(benchmark_comparison))
}
