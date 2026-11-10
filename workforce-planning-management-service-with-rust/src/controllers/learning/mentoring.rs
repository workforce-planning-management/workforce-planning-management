//! Mentorships: lifecycle and session log.

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;
use axa::extract::{Json, Path};

/// One live mentorship by pid, or 404.
async fn find_mentorship(ctx: &AppContext, pid: &str) -> Result<mentorships::Model> {
    sqlx::query_as!(
        mentorships::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, mentor_pid, mentee_pid, focus, status,
                  started_on, ended_on, deleted_at AS "deleted_at: _"
             FROM mentorships WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(crate::db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

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
    records::worker_by_pid(&ctx, payload.mentor_pid).await?;
    records::worker_by_pid(&ctx, payload.mentee_pid).await?;
    let pid = Uuid::new_v4();
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO mentorships (pid, mentor_pid, mentee_pid, focus, status) VALUES ($1, $2, $3, $4, 'proposed')",
        pid,
        payload.mentor_pid,
        payload.mentee_pid,
        payload.focus,
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(&mut tx, "mentorship", pid, "created", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(PidRef::of(pid))
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
    let row = find_mentorship(&ctx, &pid).await?;
    rules::mentorship_transition(&row.status, &payload.to)
        .map_err(|reason| unprocessable(&reason))?;
    let today = chrono::Utc::now().date_naive();
    let from = row.status.clone();
    let mut tx = crate::db::pool(&ctx).begin().await?;
    // Activating stamps `started_on`, closing stamps `ended_on`; any other move leaves both.
    let updated = sqlx::query_as!(
        mentorships::Model,
        r#"UPDATE mentorships SET status = $2::text,
                  started_on = CASE WHEN $2::text = 'active' THEN $3 ELSE started_on END,
                  ended_on = CASE WHEN $2::text IN ('completed', 'ended') THEN $3 ELSE ended_on END
            WHERE id = $1
            RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, mentor_pid, mentee_pid, focus, status,
                  started_on, ended_on, deleted_at AS "deleted_at: _""#,
        row.id,
        payload.to,
        today,
    )
    .fetch_one(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "mentorship",
        row.pid,
        "status_changed",
        caller.actor(),
        Some(serde_json::json!({ "from": from, "to": payload.to })),
    )
    .await?;
    tx.commit().await?;
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
    let mentorship = find_mentorship(&ctx, &pid).await?;
    if mentorship.status != "active" {
        return Err(unprocessable(
            "sessions can only be logged on an active mentorship",
        ));
    }
    let mut problems = Problems::new();
    problems.require_text("notes", &payload.notes);
    problems.cap_text("notes", &payload.notes);
    ensure_valid(&problems.into_vec())?;
    let session = Uuid::new_v4();
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO mentorship_sessions (pid, mentorship_pid, held_on, notes) VALUES ($1, $2, $3, $4)",
        session,
        mentorship.pid,
        payload.held_on,
        payload.notes,
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "mentorship_session",
        session,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef::of(session))
}

/// `GET /api/mentorships/{pid}` — one mentorship + its session log.
#[debug_handler]
pub(super) async fn get_mentorship(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let mentorship = find_mentorship(&ctx, &pid).await?;
    let sessions = sqlx::query_as!(
        mentorship_sessions::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, mentorship_pid, held_on, notes
             FROM mentorship_sessions WHERE mentorship_pid = $1 ORDER BY held_on DESC"#,
        mentorship.pid,
    )
    .fetch_all(crate::db::pool(&ctx))
    .await?;
    format::json(serde_json::json!({ "mentorship": mentorship, "sessions": sessions }))
}
