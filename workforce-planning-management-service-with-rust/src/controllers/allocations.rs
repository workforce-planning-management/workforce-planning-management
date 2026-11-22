//! Per-person allocation (WPM-R275, WPM-R276, WPM-D211): a share of a worker's assignment planned for a programme, product or
//! service, month by month. What a person is planned to do, never what they did or how well: no effort, no hours, no score.
//!
//! - **Who reads named allocations:** the person, their line managers, HR, and resource managers in the person's organization
//!   (`allocation::read_access`). A source's owner would too where a deployer allows it, but WPM records no owner of a programme,
//!   so that relation is never established here. Every read of another person's allocations is audited (`allocations_read`).
//! - **Who writes:** HR and resource managers, a resource manager never their own (`allocation::may_write`). A resource manager's
//!   write passes the policy on the token role `wpm-resource-manager` (`self_service::ATTRIBUTE_WRITES`).
//! - **Tracking** is per organization (`capacity_settings.allocation_tracked`): untracked is unknown, never zero (WPM-D46).
//!
//! Spec: `spec/workforce-planning-and-scenarios.md`.

use crate::absent::AbsentIsEmpty as _;
use crate::sql_time::SqlDate;
use authentication_verifier::Claims;
use axa::extract::{Json, Path, Query};
use axa::prelude::*;
use jiff::civil::Date;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use uuid::Uuid;

use super::{ensure_org_in_scope, ensure_valid, ensure_within_ceiling, forbidden, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{allocations, workers};
use crate::models::{memberships, records};
use crate::rules::allocation as rules;
use crate::rules::assignment::Assignment;
use crate::rules::capacity::{month_start, months};

/// The longest note on an allocation.
const NOTE_MAX: usize = 500;
/// The default number of months a read covers.
const DEFAULT_MONTHS: u32 = 6;

// ─── Who is asking ──────────────────────────────────────────────────────────

fn truthy(claims: &Claims, key: &str) -> bool {
    claims
        .attrs
        .get(key)
        .is_some_and(|values| values.iter().any(|v| v == "true"))
}

/// HR for allocations: the `hr` token role, the service, an administrator, or an `hr_admin` membership of the organization.
/// Payroll is not HR here: it has no part in planning who works on what.
async fn is_hr(ctx: &AppContext, claims: &Claims, organization: &str) -> Result<bool> {
    Ok(truthy(claims, "hr")
        || truthy(claims, "svc")
        || claims
            .attrs
            .get("access")
            .is_some_and(|values| values.iter().any(|v| v == "admin"))
        || memberships::has_role_in(ctx, Some(claims), organization, &["hr_admin"]).await?)
}

/// A resource manager in the organization: the token role, or the membership role (WPM-R276).
async fn is_resource_manager(
    ctx: &AppContext,
    claims: &Claims,
    organization: &str,
) -> Result<bool> {
    Ok(truthy(claims, "resource_manager")
        || memberships::has_role_in(ctx, Some(claims), organization, &["resource_manager"]).await?)
}

fn person_id(person_ref: &str) -> &str {
    person_ref.split_once(':').map_or(person_ref, |(_, id)| id)
}

/// What the caller is to `worker`, for the allocation rules. With sign-in off, the caller is treated as HR, as everywhere.
async fn reader_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
) -> Result<rules::Reader> {
    if !auth::require_auth() {
        return Ok(rules::Reader {
            hr: true,
            ..rules::Reader::default()
        });
    }
    let Some(claims) = caller.claims() else {
        return Ok(rules::Reader::default());
    };
    Ok(rules::Reader {
        is_self: person_id(&worker.person_ref) == claims.sub,
        line_manager: crate::need_to_know::is_line_manager(ctx, worker, &claims.sub).await?,
        hr: is_hr(ctx, claims, &worker.organization_ref).await?,
        resource_manager: is_resource_manager(ctx, claims, &worker.organization_ref).await?,
        source_owner: false,
    })
}

/// What the caller is to an organization as a whole (for the lists): HR or a resource manager there, or neither.
async fn org_reader(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    organization: &str,
) -> Result<rules::Reader> {
    if !auth::require_auth() {
        return Ok(rules::Reader {
            hr: true,
            ..rules::Reader::default()
        });
    }
    let Some(claims) = caller.claims() else {
        return Ok(rules::Reader::default());
    };
    Ok(rules::Reader {
        hr: is_hr(ctx, claims, organization).await?,
        resource_manager: is_resource_manager(ctx, claims, organization).await?,
        ..rules::Reader::default()
    })
}

// ─── Loading ────────────────────────────────────────────────────────────────

fn status_of(row: &allocations::Model) -> Result<rules::Status> {
    rules::Status::parse(&row.status)
        .ok_or_else(|| super::internal("an allocation status the code does not write"))
}

fn as_rule(row: &allocations::Model) -> Result<rules::Allocation> {
    Ok(rules::Allocation {
        source: row.source_ref.clone(),
        pool: row.pool_pid,
        from_month: row.from_month.get(),
        to_month: row.to_month.get(),
        fte_centi: i64::from(row.fte_centi),
        status: status_of(row)?,
    })
}

async fn allocation_by_pid(ctx: &AppContext, pid: Uuid) -> Result<allocations::Model> {
    sqlx::query_as!(
        allocations::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, assignment_pid,
                  organization_ref, source_ref, pool_pid, from_month, to_month, fte_centi, status, note, recorded_by,
                  deleted_at AS "deleted_at: _"
             FROM allocations WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// One assignment as the rules see it, with its pid.
struct AssignmentRow {
    pid: Uuid,
    worker_pid: Uuid,
    rule: Assignment,
}

async fn assignments_of(ctx: &AppContext, workers: &[Uuid]) -> Result<Vec<AssignmentRow>> {
    Ok(sqlx::query!(
        "SELECT pid, worker_pid, fte_percent, is_primary, starts_on, ends_on FROM assignments
          WHERE worker_pid = ANY($1) ORDER BY starts_on, id",
        workers,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
    .map(|a| AssignmentRow {
        pid: a.pid,
        worker_pid: a.worker_pid,
        rule: Assignment {
            fte_percent: a.fte_percent,
            starts_on: a.starts_on.get(),
            ends_on: a.ends_on.map(SqlDate::get),
            primary: a.is_primary,
        },
    })
    .collect())
}

async fn allocations_of(ctx: &AppContext, workers: &[Uuid]) -> Result<Vec<allocations::Model>> {
    Ok(sqlx::query_as!(
        allocations::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, assignment_pid,
                  organization_ref, source_ref, pool_pid, from_month, to_month, fte_centi, status, note, recorded_by,
                  deleted_at AS "deleted_at: _"
             FROM allocations WHERE worker_pid = ANY($1) AND deleted_at IS NULL ORDER BY from_month, id"#,
        workers,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// Approved leave as date ranges, by worker. The kind of leave is not selected.
async fn leave_of(ctx: &AppContext, workers: &[Uuid]) -> Result<HashMap<Uuid, Vec<(Date, Date)>>> {
    let mut out: HashMap<Uuid, Vec<(Date, Date)>> = HashMap::new();
    for row in sqlx::query!(
        "SELECT worker_pid, start_on, end_on FROM leave_requests
          WHERE worker_pid = ANY($1) AND status = 'approved' AND deleted_at IS NULL",
        workers,
    )
    .fetch_all(db::pool(ctx))
    .await?
    {
        out.entry(row.worker_pid)
            .or_default()
            .push((row.start_on.get(), row.end_on.get()));
    }
    Ok(out)
}

/// Whether the organization records allocation.
async fn tracking_of(ctx: &AppContext, organization: &str) -> Result<rules::Tracking> {
    let tracked = sqlx::query_scalar!(
        "SELECT allocation_tracked FROM capacity_settings WHERE organization_ref = $1",
        organization,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .unwrap_or(false);
    Ok(if tracked {
        rules::Tracking::Tracked
    } else {
        rules::Tracking::Untracked
    })
}

#[derive(Debug, Deserialize)]
struct Window {
    from: Option<Date>,
    months: Option<u32>,
}

impl Window {
    /// The months of the window. `capacity::months` refuses a count of 0 or more than the horizon.
    fn months(&self) -> Result<Vec<Date>> {
        let count = self.months.unwrap_or(DEFAULT_MONTHS);
        months(self.from.unwrap_or_else(crate::dates::today), count).map_err(|m| unprocessable(&m))
    }

    /// The months, and the first and last of them.
    fn bounds(&self) -> Result<(Vec<Date>, Date, Date)> {
        let span = self.months()?;
        // `capacity::months` gives at least one month, or an error above.
        let (first, last) = span
            .first()
            .zip(span.last())
            .map(|(first, last)| (*first, *last))
            .ok_or_else(|| unprocessable("the window has no months"))?;
        Ok((span, first, last))
    }
}

// ─── Rendering ──────────────────────────────────────────────────────────────

fn signal_json(signal: &rules::Signal, holdings: &[Uuid]) -> Value {
    match signal {
        rules::Signal::OverAllocated { by_centi } => {
            json!({ "kind": "over_allocated", "by_centi": by_centi })
        }
        rules::Signal::LeaveClash { short_centi } => {
            json!({ "kind": "leave_clash", "short_centi": short_centi })
        }
        rules::Signal::Available { free_centi } => {
            json!({ "kind": "available", "free_centi": free_centi })
        }
        rules::Signal::AssignmentOverAllocated { holding, by_centi } => json!({
            "kind": "assignment_over_allocated",
            "assignment_pid": holdings.get(*holding),
            "by_centi": by_centi,
        }),
    }
}

/// The allocations, their signals and the month standings for one worker.
fn worker_json(
    worker: &workers::Model,
    assignments: &[&AssignmentRow],
    rows: &[&allocations::Model],
    leave: Vec<(Date, Date)>,
    tracking: rules::Tracking,
    window: &[Date],
) -> Result<Value> {
    let engagement_ends_on = worker.engagement_ends_on.map(SqlDate::get);
    let mut holdings = Vec::new();
    let mut holding_pids = Vec::new();
    for a in assignments {
        let mut list = Vec::new();
        for row in rows.iter().filter(|r| r.assignment_pid == a.pid) {
            list.push(as_rule(row)?);
        }
        holdings.push(rules::Holding {
            assignment: a.rule.clone(),
            allocations: list,
        });
        holding_pids.push(a.pid);
    }
    let person = rules::Person {
        holdings,
        engagement_ends_on,
        approved_leave: leave,
    };
    let mut listed = Vec::new();
    for row in rows {
        let past = assignments
            .iter()
            .find(|a| a.pid == row.assignment_pid)
            .and_then(|a| rules::runs_past_end(&as_rule(row).ok()?, &a.rule, engagement_ends_on));
        listed.push(json!({
            "pid": row.pid,
            "assignment_pid": row.assignment_pid,
            "source_ref": row.source_ref,
            "pool_pid": row.pool_pid,
            "from_month": row.from_month,
            "to_month": row.to_month,
            "fte_centi": row.fte_centi,
            "status": row.status,
            "note": row.note,
            "runs_past_end": past,
        }));
    }
    let months: Vec<Value> = window
        .iter()
        .map(|month| {
            match rules::person_month(&person, *month, tracking, rules::DEFAULT_AVAILABLE_BELOW_BP) {
                rules::MonthStanding::Unknown => json!({ "month": month, "standing": "unknown" }),
                rules::MonthStanding::Known(m) => json!({
                    "month": m.month,
                    "standing": "known",
                    "contracted_centi": m.totals.contracted_centi,
                    "available_centi": m.totals.available_centi,
                    "allocated_centi": m.totals.allocated_centi,
                    "tentative_centi": m.totals.tentative_centi,
                    "signals": m.signals.iter().map(|s| signal_json(s, &holding_pids)).collect::<Vec<_>>(),
                }),
            }
        })
        .collect();
    Ok(json!({
        "worker_pid": worker.pid,
        "display_name": worker.display_name,
        "allocations": listed,
        "months": months,
    }))
}

async fn audit_read(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    entity_pid: Uuid,
    snapshot: Value,
) -> Result<()> {
    db::audit(
        &mut *db::pool(ctx).acquire().await?,
        "allocation",
        entity_pid,
        "allocations_read",
        caller.actor(),
        Some(snapshot),
    )
    .await?;
    Ok(())
}

// ─── Reads ──────────────────────────────────────────────────────────────────

/// `GET /api/workers/{pid}/allocations?from=&months=` — one person's allocations with each month's standing. The person, their line
/// managers, HR and resource managers; another person's read is audited.
#[debug_handler]
async fn of_worker(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Query(window): Query<Window>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    ensure_org_in_scope(&ctx, &caller, &worker.organization_ref).await?;
    let reader = reader_for(&ctx, &caller, &worker).await?;
    if rules::read_access(reader, false) != rules::Read::Named {
        return Err(forbidden(
            "allocations are read by the person, their line managers, HR and resource managers",
        ));
    }
    let span = window.months()?;
    let ids = [worker.pid];
    let assignments = assignments_of(&ctx, &ids).await?;
    let rows = allocations_of(&ctx, &ids).await?;
    let mut leave = leave_of(&ctx, &ids).await?;
    let tracking = tracking_of(&ctx, &worker.organization_ref).await?;
    let body = worker_json(
        &worker,
        &assignments.iter().collect::<Vec<_>>(),
        &rows.iter().collect::<Vec<_>>(),
        leave.remove(&worker.pid).or_empty(),
        tracking,
        &span,
    )?;
    if rules::named_read_is_audited(reader, false) {
        audit_read(
            &ctx,
            &caller,
            worker.pid,
            json!({ "worker_pid": worker.pid, "count": rows.len() }),
        )
        .await?;
    }
    let mut body = body;
    if let Some(object) = body.as_object_mut() {
        object.insert(
            "tracked".into(),
            json!(tracking == rules::Tracking::Tracked),
        );
        object.insert("derivation".into(), json!(rules::DERIVATION));
    }
    format::json(body)
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    organization: String,
    source: Option<String>,
    from: Option<Date>,
    months: Option<u32>,
}

/// `GET /api/allocations?organization=&source=&from=&months=` — the people of an organization with their allocations and month
/// standings, by name, for HR and resource managers. With `source`, only the people allocated to it in the window. Paged; the read
/// is audited with the count on the page.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    page: super::Page,
    Query(q): Query<ListQuery>,
) -> Result<Response> {
    ensure_org_in_scope(&ctx, &caller, &q.organization).await?;
    let reader = org_reader(&ctx, &caller, &q.organization).await?;
    if !(reader.hr || reader.resource_manager) {
        return Err(forbidden("the named list is for HR and resource managers"));
    }
    let window = super::window(&page)?;
    let (span, first, last) = Window {
        from: q.from,
        months: q.months,
    }
    .bounds()?;
    let people = sqlx::query_as!(
        workers::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, person_ref,
                  upstream_worker_ref, organization_ref, worker_number, display_name, status, employment_type,
                  fte_percent, department, job_title, location, work_jurisdiction, manager_pid, salary_minor,
                  salary_currency, hired_on, terminated_on, engagement_ends_on, deleted_at AS "deleted_at: _"
             FROM workers WHERE organization_ref = $1 AND deleted_at IS NULL
            ORDER BY display_name, pid"#,
        q.organization,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    // A count past `u64` is over any ceiling (WPM-R120), so it is refused like one.
    ensure_within_ceiling(u64::try_from(people.len()).unwrap_or(u64::MAX))?;
    // Employed at some point in the window: started by its last day and not left before its first.
    let window_end = last.last_of_month();
    let people: Vec<workers::Model> = people
        .into_iter()
        .filter(|w| {
            w.hired_on.get() <= window_end && w.terminated_on.is_none_or(|t| t.get() > first)
        })
        .collect();
    let ids: Vec<Uuid> = people.iter().map(|w| w.pid).collect();
    let assignments = assignments_of(&ctx, &ids).await?;
    let rows = allocations_of(&ctx, &ids).await?;
    let mut leave = leave_of(&ctx, &ids).await?;
    let tracking = tracking_of(&ctx, &q.organization).await?;
    let wanted: Option<BTreeSet<Uuid>> = q.source.as_ref().map(|source| {
        rows.iter()
            .filter(|r| {
                &r.source_ref == source
                    && r.from_month.get() <= last
                    && r.to_month.get() >= month_start(first)
            })
            .map(|r| r.worker_pid)
            .collect()
    });
    let kept: Vec<&workers::Model> = people
        .iter()
        .filter(|w| wanted.as_ref().is_none_or(|set| set.contains(&w.pid)))
        .collect();
    let (page_rows, more) = window.split(
        kept.into_iter()
            .skip(usize::try_from(window.offset()).unwrap_or(usize::MAX))
            .take(super::rows_to_read(window.limit()))
            .collect::<Vec<_>>(),
    );
    let mut out = Vec::new();
    for worker in &page_rows {
        let mine: Vec<&AssignmentRow> = assignments
            .iter()
            .filter(|a| a.worker_pid == worker.pid)
            .collect();
        let theirs: Vec<&allocations::Model> =
            rows.iter().filter(|r| r.worker_pid == worker.pid).collect();
        out.push(worker_json(
            worker,
            &mine,
            &theirs,
            leave.remove(&worker.pid).or_empty(),
            tracking,
            &span,
        )?);
    }
    audit_read(
        &ctx,
        &caller,
        Uuid::new_v4(),
        json!({ "organization": q.organization, "count": out.len() }),
    )
    .await?;
    Ok(super::with_next(
        axum::Json(out).into_response(),
        &window,
        more,
    ))
}

/// A summary cell: pool (none where the allocation names none), source, month.
type CellKey = (Option<Uuid>, String, Date);

#[derive(Debug, Deserialize)]
struct SummaryQuery {
    organization: String,
    from: Option<Date>,
    months: Option<u32>,
}

/// `GET /api/allocations/summary?organization=&from=&months=` — per pool, source and month: the FTE and the number of people
/// confirmed against it, the active claim, and how the two compare. Names no one. For a caller who may not read named allocations a
/// cell with fewer people than the floor withholds its figures; the claim is always shown. Untracked is unknown, never zero.
#[debug_handler]
async fn summary(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Query(q): Query<SummaryQuery>,
) -> Result<Response> {
    ensure_org_in_scope(&ctx, &caller, &q.organization).await?;
    let reader = org_reader(&ctx, &caller, &q.organization).await?;
    let exact = rules::read_access(reader, false) == rules::Read::Named;
    let (span, first, last) = Window {
        from: q.from,
        months: q.months,
    }
    .bounds()?;
    let tracking = tracking_of(&ctx, &q.organization).await?;
    let floor = crate::rules::equality_monitoring::DEFAULT_FLOOR;
    // (pool, source, month) -> (fte, people)
    let mut cells: BTreeMap<CellKey, (i64, BTreeSet<Uuid>)> = BTreeMap::new();
    let rows = sqlx::query!(
        "SELECT a.worker_pid, a.pool_pid, a.source_ref, a.from_month, a.to_month, a.fte_centi
           FROM allocations a JOIN workers w ON w.pid = a.worker_pid AND w.deleted_at IS NULL
          WHERE a.organization_ref = $1 AND a.deleted_at IS NULL AND a.status = 'confirmed'",
        q.organization,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    for row in &rows {
        for month in &span {
            if row.from_month.get() <= *month && *month <= row.to_month.get() {
                let cell = cells
                    .entry((row.pool_pid, row.source_ref.clone(), *month))
                    .or_default();
                cell.0 = cell
                    .0
                    .checked_add(i64::from(row.fte_centi))
                    .ok_or_else(|| super::internal("allocation FTE total overflowed"))?;
                cell.1.insert(row.worker_pid);
            }
        }
    }
    let mut claims: BTreeMap<CellKey, i64> = BTreeMap::new();
    for row in sqlx::query!(
        "SELECT d.pool_pid, d.programme_ref, d.month, d.fte_centi
           FROM programme_demands d JOIN skill_pools p ON p.pid = d.pool_pid AND p.deleted_at IS NULL
          WHERE p.organization_ref = $1 AND d.deleted_at IS NULL AND d.status = 'active'
            AND d.month BETWEEN $2 AND $3",
        q.organization,
        SqlDate(first),
        SqlDate(last),
    )
    .fetch_all(db::pool(&ctx))
    .await?
    {
        let claim = claims
            .entry((Some(row.pool_pid), row.programme_ref, row.month.get()))
            .or_default();
        *claim = claim
            .checked_add(row.fte_centi)
            .ok_or_else(|| super::internal("programme demand FTE total overflowed"))?;
    }
    let keys: BTreeSet<CellKey> = cells
        .keys()
        .cloned()
        .chain(claims.keys().cloned())
        .collect();
    let mut out = Vec::new();
    for key in keys {
        let (fte, people) = cells.get(&key).map_or((0, 0), |(f, p)| (*f, p.len()));
        let claim = claims.get(&key).copied();
        let withheld = !exact && people > 0 && people < floor;
        let staffing = if withheld {
            json!("withheld")
        } else {
            staffing_json(rules::reconcile(tracking, claim, fte))
        };
        let known = tracking == rules::Tracking::Tracked && !withheld;
        out.push(json!({
            "pool_pid": key.0,
            "source_ref": key.1,
            "month": key.2,
            "claim_centi": claim,
            "allocated_centi": if known { json!(fte) } else { Value::Null },
            "people": if known { json!(people) } else { Value::Null },
            "staffing": staffing,
        }));
    }
    format::json(json!({
        "organization": q.organization,
        "tracked": tracking == rules::Tracking::Tracked,
        "floor": floor,
        "cells": out,
    }))
}

fn staffing_json(s: rules::Staffing) -> Value {
    match s {
        rules::Staffing::Unknown => json!("unknown"),
        rules::Staffing::NotClaimed { .. } => json!("not_claimed"),
        rules::Staffing::Staffed => json!("staffed"),
        rules::Staffing::Understaffed { by_centi } => json!({ "understaffed_by_centi": by_centi }),
        rules::Staffing::Overstaffed { by_centi } => json!({ "overstaffed_by_centi": by_centi }),
    }
}

// ─── Writes ─────────────────────────────────────────────────────────────────

/// The worker for a write, in the caller's scope, and that the caller may write their allocations.
async fn writable(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker_pid: Uuid,
) -> Result<workers::Model> {
    let worker = records::worker_by_pid(ctx, worker_pid).await?;
    ensure_org_in_scope(ctx, caller, &worker.organization_ref).await?;
    let reader = reader_for(ctx, caller, &worker).await?;
    if !rules::may_write(reader) {
        return Err(forbidden(
            "allocations are written by HR and resource managers, and not by a resource manager for themselves",
        ));
    }
    Ok(worker)
}

fn clean_note(note: Option<String>) -> Result<Option<String>> {
    let note = note.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    if note.as_ref().is_some_and(|n| n.chars().count() > NOTE_MAX) {
        return Err(unprocessable(&format!(
            "note is at most {NOTE_MAX} characters"
        )));
    }
    Ok(note)
}

async fn ensure_pool(ctx: &AppContext, pool: Option<Uuid>, organization: &str) -> Result<()> {
    let Some(pool) = pool else { return Ok(()) };
    let found = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!" FROM skill_pools WHERE pid = $1 AND organization_ref = $2 AND deleted_at IS NULL"#,
        pool,
        organization,
    )
    .fetch_one(db::pool(ctx))
    .await?;
    if found == 0 {
        return Err(unprocessable(
            "pool_pid is not a skill pool of this organization",
        ));
    }
    Ok(())
}

/// The worker's assignment `pid`, or a `422`.
async fn assignment_of(ctx: &AppContext, worker: Uuid, pid: Uuid) -> Result<Assignment> {
    assignments_of(ctx, &[worker])
        .await?
        .into_iter()
        .find(|a| a.pid == pid)
        .map(|a| a.rule)
        .ok_or_else(|| unprocessable("assignment_pid is not one of this worker's assignments"))
}

fn fte_i32(fte: i64) -> Result<i32> {
    i32::try_from(fte).map_err(|_| unprocessable("fte_centi is out of range"))
}

async fn tell(
    tx: &mut sqlx::PgConnection,
    worker: Uuid,
    kind: &str,
    allocation: Uuid,
    source: &str,
    from: Date,
    to: Date,
) -> Result<()> {
    let body = if kind == "allocation_confirmed" {
        format!("An allocation of your time was confirmed: {source}, {from} to {to}.")
    } else {
        format!("An allocation of your time was changed: {source}.")
    };
    db::notify(
        tx,
        worker,
        kind,
        &body,
        json!({ "allocation_pid": allocation }),
    )
    .await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct CreatePayload {
    assignment_pid: Uuid,
    source_ref: String,
    pool_pid: Option<Uuid>,
    from_month: Date,
    to_month: Date,
    fte_centi: i64,
    status: Option<String>,
    note: Option<String>,
}

/// `POST /api/workers/{pid}/allocations` — allocate a share of one of the worker's assignments. Tentative unless `status` is
/// `confirmed`; a confirmed one tells the person. HR and resource managers.
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(p): Json<CreatePayload>,
) -> Result<Response> {
    let worker = writable(&ctx, &caller, pid).await?;
    let status = match p.status.as_deref() {
        None | Some("tentative") => rules::Status::Tentative,
        Some("confirmed") => rules::Status::Confirmed,
        Some(_) => return Err(unprocessable("status must be tentative or confirmed")),
    };
    let assignment = assignment_of(&ctx, worker.pid, p.assignment_pid).await?;
    let allocation = rules::Allocation {
        source: p.source_ref.trim().to_string(),
        pool: p.pool_pid,
        from_month: p.from_month,
        to_month: p.to_month,
        fte_centi: p.fte_centi,
        status,
    };
    ensure_valid(&rules::allocation_problems(&allocation, &assignment))?;
    ensure_pool(&ctx, p.pool_pid, &worker.organization_ref).await?;
    let note = clean_note(p.note)?;
    let fte = fte_i32(allocation.fte_centi)?;
    let new = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO allocations (pid, worker_pid, assignment_pid, organization_ref, source_ref, pool_pid, from_month, to_month,
                                  fte_centi, status, note, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        new,
        worker.pid,
        p.assignment_pid,
        worker.organization_ref,
        allocation.source,
        allocation.pool,
        SqlDate(allocation.from_month),
        SqlDate(allocation.to_month),
        fte,
        status.as_str(),
        note,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "allocation",
        new,
        "allocation_created",
        caller.actor(),
        Some(json!({ "worker_pid": worker.pid, "status": status.as_str() })),
    )
    .await?;
    if status == rules::Status::Confirmed {
        tell(
            &mut tx,
            worker.pid,
            "allocation_confirmed",
            new,
            &allocation.source,
            allocation.from_month,
            allocation.to_month,
        )
        .await?;
    }
    tx.commit().await?;
    format::json(json!({ "pid": new }))
}

#[derive(Debug, Deserialize)]
struct ChangePayload {
    source_ref: Option<String>,
    pool_pid: Option<Uuid>,
    from_month: Option<Date>,
    to_month: Option<Date>,
    fte_centi: Option<i64>,
    note: Option<String>,
}

/// `PUT /api/allocations/{pid}` — change what is given and keep the rest. An ended allocation is not changed. A confirmed one
/// tells the person.
#[debug_handler]
async fn change(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(p): Json<ChangePayload>,
) -> Result<Response> {
    let row = allocation_by_pid(&ctx, pid).await?;
    let worker = writable(&ctx, &caller, row.worker_pid).await?;
    let old = as_rule(&row)?;
    if old.status == rules::Status::Ended {
        return Err(unprocessable(
            "an ended allocation is not changed; make a new one",
        ));
    }
    let assignment = assignment_of(&ctx, worker.pid, row.assignment_pid).await?;
    let new = rules::Allocation {
        source: p
            .source_ref
            .map_or_else(|| old.source.clone(), |s| s.trim().to_string()),
        pool: p.pool_pid.or(old.pool),
        from_month: p.from_month.unwrap_or(old.from_month),
        to_month: p.to_month.unwrap_or(old.to_month),
        fte_centi: p.fte_centi.unwrap_or(old.fte_centi),
        status: old.status,
    };
    ensure_valid(&rules::allocation_problems(&new, &assignment))?;
    ensure_pool(&ctx, new.pool, &worker.organization_ref).await?;
    let note = match p.note {
        Some(n) => clean_note(Some(n))?,
        None => row.note.clone(),
    };
    let fte = fte_i32(new.fte_centi)?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE allocations SET source_ref = $2, pool_pid = $3, from_month = $4, to_month = $5, fte_centi = $6, note = $7,
                updated_at = CURRENT_TIMESTAMP
          WHERE id = $1",
        row.id,
        new.source,
        new.pool,
        SqlDate(new.from_month),
        SqlDate(new.to_month),
        fte,
        note,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "allocation",
        row.pid,
        "allocation_changed",
        caller.actor(),
        Some(json!({ "worker_pid": worker.pid })),
    )
    .await?;
    if old.status == rules::Status::Confirmed {
        tell(
            &mut tx,
            worker.pid,
            "allocation_changed",
            row.pid,
            &new.source,
            new.from_month,
            new.to_month,
        )
        .await?;
    }
    tx.commit().await?;
    format::json(json!({ "pid": row.pid }))
}

#[derive(Debug, Deserialize)]
struct StatusPayload {
    to: String,
}

/// `POST /api/allocations/{pid}/status` — tentative and confirmed move either way; either can end; an ended one is final.
#[debug_handler]
async fn set_status(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(p): Json<StatusPayload>,
) -> Result<Response> {
    let row = allocation_by_pid(&ctx, pid).await?;
    let worker = writable(&ctx, &caller, row.worker_pid).await?;
    let from = status_of(&row)?;
    let to = rules::Status::parse(&p.to)
        .ok_or_else(|| unprocessable("to must be tentative, confirmed or ended"))?;
    if !from.can_move_to(to) {
        return Err(unprocessable(&format!(
            "an allocation that is {} cannot become {}",
            from.as_str(),
            to.as_str()
        )));
    }
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE allocations SET status = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $1",
        row.id,
        to.as_str(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "allocation",
        row.pid,
        "allocation_status_changed",
        caller.actor(),
        Some(json!({ "worker_pid": worker.pid, "from": from.as_str(), "to": to.as_str() })),
    )
    .await?;
    let kind = match (from, to) {
        (_, rules::Status::Confirmed) => Some("allocation_confirmed"),
        (rules::Status::Confirmed, _) => Some("allocation_changed"),
        _ => None,
    };
    if let Some(kind) = kind {
        tell(
            &mut tx,
            worker.pid,
            kind,
            row.pid,
            &row.source_ref,
            row.from_month.get(),
            row.to_month.get(),
        )
        .await?;
    }
    tx.commit().await?;
    format::json(json!({ "pid": row.pid, "status": to.as_str() }))
}

/// `DELETE /api/allocations/{pid}` — remove an allocation made in error (soft delete). A confirmed one tells the person.
#[debug_handler]
async fn remove(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let row = allocation_by_pid(&ctx, pid).await?;
    let worker = writable(&ctx, &caller, row.worker_pid).await?;
    let was = status_of(&row)?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE allocations SET deleted_at = now() WHERE id = $1",
        row.id,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "allocation",
        row.pid,
        "allocation_deleted",
        caller.actor(),
        Some(json!({ "worker_pid": worker.pid })),
    )
    .await?;
    if was == rules::Status::Confirmed {
        tell(
            &mut tx,
            worker.pid,
            "allocation_changed",
            row.pid,
            &row.source_ref,
            row.from_month.get(),
            row.to_month.get(),
        )
        .await?;
    }
    tx.commit().await?;
    format::json(json!({ "pid": row.pid, "deleted": true }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/allocations", get(list))
        .add("/allocations/summary", get(summary))
        .add("/workers/{pid}/allocations", get(of_worker))
        .add("/workers/{pid}/allocations", post(create))
        .add("/allocations/{pid}", put(change))
        .add("/allocations/{pid}", delete(remove))
        .add("/allocations/{pid}/status", post(set_status))
}
