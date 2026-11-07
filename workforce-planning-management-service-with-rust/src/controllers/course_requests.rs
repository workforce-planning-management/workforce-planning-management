//! **Starting a course** (WPM-R236, WPM-D182): a person enrols at once on a course whose recorded fee is zero, and asks a manager above
//! them for any other. Every route resolves the person from the token. A request names a course and a decision, and holds no reason.

use axa::error::ErrorDetail;
use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use sqlx::PgConnection;
use uuid::Uuid;

use super::individual_goals::{reports_of, subject};
use super::me::me_worker;
use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::db::{self, is_unique_violation, pool};
use crate::models::_entities::workers;
use crate::models::records;
use crate::need_to_know::is_line_manager;
use crate::rules::course_request::{self as rules, Route};

/// One request as stored.
struct CourseRequest {
    pid: Uuid,
    worker_pid: Uuid,
    course_ref: String,
    status: String,
    decided_on: Option<NaiveDate>,
}

/// `POST /api/me/course-requests` body.
#[derive(Debug, Deserialize)]
pub(crate) struct StartPayload {
    course_ref: String,
}

fn conflict(message: &str) -> Error {
    Error::CustomError(
        axum::http::StatusCode::CONFLICT,
        ErrorDetail::new("conflict", message),
    )
}

/// One catalogue row for a course.
struct CatalogueRow {
    title: String,
    hours: i32,
    fee_minor: Option<i64>,
    fee_currency: Option<String>,
}

async fn catalogue_rows(ctx: &AppContext, course_ref: &str) -> Result<Vec<CatalogueRow>> {
    Ok(sqlx::query_as!(
        CatalogueRow,
        "SELECT title, hours, fee_minor, fee_currency FROM skill_courses
         WHERE course_ref = $1 AND deleted_at IS NULL",
        course_ref,
    )
    .fetch_all(pool(ctx))
    .await?)
}

async fn enrolled(connection: &mut PgConnection, worker: Uuid, course_ref: &str) -> Result<bool> {
    let found = sqlx::query_scalar!(
        "SELECT EXISTS (
             SELECT 1 FROM training_enrollments
             WHERE worker_pid = $1 AND course_ref = $2 AND status = 'enrolled' AND deleted_at IS NULL
         )",
        worker,
        course_ref,
    )
    .fetch_one(connection)
    .await?;
    Ok(found == Some(true))
}

async fn enrol(
    connection: &mut PgConnection,
    worker: Uuid,
    course_ref: &str,
    actor: Option<&str>,
) -> Result<Uuid> {
    let pid = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO training_enrollments (pid, worker_pid, course_ref, status) VALUES ($1, $2, $3, 'enrolled')",
        pid,
        worker,
        course_ref,
    )
    .execute(&mut *connection)
    .await?;
    db::audit(
        connection,
        "training_enrollment",
        pid,
        "created",
        actor,
        None,
    )
    .await?;
    Ok(pid)
}

/// `POST /api/me/course-requests` — start a course: enrolled at once on a recorded fee of zero, otherwise asked of a manager.
#[debug_handler]
pub(crate) async fn start(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<StartPayload>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let course_ref = payload.course_ref.trim();
    let rows = catalogue_rows(&ctx, course_ref).await?;
    let fees: Vec<Option<i64>> = rows.iter().map(|row| row.fee_minor).collect();
    let Some(route) = rules::route(&fees) else {
        return Err(unprocessable("that course is not in the catalogue"));
    };
    let mut tx = pool(&ctx).begin().await?;
    if enrolled(&mut tx, worker.pid, course_ref).await? {
        return Err(conflict("you are already enrolled on this course"));
    }
    let answer = if route == Route::Direct {
        let pid = enrol(&mut tx, worker.pid, course_ref, caller.actor()).await?;
        serde_json::json!({ "outcome": "enrolled", "pid": pid })
    } else {
        let pid = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO course_requests (pid, worker_pid, course_ref) VALUES ($1, $2, $3)",
            pid,
            worker.pid,
            course_ref,
        )
        .execute(&mut *tx)
        .await
        .map_err(|cause| {
            if is_unique_violation(&cause) {
                conflict("you have already asked for this course")
            } else {
                cause.into()
            }
        })?;
        db::audit(
            &mut tx,
            "course_request",
            pid,
            "created",
            caller.actor(),
            None,
        )
        .await?;
        if let Some(manager) = worker.manager_pid {
            db::notify(
                &mut tx,
                manager,
                "course_requested",
                "A course request is waiting for your decision.",
                serde_json::json!({ "course_request_pid": pid }),
            )
            .await?;
        }
        serde_json::json!({ "outcome": "requested", "pid": pid })
    };
    tx.commit().await?;
    format::json(serde_json::json!({
        "has_manager": worker.manager_pid.is_some(),
        "started": answer,
    }))
}

/// `GET /api/me/course-requests` — the caller's own requests, newest first.
#[debug_handler]
pub(crate) async fn mine(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let rows = sqlx::query_as!(
        CourseRequest,
        "SELECT pid, worker_pid, course_ref, status, decided_on FROM course_requests
         WHERE worker_pid = $1 ORDER BY id DESC",
        worker.pid,
    )
    .fetch_all(pool(&ctx))
    .await?;
    format::json(serde_json::json!({
        "has_manager": worker.manager_pid.is_some(),
        "requests": rows.iter().map(|row| serde_json::json!({
            "pid": row.pid,
            "course_ref": row.course_ref,
            "status": row.status,
            "decided_on": row.decided_on,
        })).collect::<Vec<_>>(),
    }))
}

/// `GET /api/me/team-course-requests` — what waits for the caller's decision, from everyone below them, with the course's fee as the
/// catalogue has it now. Rows that disagree show no fee.
#[debug_handler]
pub(crate) async fn team(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let sub = subject(&caller)?.to_string();
    let mut waiting = Vec::new();
    for worker in reports_of(&ctx, &sub).await? {
        let rows = sqlx::query_as!(
            CourseRequest,
            "SELECT pid, worker_pid, course_ref, status, decided_on FROM course_requests
             WHERE worker_pid = $1 AND status = 'requested' ORDER BY id",
            worker.pid,
        )
        .fetch_all(pool(&ctx))
        .await?;
        for row in rows {
            let courses = catalogue_rows(&ctx, &row.course_ref).await?;
            let fees: Vec<_> = courses
                .iter()
                .map(|c| (c.fee_minor, c.fee_currency.clone()))
                .collect();
            let fee = rules::shared_fee(&fees);
            waiting.push(serde_json::json!({
                "pid": row.pid,
                "worker_pid": worker.pid,
                "display_name": worker.display_name,
                "course_ref": row.course_ref,
                "title": courses.first().map(|c| c.title.clone()),
                "hours": courses.first().map(|c| c.hours),
                "fee_minor": fee.as_ref().map(|f| f.0),
                "fee_currency": fee.and_then(|f| f.1),
            }));
        }
    }
    format::json(serde_json::json!({ "waiting": waiting }))
}

async fn open_request(ctx: &AppContext, pid: &str) -> Result<CourseRequest> {
    sqlx::query_as!(
        CourseRequest,
        "SELECT pid, worker_pid, course_ref, status, decided_on FROM course_requests WHERE pid = $1",
        records::parse_pid(pid)?,
    )
    .fetch_optional(pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// Move a request out of `requested`, for one caller only: whoever gets there second is refused.
async fn settle(
    connection: &mut PgConnection,
    row: &CourseRequest,
    to: &str,
    decided_by: Option<&str>,
) -> Result<()> {
    let decided_on = decided_by.map(|_| Utc::now().date_naive());
    let moved = sqlx::query!(
        "UPDATE course_requests SET status = $1, decided_by = $2, decided_on = $3
         WHERE pid = $4 AND status = 'requested'",
        to,
        decided_by,
        decided_on,
        row.pid,
    )
    .execute(connection)
    .await?;
    if moved.rows_affected() == 1 {
        Ok(())
    } else {
        Err(conflict("this request has already been settled"))
    }
}

/// The worker a manager decides for, or a 404: to anyone else the request is not there.
async fn decidable(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    row: &CourseRequest,
) -> Result<workers::Model> {
    let sub = subject(caller)?;
    let worker = records::find_worker(&ctx.db, row.worker_pid).await?;
    if is_line_manager(ctx, &worker, sub).await? {
        Ok(worker)
    } else {
        Err(Error::NotFound)
    }
}

async fn tell_person(
    connection: &mut PgConnection,
    row: &CourseRequest,
    worker: Uuid,
) -> Result<()> {
    db::notify(
        connection,
        worker,
        "course_decided",
        "Your course request has been decided.",
        serde_json::json!({ "course_request_pid": row.pid }),
    )
    .await?;
    Ok(())
}

/// `POST /api/me/course-requests/{pid}/approve` — a manager above the person approves, which enrols them. If they were enrolled some
/// other way meanwhile, no second enrolment is made.
#[debug_handler]
pub(crate) async fn approve(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = open_request(&ctx, &pid).await?;
    let worker = decidable(&ctx, &caller, &row).await?;
    let mut tx = pool(&ctx).begin().await?;
    settle(&mut tx, &row, "approved", Some(subject(&caller)?)).await?;
    if !enrolled(&mut tx, worker.pid, &row.course_ref).await? {
        enrol(&mut tx, worker.pid, &row.course_ref, caller.actor()).await?;
    }
    db::audit(
        &mut tx,
        "course_request",
        row.pid,
        "approved",
        caller.actor(),
        None,
    )
    .await?;
    tell_person(&mut tx, &row, worker.pid).await?;
    tx.commit().await?;
    format::empty_json()
}

/// `POST /api/me/course-requests/{pid}/decline` — a manager above the person declines. No reason is kept.
#[debug_handler]
pub(crate) async fn decline(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = open_request(&ctx, &pid).await?;
    let worker = decidable(&ctx, &caller, &row).await?;
    let mut tx = pool(&ctx).begin().await?;
    settle(&mut tx, &row, "declined", Some(subject(&caller)?)).await?;
    db::audit(
        &mut tx,
        "course_request",
        row.pid,
        "declined",
        caller.actor(),
        None,
    )
    .await?;
    tell_person(&mut tx, &row, worker.pid).await?;
    tx.commit().await?;
    format::empty_json()
}

/// `POST /api/me/course-requests/{pid}/withdraw` — the person takes back their own request.
#[debug_handler]
pub(crate) async fn withdraw(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let row = open_request(&ctx, &pid).await?;
    if row.worker_pid != worker.pid {
        return Err(Error::NotFound);
    }
    let mut tx = pool(&ctx).begin().await?;
    settle(&mut tx, &row, "withdrawn", None).await?;
    db::audit(
        &mut tx,
        "course_request",
        row.pid,
        "withdrawn",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}
