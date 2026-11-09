//! Learning & development — the skills framework (catalog + declared
//! worker proficiency), learning paths (ordered course steps +
//! per-worker enrolment with honest progress), and mentorships
//! (lifecycle + session log). Declared/recorded data plus derived
//! views; the pure rules live in [`crate::rules::learning`], and the
//! progress derivation counts only real course completions.

use super::row_json;
use axa::prelude::*;
use sea_orm::{ActiveValue, PaginatorTrait, QueryOrder, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{
    development_plan_items, initiative_skill_shifts, mentorship_sessions, mentorships,
    role_skill_requirements, skill_external_refs, skills, worker_skills,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::learning as rules;
use crate::rules::skill_merge as merge_rules;
use crate::validation::Problems;

/// A `{pid}` reference response.
#[derive(serde::Serialize)]
struct PidRef {
    pid: String,
}

impl PidRef {
    fn of(pid: Uuid) -> Self {
        Self {
            pid: pid.to_string(),
        }
    }
}

mod catalogue;
mod mentoring;
mod paths;
mod views;

/// A learning path's pid and name.
struct FoundPath {
    pid: Uuid,
    name: String,
}

/// Find one live learning path by pid, or 404.
async fn find_path(ctx: &AppContext, pid: &str) -> Result<FoundPath> {
    sqlx::query_as!(
        FoundPath,
        "SELECT pid, name FROM learning_paths WHERE pid = $1 AND deleted_at IS NULL",
        records::parse_pid(pid)?,
    )
    .fetch_optional(crate::db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// The learning routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/skills", post(catalogue::create_skill))
        .add("/skills", get(catalogue::list_skills))
        .add(
            "/skills/category-suggestions",
            get(catalogue::category_suggestions),
        )
        .add(
            "/skills/category-suggestions/apply",
            post(catalogue::apply_suggestions),
        )
        .add("/skills/{pid}", put(catalogue::update_skill))
        .add("/skills/{pid}", delete(catalogue::delete_skill))
        .add("/skills/{pid}/usage", get(catalogue::skill_usage_view))
        .add("/skills/{pid}/merge", post(catalogue::merge_skill))
        .add("/skills/{pid}/refs", post(catalogue::add_skill_ref))
        .add(
            "/skills/{pid}/refs/{framework_slug}",
            delete(catalogue::remove_skill_ref),
        )
        .add("/workers/{pid}/skills", put(catalogue::declare_skill))
        .add("/workers/{pid}/skills", get(catalogue::list_worker_skills))
        .add("/learning-paths", post(paths::create_path))
        .add("/learning-paths", get(paths::list_paths))
        .add(
            "/learning-paths/{pid}/enrollments",
            post(paths::enroll_path),
        )
        .add("/learning-paths/{pid}/progress", get(paths::path_progress))
        .add("/mentorships", post(mentoring::create_mentorship))
        .add("/mentorships/{pid}", get(mentoring::get_mentorship))
        .add(
            "/mentorships/{pid}/status",
            post(mentoring::mentorship_status),
        )
        .add("/mentorships/{pid}/sessions", post(mentoring::log_session))
        .add("/learning/skills-matrix", get(views::skills_matrix))
        .add(
            "/learning/training-analytics",
            get(views::training_analytics),
        )
        .add(
            "/learning/mentorship-overview",
            get(views::mentorship_overview),
        )
}
