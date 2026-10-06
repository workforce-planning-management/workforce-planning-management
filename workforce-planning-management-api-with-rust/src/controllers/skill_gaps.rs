//! **Skills gap analysis** (`rules::skill_gap`): what people need against
//! what they have declared, ranked.
//!
//! - `GET /api/workers/{pid}/skill-gaps` — one person: the needs of their
//!   current role (the UK GDAD PCF role profile's requirements), their own
//!   skill targets, and — only for the person themself or HR — their skill
//!   aspirations; merged per skill, graded, ranked.
//! - `GET /api/workforce-intelligence/skill-gaps` — the workforce: per skill,
//!   how many people need it, meet it, are below it, or have not declared it,
//!   ranked by importance × levels short. **Counts only — nobody is named,
//!   and aspirations are never included** (they are private unless shared).
//!
//! It compares **declarations**: an undeclared skill is unknown, never zero.

use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::QueryOrder;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

use super::record_rejection;
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{
    role_skill_requirements, skills, worker_aspirations, worker_framework_roles, worker_skills,
    workers,
};
use crate::models::{memberships, records};
use crate::rules::metrics::is_employed_on;
use crate::rules::skill_gap::{self as rules, Gap, Need, Source, Status};
use crate::tasks::import_framework::PCF_SLUG;

/// Aspirations that still count as a want (not achieved, not dropped).
const ACTIVE_ASPIRATIONS: &[&str] = &["idea", "planned", "in_progress"];

/// The needs and declarations behind a set of workers, loaded in bulk.
pub(crate) struct Evidence {
    /// Role requirements by role profile.
    requirements: HashMap<Uuid, Vec<role_skill_requirements::Model>>,
    /// Each worker's current PCF role profile.
    roles: HashMap<Uuid, Uuid>,
    /// Each worker's declarations: skill → (proficiency, target).
    declared: HashMap<Uuid, BTreeMap<Uuid, (i32, Option<i32>)>>,
    /// Active skill aspirations by worker (loaded only when asked).
    aspirations: HashMap<Uuid, Vec<worker_aspirations::Model>>,
}

pub(crate) async fn load_evidence(
    ctx: &AppContext,
    worker_pids: &[Uuid],
    with_aspirations: bool,
) -> Result<Evidence> {
    let roles_rows = worker_framework_roles::Entity::find()
        .filter(worker_framework_roles::Column::WorkerPid.is_in(worker_pids.to_vec()))
        .filter(worker_framework_roles::Column::FrameworkSlug.eq(PCF_SLUG))
        .filter(worker_framework_roles::Column::EndedAt.is_null())
        .all(&ctx.db)
        .await?;
    let roles: HashMap<Uuid, Uuid> = roles_rows
        .iter()
        .filter_map(|r| r.role_profile_pid.map(|p| (r.worker_pid, p)))
        .collect();
    let profile_ids: Vec<Uuid> = roles.values().copied().collect();
    let mut requirements: HashMap<Uuid, Vec<role_skill_requirements::Model>> = HashMap::new();
    for r in role_skill_requirements::Entity::find()
        .filter(role_skill_requirements::Column::RoleProfilePid.is_in(profile_ids))
        .all(&ctx.db)
        .await?
    {
        requirements.entry(r.role_profile_pid).or_default().push(r);
    }
    let mut declared: HashMap<Uuid, BTreeMap<Uuid, (i32, Option<i32>)>> = HashMap::new();
    for d in worker_skills::Entity::find()
        .filter(worker_skills::Column::WorkerPid.is_in(worker_pids.to_vec()))
        .filter(worker_skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
    {
        declared
            .entry(d.worker_pid)
            .or_default()
            .insert(d.skill_pid, (d.proficiency, d.target));
    }
    let mut aspirations: HashMap<Uuid, Vec<worker_aspirations::Model>> = HashMap::new();
    if with_aspirations {
        for a in worker_aspirations::Entity::find()
            .filter(worker_aspirations::Column::WorkerPid.is_in(worker_pids.to_vec()))
            .filter(worker_aspirations::Column::DeletedAt.is_null())
            .filter(worker_aspirations::Column::Kind.eq("skill"))
            .all(&ctx.db)
            .await?
        {
            if ACTIVE_ASPIRATIONS.contains(&a.status.as_str()) {
                aspirations.entry(a.worker_pid).or_default().push(a);
            }
        }
    }
    Ok(Evidence {
        requirements,
        roles,
        declared,
        aspirations,
    })
}

impl Evidence {
    /// Merge one worker's needs and grade them against their declarations.
    pub(crate) fn gaps_for(&self, worker: Uuid) -> Vec<Gap> {
        let mut needs: Vec<Need> = Vec::new();
        if let Some(profile) = self.roles.get(&worker)
            && let Some(reqs) = self.requirements.get(profile)
        {
            for r in reqs {
                needs.push(Need {
                    skill: r.skill_pid,
                    required: r.min_proficiency,
                    importance: r.importance.clone(),
                    source: Source::Role,
                });
            }
        }
        let mine = self.declared.get(&worker);
        if let Some(mine) = mine {
            for (skill, (_, target)) in mine {
                if let Some(t) = target {
                    needs.push(Need {
                        skill: *skill,
                        required: *t,
                        importance: "important".to_string(),
                        source: Source::Target,
                    });
                }
            }
        }
        for a in self.aspirations.get(&worker).into_iter().flatten() {
            if let (Some(skill), Some(level)) = (a.skill_pid, a.target_level) {
                needs.push(Need {
                    skill,
                    required: level,
                    importance: "useful".to_string(),
                    source: Source::Aspiration,
                });
            }
        }
        let declared: BTreeMap<Uuid, i32> = mine
            .map(|m| m.iter().map(|(s, (p, _))| (*s, *p)).collect())
            .unwrap_or_default();
        rules::merge(&needs, &declared)
    }
}

/// Skill names and categories by pid.
async fn skill_index(ctx: &AppContext) -> Result<HashMap<Uuid, (String, String)>> {
    Ok(skills::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|s| (s.pid, (s.name, s.category)))
        .collect())
}

/// `GET /api/workers/{pid}/skill-gaps` — one person's gaps, ranked.
#[debug_handler]
async fn worker_gaps(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    // Aspirations are the person's own: only they (or HR) see them here.
    let own_view = auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .is_ok();
    let evidence = load_evidence(&ctx, &[worker.pid], own_view).await?;
    let mut gaps = evidence.gaps_for(worker.pid);
    rules::rank(&mut gaps);
    let index = skill_index(&ctx).await?;
    let count = |s: Status| gaps.iter().filter(|g| g.status == s).count();
    let (met, below, undeclared) = (
        count(Status::Met),
        count(Status::Below),
        count(Status::Undeclared),
    );
    let listed: Vec<serde_json::Value> = gaps
        .iter()
        .filter(|g| g.status != Status::Met)
        .map(|g| {
            serde_json::json!({
                "skill_pid": g.skill,
                "skill": index.get(&g.skill).map(|s| s.0.clone()),
                "category": index.get(&g.skill).map(|s| s.1.clone()),
                "required": g.required,
                "importance": g.importance,
                "sources": g.sources,
                "declared": g.declared,
                "status": g.status,
                "shortfall": g.shortfall,
                "priority": rules::priority(g),
            })
        })
        .collect();
    format::json(serde_json::json!({
        "derivation": "compares the person's DECLARED proficiency with each need: the minimum of \\
                       their current role's requirement, their own skill targets and — for them \\
                       alone — their aspirations, merged per skill at the highest level. \\
                       `undeclared` means unknown: no shortfall and no priority, a prompt to \\
                       assess. Priority = importance weight (critical 3, important 2, useful 1) \\
                       × levels short.",
        "worker_pid": worker.pid,
        "includes_aspirations": own_view,
        "counts": { "met": met, "below": below, "undeclared": undeclared },
        "gaps": listed,
    }))
}

/// Query for the workforce view.
#[derive(Debug, Deserialize)]
struct WorkforceQuery {
    /// Only people in this department (case-insensitive).
    department: Option<String>,
    /// Most skills returned (default 25, at most 100).
    limit: Option<usize>,
}

/// `GET /api/workforce-intelligence/skill-gaps?department=&limit=` — the
/// workforce's skill gaps, ranked by importance × levels short. Employed
/// workers in the caller's organizations. Counts only; nobody is named.
#[debug_handler]
async fn workforce_gaps(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(q): axum::extract::Query<WorkforceQuery>,
) -> Result<Response> {
    let limit = q.limit.unwrap_or(25).clamp(1, 100);
    let mut query = workers::Entity::find().filter(workers::Column::DeletedAt.is_null());
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await? {
        query = query.filter(workers::Column::OrganizationRef.is_in(refs));
    }
    let today = Utc::now().date_naive();
    let staff: Vec<workers::Model> = query
        .order_by_asc(workers::Column::Id)
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| is_employed_on(today, w.hired_on, w.terminated_on))
        .filter(|w| {
            q.department
                .as_deref()
                .is_none_or(|d| w.department.eq_ignore_ascii_case(d))
        })
        .collect();
    let pids: Vec<Uuid> = staff.iter().map(|w| w.pid).collect();
    let evidence = load_evidence(&ctx, &pids, false).await?;
    let mut rows = Vec::new();
    let mut with_needs = 0;
    for w in &staff {
        let gaps = evidence.gaps_for(w.pid);
        if !gaps.is_empty() {
            with_needs += 1;
        }
        for g in gaps {
            rows.push(rules::Row {
                skill: g.skill,
                department: w.department.clone(),
                status: g.status,
                shortfall: g.shortfall,
                importance: g.importance,
            });
        }
    }
    let rolled = rules::rollup(&rows);
    let total = rolled.len();
    let index = skill_index(&ctx).await?;
    let skills_out: Vec<serde_json::Value> = rolled
        .into_iter()
        .take(limit)
        .map(|r| {
            serde_json::json!({
                "skill_pid": r.skill,
                "skill": index.get(&r.skill).map(|s| s.0.clone()),
                "category": index.get(&r.skill).map(|s| s.1.clone()),
                "needed_by": r.needed_by,
                "met": r.met,
                "below": r.below,
                "undeclared": r.undeclared,
                "total_shortfall": r.total_shortfall,
                "critical_below": r.critical_below,
                "score": r.score,
                "departments": r.departments.iter()
                    .map(|(d, n)| serde_json::json!({ "department": d, "below": n }))
                    .collect::<Vec<_>>(),
            })
        })
        .collect();
    format::json(serde_json::json!({
        "derivation": "per skill, over employed workers: how many NEED it (their current role's \\
                       requirement or their own target), meet it, are below it, or have not \\
                       declared it (unknown — no shortfall, no score). Ranked by Σ importance \\
                       weight × levels short. Aggregate only: nobody is named, and private \\
                       aspirations are never included.",
        "as_of": today,
        "workers_considered": staff.len(),
        "workers_with_needs": with_needs,
        "skills_total": total,
        "skills": skills_out,
    }))
}

/// The skill-gap routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/skill-gaps", get(worker_gaps))
        .add("/workforce-intelligence/skill-gaps", get(workforce_gaps))
}
