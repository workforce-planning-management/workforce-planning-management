//! **Skills matching and internal mobility** (WPM-R37, WPM-D28).
//!
//! Employee-facing by design: a worker sees which roles fit **their own**
//! declared skills, which open requisitions exist, and may express interest.
//! Nothing here ranks people against each other or picks anyone — the only
//! ordering is of roles, for one worker, by that worker's own fit — and
//! other people see only **aggregate counts** of interest, never who.
//! The pure rules live in [`crate::rules::mobility`] and
//! [`crate::rules::gap`].

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    mobility_interests, requisitions, role_profiles, role_skill_requirements, worker_skills,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::gap as gap_rules;
use crate::rules::mobility as rules;
use crate::rules::talent::ratio;
use crate::validation::Problems;

/// A `{pid}` reference response.
#[derive(Serialize)]
struct PidRef {
    pid: String,
}

fn ratio_json(terms: Option<(usize, usize, f64)>) -> serde_json::Value {
    terms.map_or(serde_json::Value::Null, |(numerator, denominator, value)| {
        serde_json::json!({ "numerator": numerator, "denominator": denominator, "value": value })
    })
}

/// A worker's readiness for one role profile, from their declarations.
fn readiness_for(
    requirements: &[&role_skill_requirements::Model],
    declared: &BTreeMap<Uuid, i32>,
) -> gap_rules::Readiness {
    let graded: Vec<(String, &'static str)> = requirements
        .iter()
        .map(|r| {
            (
                r.importance.clone(),
                gap_rules::grade(declared.get(&r.skill_pid).copied(), r.min_proficiency),
            )
        })
        .collect();
    let pairs: Vec<(&str, &str)> = graded.iter().map(|(i, g)| (i.as_str(), *g)).collect();
    gap_rules::readiness(&pairs)
}

/// Everything needed to compute one worker's fit for any role.
struct Fits {
    profiles: Vec<role_profiles::Model>,
    requirements: Vec<role_skill_requirements::Model>,
    declared: BTreeMap<Uuid, i32>,
}

async fn load_fits(ctx: &AppContext, worker_pid: Uuid) -> Result<Fits> {
    let profiles = role_profiles::Entity::find()
        .filter(role_profiles::Column::DeletedAt.is_null())
        .order_by_asc(role_profiles::Column::JobTitle)
        .all(&ctx.db)
        .await?;
    let requirements = role_skill_requirements::Entity::find().all(&ctx.db).await?;
    let declared: BTreeMap<Uuid, i32> = worker_skills::Entity::find()
        .filter(worker_skills::Column::WorkerPid.eq(worker_pid))
        .filter(worker_skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|d| (d.skill_pid, d.proficiency))
        .collect();
    Ok(Fits {
        profiles,
        requirements,
        declared,
    })
}

impl Fits {
    fn readiness_of(&self, profile_pid: Uuid) -> gap_rules::Readiness {
        let reqs: Vec<&role_skill_requirements::Model> = self
            .requirements
            .iter()
            .filter(|r| r.role_profile_pid == profile_pid)
            .collect();
        readiness_for(&reqs, &self.declared)
    }
}

fn fit_json(readiness: &gap_rules::Readiness) -> serde_json::Value {
    serde_json::json!({
        "critical_met": ratio_json(ratio(readiness.critical_met, readiness.critical_total)),
        "all_met": ratio_json(ratio(readiness.met, readiness.total)),
    })
}

/// `GET /api/workers/{pid}/role-matches` — every role profile (other than
/// the worker's current job title) ordered by how well **this worker's own
/// declared skills** fit it: critical requirements first, roles with no
/// requirements last. A self-service view; it compares declarations and
/// never ranks people.
#[debug_handler]
async fn role_matches(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let fits = load_fits(&ctx, worker.pid).await?;
    let mut rows: Vec<(&role_profiles::Model, gap_rules::Readiness)> = fits
        .profiles
        .iter()
        .filter(|p| !rules::same_title(&p.job_title, &worker.job_title))
        .map(|p| (p, fits.readiness_of(p.pid)))
        .collect();
    rows.sort_by(|a, b| {
        rules::compare_fit(&a.1, &b.1).then_with(|| a.0.job_title.cmp(&b.0.job_title))
    });
    let out: Vec<serde_json::Value> = rows
        .iter()
        .map(|(p, readiness)| {
            serde_json::json!({
                "role_profile_pid": p.pid,
                "job_title": p.job_title,
                "requirements": readiness.total,
                "fit": fit_json(readiness),
            })
        })
        .collect();
    format::json(serde_json::json!({
        "derivation": "roles ordered by the worker's OWN declared-skill fit (critical \
                       requirements first); a role with no requirements has no evidence and \
                       sorts last. This is a self-service view — it never ranks people.",
        "worker_pid": worker.pid,
        "roles": out,
    }))
}

/// `GET /api/workers/{pid}/opportunities` — open requisitions in the
/// worker's organization (other than their own job title), each with the
/// worker's fit when a role profile exists for that title.
#[debug_handler]
async fn opportunities(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let fits = load_fits(&ctx, worker.pid).await?;
    let open = requisitions::Entity::find()
        .filter(requisitions::Column::DeletedAt.is_null())
        .filter(requisitions::Column::OrganizationRef.eq(&worker.organization_ref))
        .filter(requisitions::Column::Status.is_in(["open", "interviewing"]))
        .order_by_asc(requisitions::Column::Id)
        .all(&ctx.db)
        .await?;
    let out: Vec<serde_json::Value> = open
        .iter()
        .filter(|r| !rules::same_title(&r.job_title, &worker.job_title))
        .map(|r| {
            let profile = fits
                .profiles
                .iter()
                .find(|p| rules::same_title(&p.job_title, &r.job_title));
            serde_json::json!({
                "requisition_pid": r.pid,
                "job_title": r.job_title,
                "department": r.department,
                "status": r.status,
                "fit": profile.map(|p| fit_json(&fits.readiness_of(p.pid))),
            })
        })
        .collect();
    format::json(serde_json::json!({
        "derivation": "open or interviewing requisitions in the worker's own organization, \
                       excluding the worker's current job title; `fit` is null when no role \
                       profile exists for that title (no evidence, not a poor fit)",
        "worker_pid": worker.pid,
        "opportunities": out,
    }))
}

/// `POST /api/workers/{pid}/mobility-interests` body.
#[derive(Debug, Deserialize)]
struct InterestPayload {
    #[serde(default)]
    role_profile_pid: Option<Uuid>,
    #[serde(default)]
    requisition_pid: Option<Uuid>,
    #[serde(default)]
    note: Option<String>,
}

/// `POST /api/workers/{pid}/mobility-interests` — express interest in a
/// role profile or an open requisition.
#[debug_handler]
async fn express_interest(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<InterestPayload>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let target = rules::interest_target(
        payload.role_profile_pid.is_some(),
        payload.requisition_pid.is_some(),
    )
    .map_err(|e| unprocessable(&e))?;
    let mut problems = Problems::new();
    if let Some(note) = &payload.note
        && note.chars().count() > rules::MAX_NOTE_LEN
    {
        problems.push(format!(
            "note is longer than {} characters",
            rules::MAX_NOTE_LEN
        ));
    }
    ensure_valid(&problems.into_vec())?;
    let mut live = mobility_interests::Entity::find()
        .filter(mobility_interests::Column::WorkerPid.eq(worker.pid))
        .filter(mobility_interests::Column::DeletedAt.is_null());
    match target {
        rules::Target::Role => {
            let role = payload.role_profile_pid.ok_or(Error::NotFound)?;
            role_profiles::Entity::find()
                .filter(role_profiles::Column::Pid.eq(role))
                .filter(role_profiles::Column::DeletedAt.is_null())
                .one(&ctx.db)
                .await?
                .ok_or(Error::NotFound)?;
            live = live.filter(mobility_interests::Column::RoleProfilePid.eq(role));
        }
        rules::Target::Requisition => {
            let requisition = payload.requisition_pid.ok_or(Error::NotFound)?;
            let found = requisitions::Entity::find()
                .filter(requisitions::Column::Pid.eq(requisition))
                .filter(requisitions::Column::DeletedAt.is_null())
                .one(&ctx.db)
                .await?
                .ok_or(Error::NotFound)?;
            if !matches!(found.status.as_str(), "open" | "interviewing") {
                return Err(unprocessable("the requisition is not open"));
            }
            live = live.filter(mobility_interests::Column::RequisitionPid.eq(requisition));
        }
    }
    if live.one(&ctx.db).await?.is_some() {
        return Err(unprocessable("interest already expressed"));
    }
    let row = mobility_interests::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        role_profile_pid: ActiveValue::set(payload.role_profile_pid),
        requisition_pid: ActiveValue::set(payload.requisition_pid),
        note: ActiveValue::set(payload.note.clone()),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "mobility_interest",
        row.pid,
        "expressed",
        caller.actor(),
        None,
    )
    .await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}

/// `GET /api/workers/{pid}/mobility-interests` — the worker's own
/// expressed interests, with the target's title.
#[debug_handler]
async fn list_interests(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let rows = mobility_interests::Entity::find()
        .filter(mobility_interests::Column::WorkerPid.eq(worker.pid))
        .filter(mobility_interests::Column::DeletedAt.is_null())
        .order_by_desc(mobility_interests::Column::Id)
        .all(&ctx.db)
        .await?;
    let titles = target_titles(&ctx).await?;
    let out: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let target = r.role_profile_pid.or(r.requisition_pid);
            serde_json::json!({
                "pid": r.pid,
                "kind": if r.role_profile_pid.is_some() { "role" } else { "requisition" },
                "target_pid": target,
                "title": target.and_then(|t| titles.get(&t)),
                "note": r.note,
            })
        })
        .collect();
    format::json(out)
}

/// pid → job title for every role profile and requisition.
async fn target_titles(ctx: &AppContext) -> Result<BTreeMap<Uuid, String>> {
    let mut titles = BTreeMap::new();
    for p in role_profiles::Entity::find().all(&ctx.db).await? {
        titles.insert(p.pid, p.job_title);
    }
    for r in requisitions::Entity::find().all(&ctx.db).await? {
        titles.insert(r.pid, r.job_title);
    }
    Ok(titles)
}

/// `DELETE /api/mobility-interests/{pid}` — withdraw an expressed interest.
#[debug_handler]
async fn withdraw_interest(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = mobility_interests::Entity::find()
        .filter(mobility_interests::Column::Pid.eq(records::parse_pid(&pid)?))
        .filter(mobility_interests::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let mut active: mobility_interests::ActiveModel = row.into();
    active.deleted_at = ActiveValue::set(Some(chrono::Utc::now().into()));
    let row = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "mobility_interest",
        row.pid,
        "withdrawn",
        caller.actor(),
        None,
    )
    .await?;
    format::empty_json()
}

/// `GET /api/mobility/interest-summary` — **aggregate only**: per role
/// profile and per open requisition, how many workers have expressed
/// interest. Never who (WPM-D28).
#[debug_handler]
async fn interest_summary(State(ctx): State<AppContext>) -> Result<Response> {
    let rows = mobility_interests::Entity::find()
        .filter(mobility_interests::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let titles = target_titles(&ctx).await?;
    let mut counts: BTreeMap<(&'static str, Uuid), usize> = BTreeMap::new();
    for row in &rows {
        if let Some(role) = row.role_profile_pid {
            *counts.entry(("role", role)).or_default() += 1;
        } else if let Some(requisition) = row.requisition_pid {
            *counts.entry(("requisition", requisition)).or_default() += 1;
        }
    }
    let mut out: Vec<serde_json::Value> = counts
        .iter()
        .map(|((kind, pid), interested)| {
            serde_json::json!({
                "kind": kind,
                "target_pid": pid,
                "title": titles.get(pid),
                "interested": interested,
            })
        })
        .collect();
    out.sort_by_key(|v| std::cmp::Reverse(v["interested"].as_u64().unwrap_or(0)));
    format::json(serde_json::json!({
        "derivation": "counts of expressed, un-withdrawn interest per target. Aggregate only: \
                       no worker is named (WPM-D28).",
        "targets": out,
    }))
}

/// The mobility routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/role-matches", get(role_matches))
        .add("/workers/{pid}/opportunities", get(opportunities))
        .add("/workers/{pid}/mobility-interests", post(express_interest))
        .add("/workers/{pid}/mobility-interests", get(list_interests))
        .add("/mobility-interests/{pid}", delete(withdraw_interest))
        .add("/mobility/interest-summary", get(interest_summary))
}
