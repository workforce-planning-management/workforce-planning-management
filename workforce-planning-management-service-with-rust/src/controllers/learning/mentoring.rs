//! Mentorships: lifecycle and session log.

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

/// `POST /api/mentorships` body.
#[derive(Debug, Deserialize)]
pub(super) struct MentorshipPayload {
    mentor_pid: Uuid,
    mentee_pid: Uuid,
    focus: String,
}

/// `POST /api/mentorships` — propose a pairing (mentor ≠ mentee).
#[debug_handler]
pub(super) async fn create_mentorship(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<MentorshipPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("focus", &payload.focus);
    problems.cap_text("focus", &payload.focus);
    if payload.mentor_pid == payload.mentee_pid {
        problems.push("mentor and mentee must be different workers");
    }
    ensure_valid(&problems.into_vec())?;
    records::find_worker(&ctx.db, payload.mentor_pid).await?;
    records::find_worker(&ctx.db, payload.mentee_pid).await?;
    let txn = ctx.db.begin().await?;
    let row = mentorships::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        mentor_pid: ActiveValue::set(payload.mentor_pid),
        mentee_pid: ActiveValue::set(payload.mentee_pid),
        focus: ActiveValue::set(payload.focus.clone()),
        status: ActiveValue::set("proposed".to_string()),
        started_on: ActiveValue::set(None),
        ended_on: ActiveValue::set(None),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(&txn, "mentorship", row.pid, "created", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(PidRef::of(row.pid))
}

/// `POST /api/mentorships/{pid}/status` body.
#[derive(Debug, Deserialize)]
pub(super) struct StatusPayload {
    to: String,
}

/// `POST /api/mentorships/{pid}/status` — the lifecycle move (the
/// pure machine refuses illegal transitions); activating stamps
/// `started_on`, closing stamps `ended_on`.
#[debug_handler]
pub(super) async fn mentorship_status(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<StatusPayload>,
) -> Result<Response> {
    let row = mentorships::Entity::find()
        .filter(mentorships::Column::Pid.eq(records::parse_pid(&pid)?))
        .filter(mentorships::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    rules::mentorship_transition(&row.status, &payload.to)
        .map_err(|reason| unprocessable(&reason))?;
    let today = chrono::Utc::now().date_naive();
    let from = row.status.clone();
    let row_pid = row.pid;
    let mut active: mentorships::ActiveModel = row.into();
    active.status = ActiveValue::set(payload.to.clone());
    if payload.to == "active" {
        active.started_on = ActiveValue::set(Some(today));
    } else if matches!(payload.to.as_str(), "completed" | "ended") {
        active.ended_on = ActiveValue::set(Some(today));
    }
    let txn = ctx.db.begin().await?;
    let updated = active.update(&txn).await?;
    Audit::record(
        &txn,
        "mentorship",
        row_pid,
        "status_changed",
        caller.actor(),
        Some(serde_json::json!({ "from": from, "to": payload.to })),
    )
    .await?;
    txn.commit().await?;
    format::json(row_json(&updated)?)
}

/// `POST /api/mentorships/{pid}/sessions` body.
#[derive(Debug, Deserialize)]
pub(super) struct SessionPayload {
    held_on: chrono::NaiveDate,
    notes: String,
}

/// `POST /api/mentorships/{pid}/sessions` — log a session (only on an
/// active mentorship).
#[debug_handler]
pub(super) async fn log_session(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<SessionPayload>,
) -> Result<Response> {
    let mentorship = mentorships::Entity::find()
        .filter(mentorships::Column::Pid.eq(records::parse_pid(&pid)?))
        .filter(mentorships::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    if mentorship.status != "active" {
        return Err(unprocessable(
            "sessions can only be logged on an active mentorship",
        ));
    }
    let mut problems = Problems::new();
    problems.require_text("notes", &payload.notes);
    problems.cap_text("notes", &payload.notes);
    ensure_valid(&problems.into_vec())?;
    let txn = ctx.db.begin().await?;
    let row = mentorship_sessions::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        mentorship_pid: ActiveValue::set(mentorship.pid),
        held_on: ActiveValue::set(payload.held_on),
        notes: ActiveValue::set(payload.notes.clone()),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "mentorship_session",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(PidRef::of(row.pid))
}

/// `GET /api/mentorships/{pid}` — one mentorship + its session log.
#[debug_handler]
pub(super) async fn get_mentorship(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let mentorship = mentorships::Entity::find()
        .filter(mentorships::Column::Pid.eq(records::parse_pid(&pid)?))
        .filter(mentorships::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let sessions = mentorship_sessions::Entity::find()
        .filter(mentorship_sessions::Column::MentorshipPid.eq(mentorship.pid))
        .order_by_desc(mentorship_sessions::Column::HeldOn)
        .all(&ctx.db)
        .await?;
    format::json(serde_json::json!({ "mentorship": mentorship, "sessions": sessions }))
}
