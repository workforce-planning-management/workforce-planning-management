//! **Key-person risk** (WPM-R61, WPM-D47): `GET /api/workforce-intelligence/key-person` counts, for
//! each skill marked critical on a role profile, the employed people who hold it at the highest
//! critical minimum, their qualified backups and ready successors, and flags a single point of
//! failure. It names no one. `GET /api/workforce-intelligence/key-person/named` gives the sole
//! holder of each single point of failure by worker number: HR only, and every read is audited.
//!
//! The register exists to fund backups and succession, not to judge anyone (WPM-D47). Pure rules in
//! [`crate::rules::key_person`].

use std::collections::{BTreeMap, BTreeSet};

use loco_rs::prelude::*;
use serde_json::json;
use uuid::Uuid;

use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    role_profiles, role_skill_requirements, skills, succession_candidates, succession_plans,
    worker_backups, worker_skills, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::memberships;
use crate::rules::key_person::{self as rules, Holder, Status};

/// One assessed skill in one pool (a department, or `None` for the whole scope).
pub(super) struct Row {
    pub(super) skill: Uuid,
    pub(super) name: String,
    bar: i32,
    pub(super) department: Option<String>,
    pub(super) assessment: rules::Assessment,
    sole_holder: Option<Uuid>,
    /// With a site: employed holders of the skill with no site recorded (they might be there).
    pub(super) unplaced_holders: usize,
}

/// Everything read from the database for one assessment.
struct Loaded {
    staff: Vec<workers::Model>,
    minimums: BTreeMap<Uuid, Vec<i32>>,
    names: BTreeMap<Uuid, String>,
    level: BTreeMap<(Uuid, Uuid), i32>,
    backups_of: BTreeMap<Uuid, BTreeSet<Uuid>>,
    successors_of: BTreeMap<Uuid, BTreeSet<Uuid>>,
}

/// Read the facts for the caller's scope.
async fn load(ctx: &AppContext, scope: Option<&[String]>) -> Result<Loaded> {
    let today = chrono::Utc::now().date_naive();
    let staff = super::funding::employed_in_scope(ctx, scope, today).await?;
    let live_profiles: BTreeSet<Uuid> = role_profiles::Entity::find()
        .filter(role_profiles::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|p| p.pid)
        .collect();
    let mut minimums: BTreeMap<Uuid, Vec<i32>> = BTreeMap::new();
    for r in role_skill_requirements::Entity::find()
        .filter(role_skill_requirements::Column::Importance.eq("critical"))
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|r| live_profiles.contains(&r.role_profile_pid))
    {
        minimums
            .entry(r.skill_pid)
            .or_default()
            .push(r.min_proficiency);
    }
    let skill_pids: Vec<Uuid> = minimums.keys().copied().collect();
    let names: BTreeMap<Uuid, String> = skills::Entity::find()
        .filter(skills::Column::Pid.is_in(skill_pids.clone()))
        .filter(skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|s| (s.pid, s.name))
        .collect();
    let staff_pids: Vec<Uuid> = staff.iter().map(|w| w.pid).collect();
    let mut level: BTreeMap<(Uuid, Uuid), i32> = BTreeMap::new();
    for s in worker_skills::Entity::find()
        .filter(worker_skills::Column::DeletedAt.is_null())
        .filter(worker_skills::Column::SkillPid.is_in(skill_pids))
        .filter(worker_skills::Column::WorkerPid.is_in(staff_pids.clone()))
        .all(&ctx.db)
        .await?
    {
        let e = level.entry((s.skill_pid, s.worker_pid)).or_insert(0);
        *e = (*e).max(s.proficiency);
    }
    let mut backups_of: BTreeMap<Uuid, BTreeSet<Uuid>> = BTreeMap::new();
    for b in worker_backups::Entity::find()
        .filter(worker_backups::Column::DeletedAt.is_null())
        .filter(worker_backups::Column::WorkerPid.is_in(staff_pids.clone()))
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|b| b.ends_on.is_none_or(|end| end >= today))
    {
        backups_of
            .entry(b.worker_pid)
            .or_default()
            .insert(b.backup_pid);
    }
    let plans: BTreeMap<Uuid, Uuid> = succession_plans::Entity::find()
        .filter(succession_plans::Column::DeletedAt.is_null())
        .filter(succession_plans::Column::IncumbentPid.is_in(staff_pids))
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter_map(|p| p.incumbent_pid.map(|i| (p.pid, i)))
        .collect();
    let mut successors_of: BTreeMap<Uuid, BTreeSet<Uuid>> = BTreeMap::new();
    for c in succession_candidates::Entity::find()
        .filter(succession_candidates::Column::DeletedAt.is_null())
        .filter(succession_candidates::Column::Readiness.eq("ready_now"))
        .filter(
            succession_candidates::Column::PlanPid.is_in(plans.keys().copied().collect::<Vec<_>>()),
        )
        .all(&ctx.db)
        .await?
    {
        if let Some(incumbent) = plans.get(&c.plan_pid) {
            successors_of
                .entry(*incumbent)
                .or_default()
                .insert(c.worker_pid);
        }
    }
    Ok(Loaded {
        staff,
        minimums,
        names,
        level,
        backups_of,
        successors_of,
    })
}

/// Build every row for the caller's scope.
pub(super) async fn rows(
    ctx: &AppContext,
    scope: Option<&[String]>,
    site: Option<&[String]>,
) -> Result<(Vec<Row>, Vec<workers::Model>)> {
    let Loaded {
        staff,
        minimums,
        names,
        level,
        backups_of,
        successors_of,
    } = load(ctx, scope).await?;
    let mut out = Vec::new();
    for (skill, mins) in &minimums {
        let (Some(bar), Some(name)) = (rules::bar(mins), names.get(skill)) else {
            continue;
        };
        let holds = |pid: &Uuid| level.get(&(*skill, *pid)).is_some_and(|l| *l >= bar);
        // Read at a site, only the people recorded there count as holders or backups; the people with
        // no site on file are counted apart, since they might be there.
        let here = |w: &workers::Model| {
            site.is_none_or(|names| crate::rules::cutover::at_site(w.location.as_deref(), names))
        };
        let holders: Vec<&workers::Model> =
            staff.iter().filter(|w| holds(&w.pid) && here(w)).collect();
        let unplaced_holders = site.map_or(0, |_| {
            staff
                .iter()
                .filter(|w| holds(&w.pid) && crate::rules::cutover::unplaced(w.location.as_deref()))
                .count()
        });
        let at_the_site: BTreeSet<Uuid> = staff.iter().filter(|w| here(w)).map(|w| w.pid).collect();
        let facts = |w: &workers::Model| Holder {
            id: w.pid,
            backups: backups_of
                .get(&w.pid)
                .into_iter()
                .flatten()
                .copied()
                .filter(|b| *b != w.pid && holds(b) && at_the_site.contains(b))
                .collect(),
            ready_successors: successors_of.get(&w.pid).cloned().unwrap_or_default(),
        };
        let mut pools: Vec<(Option<String>, Vec<&workers::Model>)> = vec![(None, holders.clone())];
        let departments: BTreeSet<&str> = holders.iter().map(|w| w.department.as_str()).collect();
        for d in departments {
            pools.push((
                Some(d.to_string()),
                holders
                    .iter()
                    .copied()
                    .filter(|w| w.department == d)
                    .collect(),
            ));
        }
        for (department, members) in pools {
            let facts: Vec<Holder> = members.iter().map(|w| facts(w)).collect();
            out.push(Row {
                skill: *skill,
                name: name.clone(),
                bar,
                department,
                assessment: rules::assess(&facts),
                sole_holder: (members.len() == 1).then(|| members[0].pid),
                unplaced_holders,
            });
        }
    }
    out.sort_by(|a, b| (&a.name, &a.department).cmp(&(&b.name, &b.department)));
    Ok((out, staff))
}

/// The key-person figures of one organization for the monthly pack: the skills across the whole
/// organization with their counts, and the number of single points of failure. No department row and
/// no person, so a small department's holder cannot be worked out from it.
pub(super) async fn pack_json(ctx: &AppContext, org: &str) -> Result<serde_json::Value> {
    let scope = [org.to_string()];
    let (rows, _) = rows(ctx, Some(&scope), None).await?;
    let single = rows
        .iter()
        .filter(|r| r.department.is_some() && r.assessment.status == Status::SinglePointOfFailure)
        .count();
    Ok(json!({
        "single_points_of_failure": single,
        "skills": rows.iter().filter(|r| r.department.is_none()).map(|r| json!({
            "skill": r.name,
            "minimum_proficiency": r.bar,
            "holders": r.assessment.holders,
            "backups": r.assessment.backups,
            "ready_successors": r.assessment.ready_successors,
            "status": r.assessment.status.as_str(),
        })).collect::<Vec<_>>(),
        "basis": "Counts as at the freeze date, from the declared skill levels then on file. The number of \
                  single points of failure counts a skill in a department held by one person; no one is named.",
    }))
}

/// Query for the key-person routes.
#[derive(Debug, serde::Deserialize)]
struct OrgQuery {
    /// Read one organization instead of the whole of the caller's scope.
    organization: Option<String>,
}

/// The scope to read: the caller's, or the one organization asked for. An organization outside the
/// caller's scope is `404`, as for every scoped read.
async fn scope_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    query: &OrgQuery,
) -> Result<Option<Vec<String>>> {
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let Some(org) = query.organization.as_deref() else {
        return Ok(scope);
    };
    if let Some(refs) = &scope
        && !refs.iter().any(|r| r == org)
    {
        return Err(Error::NotFound);
    }
    Ok(Some(vec![org.to_string()]))
}

/// `GET /api/workforce-intelligence/key-person` — counts only; no one is named. With
/// `?organization=`, only that organization.
#[debug_handler]
async fn aggregate(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<OrgQuery>,
) -> Result<Response> {
    let scope = scope_for(&ctx, &caller, &query).await?;
    let (rows, _) = rows(&ctx, scope.as_deref(), None).await?;
    let single = rows
        .iter()
        .filter(|r| r.department.is_some() && r.assessment.status == Status::SinglePointOfFailure)
        .count();
    format::json(json!({
        "single_points_of_failure": single,
        "skills": rows.iter().map(|r| json!({
            "skill_pid": r.skill,
            "skill": r.name,
            "minimum_proficiency": r.bar,
            "department": r.department,
            "holders": r.assessment.holders,
            "backups": r.assessment.backups,
            "ready_successors": r.assessment.ready_successors,
            "status": r.assessment.status.as_str(),
        })).collect::<Vec<_>>(),
        "basis": "a critical skill is one marked critical on a live role profile; a holder is an \
                  employed person whose declared proficiency meets the highest critical minimum; a \
                  backup is a named, current backup who also holds it; a ready successor is ranked \
                  ready_now on a succession plan whose incumbent is a holder. Counts only: it exists \
                  to fund backups and succession, not to judge anyone.",
    }))
}

/// `GET /api/workforce-intelligence/key-person/named` — the sole holder behind each single point of
/// failure, by worker number. HR only, and the read is audited.
#[debug_handler]
async fn named(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<OrgQuery>,
) -> Result<Response> {
    let scope = scope_for(&ctx, &caller, &query).await?;
    let (rows, staff) = rows(&ctx, scope.as_deref(), None).await?;
    let by_pid: BTreeMap<Uuid, &workers::Model> = staff.iter().map(|w| (w.pid, w)).collect();
    let people: Vec<serde_json::Value> = rows
        .iter()
        .filter(|r| r.department.is_some() && r.assessment.status == Status::SinglePointOfFailure)
        .filter_map(|r| {
            let w = by_pid.get(&r.sole_holder?)?;
            Some(json!({
                "skill": r.name,
                "department": r.department,
                "worker_pid": w.pid,
                "worker_number": w.worker_number,
            }))
        })
        .collect();
    Audit::record(
        &ctx.db,
        "key_person",
        Uuid::nil(),
        "key_person_read",
        caller.actor(),
        None,
    )
    .await?;
    format::json(json!({ "people": people }))
}

/// The key-person routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workforce-intelligence/key-person", get(aggregate))
        .add("/workforce-intelligence/key-person/named", get(named))
}
