//! Workforce management (WPM-R4–R6): time & attendance, leave, and
//! shift scheduling. Approval races serialize on the request row
//! (`FOR UPDATE`, WPM-D9); balances change in the decision's
//! transaction.

use super::row_json;
use axa::extract::{Json, Path, Query};
use axa::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::metrics::Metrics;
use crate::models::_entities::{
    leave_entitlements, leave_requests, shift_assignments, shifts, time_entries,
};
use crate::models::records;
use crate::rules::{leave, lifecycle, tokens, workforce, working_time};
use crate::streaming;
use crate::validation::Problems;

/// `POST /api/workers/{pid}/time-entries` body.
#[derive(Debug, Deserialize)]
struct TimeEntryPayload {
    worked_on: chrono::NaiveDate,
    minutes: i32,
    #[serde(default = "default_kind")]
    kind: String,
    #[serde(default)]
    notes: Option<String>,
}

/// `POST /api/workers/{pid}/leave-entitlements` body.
#[derive(Debug, Deserialize)]
struct EntitlementPayload {
    kind: String,
    year: i32,
    entitled_days: i32,
}

/// `POST /api/workers/{pid}/leave-requests` body.
#[derive(Debug, Deserialize)]
pub(crate) struct LeaveRequestPayload {
    kind: String,
    start_on: chrono::NaiveDate,
    end_on: chrono::NaiveDate,
    #[serde(default)]
    reason: Option<String>,
}

/// `POST /api/shifts` body.
#[derive(Debug, Deserialize)]
struct ShiftPayload {
    department: String,
    starts_at: chrono::DateTime<chrono::FixedOffset>,
    ends_at: chrono::DateTime<chrono::FixedOffset>,
    #[serde(default = "default_headcount")]
    required_headcount: i32,
}

/// `POST /api/shifts/{pid}/assignments` body.
#[derive(Debug, Deserialize)]
struct AssignmentPayload {
    worker_pid: Uuid,
}

/// A `{pid}` reference response.
#[derive(Debug, Serialize)]
struct PidRef {
    pid: String,
}

fn default_kind() -> String {
    "regular".to_string()
}
const fn default_headcount() -> i32 {
    1
}

/// `POST /api/workers/{pid}/time-entries` — record time; the day
/// total is capped at 24 h (WPM-R4).
#[debug_handler]
async fn create_time_entry(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<TimeEntryPayload>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let mut problems = Problems::new();
    problems.require_token("kind", tokens::TIME_KINDS, &payload.kind);
    problems.cap_opt("notes", payload.notes.as_deref());
    ensure_valid(&problems.into_vec())?;
    let existing: Vec<i32> = sqlx::query_scalar!(
        "SELECT minutes FROM time_entries WHERE worker_pid = $1 AND worked_on = $2 AND deleted_at IS NULL",
        worker.pid,
        payload.worked_on,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let existing_minutes: i32 = existing.iter().sum();
    workforce::check_day_minutes(existing_minutes, payload.minutes)
        .map_err(|e| unprocessable(&e))?;
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO time_entries (pid, worker_pid, worked_on, minutes, kind, status, notes)
         VALUES ($1, $2, $3, $4, $5, 'recorded', $6)",
        pid,
        worker.pid,
        payload.worked_on,
        payload.minutes,
        payload.kind,
        payload.notes,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "time_entry",
        pid,
        "time_recorded",
        caller.actor(),
        None,
    )
    .await?;
    streaming::emit_in(
        &mut tx,
        "time_entry",
        "time_recorded",
        &pid.to_string(),
        &worker.worker_number,
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: pid.to_string(),
    })
}

/// `GET /api/workers/{pid}/time-entries?from=&to=` — entries plus
/// the derived per-day overtime (WPM-R4).
#[derive(Debug, Deserialize)]
struct TimeListParams {
    #[serde(default)]
    from: Option<chrono::NaiveDate>,
    #[serde(default)]
    to: Option<chrono::NaiveDate>,
}

#[debug_handler]
async fn list_time_entries(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
    Query(params): Query<TimeListParams>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let rows = sqlx::query_as!(
        time_entries::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, worked_on, minutes,
                  kind, status, notes, deleted_at AS "deleted_at: _"
             FROM time_entries
            WHERE worker_pid = $1 AND deleted_at IS NULL AND ($2::date IS NULL OR worked_on >= $2)
              AND ($3::date IS NULL OR worked_on <= $3)
            ORDER BY worked_on, id
            LIMIT 1000"#,
        worker.pid,
        params.from,
        params.to,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    // Per-day overtime derivation over the returned window, against the deployment's day (WPM-T383).
    let full_day = super::payroll::contracted_day();
    let mut days: std::collections::BTreeMap<chrono::NaiveDate, (i32, i32)> =
        std::collections::BTreeMap::new();
    for entry in &rows {
        let slot = days.entry(entry.worked_on).or_default();
        if entry.kind == "overtime" {
            slot.1 += entry.minutes;
        } else {
            slot.0 += entry.minutes;
        }
    }
    let overtime: Vec<serde_json::Value> = days
        .iter()
        .map(|(day, (regular, explicit))| {
            serde_json::json!({
                "worked_on": day,
                "overtime_minutes": workforce::overtime_minutes(*regular, *explicit, worker.fte_percent, full_day),
            })
        })
        .collect();
    format::json(serde_json::json!({ "entries": rows, "overtime": overtime }))
}

/// `POST /api/time-entries/{pid}/approve` — manager approval; only
/// approved time feeds payroll (WPM-R4).
#[debug_handler]
async fn approve_time_entry(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let entry = sqlx::query!(
        "SELECT id, status FROM time_entries WHERE pid = $1 AND deleted_at IS NULL",
        records::parse_pid(&pid)?,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    if entry.status == "approved" {
        return Err(unprocessable("time entry is already approved"));
    }
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        time_entries::Model,
        r#"UPDATE time_entries SET status = 'approved' WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, worked_on, minutes,
                  kind, status, notes, deleted_at AS "deleted_at: _""#,
        entry.id,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "time_entry",
        row.pid,
        "time_approved",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&row)?)
}

/// `POST /api/workers/{pid}/leave-entitlements` — grant.
#[debug_handler]
async fn create_entitlement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<EntitlementPayload>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let mut problems = Problems::new();
    problems.require_token("kind", tokens::LEAVE_KINDS, &payload.kind);
    if !(2000..=2100).contains(&payload.year) {
        problems.push(format!("year {} out of range", payload.year));
    }
    if payload.entitled_days < 0 || payload.entitled_days > 366 {
        problems.push(format!(
            "entitled_days {} out of range 0-366",
            payload.entitled_days
        ));
    }
    ensure_valid(&problems.into_vec())?;
    let existing = sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM leave_entitlements
            WHERE worker_pid = $1 AND kind = $2 AND year = $3 AND deleted_at IS NULL LIMIT 1"#,
        worker.pid,
        payload.kind,
        payload.year,
    )
    .fetch_optional(db::pool(&ctx))
    .await?;
    if existing.is_some() {
        return Err(unprocessable(
            "entitlement already exists for this kind and year",
        ));
    }
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO leave_entitlements (pid, worker_pid, kind, year, entitled_days, used_days)
         VALUES ($1, $2, $3, $4, $5, 0)",
        pid,
        worker.pid,
        payload.kind,
        payload.year,
        payload.entitled_days,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "leave_entitlement",
        pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: pid.to_string(),
    })
}

/// `GET /api/workers/{pid}/leave-entitlements` — balances.
#[debug_handler]
async fn list_entitlements(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let rows = sqlx::query_as!(
        leave_entitlements::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, kind, year,
                  entitled_days, used_days, deleted_at AS "deleted_at: _"
             FROM leave_entitlements WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY id"#,
        worker.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    format::json(rows)
}

/// `POST /api/workers/{pid}/leave-requests` — request leave. The
/// balance is **checked** here (annual over-balance ⇒ 422; sick may
/// flag negative) and **decremented on approval** (WPM-R5).
#[debug_handler]
async fn create_leave_request(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<LeaveRequestPayload>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    create_leave_for(&ctx, &caller, &worker, payload).await
}

/// Request leave for this worker: validated, balance-checked, audited. Shared by the HR route
/// and `/api/me/leave-requests`.
pub(crate) async fn create_leave_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &crate::models::_entities::workers::Model,
    payload: LeaveRequestPayload,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token("kind", tokens::LEAVE_KINDS, &payload.kind);
    problems.cap_opt("reason", payload.reason.as_deref());
    ensure_valid(&problems.into_vec())?;
    leave::validate_reason(&payload.kind, payload.reason.as_deref())
        .map_err(|e| unprocessable(&e))?;
    let days = leave::day_span(payload.start_on, payload.end_on).map_err(|e| unprocessable(&e))?;
    let check = balance_check(ctx, worker, &payload.kind, payload.start_on, days).await?;
    let negative = match check {
        leave::BalanceCheck::Ok { .. } => false,
        leave::BalanceCheck::NegativeFlagged { .. } => true,
        leave::BalanceCheck::OverBalance {
            remaining,
            requested,
        } => {
            return Err(unprocessable(&format!(
                "requested {requested} calendar days exceeds remaining balance {remaining} calendar days"
            )));
        }
    };
    let pid = Uuid::new_v4();
    let mut tx = db::pool(ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO leave_requests (pid, worker_pid, kind, start_on, end_on, days, status, negative_balance, reason)
         VALUES ($1, $2, $3, $4, $5, $6, 'requested', $7, $8)",
        pid,
        worker.pid,
        payload.kind,
        payload.start_on,
        payload.end_on,
        days,
        negative,
        payload.reason,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "leave_request",
        pid,
        "leave_requested",
        caller.actor(),
        None,
    )
    .await?;
    streaming::emit_in(
        &mut tx,
        "leave_request",
        "leave_requested",
        &pid.to_string(),
        &worker.worker_number,
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": pid, "days": days, "negative_balance": negative }))
}

/// The balance verdict for `kind` in the request's starting year.
async fn balance_check(
    ctx: &AppContext,
    worker: &crate::models::_entities::workers::Model,
    kind: &str,
    start_on: chrono::NaiveDate,
    days: i32,
) -> Result<leave::BalanceCheck> {
    use chrono::Datelike;
    let year = start_on.year();
    let entitlement = sqlx::query!(
        "SELECT entitled_days, used_days FROM leave_entitlements
          WHERE worker_pid = $1 AND kind = $2 AND year = $3 AND deleted_at IS NULL LIMIT 1",
        worker.pid,
        kind,
        year,
    )
    .fetch_optional(db::pool(ctx))
    .await?;
    let (entitled, used) = entitlement.map_or((0, 0), |e| (e.entitled_days, e.used_days));
    Ok(leave::check_balance(kind, entitled, used, days))
}

/// `GET /api/workers/{pid}/leave-requests`.
#[debug_handler]
async fn list_leave_requests(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let rows = sqlx::query_as!(
        leave_requests::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, kind, start_on, end_on,
                  days, status, negative_balance, reason, decided_by, decided_at AS "decided_at: _",
                  deleted_at AS "deleted_at: _"
             FROM leave_requests WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY id"#,
        worker.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    format::json(rows)
}

/// One leave decision (`approved` / `rejected` / `cancelled`),
/// serialized on the locked request row; approval decrements the
/// balance in the same transaction (WPM-R5, WPM-D9).
#[allow(clippy::too_many_lines)] // the audit is written per decision, each with its literal action
pub(crate) async fn decide_leave(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
    to: &str,
) -> Result<Response> {
    use chrono::Datelike;
    let pid = records::parse_pid(pid)?;
    let mut tx = db::pool(ctx).begin().await?;
    // Lock the request row: two racing approvers serialize here; the
    // loser sees the already-decided status and gets the 422.
    let request = sqlx::query!(
        "SELECT id, status, kind, start_on, days, worker_pid FROM leave_requests
          WHERE pid = $1 AND deleted_at IS NULL LIMIT 1 FOR UPDATE",
        pid,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;
    lifecycle::check("leave request", lifecycle::LEAVE, &request.status, to)
        .map_err(|e| unprocessable(&e))?;
    let was = request.status.clone();
    let year = request.start_on.year();
    let days = request.days;
    let row = sqlx::query_as!(
        leave_requests::Model,
        r#"UPDATE leave_requests SET status = $2, decided_by = $3, decided_at = now() WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, kind, start_on, end_on,
                  days, status, negative_balance, reason, decided_by, decided_at AS "decided_at: _",
                  deleted_at AS "deleted_at: _""#,
        request.id,
        to,
        caller.actor(),
    )
    .fetch_one(&mut *tx)
    .await?;
    // Balance mutation: approve uses days; cancelling an approved
    // request restores them.
    let delta = if to == "approved" {
        days
    } else if to == "cancelled" && was == "approved" {
        -days
    } else {
        0
    };
    if delta != 0 {
        let entitlement = sqlx::query_scalar!(
            "SELECT id FROM leave_entitlements
              WHERE worker_pid = $1 AND kind = $2 AND year = $3 AND deleted_at IS NULL LIMIT 1 FOR UPDATE",
            request.worker_pid,
            request.kind,
            year,
        )
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(id) = entitlement {
            sqlx::query!(
                "UPDATE leave_entitlements SET used_days = used_days + $2 WHERE id = $1",
                id,
                delta,
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    let snapshot = Some(serde_json::json!({ "from": was, "days": days }));
    // One literal action per decision, so the audit guard can read each.
    match to {
        "approved" => {
            db::audit(
                &mut tx,
                "leave_request",
                row.pid,
                "leave_approved",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
        "rejected" => {
            db::audit(
                &mut tx,
                "leave_request",
                row.pid,
                "leave_rejected",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
        "cancelled" => {
            db::audit(
                &mut tx,
                "leave_request",
                row.pid,
                "leave_cancelled",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
        _ => {
            return Err(super::internal(
                "a leave decision is approved, rejected or cancelled",
            ));
        }
    }
    let event = format!("leave_{to}");
    streaming::emit_in(
        &mut tx,
        "leave_request",
        &event,
        &row.pid.to_string(),
        "",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    if to == "approved" || to == "rejected" {
        Metrics::global().leave_decided_total.inc();
    }
    format::json(row_json(&row)?)
}

/// `POST /api/leave-requests/{pid}/approve`.
#[debug_handler]
async fn approve_leave(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    decide_leave(&ctx, &caller, &pid, "approved").await
}

/// `POST /api/leave-requests/{pid}/reject`.
#[debug_handler]
async fn reject_leave(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    decide_leave(&ctx, &caller, &pid, "rejected").await
}

/// `POST /api/leave-requests/{pid}/cancel`.
#[debug_handler]
async fn cancel_leave(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    decide_leave(&ctx, &caller, &pid, "cancelled").await
}

/// `POST /api/shifts` — plan a shift.
#[debug_handler]
async fn create_shift(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<ShiftPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("department", &payload.department);
    if payload.ends_at <= payload.starts_at {
        problems.push("ends_at must be after starts_at".to_string());
    }
    if payload.required_headcount < 1 {
        problems.push("required_headcount must be at least 1".to_string());
    }
    ensure_valid(&problems.into_vec())?;
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO shifts (pid, department, starts_at, ends_at, required_headcount) VALUES ($1, $2, $3, $4, $5)",
        pid,
        payload.department,
        payload.starts_at,
        payload.ends_at,
        payload.required_headcount,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "shift", pid, "created", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: pid.to_string(),
    })
}

/// `GET /api/shifts?department=&date=` — the day rota (WPM-R6).
#[derive(Debug, Deserialize)]
struct RotaParams {
    #[serde(default)]
    department: Option<String>,
    #[serde(default)]
    date: Option<chrono::NaiveDate>,
}

#[debug_handler]
async fn list_shifts(
    State(ctx): State<AppContext>,
    Query(params): Query<RotaParams>,
) -> Result<Response> {
    let rows = sqlx::query_as!(
        shifts::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, department,
                  starts_at AS "starts_at: _", ends_at AS "ends_at: _", required_headcount, deleted_at AS "deleted_at: _"
             FROM shifts WHERE deleted_at IS NULL AND ($1::text IS NULL OR department = $1)
            ORDER BY starts_at, id
            LIMIT 1000"#,
        params.department,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let rows: Vec<_> = if let Some(date) = params.date {
        rows.into_iter()
            .filter(|s| s.starts_at.date_naive() == date)
            .collect()
    } else {
        rows
    };
    // Attach assignments per shift, read in one statement.
    let shift_pids: Vec<Uuid> = rows.iter().map(|s| s.pid).collect();
    let mut by_shift: std::collections::HashMap<Uuid, Vec<shift_assignments::Model>> =
        std::collections::HashMap::new();
    for assignment in sqlx::query_as!(
        shift_assignments::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, shift_pid, worker_pid,
                  deleted_at AS "deleted_at: _"
             FROM shift_assignments WHERE shift_pid = ANY($1) AND deleted_at IS NULL ORDER BY id"#,
        &shift_pids,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    {
        by_shift
            .entry(assignment.shift_pid)
            .or_default()
            .push(assignment);
    }
    let mut out = Vec::with_capacity(rows.len());
    for shift in rows {
        let assignments = by_shift.remove(&shift.pid).unwrap_or_default();
        out.push(serde_json::json!({ "shift": shift, "assignments": assignments }));
    }
    format::json(out)
}

/// `POST /api/shifts/{pid}/assignments` — assign a worker. Refuses
/// a double booking (overlapping assigned shift) and an assignment
/// overlapping approved leave (WPM-R6).
#[debug_handler]
async fn assign_shift(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<AssignmentPayload>,
) -> Result<Response> {
    let shift = sqlx::query!(
        r#"SELECT pid, starts_at AS "starts_at: chrono::DateTime<chrono::FixedOffset>",
                  ends_at AS "ends_at: chrono::DateTime<chrono::FixedOffset>"
             FROM shifts WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(&pid)?,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let worker = records::worker_by_pid(&ctx, payload.worker_pid).await?;
    // Double-booking: any live assignment to an overlapping live shift.
    let theirs = sqlx::query!(
        r#"SELECT s.pid, s.starts_at AS "starts_at: chrono::DateTime<chrono::FixedOffset>",
                  s.ends_at AS "ends_at: chrono::DateTime<chrono::FixedOffset>"
             FROM shift_assignments a JOIN shifts s ON s.pid = a.shift_pid AND s.deleted_at IS NULL
            WHERE a.worker_pid = $1 AND a.deleted_at IS NULL
            ORDER BY a.id"#,
        worker.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    for other in &theirs {
        if workforce::windows_overlap(
            shift.starts_at,
            shift.ends_at,
            other.starts_at,
            other.ends_at,
        ) {
            return Err(unprocessable(&format!(
                "double booking: worker already assigned to an overlapping shift ({})",
                other.pid
            )));
        }
    }
    // Leave conflict: any approved leave overlapping the shift's dates.
    let approved_leave = sqlx::query!(
        "SELECT kind, start_on, end_on FROM leave_requests
          WHERE worker_pid = $1 AND status = 'approved' AND deleted_at IS NULL ORDER BY id",
        worker.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let shift_start = shift.starts_at.date_naive();
    let shift_end = shift.ends_at.date_naive();
    for request in &approved_leave {
        if leave::ranges_overlap(shift_start, shift_end, request.start_on, request.end_on) {
            return Err(unprocessable(&format!(
                "worker is on approved {} leave {} to {}",
                request.kind, request.start_on, request.end_on
            )));
        }
    }
    let new_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO shift_assignments (pid, shift_pid, worker_pid) VALUES ($1, $2, $3)",
        new_pid,
        shift.pid,
        worker.pid,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "shift_assignment",
        new_pid,
        "shift_assigned",
        caller.actor(),
        None,
    )
    .await?;
    streaming::emit_in(
        &mut tx,
        "shift_assignment",
        "shift_assigned",
        &new_pid.to_string(),
        &worker.worker_number,
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: new_pid.to_string(),
    })
}

/// `DELETE /api/shift-assignments/{pid}` — unassign (soft delete).
#[debug_handler]
async fn unassign_shift(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let pid = records::parse_pid(&pid)?;
    let id = sqlx::query_scalar!(
        "SELECT id FROM shift_assignments WHERE pid = $1 AND deleted_at IS NULL",
        pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE shift_assignments SET deleted_at = now() WHERE id = $1",
        id
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "shift_assignment",
        pid,
        "deleted",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}

/// `GET /api/workforce/working-time?department=&as_of=` query.
#[derive(Debug, Deserialize)]
struct WorkingTimeQuery {
    department: Option<String>,
    as_of: Option<chrono::NaiveDate>,
}

/// How far the rest-gap check looks around `as_of` (days, each way) —
/// recent turnarounds and the *planned* rota alike.
const REST_WINDOW_DAYS: i64 = 28;

/// One shift's `(starts, ends)` in UTC, for the rest-gap check.
type ShiftInterval = (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>);

/// One worker's three resolved working-time limits.
struct WorkerLimits {
    weeks: crate::rules::jurisdiction::Resolution,
    ceiling: crate::rules::jurisdiction::Resolution,
    rest: crate::rules::jurisdiction::Resolution,
}

impl WorkerLimits {
    /// The window and ceiling, when both are configured.
    fn average(&self) -> Option<(i64, i64)> {
        Some((self.weeks.integer()?, self.ceiling.integer()?))
    }
}

/// `GET /api/workforce/working-time` — the advisory working-time
/// guardrails (WPM-R27): per worker, the average of **recorded** minutes
/// (all non-deleted entries — a safety signal does not wait for approval)
/// over the trailing reference window, with WPM-D16 terms and the ceiling
/// flag, plus rest-gap breaches across recent and planned shift assignments.
/// Each worker's three limits come from their organization's jurisdiction
/// profile over the chain of their `work_jurisdiction` (WPM-D144, WPM-T380);
/// a limit with no row is **not configured**, counted, never a pass.
/// Flags only — nothing is refused (WPM-D19). Visibility equals the rota's:
/// flagged workers are named.
#[debug_handler]
#[allow(clippy::too_many_lines)] // limits → minutes → shifts → rest breaches → flagged, linearly
async fn working_time(
    axa::extract::Query(query): axa::extract::Query<WorkingTimeQuery>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    use crate::rules::jurisdiction::Resolution;
    let as_of = query
        .as_of
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    let department = query.department.as_deref();
    // Every worker in scope, with where their work is done: the limits are per worker.
    let places: Vec<(Uuid, String, Option<String>)> = sqlx::query!(
        "SELECT pid, organization_ref, work_jurisdiction FROM workers
          WHERE deleted_at IS NULL AND ($1::text IS NULL OR department = $1) ORDER BY id",
        department,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|w| (w.pid, w.organization_ref, w.work_jurisdiction))
    .collect();
    let workers_checked = places.len();
    let organizations: Vec<String> = places
        .iter()
        .map(|(_, org, _)| org.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let profiles =
        super::jurisdictions::profiles_for(crate::db::pool(&ctx), &organizations).await?;
    let empty = super::jurisdictions::Profile::default();
    let limits: std::collections::BTreeMap<Uuid, WorkerLimits> = places
        .iter()
        .map(|(pid, org, place)| {
            let profile = profiles.get(org).unwrap_or(&empty);
            let get = |key| profile.resolve(key, place.as_deref(), as_of);
            (
                *pid,
                WorkerLimits {
                    weeks: get(working_time::REFERENCE_WEEKS_KEY),
                    ceiling: get(working_time::MAX_AVERAGE_WEEKLY_MINUTES_KEY),
                    rest: get(working_time::MIN_DAILY_REST_MINUTES_KEY),
                },
            )
        })
        .collect();
    // Workers grouped by their (window, ceiling): one database pass per distinct pair.
    let mut groups: std::collections::BTreeMap<(i64, i64), Vec<Uuid>> =
        std::collections::BTreeMap::new();
    for (pid, limit) in &limits {
        if let Some(pair) = limit.average() {
            groups.entry(pair).or_default().push(*pid);
        }
    }
    // Recorded minutes per worker over their window, summed by the database: only the workers over
    // their ceiling come back (the same comparison as `working_time::over_average`). The weekly
    // rollup (`time_entry_weeks`, kept by triggers) first names the workers whose weeks touching the
    // window add up to more than the ceiling, a superset of the answer, looking only at workers with
    // a week at the heavy floor; only their entries are then summed over the exact window.
    let mut minutes_of: std::collections::BTreeMap<Uuid, i64> = std::collections::BTreeMap::new();
    for (&(weeks, ceiling), pids) in &groups {
        let window_start = as_of - chrono::Duration::weeks(weeks);
        let total_ceiling = ceiling.saturating_mul(weeks);
        let heavy = working_time::heavy_week_minutes(ceiling, weeks).unwrap_or(0);
        // One bind for the whole list (`ANY`), so the statement's parameter limit does not apply.
        let rows = sqlx::query!(
            r#"WITH heavy AS (
                   SELECT DISTINCT worker_pid FROM time_entry_weeks
                    WHERE week_start >= date_trunc('week', $1::date + 1)::date AND week_start <= $2::date
                      AND minutes >= $4::bigint AND worker_pid = ANY($5::uuid[])),
                    maybe AS (
                   SELECT w.worker_pid FROM time_entry_weeks w
                    WHERE w.worker_pid IN (SELECT worker_pid FROM heavy)
                      AND w.week_start >= date_trunc('week', $1::date + 1)::date AND w.week_start <= $2::date
                    GROUP BY w.worker_pid HAVING SUM(w.minutes) > $3::bigint)
               SELECT t.worker_pid, SUM(t.minutes)::bigint AS "total!"
                 FROM time_entries t
                WHERE t.worker_pid IN (SELECT worker_pid FROM maybe)
                  AND t.deleted_at IS NULL AND t.worked_on > $1::date AND t.worked_on <= $2::date
                GROUP BY t.worker_pid HAVING SUM(t.minutes) > $3::bigint"#,
            window_start,
            as_of,
            total_ceiling,
            heavy,
            pids.as_slice(),
        )
        .fetch_all(db::pool(&ctx))
        .await?;
        for row in rows {
            minutes_of.insert(row.worker_pid, row.total);
        }
    }
    // Shift intervals per worker around `as_of` (recent + planned): only the shifts in the window.
    let rest_start = as_of - chrono::Duration::days(REST_WINDOW_DAYS);
    let rest_end = as_of + chrono::Duration::days(REST_WINDOW_DAYS);
    let utc_midnight = |date: chrono::NaiveDate| date.and_time(chrono::NaiveTime::MIN).and_utc();
    let shift_rows = sqlx::query!(
        r#"SELECT pid, starts_at AS "starts_at: chrono::DateTime<chrono::Utc>",
                  ends_at AS "ends_at: chrono::DateTime<chrono::Utc>"
             FROM shifts WHERE deleted_at IS NULL AND starts_at >= $1 AND starts_at < $2"#,
        utc_midnight(rest_start),
        utc_midnight(rest_end + chrono::Duration::days(1)),
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let window_shifts: std::collections::BTreeMap<Uuid, ShiftInterval> = shift_rows
        .iter()
        .map(|shift| (shift.pid, (shift.starts_at, shift.ends_at)))
        .collect();
    let window_pids: Vec<Uuid> = window_shifts.keys().copied().collect();
    let mut assigned: Vec<(Uuid, ShiftInterval)> = Vec::new();
    for assignment in sqlx::query!(
        "SELECT shift_pid, worker_pid FROM shift_assignments WHERE deleted_at IS NULL AND shift_pid = ANY($1) ORDER BY id",
        &window_pids,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    {
        if let Some(interval) = window_shifts.get(&assignment.shift_pid) {
            assigned.push((assignment.worker_pid, *interval));
        }
    }
    let breaches_of = working_time::breaches_by_worker(&assigned, |pid| {
        limits.get(&pid).and_then(|limit| limit.rest.integer())
    });
    // A worker flagged only for rest still shows their total over their own window (if configured).
    for pid in breaches_of.keys() {
        if minutes_of.contains_key(pid) {
            continue;
        }
        let Some((weeks, _)) = limits.get(pid).and_then(WorkerLimits::average) else {
            continue;
        };
        let total = sqlx::query_scalar!(
            "SELECT SUM(minutes)::bigint FROM time_entries
              WHERE deleted_at IS NULL AND worker_pid = $1 AND worked_on > $2 AND worked_on <= $3",
            *pid,
            as_of - chrono::Duration::weeks(weeks),
            as_of,
        )
        .fetch_one(db::pool(&ctx))
        .await?;
        minutes_of.insert(*pid, total.unwrap_or(0));
    }
    let mut candidates: Vec<Uuid> = minutes_of
        .keys()
        .chain(breaches_of.keys())
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    candidates.sort_unstable();
    let limit_json = |resolution: &Resolution| serde_json::to_value(resolution).unwrap_or_default();
    let mut flagged = Vec::new();
    {
        let people = sqlx::query!(
            "SELECT pid, display_name, department, work_jurisdiction FROM workers
              WHERE deleted_at IS NULL AND ($1::text IS NULL OR department = $1) AND pid = ANY($2)
              ORDER BY id",
            department,
            &candidates,
        )
        .fetch_all(db::pool(&ctx))
        .await?;
        for worker in &people {
            let Some(limit) = limits.get(&worker.pid) else {
                continue;
            };
            let total_minutes = minutes_of.get(&worker.pid).copied().unwrap_or(0);
            let average = limit.average();
            let over = average.is_some_and(|(weeks, ceiling)| {
                working_time::over_average(total_minutes, weeks, ceiling)
            });
            let breaches = breaches_of.get(&worker.pid).map_or(&[][..], Vec::as_slice);
            if !working_time::is_flagged(over, breaches.len()) {
                continue;
            }
            flagged.push(serde_json::json!({
                "worker_pid": worker.pid,
                "display_name": worker.display_name,
                "department": worker.department,
                "work_jurisdiction": worker.work_jurisdiction,
                "average_weekly": average.map(|(weeks, _)| serde_json::json!({
                    "numerator_minutes": total_minutes,
                    "denominator_weeks": weeks,
                    "value_minutes_per_week": working_time::weekly_average(total_minutes, weeks),
                })),
                "over_average": over,
                "limits": {
                    "reference_weeks": limit_json(&limit.weeks),
                    "max_average_weekly_minutes": limit_json(&limit.ceiling),
                    "min_daily_rest_minutes": limit_json(&limit.rest),
                },
                "rest_breaches": breaches.iter().map(|breach| serde_json::json!({
                    "prev_end": breach.prev_end,
                    "next_start": breach.next_start,
                    "gap_minutes": breach.gap_minutes,
                })).collect::<Vec<_>>(),
            }));
        }
    }
    // Counts only: who has no limit is not a flag and names no one.
    let count =
        |test: &dyn Fn(&WorkerLimits) -> bool| limits.values().filter(|limit| test(limit)).count();
    let not_configured = serde_json::json!({
        "average": count(&|limit| limit.average().is_none()),
        "rest": count(&|limit| limit.rest.integer().is_none()),
        "no_work_jurisdiction": places.iter().filter(|(_, _, place)| place.is_none()).count(),
    });
    format::json(serde_json::json!({
        "as_of": as_of,
        "rest_window_days": REST_WINDOW_DAYS,
        "workers_checked": workers_checked,
        "flagged": flagged,
        "not_configured": not_configured,
        "derivation": "advisory only, nothing is refused (WPM-D19); each worker's limits come from their organization's \
                       jurisdiction profile for where the work is done (WPM-D144), and a limit with no profile row is \
                       not configured, never a pass; average = all recorded (not merely approved) minutes over the \
                       trailing reference window; rest breaches = consecutive shift assignments (recent and planned, \
                       ±28 days) closer than the rest floor",
    }))
}

/// The workforce routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/time-entries", post(create_time_entry))
        .add("/workers/{pid}/time-entries", get(list_time_entries))
        .add("/time-entries/{pid}/approve", post(approve_time_entry))
        .add(
            "/workers/{pid}/leave-entitlements",
            post(create_entitlement),
        )
        .add("/workers/{pid}/leave-entitlements", get(list_entitlements))
        .add("/workers/{pid}/leave-requests", post(create_leave_request))
        .add("/workers/{pid}/leave-requests", get(list_leave_requests))
        .add("/leave-requests/{pid}/approve", post(approve_leave))
        .add("/leave-requests/{pid}/reject", post(reject_leave))
        .add("/leave-requests/{pid}/cancel", post(cancel_leave))
        .add("/shifts", post(create_shift))
        .add("/shifts", get(list_shifts))
        .add("/workforce/working-time", get(working_time))
        .add("/shifts/{pid}/assignments", post(assign_shift))
        .add("/shift-assignments/{pid}", delete(unassign_shift))
}
