//! **Starting a course** (WPM-R236, WPM-D182): a person enrols at once on a course whose recorded fee is zero, and asks a manager above
//! them for any other. Every route resolves the person from the token. A request names a course and a decision, and holds no reason.

use axa::error::ErrorDetail;
use axa::prelude::*;
use chrono::Utc;
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveValue, ConnectionTrait, QueryOrder, SqlErr, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;

use super::individual_goals::{reports_of, subject};
use super::me::me_worker;
use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{course_requests, skill_courses, training_enrollments, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::notifications::Model as Notification;
use crate::models::records;
use crate::need_to_know::is_line_manager;
use crate::rules::course_request::{self as rules, Route};

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

async fn catalogue_rows(ctx: &AppContext, course_ref: &str) -> Result<Vec<skill_courses::Model>> {
    Ok(skill_courses::Entity::find()
        .filter(skill_courses::Column::CourseRef.eq(course_ref))
        .filter(skill_courses::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?)
}

async fn enrolled<C: ConnectionTrait>(db: &C, worker: Uuid, course_ref: &str) -> Result<bool> {
    Ok(training_enrollments::Entity::find()
        .filter(training_enrollments::Column::WorkerPid.eq(worker))
        .filter(training_enrollments::Column::CourseRef.eq(course_ref))
        .filter(training_enrollments::Column::Status.eq("enrolled"))
        .filter(training_enrollments::Column::DeletedAt.is_null())
        .one(db)
        .await?
        .is_some())
}

async fn enrol<C: ConnectionTrait>(
    db: &C,
    worker: Uuid,
    course_ref: &str,
    actor: Option<&str>,
) -> Result<Uuid> {
    let row = training_enrollments::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker),
        course_ref: ActiveValue::set(course_ref.to_string()),
        status: ActiveValue::set("enrolled".to_string()),
        completed_on: ActiveValue::set(None),
        certificate_expires_on: ActiveValue::set(None),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Audit::record(db, "training_enrollment", row.pid, "created", actor, None).await?;
    Ok(row.pid)
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
    if enrolled(&ctx.db, worker.pid, course_ref).await? {
        return Err(conflict("you are already enrolled on this course"));
    }
    let txn = ctx.db.begin().await?;
    let answer = if route == Route::Direct {
        let pid = enrol(&txn, worker.pid, course_ref, caller.actor()).await?;
        serde_json::json!({ "outcome": "enrolled", "pid": pid })
    } else {
        let row = course_requests::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            worker_pid: ActiveValue::set(worker.pid),
            course_ref: ActiveValue::set(course_ref.to_string()),
            status: ActiveValue::set("requested".to_string()),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(|cause| match cause.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                conflict("you have already asked for this course")
            }
            _ => cause.into(),
        })?;
        Audit::record(
            &txn,
            "course_request",
            row.pid,
            "created",
            caller.actor(),
            None,
        )
        .await?;
        if let Some(manager) = worker.manager_pid {
            Notification::push(
                &txn,
                manager,
                "course_requested",
                "A course request is waiting for your decision.",
                serde_json::json!({ "course_request_pid": row.pid }),
            )
            .await?;
        }
        serde_json::json!({ "outcome": "requested", "pid": row.pid })
    };
    txn.commit().await?;
    format::json(serde_json::json!({
        "has_manager": worker.manager_pid.is_some(),
        "started": answer,
    }))
}

/// `GET /api/me/course-requests` — the caller's own requests, newest first.
#[debug_handler]
pub(crate) async fn mine(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let rows = course_requests::Entity::find()
        .filter(course_requests::Column::WorkerPid.eq(worker.pid))
        .order_by_desc(course_requests::Column::Id)
        .all(&ctx.db)
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
        let rows = course_requests::Entity::find()
            .filter(course_requests::Column::WorkerPid.eq(worker.pid))
            .filter(course_requests::Column::Status.eq("requested"))
            .order_by_asc(course_requests::Column::Id)
            .all(&ctx.db)
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

async fn open_request(ctx: &AppContext, pid: &str) -> Result<course_requests::Model> {
    course_requests::Entity::find()
        .filter(course_requests::Column::Pid.eq(records::parse_pid(pid)?))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

/// Move a request out of `requested`, for one caller only: whoever gets there second is refused.
async fn settle<C: ConnectionTrait>(
    db: &C,
    row: &course_requests::Model,
    to: &str,
    decided_by: Option<&str>,
) -> Result<()> {
    let moved = course_requests::Entity::update_many()
        .col_expr(course_requests::Column::Status, Expr::value(to))
        .col_expr(course_requests::Column::DecidedBy, Expr::value(decided_by))
        .col_expr(
            course_requests::Column::DecidedOn,
            Expr::value(decided_by.map(|_| Utc::now().date_naive())),
        )
        .filter(course_requests::Column::Pid.eq(row.pid))
        .filter(course_requests::Column::Status.eq("requested"))
        .exec(db)
        .await?;
    if moved.rows_affected == 1 {
        Ok(())
    } else {
        Err(conflict("this request has already been settled"))
    }
}

/// The worker a manager decides for, or a 404: to anyone else the request is not there.
async fn decidable(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    row: &course_requests::Model,
) -> Result<workers::Model> {
    let sub = subject(caller)?;
    let worker = records::find_worker(&ctx.db, row.worker_pid).await?;
    if is_line_manager(ctx, &worker, sub).await? {
        Ok(worker)
    } else {
        Err(Error::NotFound)
    }
}

async fn tell_person<C: ConnectionTrait>(
    db: &C,
    row: &course_requests::Model,
    worker: Uuid,
) -> Result<()> {
    Notification::push(
        db,
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
    let txn = ctx.db.begin().await?;
    settle(&txn, &row, "approved", Some(subject(&caller)?)).await?;
    if !enrolled(&txn, worker.pid, &row.course_ref).await? {
        enrol(&txn, worker.pid, &row.course_ref, caller.actor()).await?;
    }
    Audit::record(
        &txn,
        "course_request",
        row.pid,
        "approved",
        caller.actor(),
        None,
    )
    .await?;
    tell_person(&txn, &row, worker.pid).await?;
    txn.commit().await?;
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
    let txn = ctx.db.begin().await?;
    settle(&txn, &row, "declined", Some(subject(&caller)?)).await?;
    Audit::record(
        &txn,
        "course_request",
        row.pid,
        "declined",
        caller.actor(),
        None,
    )
    .await?;
    tell_person(&txn, &row, worker.pid).await?;
    txn.commit().await?;
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
    let txn = ctx.db.begin().await?;
    settle(&txn, &row, "withdrawn", None).await?;
    Audit::record(
        &txn,
        "course_request",
        row.pid,
        "withdrawn",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::empty_json()
}
