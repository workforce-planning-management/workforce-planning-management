//! Learning paths: ordered course steps, enrolment, and honest progress.

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;
use axa::extract::{Json, Path};

/// `POST /api/learning-paths` body (steps declared inline, ordered).
#[derive(Debug, Deserialize)]
pub(super) struct PathPayload {
    name: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    steps: Vec<PathStepPayload>,
}

/// One step of a learning path.
#[derive(Debug, Deserialize)]
pub(super) struct PathStepPayload {
    course_ref: String,
    title: String,
}

/// `POST /api/learning-paths` — create a path with its ordered steps.
#[debug_handler]
pub(super) async fn create_path(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<PathPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("name", &payload.name);
    problems.cap_text("name", &payload.name);
    for (index, step) in payload.steps.iter().enumerate() {
        problems.require_text(&format!("steps[{index}].course_ref"), &step.course_ref);
        problems.require_text(&format!("steps[{index}].title"), &step.title);
    }
    ensure_valid(&problems.into_vec())?;
    let path = Uuid::new_v4();
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO learning_paths (pid, name, summary) VALUES ($1, $2, $3)",
        path,
        payload.name,
        payload.summary,
    )
    .execute(&mut *tx)
    .await?;
    // The steps in one statement, each at its place in the order given (from 0).
    let courses: Vec<String> = payload.steps.iter().map(|s| s.course_ref.clone()).collect();
    let titles: Vec<String> = payload.steps.iter().map(|s| s.title.clone()).collect();
    sqlx::query!(
        "INSERT INTO learning_path_steps (pid, path_pid, course_ref, title, position)
         SELECT gen_random_uuid(), $1, step.course_ref, step.title, (step.n - 1)::int
           FROM unnest($2::text[], $3::text[]) WITH ORDINALITY AS step(course_ref, title, n)",
        path,
        &courses,
        &titles,
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "learning_path",
        path,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef::of(path))
}

/// `GET /api/learning-paths` — all paths with their step counts.
#[debug_handler]
pub(super) async fn list_paths(State(ctx): State<AppContext>) -> Result<Response> {
    // Each path with its step count, counted by the database.
    let view: Vec<serde_json::Value> = sqlx::query!(
        r#"SELECT p.pid, p.name, p.summary,
                  (SELECT COUNT(*) FROM learning_path_steps s WHERE s.path_pid = p.pid) AS "steps!"
             FROM learning_paths p WHERE p.deleted_at IS NULL ORDER BY p.id"#
    )
    .fetch_all(crate::db::pool(&ctx))
    .await?
    .into_iter()
    .map(|path| {
        serde_json::json!({
            "pid": path.pid, "name": path.name, "summary": path.summary, "steps": path.steps,
        })
    })
    .collect();
    format::json(view)
}

/// `POST /api/learning-paths/{pid}/enrollments` body.
#[derive(Debug, Deserialize)]
pub(super) struct PathEnrollPayload {
    worker_pid: Uuid,
}

/// `POST /api/learning-paths/{pid}/enrollments` — enrol a worker
/// (idempotent per worker+path).
#[debug_handler]
pub(super) async fn enroll_path(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<PathEnrollPayload>,
) -> Result<Response> {
    let path = find_path(&ctx, &pid).await?;
    let worker = records::worker_by_pid(&ctx, payload.worker_pid).await?;
    let db = crate::db::pool(&ctx);
    let existing = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM path_enrollments WHERE path_pid = $1 AND worker_pid = $2 AND deleted_at IS NULL)
             AS "exists!""#,
        path.pid,
        worker.pid,
    )
    .fetch_one(db)
    .await?;
    if existing {
        return Err(unprocessable("worker is already enrolled in this path"));
    }
    let enrollment = Uuid::new_v4();
    let mut tx = db.begin().await?;
    sqlx::query!(
        "INSERT INTO path_enrollments (pid, path_pid, worker_pid, enrolled_on) VALUES ($1, $2, $3, $4)",
        enrollment,
        path.pid,
        worker.pid,
        chrono::Utc::now().date_naive(),
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "path_enrollment",
        enrollment,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef::of(enrollment))
}

/// `GET /api/learning-paths/{pid}/progress` — each enrolled worker's
/// honest progress: steps completed (a step counts only if the
/// worker has a **completed** training enrolment for that
/// `course_ref`) out of the path's step count.
#[debug_handler]
pub(super) async fn path_progress(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let path = find_path(&ctx, &pid).await?;
    let db = crate::db::pool(&ctx);
    let steps = sqlx::query!(
        "SELECT course_ref, title, position FROM learning_path_steps WHERE path_pid = $1 ORDER BY position, id",
        path.pid,
    )
    .fetch_all(db)
    .await?;
    let step_courses: Vec<String> = steps.iter().map(|s| s.course_ref.clone()).collect();
    // Each enrolled worker with their name and the courses they have completed, in one read, where it was two reads
    // per worker.
    let enrolled = sqlx::query!(
        r#"SELECT e.worker_pid, w.display_name AS "display_name?",
                  ARRAY(SELECT t.course_ref FROM training_enrollments t
                         WHERE t.worker_pid = e.worker_pid AND t.deleted_at IS NULL AND t.status = 'completed')
                      AS "completed!"
             FROM path_enrollments e LEFT JOIN workers w ON w.pid = e.worker_pid
            WHERE e.path_pid = $1 AND e.deleted_at IS NULL
            ORDER BY e.id"#,
        path.pid,
    )
    .fetch_all(db)
    .await?;
    let members: Vec<serde_json::Value> = enrolled
        .iter()
        .map(|e| {
            let (done, total) = rules::path_progress(&step_courses, &e.completed);
            serde_json::json!({
                "worker_pid": e.worker_pid,
                "display_name": e.display_name,
                "completed_steps": done,
                "total_steps": total,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "as_of": chrono::Utc::now(),
        "path": { "pid": path.pid, "name": path.name },
        "steps": steps.iter().map(|s| serde_json::json!({
            "course_ref": s.course_ref, "title": s.title, "position": s.position,
            "lms_url": super::super::lms::course_url(&s.course_ref),
        })).collect::<Vec<_>>(),
        "derivation": "a step is complete iff the worker has a completed training \
                       enrolment for its course_ref; real completions only",
        "members": members,
    }))
}
