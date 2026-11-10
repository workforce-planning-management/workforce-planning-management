//! Talent development (WPM-R10–R12): review cycles / reviews / goals
//! / feedback, training enrollments over the course registry, and
//! succession plans with the gap report. Review content is
//! high-sensitivity: reads are audited (WPM-D7).

use super::row_json;
use axa::extract::{Json, Path, Query};
use axa::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::models::_entities::{
    feedback_entries, goals, review_cycles, reviews, succession_candidates, succession_plans,
    training_enrollments, worker_goals,
};
use crate::models::records;
use crate::rules::{lifecycle, tokens};
use crate::streaming;
use crate::validation::Problems;

/// The live review cycle with this pid, else a 404.
async fn find_review_cycle(ctx: &AppContext, pid: Uuid) -> Result<review_cycles::Model> {
    sqlx::query_as!(
        review_cycles::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, period_start, period_end, status,
                  deleted_at AS "deleted_at: _"
             FROM review_cycles WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// The live performance review with this pid, else a 404. A PADR answers only in `padr.rs`.
async fn find_review(ctx: &AppContext, pid: Uuid) -> Result<reviews::Model> {
    sqlx::query_as!(
        reviews::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, cycle_pid, kind, worker_pid,
                  reviewer_ref, status, rating, content, deleted_at AS "deleted_at: _"
             FROM reviews WHERE pid = $1 AND kind = 'performance' AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// The live training enrolment with this pid, else a 404.
async fn find_training_enrollment(
    ctx: &AppContext,
    pid: Uuid,
) -> Result<training_enrollments::Model> {
    sqlx::query_as!(
        training_enrollments::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, course_ref, status,
                  completed_on, certificate_expires_on, deleted_at AS "deleted_at: _"
             FROM training_enrollments WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// The live succession plan with this pid, else a 404.
async fn find_succession_plan(ctx: &AppContext, pid: Uuid) -> Result<succession_plans::Model> {
    sqlx::query_as!(
        succession_plans::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, role_title, department, criticality,
                  incumbent_pid, risk_of_loss, vacancy_expected_on, deleted_at AS "deleted_at: _"
             FROM succession_plans WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// Freeze the goals of a review as it is shared (WPM-D165): each goal in the cycle's period gets a snapshot beside its outcomes,
/// in the transaction that shares it. The `sqlx` twin of `individual_goals::freeze_review_goals`, which is still `SeaORM`.
async fn freeze_review_goals(
    connection: &mut sqlx::PgConnection,
    review: &reviews::Model,
) -> Result<()> {
    let Some(cycle_pid) = review.cycle_pid else {
        return Ok(());
    };
    let Some(cycle) = sqlx::query!(
        "SELECT period_start, period_end FROM review_cycles WHERE pid = $1 LIMIT 1",
        cycle_pid,
    )
    .fetch_optional(&mut *connection)
    .await?
    else {
        return Ok(());
    };
    let goals = sqlx::query_as!(
        worker_goals::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, organization_ref, title,
                  description, period_start, period_end, status, proposed_by, agreed_on, skill_pid, skill_target_level,
                  plan_item_pid, aligns_to, deleted_at AS "deleted_at: _"
             FROM worker_goals WHERE worker_pid = $1 AND deleted_at IS NULL
            ORDER BY id"#,
        review.worker_pid,
    )
    .fetch_all(&mut *connection)
    .await?;
    for goal in goals.into_iter().filter(|g| {
        crate::rules::individual_goals::overlaps(
            (g.period_start, g.period_end),
            (cycle.period_start, cycle.period_end),
        )
    }) {
        let snapshot = super::padr::goal_snapshot(&mut *connection, &goal).await?;
        sqlx::query!(
            "INSERT INTO review_goal_outcomes (pid, review_pid, goal_pid, worker_pid, snapshot)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (review_pid, goal_pid) DO UPDATE SET snapshot = EXCLUDED.snapshot, updated_at = now()",
            Uuid::new_v4(),
            review.pid,
            goal.pid,
            goal.worker_pid,
            snapshot,
        )
        .execute(&mut *connection)
        .await?;
    }
    Ok(())
}

/// `POST /api/review-cycles` body.
#[derive(Debug, Deserialize)]
struct CyclePayload {
    name: String,
    period_start: chrono::NaiveDate,
    period_end: chrono::NaiveDate,
}

/// `POST /api/review-cycles/{pid}/reviews` body.
#[derive(Debug, Deserialize)]
struct ReviewPayload {
    worker_pid: Uuid,
    reviewer_ref: String,
}

/// `PUT /api/reviews/{pid}` body — author edits (draft only).
#[derive(Debug, Deserialize)]
struct ReviewUpdate {
    #[serde(default)]
    rating: Option<i32>,
    #[serde(default)]
    content: Option<String>,
}

/// `POST /api/reviews/{pid}/status` body.
#[derive(Debug, Deserialize)]
struct ReviewStatusPayload {
    to: String,
}

/// `POST /api/reviews/{pid}/goals` body.
#[derive(Debug, Deserialize)]
struct GoalPayload {
    title: String,
    #[serde(default = "default_weight")]
    weight_percent: i32,
}

/// `PUT /api/goals/{pid}` body.
#[derive(Debug, Deserialize)]
struct GoalUpdate {
    status: String,
}

/// `POST /api/reviews/{pid}/feedback` body.
#[derive(Debug, Deserialize)]
struct FeedbackPayload {
    author_ref: String,
    content: String,
}

/// `POST /api/workers/{pid}/training-enrollments` body.
#[derive(Debug, Deserialize)]
struct TrainingPayload {
    course_ref: String,
}

/// `PUT /api/training-enrollments/{pid}` body.
#[derive(Debug, Deserialize)]
struct TrainingUpdate {
    status: String,
    #[serde(default)]
    completed_on: Option<chrono::NaiveDate>,
    #[serde(default)]
    certificate_expires_on: Option<chrono::NaiveDate>,
}

/// `POST /api/succession-plans` body.
#[derive(Debug, Deserialize)]
struct SuccessionPayload {
    role_title: String,
    department: String,
    criticality: i32,
    #[serde(default)]
    incumbent_pid: Option<Uuid>,
    /// How likely the incumbent is to leave: `low` | `medium` | `high`.
    /// Together with `criticality` this is what makes a role a single
    /// point of failure (`/api/workforce-intelligence/succession`).
    #[serde(default)]
    risk_of_loss: Option<String>,
    /// When the role is expected to fall vacant, when that is known.
    #[serde(default)]
    vacancy_expected_on: Option<chrono::NaiveDate>,
}

/// `PUT /api/succession-plans/{pid}` body — restate the planning
/// judgements as they change. Every field is optional.
#[derive(Debug, Deserialize)]
struct SuccessionUpdate {
    #[serde(default)]
    criticality: Option<i32>,
    #[serde(default)]
    risk_of_loss: Option<String>,
    #[serde(default)]
    vacancy_expected_on: Option<chrono::NaiveDate>,
    #[serde(default)]
    incumbent_pid: Option<Uuid>,
}

/// `PUT /api/succession-candidates/{pid}` body — readiness moves as a
/// successor develops (or regresses).
#[derive(Debug, Deserialize)]
struct SuccessionCandidateUpdate {
    #[serde(default)]
    readiness: Option<String>,
    #[serde(default)]
    rank: Option<i32>,
}

/// `POST /api/succession-plans/{pid}/candidates` body.
#[derive(Debug, Deserialize)]
struct SuccessionCandidatePayload {
    worker_pid: Uuid,
    readiness: String,
    #[serde(default = "default_rank")]
    rank: i32,
}

/// A `{pid}` reference response.
#[derive(Debug, Serialize)]
struct PidRef {
    pid: String,
}

const fn default_weight() -> i32 {
    100
}
const fn default_rank() -> i32 {
    1
}

/// `POST /api/review-cycles`.
#[debug_handler]
async fn create_cycle(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<CyclePayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("name", &payload.name);
    if payload.period_end < payload.period_start {
        problems.push("period_end is before period_start".to_string());
    }
    ensure_valid(&problems.into_vec())?;
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO review_cycles (pid, name, period_start, period_end, status) VALUES ($1, $2, $3, $4, 'open')",
        pid,
        payload.name,
        payload.period_start,
        payload.period_end,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "review_cycle",
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

/// `GET /api/review-cycles`.
#[debug_handler]
async fn list_cycles(State(ctx): State<AppContext>) -> Result<Response> {
    let rows = sqlx::query_as!(
        review_cycles::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, period_start, period_end, status,
                  deleted_at AS "deleted_at: _"
             FROM review_cycles WHERE deleted_at IS NULL
            ORDER BY id LIMIT 200"#,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    format::json(rows)
}

/// `POST /api/review-cycles/{pid}/reviews` — open a draft review.
#[debug_handler]
async fn create_review(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ReviewPayload>,
) -> Result<Response> {
    let cycle = find_review_cycle(&ctx, records::parse_pid(&pid)?).await?;
    if cycle.status != "open" {
        return Err(unprocessable("review cycle is closed"));
    }
    let worker = records::worker_by_pid(&ctx, payload.worker_pid).await?;
    let mut problems = Problems::new();
    problems.require_ref(
        "reviewer_ref",
        entity_ref::EntityType::Worker,
        &payload.reviewer_ref,
    );
    ensure_valid(&problems.into_vec())?;
    let review_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO reviews (pid, cycle_pid, kind, worker_pid, reviewer_ref, status)
         VALUES ($1, $2, 'performance', $3, $4, 'draft')",
        review_pid,
        cycle.pid,
        worker.pid,
        payload.reviewer_ref,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "review",
        review_pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: review_pid.to_string(),
    })
}

/// `GET /api/workers/{pid}/reviews` — a worker's reviews.
/// Draft/submitted/calibrated content is redacted; `shared` reviews
/// carry content, and the content read is audited (WPM-R10).
#[debug_handler]
async fn list_reviews(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let rows = sqlx::query_as!(
        reviews::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, cycle_pid, kind, worker_pid,
                  reviewer_ref, status, rating, content, deleted_at AS "deleted_at: _"
             FROM reviews WHERE worker_pid = $1 AND kind = 'performance' AND deleted_at IS NULL
            ORDER BY id"#,
        worker.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let mut any_shared = false;
    let rows: Vec<reviews::Model> = rows
        .into_iter()
        .map(|mut review| {
            if review.status == "shared" {
                any_shared = true;
            } else {
                review.content = None;
                review.rating = None;
            }
            review
        })
        .collect();
    if any_shared {
        db::audit(
            &mut *db::pool(&ctx).acquire().await?,
            "worker",
            worker.pid,
            "review_content_read",
            caller.actor(),
            None,
        )
        .await?;
    }
    format::json(rows)
}

/// `PUT /api/reviews/{pid}` — author edits, drafts only.
#[debug_handler]
async fn update_review(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ReviewUpdate>,
) -> Result<Response> {
    let review = find_review(&ctx, records::parse_pid(&pid)?).await?;
    if review.status != "draft" {
        return Err(unprocessable(&format!(
            "review is {} — only drafts are editable",
            review.status
        )));
    }
    let mut problems = Problems::new();
    if let Some(rating) = payload.rating
        && !(1..=5).contains(&rating)
    {
        problems.push(format!("rating {rating} out of range 1-5"));
    }
    problems.cap_opt("content", payload.content.as_deref());
    ensure_valid(&problems.into_vec())?;
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        reviews::Model,
        r#"UPDATE reviews SET rating = COALESCE($2, rating), content = COALESCE($3, content), updated_at = now()
            WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, cycle_pid, kind, worker_pid,
                  reviewer_ref, status, rating, content, deleted_at AS "deleted_at: _""#,
        review.id,
        payload.rating,
        payload.content,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(&mut tx, "review", row.pid, "updated", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(row_json(&row)?)
}

/// `POST /api/reviews/{pid}/status` — one review transition
/// (`draft → submitted → calibrated → shared`).
#[debug_handler]
async fn review_status(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ReviewStatusPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token("to", tokens::REVIEW_STATUSES, &payload.to);
    ensure_valid(&problems.into_vec())?;
    let review = find_review(&ctx, records::parse_pid(&pid)?).await?;
    lifecycle::check("review", lifecycle::REVIEW, &review.status, &payload.to)
        .map_err(|e| unprocessable(&e))?;
    let mut tx = db::pool(&ctx).begin().await?;
    let from = review.status.clone();
    let row = sqlx::query_as!(
        reviews::Model,
        r#"UPDATE reviews SET status = $2, updated_at = now()
            WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, cycle_pid, kind, worker_pid,
                  reviewer_ref, status, rating, content, deleted_at AS "deleted_at: _""#,
        review.id,
        payload.to,
    )
    .fetch_one(&mut *tx)
    .await?;
    if row.status == "shared" {
        freeze_review_goals(&mut tx, &row).await?;
    }
    let snapshot = Some(serde_json::json!({ "from": from, "to": payload.to }));
    // One literal action per kind, so the audit guard can read each (WPM-D107).
    match payload.to.as_str() {
        "submitted" => {
            db::audit(
                &mut tx,
                "review",
                row.pid,
                "review_submitted",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
        "shared" => {
            db::audit(
                &mut tx,
                "review",
                row.pid,
                "review_shared",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
        _ => {
            db::audit(
                &mut tx,
                "review",
                row.pid,
                "review_status_changed",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
    }
    let kind = match payload.to.as_str() {
        "submitted" => "review_submitted",
        "shared" => "review_shared",
        _ => "review_status_changed",
    };
    streaming::emit_in(
        &mut tx,
        "review",
        kind,
        &row.pid.to_string(),
        "",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&row)?)
}

/// `POST /api/reviews/{pid}/goals`.
#[debug_handler]
async fn create_goal(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<GoalPayload>,
) -> Result<Response> {
    let review = find_review(&ctx, records::parse_pid(&pid)?).await?;
    let mut problems = Problems::new();
    problems.require_text("title", &payload.title);
    if !(1..=100).contains(&payload.weight_percent) {
        problems.push(format!(
            "weight_percent {} out of range 1-100",
            payload.weight_percent
        ));
    }
    ensure_valid(&problems.into_vec())?;
    let goal_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO goals (pid, review_pid, title, weight_percent, status) VALUES ($1, $2, $3, $4, 'open')",
        goal_pid,
        review.pid,
        payload.title,
        payload.weight_percent,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "goal", goal_pid, "created", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: goal_pid.to_string(),
    })
}

/// `PUT /api/goals/{pid}` — set the status.
#[debug_handler]
async fn update_goal(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<GoalUpdate>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token("status", tokens::GOAL_STATUSES, &payload.status);
    ensure_valid(&problems.into_vec())?;
    let pid = records::parse_pid(&pid)?;
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        goals::Model,
        r#"UPDATE goals SET status = $2, updated_at = now()
            WHERE pid = $1 AND deleted_at IS NULL
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, review_pid, title, weight_percent,
                  status, deleted_at AS "deleted_at: _""#,
        pid,
        payload.status,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;
    db::audit(&mut tx, "goal", row.pid, "updated", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(row_json(&row)?)
}

/// `GET /api/reviews/{pid}/goals` (+ feedback list alongside).
#[debug_handler]
async fn review_detail(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let review = find_review(&ctx, records::parse_pid(&pid)?).await?;
    let goal_rows = sqlx::query_as!(
        goals::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, review_pid, title, weight_percent,
                  status, deleted_at AS "deleted_at: _"
             FROM goals WHERE review_pid = $1 AND deleted_at IS NULL
            ORDER BY id"#,
        review.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let feedback = sqlx::query_as!(
        feedback_entries::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, review_pid, author_ref, content,
                  deleted_at AS "deleted_at: _"
             FROM feedback_entries WHERE review_pid = $1 AND deleted_at IS NULL
            ORDER BY id"#,
        review.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    // Reading a review with content is a sensitive read (WPM-D7).
    if review.content.is_some() {
        db::audit(
            &mut *db::pool(&ctx).acquire().await?,
            "review",
            review.pid,
            "review_content_read",
            caller.actor(),
            None,
        )
        .await?;
    }
    format::json(serde_json::json!({
        "review": review, "goals": goal_rows, "feedback": feedback,
    }))
}

/// `POST /api/reviews/{pid}/feedback`.
#[debug_handler]
async fn create_feedback(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<FeedbackPayload>,
) -> Result<Response> {
    let review = find_review(&ctx, records::parse_pid(&pid)?).await?;
    let mut problems = Problems::new();
    problems.require_ref(
        "author_ref",
        entity_ref::EntityType::Worker,
        &payload.author_ref,
    );
    problems.require_text("content", &payload.content);
    ensure_valid(&problems.into_vec())?;
    let entry_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO feedback_entries (pid, review_pid, author_ref, content) VALUES ($1, $2, $3, $4)",
        entry_pid,
        review.pid,
        payload.author_ref,
        payload.content,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "feedback_entry",
        entry_pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: entry_pid.to_string(),
    })
}

/// `POST /api/workers/{pid}/training-enrollments` — enrol against a
/// `course:` / `courseinstance:` URN (WPM-D10).
#[debug_handler]
async fn create_training(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<TrainingPayload>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    // Accept either course-family entity type.
    let parsed: std::result::Result<entity_ref::EntityRef, _> = payload.course_ref.parse();
    match parsed {
        Ok(r)
            if r.entity_type == entity_ref::EntityType::Course
                || r.entity_type == entity_ref::EntityType::CourseInstance => {}
        _ => {
            return Err(unprocessable(
                "course_ref must be a course: or courseinstance: URN",
            ));
        }
    }
    let enrollment_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO training_enrollments (pid, worker_pid, course_ref, status) VALUES ($1, $2, $3, 'enrolled')",
        enrollment_pid,
        worker.pid,
        payload.course_ref,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "training_enrollment",
        enrollment_pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: enrollment_pid.to_string(),
    })
}

/// `GET /api/workers/{pid}/training-enrollments`.
#[debug_handler]
async fn list_training(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let rows = sqlx::query_as!(
        training_enrollments::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, course_ref, status,
                  completed_on, certificate_expires_on, deleted_at AS "deleted_at: _"
             FROM training_enrollments WHERE worker_pid = $1 AND deleted_at IS NULL
            ORDER BY id"#,
        worker.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    // each row as it is, plus the link to its course in the learning system (WPM-R138)
    let rows: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            let lms_url = super::lms::course_url(&row.course_ref);
            let mut json = serde_json::to_value(&row).unwrap_or(serde_json::Value::Null);
            if let Some(object) = json.as_object_mut() {
                object.insert("lms_url".to_string(), serde_json::json!(lms_url));
            }
            json
        })
        .collect();
    format::json(rows)
}

/// `PUT /api/training-enrollments/{pid}` — progress the enrolment.
#[debug_handler]
async fn update_training(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<TrainingUpdate>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token("status", tokens::TRAINING_STATUSES, &payload.status);
    ensure_valid(&problems.into_vec())?;
    let enrollment = find_training_enrollment(&ctx, records::parse_pid(&pid)?).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        training_enrollments::Model,
        r#"UPDATE training_enrollments
              SET status = $2, completed_on = COALESCE($3, completed_on),
                  certificate_expires_on = COALESCE($4, certificate_expires_on), updated_at = now()
            WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, course_ref, status,
                  completed_on, certificate_expires_on, deleted_at AS "deleted_at: _""#,
        enrollment.id,
        payload.status,
        payload.completed_on,
        payload.certificate_expires_on,
    )
    .fetch_one(&mut *tx)
    .await?;
    let kind = if payload.status == "completed" {
        db::audit(
            &mut tx,
            "training_enrollment",
            row.pid,
            "training_completed",
            caller.actor(),
            None,
        )
        .await?;
        "training_completed"
    } else {
        db::audit(
            &mut tx,
            "training_enrollment",
            row.pid,
            "updated",
            caller.actor(),
            None,
        )
        .await?;
        "updated"
    };
    streaming::emit_in(
        &mut tx,
        "training_enrollment",
        kind,
        &row.pid.to_string(),
        &row.course_ref,
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&row)?)
}

/// `GET /api/training/expiring?within_days=` — certificates expiring
/// soon (WPM-R11).
#[derive(Debug, Deserialize)]
struct ExpiringParams {
    #[serde(default = "default_within")]
    within_days: i64,
}

const fn default_within() -> i64 {
    90
}

#[debug_handler]
async fn expiring_training(
    State(ctx): State<AppContext>,
    Query(params): Query<ExpiringParams>,
) -> Result<Response> {
    let today = chrono::Utc::now().date_naive();
    let horizon = today + chrono::Duration::days(params.within_days.clamp(0, 3650));
    let rows = sqlx::query_as!(
        training_enrollments::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, course_ref, status,
                  completed_on, certificate_expires_on, deleted_at AS "deleted_at: _"
             FROM training_enrollments
            WHERE deleted_at IS NULL AND certificate_expires_on IS NOT NULL AND certificate_expires_on <= $1
            ORDER BY certificate_expires_on, id LIMIT 500"#,
        horizon,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    format::json(serde_json::json!({ "as_of": today, "horizon": horizon, "expiring": rows }))
}

/// `POST /api/succession-plans`.
#[debug_handler]
async fn create_succession(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<SuccessionPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("role_title", &payload.role_title);
    problems.require_text("department", &payload.department);
    if !(1..=5).contains(&payload.criticality) {
        problems.push(format!(
            "criticality {} out of range 1-5",
            payload.criticality
        ));
    }
    problems.token_opt(
        "risk_of_loss",
        crate::rules::talent::RISK_OF_LOSS,
        payload.risk_of_loss.as_deref(),
    );
    ensure_valid(&problems.into_vec())?;
    if let Some(incumbent) = payload.incumbent_pid {
        records::worker_by_pid(&ctx, incumbent).await?;
    }
    let plan_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO succession_plans (pid, role_title, department, criticality, incumbent_pid, risk_of_loss,
                                       vacancy_expected_on)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        plan_pid,
        payload.role_title,
        payload.department,
        payload.criticality,
        payload.incumbent_pid,
        payload.risk_of_loss,
        payload.vacancy_expected_on,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "succession_plan",
        plan_pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: plan_pid.to_string(),
    })
}

/// `GET /api/succession-plans` — plans + candidates; the read is
/// audited (high-sensitivity, WPM-R12).
#[debug_handler]
async fn list_succession(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let plans = sqlx::query_as!(
        succession_plans::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, role_title, department, criticality,
                  incumbent_pid, risk_of_loss, vacancy_expected_on, deleted_at AS "deleted_at: _"
             FROM succession_plans WHERE deleted_at IS NULL
            ORDER BY id LIMIT 500"#,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let plan_pids: Vec<Uuid> = plans.iter().map(|p| p.pid).collect();
    let mut bench: std::collections::HashMap<Uuid, Vec<succession_candidates::Model>> =
        std::collections::HashMap::new();
    for candidate in sqlx::query_as!(
        succession_candidates::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, plan_pid, worker_pid, readiness, rank,
                  deleted_at AS "deleted_at: _"
             FROM succession_candidates WHERE plan_pid = ANY($1) AND deleted_at IS NULL
            ORDER BY rank, id"#,
        &plan_pids,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    {
        bench.entry(candidate.plan_pid).or_default().push(candidate);
    }
    let mut out = Vec::with_capacity(plans.len());
    for plan in plans {
        let candidates = bench.remove(&plan.pid).unwrap_or_default();
        out.push(serde_json::json!({ "plan": plan, "candidates": candidates }));
    }
    if let Some(actor) = caller.actor() {
        db::audit(
            &mut *db::pool(&ctx).acquire().await?,
            "succession_plan",
            Uuid::nil(),
            "succession_read",
            Some(actor),
            None,
        )
        .await?;
    }
    format::json(out)
}

/// `POST /api/succession-plans/{pid}/candidates`.
#[debug_handler]
async fn add_succession_candidate(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<SuccessionCandidatePayload>,
) -> Result<Response> {
    let plan = find_succession_plan(&ctx, records::parse_pid(&pid)?).await?;
    let worker = records::worker_by_pid(&ctx, payload.worker_pid).await?;
    let mut problems = Problems::new();
    problems.require_token("readiness", tokens::READINESS, &payload.readiness);
    if payload.rank < 1 {
        problems.push("rank must be at least 1".to_string());
    }
    ensure_valid(&problems.into_vec())?;
    let candidate_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO succession_candidates (pid, plan_pid, worker_pid, readiness, rank) VALUES ($1, $2, $3, $4, $5)",
        candidate_pid,
        plan.pid,
        worker.pid,
        payload.readiness,
        payload.rank,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "succession_candidate",
        candidate_pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: candidate_pid.to_string(),
    })
}

/// `GET /api/succession-plans/gaps` — critical roles (criticality ≥ 4)
/// with no `ready_now` candidate (WPM-R12).
#[debug_handler]
async fn succession_gaps(State(ctx): State<AppContext>) -> Result<Response> {
    let gaps = sqlx::query_as!(
        succession_plans::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, role_title, department, criticality,
                  incumbent_pid, risk_of_loss, vacancy_expected_on, deleted_at AS "deleted_at: _"
             FROM succession_plans p
            WHERE deleted_at IS NULL AND criticality >= 4
              AND NOT EXISTS (SELECT 1 FROM succession_candidates c
                               WHERE c.plan_pid = p.pid AND c.deleted_at IS NULL AND c.readiness = 'ready_now')
            ORDER BY id"#,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    format::json(serde_json::json!({ "gaps": gaps }))
}

/// `PUT /api/succession-plans/{pid}` — restate a plan's criticality,
/// risk of loss, expected vacancy date, or incumbent as the picture
/// changes. Succession judgements go stale faster than anything else in
/// WPM, so they must be updatable without deleting the plan and its
/// bench.
#[debug_handler]
async fn update_succession(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<SuccessionUpdate>,
) -> Result<Response> {
    let plan = find_succession_plan(&ctx, records::parse_pid(&pid)?).await?;
    let mut problems = Problems::new();
    if let Some(criticality) = payload.criticality
        && !(1..=5).contains(&criticality)
    {
        problems.push(format!("criticality {criticality} out of range 1-5"));
    }
    problems.token_opt(
        "risk_of_loss",
        crate::rules::talent::RISK_OF_LOSS,
        payload.risk_of_loss.as_deref(),
    );
    ensure_valid(&problems.into_vec())?;
    if let Some(incumbent) = payload.incumbent_pid {
        records::worker_by_pid(&ctx, incumbent).await?;
    }

    let plan_pid = plan.pid;
    let before = serde_json::json!({
        "criticality": plan.criticality,
        "risk_of_loss": plan.risk_of_loss,
        "vacancy_expected_on": plan.vacancy_expected_on,
        "incumbent_pid": plan.incumbent_pid,
    });
    let mut tx = db::pool(&ctx).begin().await?;
    let updated = sqlx::query_as!(
        succession_plans::Model,
        r#"UPDATE succession_plans
              SET criticality = COALESCE($2, criticality), risk_of_loss = COALESCE($3, risk_of_loss),
                  vacancy_expected_on = COALESCE($4, vacancy_expected_on), incumbent_pid = COALESCE($5, incumbent_pid),
                  updated_at = now()
            WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, role_title, department, criticality,
                  incumbent_pid, risk_of_loss, vacancy_expected_on, deleted_at AS "deleted_at: _""#,
        plan.id,
        payload.criticality,
        payload.risk_of_loss,
        payload.vacancy_expected_on,
        payload.incumbent_pid,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "succession_plan",
        plan_pid,
        "updated",
        caller.actor(),
        Some(serde_json::json!({ "before": before })),
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&updated)?)
}

/// `PUT /api/succession-candidates/{pid}` — move a successor's
/// readiness or rank. Readiness may go **down** as well as up: a bench
/// that can only improve on paper would overstate the organisation's
/// cover.
#[debug_handler]
async fn update_succession_candidate(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<SuccessionCandidateUpdate>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.token_opt("readiness", tokens::READINESS, payload.readiness.as_deref());
    if payload.rank.is_some_and(|r| r < 1) {
        problems.push("rank must be 1 or greater");
    }
    ensure_valid(&problems.into_vec())?;

    let candidate = sqlx::query!(
        "SELECT id, pid, readiness FROM succession_candidates WHERE pid = $1 AND deleted_at IS NULL",
        records::parse_pid(&pid)?,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let candidate_pid = candidate.pid;
    let from = candidate.readiness;
    let mut tx = db::pool(&ctx).begin().await?;
    let updated = sqlx::query_as!(
        succession_candidates::Model,
        r#"UPDATE succession_candidates
              SET readiness = COALESCE($2, readiness), rank = COALESCE($3, rank), updated_at = now()
            WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, plan_pid, worker_pid, readiness, rank,
                  deleted_at AS "deleted_at: _""#,
        candidate.id,
        payload.readiness,
        payload.rank,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "succession_candidate",
        candidate_pid,
        "updated",
        caller.actor(),
        Some(serde_json::json!({ "readiness_from": from, "readiness_to": payload.readiness })),
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&updated)?)
}

/// The development routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/review-cycles", post(create_cycle))
        .add("/review-cycles", get(list_cycles))
        .add("/review-cycles/{pid}/reviews", post(create_review))
        .add("/workers/{pid}/reviews", get(list_reviews))
        .add("/reviews/{pid}", put(update_review))
        .add("/reviews/{pid}", get(review_detail))
        .add("/reviews/{pid}/status", post(review_status))
        .add("/reviews/{pid}/goals", post(create_goal))
        .add("/goals/{pid}", put(update_goal))
        .add("/reviews/{pid}/feedback", post(create_feedback))
        .add("/workers/{pid}/training-enrollments", post(create_training))
        .add("/workers/{pid}/training-enrollments", get(list_training))
        .add("/training-enrollments/{pid}", put(update_training))
        .add("/training/expiring", get(expiring_training))
        .add("/succession-plans", post(create_succession))
        .add("/succession-plans", get(list_succession))
        .add("/succession-plans/gaps", get(succession_gaps))
        .add("/succession-plans/{pid}", put(update_succession))
        .add(
            "/succession-plans/{pid}/candidates",
            post(add_succession_candidate),
        )
        .add(
            "/succession-candidates/{pid}",
            put(update_succession_candidate),
        )
}
