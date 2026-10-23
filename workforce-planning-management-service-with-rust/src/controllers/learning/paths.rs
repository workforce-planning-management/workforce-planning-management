//! Learning paths: ordered course steps, enrolment, and honest progress.

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

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
    let txn = ctx.db.begin().await?;
    let path = learning_paths::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        name: ActiveValue::set(payload.name.clone()),
        summary: ActiveValue::set(payload.summary.clone()),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    for (index, step) in payload.steps.iter().enumerate() {
        learning_path_steps::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            path_pid: ActiveValue::set(path.pid),
            course_ref: ActiveValue::set(step.course_ref.clone()),
            title: ActiveValue::set(step.title.clone()),
            position: ActiveValue::set(i32::try_from(index).unwrap_or(i32::MAX)),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
    }
    Audit::record(
        &txn,
        "learning_path",
        path.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(PidRef::of(path.pid))
}

/// `GET /api/learning-paths` — all paths with their step counts.
#[debug_handler]
pub(super) async fn list_paths(State(ctx): State<AppContext>) -> Result<Response> {
    let paths = learning_paths::Entity::find()
        .filter(learning_paths::Column::DeletedAt.is_null())
        .order_by_asc(learning_paths::Column::Id)
        .all(&ctx.db)
        .await?;
    let steps = learning_path_steps::Entity::find().all(&ctx.db).await?;
    let mut counts: std::collections::BTreeMap<Uuid, usize> = std::collections::BTreeMap::new();
    for step in &steps {
        *counts.entry(step.path_pid).or_default() += 1;
    }
    let view: Vec<serde_json::Value> = paths
        .iter()
        .map(|path| {
            serde_json::json!({
                "pid": path.pid, "name": path.name, "summary": path.summary,
                "steps": counts.get(&path.pid).copied().unwrap_or(0),
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
    let worker = records::find_worker(&ctx.db, payload.worker_pid).await?;
    let existing = path_enrollments::Entity::find()
        .filter(path_enrollments::Column::PathPid.eq(path.pid))
        .filter(path_enrollments::Column::WorkerPid.eq(worker.pid))
        .filter(path_enrollments::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?;
    if existing.is_some() {
        return Err(unprocessable("worker is already enrolled in this path"));
    }
    let txn = ctx.db.begin().await?;
    let row = path_enrollments::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        path_pid: ActiveValue::set(path.pid),
        worker_pid: ActiveValue::set(worker.pid),
        enrolled_on: ActiveValue::set(chrono::Utc::now().date_naive()),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "path_enrollment",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(PidRef::of(row.pid))
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
    let mut steps = learning_path_steps::Entity::find()
        .filter(learning_path_steps::Column::PathPid.eq(path.pid))
        .all(&ctx.db)
        .await?;
    steps.sort_by_key(|step| step.position);
    let step_courses: Vec<String> = steps.iter().map(|s| s.course_ref.clone()).collect();
    let enrollments = path_enrollments::Entity::find()
        .filter(path_enrollments::Column::PathPid.eq(path.pid))
        .filter(path_enrollments::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let mut members = Vec::new();
    for enrollment in &enrollments {
        let completed: Vec<String> = training_enrollments::Entity::find()
            .filter(training_enrollments::Column::WorkerPid.eq(enrollment.worker_pid))
            .filter(training_enrollments::Column::DeletedAt.is_null())
            .filter(training_enrollments::Column::Status.eq("completed"))
            .all(&ctx.db)
            .await?
            .into_iter()
            .map(|t| t.course_ref)
            .collect();
        let (done, total) = rules::path_progress(&step_courses, &completed);
        let worker = workers::Entity::find()
            .filter(workers::Column::Pid.eq(enrollment.worker_pid))
            .one(&ctx.db)
            .await?;
        members.push(serde_json::json!({
            "worker_pid": enrollment.worker_pid,
            "display_name": worker.map(|e| e.display_name),
            "completed_steps": done,
            "total_steps": total,
        }));
    }
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
