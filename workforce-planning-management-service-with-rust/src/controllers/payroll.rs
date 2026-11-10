//! Payroll & compensation (WPM-R13, WPM-R14): runs (draft →
//! calculated → approved → paid), derived payslips (WPM-D5: payroll
//! is a derivation, not an editor), and salary benchmarking.
//! Payslip reads honour the `mask` obligation via the owning
//! worker's record-level ABAC pass (WPM-R15).

use super::row_json;
use axa::extract::{Json, Path, Query};
use axa::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ensure_valid, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::metrics::Metrics;
use crate::models::_entities::{benchmarks, payroll_runs, payslips, workers};
use crate::models::{memberships, records};
use crate::rules::{benchmark, lifecycle, payroll as rules, workforce};
use crate::streaming;
use crate::validation::Problems;

/// A live payroll run by pid, else 404.
async fn find_run(ctx: &AppContext, pid: Uuid) -> Result<payroll_runs::Model> {
    sqlx::query_as!(
        payroll_runs::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, period_start,
                  period_end, status, approved_by, notes, deleted_at AS "deleted_at: _"
             FROM payroll_runs WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// A run's live payslips, or one worker's, in the order they were made.
async fn live_payslips(
    ctx: &AppContext,
    run: Option<Uuid>,
    worker: Option<Uuid>,
) -> Result<Vec<payslips::Model>> {
    Ok(sqlx::query_as!(
        payslips::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, run_pid, worker_pid, currency,
                  gross_minor, deductions, net_minor, statutory_overtime_minor, employer_cost_minor, employer_costs,
                  deleted_at AS "deleted_at: _"
             FROM payslips
            WHERE deleted_at IS NULL AND ($1::uuid IS NULL OR run_pid = $1) AND ($2::uuid IS NULL OR worker_pid = $2)
            ORDER BY id"#,
        run,
        worker,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// The workers with these pids, soft-deleted or not, by pid.
async fn workers_by_pid(
    ctx: &AppContext,
    pids: &[Uuid],
) -> Result<std::collections::BTreeMap<Uuid, workers::Model>> {
    Ok(sqlx::query_as!(
        workers::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, person_ref,
                  upstream_worker_ref, organization_ref, worker_number, display_name, status, employment_type,
                  fte_percent, department, job_title, location, work_jurisdiction, manager_pid, salary_minor,
                  salary_currency, hired_on, terminated_on, engagement_ends_on, deleted_at AS "deleted_at: _"
             FROM workers WHERE pid = ANY($1) ORDER BY id"#,
        pids,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
    .map(|worker| (worker.pid, worker))
    .collect())
}

/// An audit of a read, which has no write to share a transaction with.
async fn read_connection(ctx: &AppContext) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>> {
    Ok(db::pool(ctx).acquire().await?)
}

// ─── The stub's parameters (the deployer's, with example defaults; WPM-T383) ──

/// Where a setting is read from: the process environment in the service, a table in a test
/// (WPM-T401), so the parsing is tested without mutating the environment.
type Lookup<'a> = &'a dyn Fn(&str) -> Option<String>;

/// The process environment, through the legacy-name fallback.
fn from_env(name: &str) -> Option<String> {
    crate::compat::env_var(name)
}

fn setting(lookup: Lookup<'_>, name: &str) -> Option<String> {
    lookup(name).filter(|v| !v.trim().is_empty())
}

fn parsed<T: std::str::FromStr>(lookup: Lookup<'_>, name: &str) -> Option<T> {
    let text = setting(lookup, name)?;
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
    contracted_day_from(&from_env)
}

/// [`contracted_day`] read through `lookup`.
fn contracted_day_from(lookup: Lookup<'_>) -> i32 {
    parsed::<i32>(lookup, "WPM_CONTRACTED_DAY_MINUTES")
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
    tax_stub_from(&from_env)
}

/// [`tax_stub`] read through `lookup`.
fn tax_stub_from(lookup: Lookup<'_>) -> (rules::TaxStub, bool) {
    let example = rules::TaxStub::example();
    let currency = setting(lookup, "WPM_PAYROLL_STUB_CURRENCY").map(|c| c.trim().to_string());
    let allowance = parsed::<i64>(lookup, "WPM_PAYROLL_STUB_ALLOWANCE_MINOR");
    let rate = parsed::<i64>(lookup, "WPM_PAYROLL_STUB_RATE_PERCENT");
    let month = parsed::<i64>(lookup, "WPM_PAYROLL_MONTH_MINUTES");
    // Configured means a deployer's value is in force, not merely present: a currency refused for want
    // of its own allowance and rate leaves the example in force and is not "configured" (WPM-T401).
    let mut configured = false;
    let mut stub = example.clone();
    if let Some(m) = month {
        stub.month_minutes = m;
        configured = true;
    }
    match currency {
        Some(code) if code != example.currency && (allowance.is_none() || rate.is_none()) => {
            tracing::warn!(
                "WPM_PAYROLL_STUB_CURRENCY needs WPM_PAYROLL_STUB_ALLOWANCE_MINOR and \
                 WPM_PAYROLL_STUB_RATE_PERCENT for that currency; using the example"
            );
        }
        other => {
            configured |= other.is_some() || allowance.is_some() || rate.is_some();
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
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO payroll_runs (pid, organization_ref, period_start, period_end, status, approved_by, notes, deleted_at)
         VALUES ($1, $2, $3, $4, 'draft', NULL, $5, NULL)",
        pid,
        payload.organization_ref,
        payload.period_start,
        payload.period_end,
        payload.notes,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "payroll_run", pid, "created", caller.actor(), None).await?;
    streaming::emit_in(
        &mut tx,
        "payroll_run",
        "created",
        &pid.to_string(),
        "",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: pid.to_string(),
    })
}

/// `GET /api/payroll-runs`.
#[debug_handler]
async fn list_runs(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    page: super::Page,
) -> Result<Response> {
    let window = super::window(&page)?;
    let refs = memberships::scope_organization_refs(&ctx, caller.claims()).await?;
    let (limit, offset) = window.bounds();
    let rows = sqlx::query_as!(
        payroll_runs::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, period_start,
                  period_end, status, approved_by, notes, deleted_at AS "deleted_at: _"
             FROM payroll_runs
            WHERE deleted_at IS NULL AND ($1::text[] IS NULL OR organization_ref = ANY($1))
            ORDER BY id LIMIT $2 OFFSET $3"#,
        refs.as_deref(),
        limit,
        offset,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    Ok(window.respond(rows.iter().map(row_json).collect::<Result<Vec<_>>>()?))
}

/// `GET /api/payroll-runs/{pid}`.
#[debug_handler]
async fn get_run(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    format::json(find_run(&ctx, records::parse_pid(&pid)?).await?)
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
    let run = find_run(&ctx, records::parse_pid(&pid)?).await?;
    lifecycle::check("payroll run", lifecycle::PAYROLL, &run.status, "calculated")
        .map_err(|e| unprocessable(&e))?;
    // Contractors are paid against invoices, not through payroll (WPM-D58).
    let off_payroll: Vec<String> = rules::OFF_PAYROLL_TYPES
        .iter()
        .map(|t| (*t).to_string())
        .collect();
    let staff = sqlx::query_as!(
        workers::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, person_ref,
                  upstream_worker_ref, organization_ref, worker_number, display_name, status, employment_type,
                  fte_percent, department, job_title, location, work_jurisdiction, manager_pid, salary_minor,
                  salary_currency, hired_on, terminated_on, engagement_ends_on, deleted_at AS "deleted_at: _"
             FROM workers
            WHERE deleted_at IS NULL AND organization_ref = $1 AND status IN ('active', 'on_leave')
              AND salary_minor IS NOT NULL AND NOT (employment_type = ANY($2))
            ORDER BY id"#,
        run.organization_ref,
        &off_payroll,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    // Jurisdiction parameters for the statutory overtime premium (WPM-D169), resolved once for the organization. Read before
    // the transaction: they are append-only and this run writes none.
    let profiles = super::jurisdictions::profiles_for(
        crate::db::pool(&ctx),
        std::slice::from_ref(&run.organization_ref),
    )
    .await?;
    let mut tx = db::pool(&ctx).begin().await?;
    // Replace any prior calculation for this run.
    sqlx::query!("DELETE FROM payslips WHERE run_pid = $1", run.pid)
        .execute(&mut *tx)
        .await?;
    let (stub, stub_configured) = tax_stub();
    let full_day = contracted_day();
    let no_profile = super::jurisdictions::Profile::default();
    let profile = profiles.get(&run.organization_ref).unwrap_or(&no_profile);
    let holidays = holiday_dates(
        &mut tx,
        &run.organization_ref,
        run.period_start,
        run.period_end,
    )
    .await?;
    // Earlier runs of the same calendar year, for the wage base of an employer cost line (WPM-D170): approved or paid, ending before
    // this period starts.
    let earlier_runs: Vec<Uuid> = {
        use chrono::Datelike;
        sqlx::query!(
            "SELECT pid, period_end FROM payroll_runs
              WHERE organization_ref = $1 AND status IN ('approved', 'paid') AND period_end < $2 AND pid <> $3",
            run.organization_ref,
            run.period_start,
            run.pid,
        )
        .fetch_all(&mut *tx)
        .await?
        .into_iter()
        .filter(|r| r.period_end.year() == run.period_end.year())
        .map(|r| r.pid)
        .collect()
    };
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
        let entries = sqlx::query!(
            "SELECT worked_on, kind, minutes FROM time_entries
              WHERE worker_pid = $1 AND status = 'approved' AND worked_on >= $2 AND worked_on <= $3 AND deleted_at IS NULL
              ORDER BY id",
            worker.pid,
            run.period_start,
            run.period_end,
        )
        .fetch_all(&mut *tx)
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
        // Each live enrollment's live plan, in enrollment order.
        let plans = sqlx::query!(
            r#"SELECT p.name, p.currency, p.worker_cost_minor
                 FROM benefit_enrollments e
                 CROSS JOIN LATERAL (
                     SELECT name, currency, worker_cost_minor FROM benefit_plans
                      WHERE pid = e.plan_pid AND deleted_at IS NULL LIMIT 1
                 ) p
                WHERE e.worker_pid = $1 AND e.deleted_at IS NULL
                ORDER BY e.id"#,
            worker.pid,
        )
        .fetch_all(&mut *tx)
        .await?;
        let mut benefit_costs = Vec::new();
        for plan in &plans {
            if plan.currency.eq_ignore_ascii_case(&currency) && plan.worker_cost_minor > 0 {
                benefit_costs.push((plan.name.clone(), plan.worker_cost_minor));
            }
        }
        // The statutory premium: derived only for a person recorded as non-exempt, in a place with bands configured. Anything else adds
        // nothing, and the advisory check says why (WPM-D169).
        let seats: Vec<String> = sqlx::query_scalar!(
            "SELECT overtime_status FROM positions WHERE holder_worker_pid = $1 AND state <> 'abolished' ORDER BY id",
            worker.pid,
        )
        .fetch_all(&mut *tx)
        .await?;
        let mut worked: std::collections::BTreeMap<chrono::NaiveDate, u32> =
            std::collections::BTreeMap::new();
        for entry in entries.iter().filter(|entry| entry.kind != "on_call") {
            *worked.entry(entry.worked_on).or_default() +=
                u32::try_from(entry.minutes.max(0)).unwrap_or(0);
        }
        let found = statutory_outcome(
            Some(worker),
            &seats,
            &worked,
            None,
            &OutcomeContext {
                profile,
                as_of: run.period_end,
                currency: &currency,
                month_minutes: stub.month_minutes,
                holidays: &holidays,
            },
        );
        let premium = if found.name == "derived" {
            found.extra["premium_minor"].as_i64().unwrap_or(0).max(0)
        } else {
            0
        };
        let slip = rules::compute_payslip_with_premium(
            &stub,
            &currency,
            salary,
            worker.fte_percent,
            overtime,
            premium,
            &benefit_costs,
        )
        .map_err(|e| unprocessable(&format!("payslip for {}: {e}", worker.worker_number)))?;
        // The persist gate re-checks the invariant (WPM-R13).
        rules::reconcile(&slip).map_err(|e| unprocessable(&e))?;
        let earlier: i64 = if earlier_runs.is_empty() {
            0
        } else {
            sqlx::query_scalar!(
                "SELECT gross_minor FROM payslips WHERE run_pid = ANY($1) AND worker_pid = $2 AND deleted_at IS NULL",
                &earlier_runs,
                worker.pid,
            )
            .fetch_all(&mut *tx)
            .await?
            .iter()
            .sum()
        };
        let (employer_cost, employer_costs) =
            employer_costs_for(profile, worker, run.period_end, slip.gross_minor, earlier)
                .map_err(|e| {
                    unprocessable(&format!("payslip for {}: {e}", worker.worker_number))
                })?;
        sqlx::query!(
            "INSERT INTO payslips (pid, run_pid, worker_pid, currency, gross_minor, deductions, net_minor,
                                   statutory_overtime_minor, employer_cost_minor, employer_costs, deleted_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NULL)",
            Uuid::new_v4(),
            run.pid,
            worker.pid,
            currency,
            slip.gross_minor,
            serde_json::to_value(&slip.deductions).unwrap_or_default(),
            slip.net_minor,
            premium,
            employer_cost,
            employer_costs,
        )
        .execute(&mut *tx)
        .await?;
        count += 1;
    }
    sqlx::query!(
        "UPDATE payroll_runs SET status = 'calculated', updated_at = now() WHERE id = $1",
        run.id
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "payroll_run",
        run.pid,
        "payroll_run_calculated",
        caller.actor(),
        Some(serde_json::json!({ "payslips": count })),
    )
    .await?;
    streaming::emit_in(
        &mut tx,
        "payroll_run",
        "payroll_run_calculated",
        &run.pid.to_string(),
        "",
        caller.actor(),
        Some(serde_json::json!({ "payslips": count })),
    )
    .await?;
    tx.commit().await?;
    Metrics::global().payroll_calculated_total.inc();
    format::json(serde_json::json!({
        "pid": run.pid,
        "status": "calculated",
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
    let run = find_run(ctx, records::parse_pid(pid)?).await?;
    lifecycle::check("payroll run", lifecycle::PAYROLL, &run.status, to)
        .map_err(|e| unprocessable(&e))?;
    let mut tx = db::pool(ctx).begin().await?;
    let from = run.status.clone();
    let row = sqlx::query_as!(
        payroll_runs::Model,
        r#"UPDATE payroll_runs
              SET status = $2::text, approved_by = CASE WHEN $2::text = 'approved' THEN $3 ELSE approved_by END, updated_at = now()
            WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, period_start,
                  period_end, status, approved_by, notes, deleted_at AS "deleted_at: _""#,
        run.id,
        to,
        caller.actor(),
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;
    let snapshot = Some(serde_json::json!({ "from": from }));
    // One literal action per transition.
    match to {
        "approved" => {
            db::audit(
                &mut tx,
                "payroll_run",
                row.pid,
                "payroll_run_approved",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
        "paid" => {
            db::audit(
                &mut tx,
                "payroll_run",
                row.pid,
                "payroll_run_paid",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
        "draft" => {
            db::audit(
                &mut tx,
                "payroll_run",
                row.pid,
                "payroll_run_draft",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
        _ => return Err(super::internal("an unknown payroll run transition")),
    }
    let kind = format!("payroll_run_{to}");
    streaming::emit_in(
        &mut tx,
        "payroll_run",
        &kind,
        &row.pid.to_string(),
        "",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
        let worker = records::worker_by_pid(ctx, slip.worker_pid).await?;
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
    let run = find_run(&ctx, records::parse_pid(&pid)?).await?;
    let rows = live_payslips(&ctx, Some(run.pid), None).await?;
    db::audit(
        &mut *read_connection(&ctx).await?,
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
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let rows = live_payslips(&ctx, None, Some(worker.pid)).await?;
    db::audit(
        &mut *read_connection(&ctx).await?,
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
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO benchmarks (pid, job_title, currency, min_minor, median_minor, max_minor, source, as_of, deleted_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL)",
        pid,
        payload.job_title,
        payload.currency,
        payload.min_minor,
        payload.median_minor,
        payload.max_minor,
        payload.source,
        payload.as_of,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "benchmark", pid, "created", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: pid.to_string(),
    })
}

/// `GET /api/benchmarks`.
#[debug_handler]
async fn list_benchmarks(State(ctx): State<AppContext>) -> Result<Response> {
    let rows = sqlx::query_as!(
        benchmarks::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, job_title, currency, min_minor,
                  median_minor, max_minor, source, as_of, deleted_at AS "deleted_at: _"
             FROM benchmarks WHERE deleted_at IS NULL ORDER BY id LIMIT 500"#
    )
    .fetch_all(db::pool(&ctx))
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
    // `params.organization` outside the caller's scope ANDs down to
    // zero rows here — no separate 403 needed for this read.
    let refs = memberships::scope_organization_refs(&ctx, caller.claims()).await?;
    let staff = sqlx::query!(
        "SELECT pid, job_title, department, salary_minor, salary_currency FROM workers
          WHERE deleted_at IS NULL AND organization_ref = $1 AND salary_minor IS NOT NULL
            AND ($2::text[] IS NULL OR organization_ref = ANY($2))
          ORDER BY id",
        params.organization,
        refs.as_deref(),
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let bands = sqlx::query!(
        "SELECT pid, job_title, currency, min_minor, max_minor FROM benchmarks
          WHERE deleted_at IS NULL ORDER BY as_of DESC, id"
    )
    .fetch_all(db::pool(&ctx))
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
    db::audit(
        &mut *read_connection(&ctx).await?,
        "benchmark",
        Uuid::nil(),
        "comparison_read",
        caller.actor(),
        None,
    )
    .await?;
    format::json(serde_json::json!({ "organization": params.organization, "rows": rows }))
}

/// What the minimum-wage check says about itself on every answer (WPM-D156).
const MINIMUM_WAGE_NOTE: &str = "Indicative only. Each payslip's gross is compared with the minimum wage for the worker's age band \
     where the worker's place of work is recorded and a floor is configured. Accommodation, tips, piece work, apprentices and which pay \
     the law counts are not modelled. An ok is not a statement that an employer complies, and nothing here blocks the run.";

/// One run's minimum-wage comparison: what the check route answers and the human capital report counts.
pub(crate) struct WageCheck {
    /// Payslips compared.
    pub payslips: usize,
    /// Payslips by outcome, every outcome present.
    pub counts: std::collections::BTreeMap<&'static str, u64>,
    /// Why payslips were `unknown`.
    pub reasons: std::collections::BTreeMap<&'static str, u64>,
    /// A row per payslip: worker, outcome, reason, band.
    pub rows: Vec<serde_json::Value>,
}

/// Compare each payslip of a run with the minimum wage for the worker's age band, as of the period's last day. `known_births` holds the
/// birth dates already read in this request, so a report over several runs asks the person service once for each person; it is dropped
/// with the request and nothing is stored. Authorizing and auditing are the caller's.
#[allow(clippy::too_many_lines)] // payslips → workers → hours → floors → outcome, linearly
pub(crate) async fn check_minimum_wage(
    ctx: &AppContext,
    run: &payroll_runs::Model,
    known_births: &mut std::collections::BTreeMap<String, Option<chrono::NaiveDate>>,
) -> Result<WageCheck> {
    use crate::rules::jurisdiction::Resolution;
    use crate::rules::minimum_wage as mw;
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::str::FromStr;

    let slips = live_payslips(ctx, Some(run.pid), None).await?;
    let worker_pids: Vec<Uuid> = slips.iter().map(|slip| slip.worker_pid).collect();
    let people: BTreeMap<Uuid, workers::Model> = workers_by_pid(ctx, &worker_pids).await?;
    // Recorded minutes in the period, by worker. On-call entries are not hours worked.
    let entries = sqlx::query!(
        "SELECT worker_pid, minutes FROM time_entries
          WHERE worker_pid = ANY($1) AND worked_on >= $2 AND worked_on <= $3 AND kind <> 'on_call' AND deleted_at IS NULL",
        &worker_pids,
        run.period_start,
        run.period_end,
    )
    .fetch_all(db::pool(ctx))
    .await?;
    let mut minutes: BTreeMap<Uuid, i64> = BTreeMap::new();
    for entry in entries {
        *minutes.entry(entry.worker_pid).or_default() += i64::from(entry.minutes);
    }
    let profiles = super::jurisdictions::profiles_for(
        crate::db::pool(ctx),
        std::slice::from_ref(&run.organization_ref),
    )
    .await?;
    let empty = super::jurisdictions::Profile::default();
    let profile = profiles.get(&run.organization_ref).unwrap_or(&empty);
    let as_of = run.period_end;
    let period_days = (run.period_end - run.period_start).num_days() + 1;

    let mut counts: BTreeMap<&'static str, u64> = ["below", "near", "ok", "none_by_law", "unknown"]
        .into_iter()
        .map(|name| (name, 0))
        .collect();
    let mut reasons: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut rows = Vec::with_capacity(slips.len());
    for slip in &slips {
        let Some(worker) = people.get(&slip.worker_pid) else {
            *counts.entry("unknown").or_default() += 1;
            *reasons.entry("worker_not_found").or_default() += 1;
            rows.push(json!({ "worker_pid": slip.worker_pid, "outcome": "unknown", "reason": "worker_not_found", "band": null }));
            continue;
        };
        let place = worker.work_jurisdiction.as_deref();
        let (outcome, band_name) = if place.is_none() {
            (mw::Outcome::Unknown("no_work_jurisdiction"), None)
        } else {
            let get = |key: &str| profile.resolve(key, place, as_of);
            let int = |key: &str| get(key).integer();
            let mut floors = mw::Floors::default();
            for band in mw::Band::ALL {
                let floor = mw::Floor {
                    hourly_minor: int(band.hourly_key()),
                    monthly_minor: int(band.monthly_key()),
                };
                match band {
                    mw::Band::Adult => floors.adult = floor,
                    mw::Band::YoungAdult => floors.young_adult = floor,
                    mw::Band::Youth => floors.youth = floor,
                }
            }
            let none_by_law = matches!(
                get(mw::NONE_BY_LAW_KEY),
                Resolution::Resolved { value, .. } if value.as_flag() == Some(true)
            );
            let currency = match get(mw::CURRENCY_KEY) {
                Resolution::Resolved { value, .. } => {
                    value.as_names().and_then(|names| names.into_iter().next())
                }
                _ => None,
            };
            // The birth date is read from the person service only when the profile has age bands, used for the whole-year age
            // at the period's end, and stored nowhere (WPM-D114, WPM-D156). None where it is not known.
            let adult_from = int(mw::ADULT_FROM_AGE_KEY);
            let age = if adult_from.is_some_and(|years| years > 0) {
                let known = if let Some(known) = known_births.get(&worker.person_ref) {
                    *known
                } else {
                    let read =
                        if let Ok(person) = entity_ref::EntityRef::from_str(&worker.person_ref) {
                            crate::clients::birth_date(&person).await
                        } else {
                            None
                        };
                    known_births.insert(worker.person_ref.clone(), read);
                    read
                };
                known.map(|born| mw::completed_years(born, as_of))
            } else {
                None
            };
            let band = mw::band_for(age, adult_from, int(mw::YOUNG_ADULT_FROM_AGE_KEY));
            let inputs = mw::Inputs {
                pay_minor: slip.gross_minor,
                worked_minutes: minutes.get(&slip.worker_pid).copied().unwrap_or(0),
                period_days,
                fte_percent: i64::from(worker.fte_percent),
                payments_per_year: int(mw::PAYMENTS_PER_YEAR_KEY)
                    .unwrap_or(mw::DEFAULT_PAYMENTS_PER_YEAR),
                near_margin_bp: int(mw::NEAR_MARGIN_BP_KEY).unwrap_or(0),
            };
            let outcome = mw::check(
                band,
                &floors,
                none_by_law,
                currency.as_deref(),
                &slip.currency,
                &inputs,
            );
            (
                outcome,
                band.ok().filter(|_| floors.any()).map(mw::Band::name),
            )
        };
        *counts.entry(outcome.name()).or_default() += 1;
        if let Some(reason) = outcome.reason() {
            *reasons.entry(reason).or_default() += 1;
        }
        rows.push(json!({
            "worker_pid": slip.worker_pid,
            "outcome": outcome.name(),
            "reason": outcome.reason(),
            "band": band_name,
        }));
    }
    Ok(WageCheck {
        payslips: slips.len(),
        counts,
        reasons,
        rows,
    })
}

/// `GET /api/payroll-runs/{pid}/minimum-wage-check` — an advisory comparison of each payslip's gross with the minimum wage for the
/// worker's age band, as of the period's last day (WPM-R218, WPM-D156, WPM-D160). HR and payroll only, and only for organizations the
/// caller holds. The floors are jurisdiction parameters resolved from each worker's place of work; with none configured the answer is
/// `unknown`, never `ok`, and a jurisdiction recorded as setting no statutory floor answers `none_by_law`. The date of birth is used
/// only to pick a band and no age is returned or stored. The audit entry holds counts, never names.
#[debug_handler]
async fn minimum_wage_check(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    use serde_json::json;

    let run = find_run(&ctx, records::parse_pid(&pid)?).await?;
    super::ensure_org_in_scope(&ctx, &caller, &run.organization_ref).await?;
    let check = check_minimum_wage(&ctx, &run, &mut std::collections::BTreeMap::new()).await?;
    db::audit(
        &mut *read_connection(&ctx).await?,
        "payroll_run",
        run.pid,
        "minimum_wage_checked",
        caller.actor(),
        Some(json!({ "payslips": check.payslips, "counts": check.counts, "unknown_reasons": check.reasons })),
    )
    .await?;
    format::json(json!({
        "run_pid": run.pid,
        "period_start": run.period_start,
        "period_end": run.period_end,
        "indicative": true,
        "note": MINIMUM_WAGE_NOTE,
        "payslips": check.payslips,
        "counts": check.counts,
        "unknown_reasons": check.reasons,
        "workers": check.rows,
    }))
}

/// The employer-side cost lines for one payslip, from the worker's place of work (WPM-R223, WPM-D159, WPM-D170): the total and the JSON
/// stored on the payslip. Nothing configured, or no place of work, is no lines; a class-keyed line with no class set is stored as
/// `{"name", "unresolved": true}` and costs nothing, so it is said and never read as zero cost. A setting that does not read refuses the
/// calculation, because skipping a line would understate cost.
fn employer_costs_for(
    profile: &super::jurisdictions::Profile,
    worker: &workers::Model,
    as_of: chrono::NaiveDate,
    gross_minor: i64,
    earlier_gross_minor: i64,
) -> std::result::Result<(i64, serde_json::Value), String> {
    use crate::rules::employer_contribution as ec;
    use crate::rules::jurisdiction::Resolution;
    let Some(place) = worker.work_jurisdiction.as_deref() else {
        return Ok((0, serde_json::json!([])));
    };
    let get = |key: &str| profile.resolve(key, Some(place), as_of);
    let Resolution::Resolved { value, .. } = get("employer_contribution.lines") else {
        return Ok((0, serde_json::json!([])));
    };
    let lines = ec::parse_lines(&value.as_names().unwrap_or_default())?;
    let class = match get("employer_contribution.class") {
        Resolution::Resolved { value, .. } => {
            value.as_names().and_then(|names| names.into_iter().next())
        }
        _ => None,
    };
    let outcome = ec::cost_lines(&lines, class.as_deref(), gross_minor, earlier_gross_minor)?;
    let total = outcome.total_minor()?;
    let mut shown = serde_json::to_value(&outcome.costs).map_err(|e| e.to_string())?;
    if let Some(list) = shown.as_array_mut() {
        list.extend(
            outcome
                .unresolved
                .iter()
                .map(|n| serde_json::json!({ "name": n, "unresolved": true })),
        );
    }
    Ok((total, shown))
}

/// What the overtime check says about itself on every answer (WPM-D157).
const OVERTIME_NOTE: &str = "Indicative only. Statutory overtime is derived from approved time, week by week, using the bands configured for the \
     worker's place of work, and only for a post a person recorded as not exempt. The premium is the extra over the base rate (the \
     monthly base over the minutes in a month). A calculated payslip includes this premium in gross (WPM-D169), so this check shows what a calculation adds. Which pay the law counts, a regular rate with bonuses, \
     rest days and holidays are not modelled, and a monthly band is not derived. Exemption is a person's entry, never worked out from duties.";

/// One worker's statutory-overtime outcome for a period (WPM-R219, WPM-D157, WPM-D162): the reason it could not be derived (if any),
/// the outcome name, and any extra fields (`overtime_minutes`, `minutes_at`, `premium_minor`, `currency`, or `below_exempt_floor`).
/// Shared by the advisory check and by the payroll calculation, so the two can never disagree. It reads nothing; every input is passed.
struct StatutoryOutcome {
    /// Why the answer is `unknown` or `not_derived`; `None` for every other outcome.
    reason: Option<&'static str>,
    /// `derived`, `exempt`, `unknown`, `no_limit_configured`, `none_by_law` or `not_derived`.
    name: &'static str,
    /// The figures of a derived or exempt row; an empty object otherwise.
    extra: serde_json::Value,
}

/// The observed dates of an organization's stored holidays between two dates, by region. A holiday is a day of the place it was recorded
/// for, never of another.
async fn holiday_dates(
    connection: &mut sqlx::PgConnection,
    organization: &str,
    from: chrono::NaiveDate,
    to: chrono::NaiveDate,
) -> Result<std::collections::BTreeMap<String, std::collections::BTreeSet<chrono::NaiveDate>>> {
    let mut found: std::collections::BTreeMap<
        String,
        std::collections::BTreeSet<chrono::NaiveDate>,
    > = std::collections::BTreeMap::new();
    for h in sqlx::query!(
        "SELECT region, observed_on FROM holidays WHERE organization_ref = $1 AND observed_on >= $2 AND observed_on <= $3",
        organization,
        from,
        to,
    )
    .fetch_all(connection)
    .await?
    {
        found.entry(h.region).or_default().insert(h.observed_on);
    }
    Ok(found)
}

/// What [`statutory_outcome`] reads besides the worker: the organization's resolved profile, the date it is resolved at, the currency,
/// the minutes in a pay month and the stored holidays by region.
#[derive(Clone, Copy)]
struct OutcomeContext<'a> {
    profile: &'a super::jurisdictions::Profile,
    as_of: chrono::NaiveDate,
    currency: &'a str,
    month_minutes: i64,
    holidays: &'a std::collections::BTreeMap<String, std::collections::BTreeSet<chrono::NaiveDate>>,
}

/// Derives [`StatutoryOutcome`] for one worker. `seats` is the overtime status recorded on each seat the worker holds, `on_days` the
/// approved minutes worked by day (on-call excluded), `as_of` the date the jurisdiction parameters are resolved at.
#[allow(clippy::too_many_lines)] // worker → place → seats → status → bands → period, linearly
fn statutory_outcome(
    worker: Option<&workers::Model>,
    seats: &[String],
    on_days: &std::collections::BTreeMap<chrono::NaiveDate, u32>,
    year_days: Option<&std::collections::BTreeMap<chrono::NaiveDate, u32>>,
    at: &OutcomeContext<'_>,
) -> StatutoryOutcome {
    use crate::rules::jurisdiction::Resolution;
    use crate::rules::statutory_overtime as ot;
    use serde_json::json;

    let OutcomeContext {
        profile,
        as_of,
        currency,
        month_minutes,
        holidays,
    } = *at;
    let unknown = |reason: &'static str| (reason, "unknown");
    let mut extra = json!({});
    let (reason, name): (Option<&'static str>, &'static str) = 'outcome: {
        let Some(worker) = worker else {
            let (r, n) = unknown("worker_not_found");
            break 'outcome (Some(r), n);
        };
        let Some(place) = worker.work_jurisdiction.as_deref() else {
            let (r, n) = unknown("no_work_jurisdiction");
            break 'outcome (Some(r), n);
        };
        let get = |key: &str| profile.resolve(key, Some(place), as_of);
        if matches!(
            get("overtime.none_by_law"),
            Resolution::Resolved { value, .. } if value.as_flag() == Some(true)
        ) {
            break 'outcome (None, "none_by_law");
        }
        let status = match seats {
            [] => {
                let (r, n) = unknown("no_position");
                break 'outcome (Some(r), n);
            }
            [first, rest @ ..] if rest.iter().all(|s| s == first) => {
                ot::Status::parse(first).unwrap_or(ot::Status::Unknown)
            }
            _ => {
                let (r, n) = unknown("positions_disagree");
                break 'outcome (Some(r), n);
            }
        };
        if status == ot::Status::Exempt {
            let below = ot::exempt_below_floor(
                worker.salary_minor.unwrap_or(0),
                worker.fte_percent,
                get("overtime.exempt_salary_floor_minor_weekly").integer(),
            );
            extra = json!({ "below_exempt_floor": below });
            break 'outcome (None, "exempt");
        }
        if status == ot::Status::Unknown {
            let (r, n) = unknown("status_not_recorded");
            break 'outcome (Some(r), n);
        }
        let bands = match get("overtime.bands") {
            Resolution::Resolved { value, .. } => {
                value.as_names().map(|names| ot::parse_premiums(&names))
            }
            _ => None,
        };
        let kinds = ot::Kinds {
            rest_day_bp: get("overtime.rest_day_multiplier_bp")
                .integer()
                .and_then(|n| u32::try_from(n).ok()),
            holiday_bp: get("overtime.holiday_multiplier_bp")
                .integer()
                .and_then(|n| u32::try_from(n).ok()),
        };
        let bands = match bands {
            None if kinds == ot::Kinds::default() => {
                break 'outcome (None, "no_limit_configured");
            }
            None => Vec::new(),
            Some(Err(_)) => {
                let (r, n) = unknown("bands_unreadable");
                break 'outcome (Some(r), n);
            }
            Some(Ok(bands)) => bands,
        };
        let rules = ot::Rules {
            bands,
            stacking: if matches!(
                get("overtime.premiums_add"),
                Resolution::Resolved { value, .. } if value.as_flag() == Some(true)
            ) {
                ot::Stacking::Add
            } else {
                ot::Stacking::Highest
            },
            kinds,
        };
        let week_start = get("calendar.week_start")
            .integer()
            .and_then(|d| u32::try_from(d).ok())
            .unwrap_or(1);
        // A day's kind comes from the place's stored holidays and its configured weekend. With no weekend configured no day is a
        // rest day: it is never guessed.
        let weekend = match get("holidays.weekend_days") {
            Resolution::Resolved { value, .. } => value
                .as_weekdays()
                .and_then(|days| crate::rules::holiday::weekend_from_iso(&days))
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        let kind_of = |date: chrono::NaiveDate| {
            if holidays.get(place).is_some_and(|set| set.contains(&date)) {
                ot::DayKind::Holiday
            } else if weekend.contains(&chrono::Datelike::weekday(&date)) {
                ot::DayKind::Rest
            } else {
                ot::DayKind::Ordinary
            }
        };
        match ot::derive_period_kinds(
            status,
            &rules,
            &ot::weeks_with(on_days, week_start, kind_of),
        ) {
            ot::Outcome::Derived(d) => {
                let base =
                    rules::monthly_base_minor(worker.salary_minor.unwrap_or(0), worker.fte_percent)
                        .ok();
                extra = json!({
                    "overtime_minutes": d.overtime_minutes(),
                    "minutes_at": d.minutes_at,
                    "premium_minor": base.and_then(|b| d.premium_minor(b, month_minutes)),
                    "currency": currency,
                });
                // The advisory caps: overtime in the calendar month up to the run's end and in the calendar year, over the weeks that
                // begin there. A cap that is not configured is absent, never "within".
                if let Some(year_days) = year_days {
                    use chrono::Datelike;
                    let month_start = as_of.with_day(1).unwrap_or(as_of);
                    let year_start =
                        chrono::NaiveDate::from_ymd_opt(as_of.year(), 1, 1).unwrap_or(as_of);
                    let used = |from: chrono::NaiveDate| {
                        let days = ot::weeks_beginning_between(year_days, week_start, from, as_of);
                        match ot::derive_period_kinds(
                            status,
                            &rules,
                            &ot::weeks_with(&days, week_start, kind_of),
                        ) {
                            ot::Outcome::Derived(d) => Some(d.overtime_minutes()),
                            _ => None,
                        }
                    };
                    let mut caps = serde_json::Map::new();
                    for (name, key, from) in [
                        ("monthly", "overtime.max_monthly_minutes", month_start),
                        ("annual", "overtime.max_annual_minutes", year_start),
                    ] {
                        let limit = get(key).integer();
                        if let (Some(limit), Some(minutes)) = (limit, used(from)) {
                            caps.insert(
                                name.to_string(),
                                json!({
                                    "limit_minutes": limit,
                                    "overtime_minutes": minutes,
                                    "over": ot::over_cap(minutes, Some(limit)),
                                }),
                            );
                        }
                    }
                    if let Some(object) = extra.as_object_mut() {
                        object.insert("caps".to_string(), serde_json::Value::Object(caps));
                    }
                }
                (None, "derived")
            }
            ot::Outcome::NotDerived(why) => (Some(why), "not_derived"),
            ot::Outcome::NoLimitConfigured => (None, "no_limit_configured"),
            ot::Outcome::Exempt => (None, "exempt"),
            ot::Outcome::Unknown => (Some("status_not_recorded"), "unknown"),
        }
    };
    StatutoryOutcome {
        reason,
        name,
        extra,
    }
}

/// `GET /api/payroll-runs/{pid}/overtime-check` — an advisory derivation of statutory overtime for each payslip's worker over the run's
/// period (WPM-R219, WPM-D157, WPM-D162). HR and payroll only, for organizations the caller holds. It changes no payslip. The bands are
/// jurisdiction parameters resolved from the worker's place of work; with none configured the answer is `no_limit_configured`, a
/// jurisdiction that sets none by law answers `none_by_law`, an exempt post answers `exempt` (with whether its salary is below the
/// configured weekly floor, or `null` where none is configured), and a post nobody recorded answers `unknown`, never "no overtime".
/// The audit entry holds counts, never names.
#[debug_handler]
#[allow(clippy::too_many_lines)] // payslips → workers → posts → time → rules → outcome, linearly
async fn overtime_check(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    use serde_json::json;
    use std::collections::BTreeMap;

    let run = find_run(&ctx, records::parse_pid(&pid)?).await?;
    super::ensure_org_in_scope(&ctx, &caller, &run.organization_ref).await?;
    let slips = live_payslips(&ctx, Some(run.pid), None).await?;
    let worker_pids: Vec<Uuid> = slips.iter().map(|slip| slip.worker_pid).collect();
    let people: BTreeMap<Uuid, workers::Model> = workers_by_pid(&ctx, &worker_pids).await?;
    // The status a person recorded on each seat a worker holds.
    let mut seats: BTreeMap<Uuid, Vec<String>> = BTreeMap::new();
    for seat in sqlx::query!(
        "SELECT holder_worker_pid, overtime_status FROM positions
          WHERE holder_worker_pid = ANY($1) AND state <> 'abolished' ORDER BY id",
        &worker_pids,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    {
        if let Some(holder) = seat.holder_worker_pid {
            seats.entry(holder).or_default().push(seat.overtime_status);
        }
    }
    // The caps count whole weeks from the start of the year, so the time and the holidays are read from a week before 1 January.
    let year_from = {
        use chrono::Datelike;
        chrono::NaiveDate::from_ymd_opt(run.period_end.year(), 1, 1).unwrap_or(run.period_start)
            - chrono::Duration::days(6)
    }
    .min(run.period_start);
    // Approved minutes by worker and day. On-call entries are not hours worked.
    let entries = sqlx::query!(
        "SELECT worker_pid, worked_on, minutes FROM time_entries
          WHERE worker_pid = ANY($1) AND status = 'approved' AND worked_on >= $2 AND worked_on <= $3
            AND kind <> 'on_call' AND deleted_at IS NULL",
        &worker_pids,
        year_from,
        run.period_end,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let mut days: BTreeMap<Uuid, BTreeMap<chrono::NaiveDate, u32>> = BTreeMap::new();
    let mut year_days: BTreeMap<Uuid, BTreeMap<chrono::NaiveDate, u32>> = BTreeMap::new();
    for entry in entries {
        let (worker, on) = (entry.worker_pid, entry.worked_on);
        let minutes = u32::try_from(entry.minutes.max(0)).unwrap_or(0);
        *year_days.entry(worker).or_default().entry(on).or_default() += minutes;
        if on >= run.period_start {
            *days.entry(worker).or_default().entry(on).or_default() += minutes;
        }
    }
    let profiles = super::jurisdictions::profiles_for(
        crate::db::pool(&ctx),
        std::slice::from_ref(&run.organization_ref),
    )
    .await?;
    let empty = super::jurisdictions::Profile::default();
    let profile = profiles.get(&run.organization_ref).unwrap_or(&empty);
    let as_of = run.period_end;
    let month_minutes = tax_stub().0.month_minutes;

    let mut counts: BTreeMap<&'static str, u64> = [
        "derived",
        "exempt",
        "unknown",
        "no_limit_configured",
        "none_by_law",
        "not_derived",
    ]
    .into_iter()
    .map(|name| (name, 0))
    .collect();
    let mut reasons: BTreeMap<&'static str, u64> = BTreeMap::new();
    let no_days = BTreeMap::new();
    let holidays = holiday_dates(
        &mut *read_connection(&ctx).await?,
        &run.organization_ref,
        year_from,
        run.period_end,
    )
    .await?;
    let mut rows = Vec::with_capacity(slips.len());
    for slip in &slips {
        let found = statutory_outcome(
            people.get(&slip.worker_pid),
            seats.get(&slip.worker_pid).map_or(&[][..], Vec::as_slice),
            days.get(&slip.worker_pid).unwrap_or(&no_days),
            Some(year_days.get(&slip.worker_pid).unwrap_or(&no_days)),
            &OutcomeContext {
                profile,
                as_of,
                currency: &slip.currency,
                month_minutes,
                holidays: &holidays,
            },
        );
        let (reason, name, extra) = (found.reason, found.name, found.extra);
        *counts.entry(name).or_default() += 1;
        if let Some(reason) = reason {
            *reasons.entry(reason).or_default() += 1;
        }
        let mut row = json!({ "worker_pid": slip.worker_pid, "outcome": name, "reason": reason });
        if let (Some(object), Some(more)) = (row.as_object_mut(), extra.as_object()) {
            object.extend(more.clone());
        }
        rows.push(row);
    }
    db::audit(
        &mut *read_connection(&ctx).await?,
        "payroll_run",
        run.pid,
        "overtime_checked",
        caller.actor(),
        Some(json!({ "payslips": slips.len(), "counts": counts, "reasons": reasons })),
    )
    .await?;
    format::json(json!({
        "run_pid": run.pid,
        "period_start": run.period_start,
        "period_end": run.period_end,
        "indicative": true,
        "note": OVERTIME_NOTE,
        "payslips": slips.len(),
        "counts": counts,
        "reasons": reasons,
        "workers": rows,
    }))
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
        .add(
            "/payroll-runs/{pid}/minimum-wage-check",
            get(minimum_wage_check),
        )
        .add("/payroll-runs/{pid}/overtime-check", get(overtime_check))
        .add("/workers/{pid}/payslips", get(worker_payslips))
        .add("/benchmarks", post(create_benchmark))
        .add("/benchmarks", get(list_benchmarks))
        .add("/benchmarks/comparison", get(benchmark_comparison))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A lookup over a fixed table, standing in for the environment.
    fn table(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
        let owned: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        move |name: &str| {
            owned
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone())
        }
    }

    fn stub(pairs: &[(&str, &str)]) -> (rules::TaxStub, bool) {
        tax_stub_from(&table(pairs))
    }

    fn day(pairs: &[(&str, &str)]) -> i32 {
        contracted_day_from(&table(pairs))
    }

    #[test]
    fn nothing_set_is_the_example() {
        assert_eq!(stub(&[]), (rules::TaxStub::example(), false));
        // Blank counts as unset.
        assert_eq!(
            stub(&[
                ("WPM_PAYROLL_STUB_CURRENCY", "  "),
                ("WPM_PAYROLL_STUB_RATE_PERCENT", "")
            ]),
            (rules::TaxStub::example(), false)
        );
    }

    #[test]
    fn a_whole_stub_in_another_currency_is_taken() {
        let (yen, configured) = stub(&[
            ("WPM_PAYROLL_STUB_CURRENCY", " JPY "),
            ("WPM_PAYROLL_STUB_ALLOWANCE_MINOR", "100000"),
            ("WPM_PAYROLL_STUB_RATE_PERCENT", "10"),
            ("WPM_PAYROLL_MONTH_MINUTES", "9600"),
        ]);
        assert!(configured);
        assert_eq!(
            yen,
            rules::TaxStub {
                currency: "JPY".to_string(),
                allowance_minor: 100_000,
                rate_percent: 10,
                month_minutes: 9_600,
            }
        );
    }

    #[test]
    fn another_currency_needs_its_own_allowance_and_rate() {
        // The example's pounds are never read as yen: with either missing, the example stays in force
        // and is not reported as configured.
        for pairs in [
            &[("WPM_PAYROLL_STUB_CURRENCY", "JPY")][..],
            &[
                ("WPM_PAYROLL_STUB_CURRENCY", "JPY"),
                ("WPM_PAYROLL_STUB_ALLOWANCE_MINOR", "100000"),
            ][..],
            &[
                ("WPM_PAYROLL_STUB_CURRENCY", "JPY"),
                ("WPM_PAYROLL_STUB_RATE_PERCENT", "10"),
            ][..],
            &[
                ("WPM_PAYROLL_STUB_CURRENCY", "JPY"),
                ("WPM_PAYROLL_STUB_ALLOWANCE_MINOR", "lots"),
                ("WPM_PAYROLL_STUB_RATE_PERCENT", "10"),
            ][..],
        ] {
            assert_eq!(stub(pairs), (rules::TaxStub::example(), false), "{pairs:?}");
        }
        // The month minutes are their own setting and still apply.
        let (s, configured) = stub(&[
            ("WPM_PAYROLL_STUB_CURRENCY", "JPY"),
            ("WPM_PAYROLL_MONTH_MINUTES", "9600"),
        ]);
        assert!(configured);
        assert_eq!(s.currency, "GBP");
        assert_eq!(s.month_minutes, 9_600);
    }

    #[test]
    fn the_examples_own_currency_may_change_one_figure() {
        let (s, configured) = stub(&[("WPM_PAYROLL_STUB_RATE_PERCENT", "25")]);
        assert!(configured);
        assert_eq!(
            s,
            rules::TaxStub {
                rate_percent: 25,
                ..rules::TaxStub::example()
            }
        );
        let (s, configured) = stub(&[
            ("WPM_PAYROLL_STUB_CURRENCY", "GBP"),
            ("WPM_PAYROLL_STUB_ALLOWANCE_MINOR", "0"),
        ]);
        assert!(configured);
        assert_eq!(s.allowance_minor, 0);
    }

    #[test]
    fn a_setting_that_does_not_check_is_never_half_applied() {
        for pairs in [
            // Not ISO 4217, or not upper case.
            &[
                ("WPM_PAYROLL_STUB_CURRENCY", "XYZ"),
                ("WPM_PAYROLL_STUB_ALLOWANCE_MINOR", "1"),
                ("WPM_PAYROLL_STUB_RATE_PERCENT", "1"),
            ][..],
            &[
                ("WPM_PAYROLL_STUB_CURRENCY", "jpy"),
                ("WPM_PAYROLL_STUB_ALLOWANCE_MINOR", "1"),
                ("WPM_PAYROLL_STUB_RATE_PERCENT", "1"),
            ][..],
            // Out of range: the whole stub falls back, the month minutes with it.
            &[
                ("WPM_PAYROLL_STUB_RATE_PERCENT", "101"),
                ("WPM_PAYROLL_MONTH_MINUTES", "9600"),
            ][..],
            &[("WPM_PAYROLL_STUB_ALLOWANCE_MINOR", "-1")][..],
            &[("WPM_PAYROLL_MONTH_MINUTES", "0")][..],
            &[("WPM_PAYROLL_MONTH_MINUTES", "99999999")][..],
        ] {
            assert_eq!(stub(pairs), (rules::TaxStub::example(), false), "{pairs:?}");
        }
        // Not a whole number: ignored, as if unset.
        assert_eq!(
            stub(&[("WPM_PAYROLL_STUB_RATE_PERCENT", "12.5")]),
            (rules::TaxStub::example(), false)
        );
    }

    #[test]
    fn the_contracted_day_is_one_to_a_full_day() {
        assert_eq!(day(&[]), workforce::CONTRACTED_DAY_MINUTES);
        assert_eq!(day(&[("WPM_CONTRACTED_DAY_MINUTES", " 480 ")]), 480);
        assert_eq!(day(&[("WPM_CONTRACTED_DAY_MINUTES", "1")]), 1);
        assert_eq!(day(&[("WPM_CONTRACTED_DAY_MINUTES", "1440")]), 1_440);
        for junk in ["0", "1441", "-5", "7.5h", "", "eight hours"] {
            assert_eq!(
                day(&[("WPM_CONTRACTED_DAY_MINUTES", junk)]),
                workforce::CONTRACTED_DAY_MINUTES,
                "{junk}"
            );
        }
    }
}
