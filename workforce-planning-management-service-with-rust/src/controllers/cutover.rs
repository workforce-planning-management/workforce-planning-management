//! **Cutover readiness** (WPM-R70, WPM-T117): for a programme's go-live at a site, whether the people
//! side is ready. Three checks (trained staff per required role at the site, hypercare cover from the
//! on-call rota for the window, key-person risk in the pools the cutover depends on) each report
//! `ready`, `at_risk` or `unknown` with reasons (pure rules in [`crate::rules::cutover`]).
//!
//! Writes are for an administrator of the organization (as for the monthly pack); reads are open to
//! anyone in the organization's scope.
//!
//! The report names no one: counts, dates and skill or role names. **The go/no-go stays a human
//! decision outside WPM**, and the answer says so.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use chrono::NaiveDate;
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::Deserialize;
use serde_json::json;

use super::oversight_packs::ensure_may_freeze;
use super::{ensure_valid, key_person, rotas, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    cutover_pools, cutover_roles, cutovers, role_profiles, rotas as rota_table, skill_courses,
    skill_pool_members, skill_pools, training_enrollments, worker_framework_roles, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::rules::cutover as rules;
use crate::rules::metrics::is_employed_on;
use crate::validation::Problems;

async fn ensure_org_in_scope(ctx: &AppContext, caller: &MaybeAuthUser, org: &str) -> Result<()> {
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.iter().any(|r| r == org)
    {
        return Err(Error::NotFound);
    }
    Ok(())
}

async fn audit(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    kind: &str,
    pid: Uuid,
    action: &str,
) -> Result<()> {
    Audit::record(&ctx.db, kind, pid, action, caller.actor(), None).await?;
    Ok(())
}

async fn cutover_in_scope(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<cutovers::Model> {
    let row = cutovers::Entity::find()
        .filter(cutovers::Column::Pid.eq(records::parse_pid(pid)?))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &row.organization_ref).await?;
    Ok(row)
}

fn cutover_json(c: &cutovers::Model) -> serde_json::Value {
    json!({
        "pid": c.pid, "organization_ref": c.organization_ref, "programme_ref": c.programme_ref,
        "name": c.name, "site": c.site, "window_start": c.window_start, "window_end": c.window_end,
        "rota_pid": c.rota_pid, "site_aliases": c.site_aliases,
    })
}

/// The names the site goes by: its own and its aliases.
fn site_names(c: &cutovers::Model) -> Vec<String> {
    std::iter::once(c.site.clone())
        .chain(
            c.site_aliases
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str().map(ToString::to_string)),
        )
        .collect()
}

/// `POST /api/cutovers` body.
#[derive(Debug, Deserialize)]
struct CutoverPayload {
    organization_ref: String,
    programme_ref: String,
    name: String,
    site: String,
    window_start: NaiveDate,
    window_end: NaiveDate,
    rota_pid: Option<Uuid>,
    /// Other names the site is recorded under (a worker's location is text, so a site spelt another way
    /// needs an alias). Up to ten.
    site_aliases: Option<Vec<String>>,
}

/// `POST /api/cutovers` — a go-live at a site over a window of dates, with the on-call rota that gives
/// hypercare cover if there is one.
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<CutoverPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        &p.organization_ref,
    );
    problems.require_ref(
        "programme_ref",
        entity_ref::EntityType::Thing,
        &p.programme_ref,
    );
    problems.require_text("name", &p.name);
    problems.require_text("site", &p.site);
    ensure_valid(&problems.into_vec())?;
    rules::validate_window(p.window_start, p.window_end).map_err(|e| unprocessable(&e))?;
    let aliases = p.site_aliases.clone().unwrap_or_default();
    if aliases.len() > 10 || aliases.iter().any(|a| a.trim().is_empty() || a.len() > 200) {
        return Err(unprocessable(
            "site_aliases takes up to ten names, none blank and none over 200 characters",
        ));
    }
    ensure_org_in_scope(&ctx, &caller, &p.organization_ref).await?;
    ensure_may_freeze(&ctx, &caller, &p.organization_ref).await?;
    if let Some(rota) = p.rota_pid {
        let found = rota_table::Entity::find()
            .filter(rota_table::Column::Pid.eq(rota))
            .filter(rota_table::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?;
        if found.is_none_or(|r| r.organization_ref != p.organization_ref) {
            return Err(unprocessable(
                "that rota is not one of this organization's rotas",
            ));
        }
    }
    let row = cutovers::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        organization_ref: ActiveValue::set(p.organization_ref),
        programme_ref: ActiveValue::set(p.programme_ref),
        name: ActiveValue::set(p.name),
        site: ActiveValue::set(p.site),
        window_start: ActiveValue::set(p.window_start),
        window_end: ActiveValue::set(p.window_end),
        rota_pid: ActiveValue::set(p.rota_pid),
        site_aliases: ActiveValue::set(json!(aliases)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    audit(&ctx, &caller, "cutover", row.pid, "created").await?;
    format::json(cutover_json(&row))
}

/// Whether the learning records know the course: a catalogue entry or any enrollment carries the reference.
async fn course_known(ctx: &AppContext, course_ref: &str) -> Result<bool> {
    let catalogued = skill_courses::Entity::find()
        .filter(skill_courses::Column::CourseRef.eq(course_ref))
        .filter(skill_courses::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .is_some();
    if catalogued {
        return Ok(true);
    }
    Ok(training_enrollments::Entity::find()
        .filter(training_enrollments::Column::CourseRef.eq(course_ref))
        .filter(training_enrollments::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .is_some())
}

/// `POST /api/cutovers/{pid}/roles` body.
#[derive(Debug, Deserialize)]
struct RolePayload {
    role_profile_pid: Uuid,
    course_ref: String,
    required_headcount: i32,
}

/// `POST /api/cutovers/{pid}/roles` — a role that must be staffed at the site with people who completed
/// the named course.
#[debug_handler]
async fn add_role(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(p): Json<RolePayload>,
) -> Result<Response> {
    let c = cutover_in_scope(&ctx, &caller, &pid).await?;
    ensure_may_freeze(&ctx, &caller, &c.organization_ref).await?;
    let mut problems = Problems::new();
    problems.require_text("course_ref", &p.course_ref);
    ensure_valid(&problems.into_vec())?;
    if !course_known(&ctx, &p.course_ref).await? {
        return Err(unprocessable(
            "that course is not in the learning records (no catalogue entry and no enrollment); check the reference",
        ));
    }
    if p.required_headcount < 1 {
        return Err(unprocessable("required_headcount must be at least 1"));
    }
    let profile = role_profiles::Entity::find()
        .filter(role_profiles::Column::Pid.eq(p.role_profile_pid))
        .filter(role_profiles::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?;
    if profile.is_none() {
        return Err(unprocessable("that role profile does not exist"));
    }
    let taken = cutover_roles::Entity::find()
        .filter(cutover_roles::Column::CutoverPid.eq(c.pid))
        .filter(cutover_roles::Column::RoleProfilePid.eq(p.role_profile_pid))
        .one(&ctx.db)
        .await?
        .is_some();
    if taken {
        return Err(unprocessable(
            "that role is already listed for this cutover",
        ));
    }
    let row = cutover_roles::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        cutover_pid: ActiveValue::set(c.pid),
        role_profile_pid: ActiveValue::set(p.role_profile_pid),
        course_ref: ActiveValue::set(p.course_ref),
        required_headcount: ActiveValue::set(p.required_headcount),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    audit(&ctx, &caller, "cutover_role", row.pid, "added").await?;
    format::json(json!({
        "pid": row.pid, "cutover_pid": row.cutover_pid, "role_profile_pid": row.role_profile_pid,
        "course_ref": row.course_ref, "required_headcount": row.required_headcount,
    }))
}

/// `POST /api/cutovers/{pid}/pools` body.
#[derive(Debug, Deserialize)]
struct PoolPayload {
    pool_pid: Uuid,
}

/// `POST /api/cutovers/{pid}/pools` — a skill pool the cutover depends on.
#[debug_handler]
async fn add_pool(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(p): Json<PoolPayload>,
) -> Result<Response> {
    let c = cutover_in_scope(&ctx, &caller, &pid).await?;
    ensure_may_freeze(&ctx, &caller, &c.organization_ref).await?;
    let pool = skill_pools::Entity::find()
        .filter(skill_pools::Column::Pid.eq(p.pool_pid))
        .filter(skill_pools::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?;
    if pool.is_none_or(|x| x.organization_ref != c.organization_ref) {
        return Err(unprocessable(
            "that pool is not one of this organization's pools",
        ));
    }
    let taken = cutover_pools::Entity::find()
        .filter(cutover_pools::Column::CutoverPid.eq(c.pid))
        .filter(cutover_pools::Column::PoolPid.eq(p.pool_pid))
        .one(&ctx.db)
        .await?
        .is_some();
    if taken {
        return Err(unprocessable(
            "that pool is already listed for this cutover",
        ));
    }
    let row = cutover_pools::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        cutover_pid: ActiveValue::set(c.pid),
        pool_pid: ActiveValue::set(p.pool_pid),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    audit(&ctx, &caller, "cutover_pool", row.pid, "added").await?;
    format::json(
        json!({ "pid": row.pid, "cutover_pid": row.cutover_pid, "pool_pid": row.pool_pid }),
    )
}

/// `DELETE /api/cutovers/{pid}` — the cutover with its roles and pools.
#[debug_handler]
async fn remove(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let c = cutover_in_scope(&ctx, &caller, &pid).await?;
    ensure_may_freeze(&ctx, &caller, &c.organization_ref).await?;
    cutover_roles::Entity::delete_many()
        .filter(cutover_roles::Column::CutoverPid.eq(c.pid))
        .exec(&ctx.db)
        .await?;
    cutover_pools::Entity::delete_many()
        .filter(cutover_pools::Column::CutoverPid.eq(c.pid))
        .exec(&ctx.db)
        .await?;
    cutovers::Entity::delete_by_id(c.id).exec(&ctx.db).await?;
    audit(&ctx, &caller, "cutover", c.pid, "removed").await?;
    format::json(json!({ "removed": c.pid }))
}

/// Query naming the organization.
#[derive(Debug, Deserialize)]
pub struct OrgQuery {
    organization: Option<String>,
}

/// `GET /api/cutovers?organization=`.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(q): axum::extract::Query<OrgQuery>,
) -> Result<Response> {
    let org = q
        .organization
        .ok_or_else(|| unprocessable("organization is required"))?;
    ensure_org_in_scope(&ctx, &caller, &org).await?;
    let rows = cutovers::Entity::find()
        .filter(cutovers::Column::OrganizationRef.eq(&org))
        .order_by_asc(cutovers::Column::WindowStart)
        .order_by_asc(cutovers::Column::Id)
        .all(&ctx.db)
        .await?;
    format::json(rows.iter().map(cutover_json).collect::<Vec<_>>())
}

/// The trained-staff check: for each required role, the people of the role at the site who completed
/// the course and hold it through the window, and the people of the role with no site on file.
async fn trained_check(ctx: &AppContext, c: &cutovers::Model) -> Result<rules::Check> {
    let roles = cutover_roles::Entity::find()
        .filter(cutover_roles::Column::CutoverPid.eq(c.pid))
        .order_by_asc(cutover_roles::Column::Id)
        .all(&ctx.db)
        .await?;
    if roles.is_empty() {
        return Ok(rules::trained_staff(&[]));
    }
    let staff: Vec<workers::Model> = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&c.organization_ref))
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| is_employed_on(c.window_start, w.hired_on, w.terminated_on))
        .collect();
    let pids: Vec<Uuid> = staff.iter().map(|w| w.pid).collect();
    let profiles: BTreeMap<Uuid, role_profiles::Model> = role_profiles::Entity::find()
        .filter(role_profiles::Column::Pid.is_in(roles.iter().map(|r| r.role_profile_pid)))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|p| (p.pid, p))
        .collect();
    let mut chosen: HashMap<Uuid, BTreeSet<Uuid>> = HashMap::new();
    for row in worker_framework_roles::Entity::find()
        .filter(worker_framework_roles::Column::WorkerPid.is_in(pids.clone()))
        .filter(worker_framework_roles::Column::EndedAt.is_null())
        .all(&ctx.db)
        .await?
    {
        if let Some(role) = row.role_profile_pid {
            chosen.entry(row.worker_pid).or_default().insert(role);
        }
    }
    let names = site_names(c);
    let mut out = Vec::new();
    for role in &roles {
        let Some(profile) = profiles.get(&role.role_profile_pid) else {
            continue;
        };
        let title = profile.job_title.trim().to_lowercase();
        let of_role: Vec<&workers::Model> = staff
            .iter()
            .filter(|w| {
                chosen
                    .get(&w.pid)
                    .is_some_and(|set| set.contains(&role.role_profile_pid))
                    || w.job_title.trim().to_lowercase() == title
            })
            .collect();
        let done: BTreeSet<Uuid> = training_enrollments::Entity::find()
            .filter(training_enrollments::Column::CourseRef.eq(&role.course_ref))
            .filter(training_enrollments::Column::Status.eq("completed"))
            .filter(training_enrollments::Column::DeletedAt.is_null())
            .filter(training_enrollments::Column::WorkerPid.is_in(of_role.iter().map(|w| w.pid)))
            .all(&ctx.db)
            .await?
            .into_iter()
            .filter(|e| e.completed_on.is_some_and(|d| d <= c.window_start))
            .filter(|e| e.certificate_expires_on.is_none_or(|d| d >= c.window_end))
            .map(|e| e.worker_pid)
            .collect();
        let at = |w: &&workers::Model| rules::at_site(w.location.as_deref(), &names);
        let unplaced = |w: &&workers::Model| rules::unplaced(w.location.as_deref());
        let count = |f: &dyn Fn(&&workers::Model) -> bool| {
            u32::try_from(
                of_role
                    .iter()
                    .filter(|w| done.contains(&w.pid))
                    .filter(|w| f(w))
                    .count(),
            )
            .unwrap_or(u32::MAX)
        };
        out.push((
            profile.job_title.clone(),
            rules::RoleCount {
                required: u32::try_from(role.required_headcount).unwrap_or(u32::MAX),
                trained: count(&at),
                unplaced: count(&unplaced),
                course_known: course_known(ctx, &role.course_ref).await?,
            },
        ));
    }
    Ok(rules::trained_staff(&out))
}

/// The hypercare check: the days of the window with nobody on call.
async fn hypercare_check(ctx: &AppContext, c: &cutovers::Model) -> Result<rules::Check> {
    let Some(rota_pid) = c.rota_pid else {
        return Ok(rules::hypercare(false, &[]));
    };
    let rota = rota_table::Entity::find()
        .filter(rota_table::Column::Pid.eq(rota_pid))
        .filter(rota_table::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?;
    let Some(rota) = rota else {
        return Ok(rules::hypercare(false, &[]));
    };
    let (_, _, assignments, _) = rotas::compute(ctx, &rota, c.window_start, c.window_end).await?;
    let uncovered: Vec<NaiveDate> = assignments
        .iter()
        .filter(|a| a.worker.is_none())
        .map(|a| a.date)
        .collect();
    Ok(rules::hypercare(true, &uncovered))
}

/// The key-person check over the skills of the listed pools.
async fn key_person_check(ctx: &AppContext, c: &cutovers::Model) -> Result<rules::Check> {
    let pools: Vec<Uuid> = cutover_pools::Entity::find()
        .filter(cutover_pools::Column::CutoverPid.eq(c.pid))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|p| p.pool_pid)
        .collect();
    if pools.is_empty() {
        return Ok(rules::key_person(false, &[]));
    }
    let wanted: BTreeSet<Uuid> = skill_pool_members::Entity::find()
        .filter(skill_pool_members::Column::PoolPid.is_in(pools))
        .filter(skill_pool_members::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter_map(|m| m.skill_pid)
        .collect();
    let scope = [c.organization_ref.clone()];
    let (rows, _) = key_person::rows(ctx, Some(&scope), Some(&site_names(c))).await?;
    let skills: Vec<(String, _, usize)> = rows
        .iter()
        .filter(|r| r.department.is_none() && wanted.contains(&r.skill))
        .map(|r| (r.name.clone(), r.assessment.status, r.unplaced_holders))
        .collect();
    Ok(rules::key_person(true, &skills))
}

fn check_json(c: &rules::Check) -> serde_json::Value {
    json!({ "status": c.status.as_str(), "reasons": c.reasons })
}

/// The three checks for a cutover.
pub(super) async fn assess(
    ctx: &AppContext,
    c: &cutovers::Model,
) -> Result<(rules::Check, rules::Check, rules::Check)> {
    Ok((
        trained_check(ctx, c).await?,
        hypercare_check(ctx, c).await?,
        key_person_check(ctx, c).await?,
    ))
}

/// `GET /api/cutovers/{pid}/readiness` — the people side of the go-live: each check and the worst of
/// them. Counts, dates and names of roles and skills; no person.
#[debug_handler]
async fn readiness(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let c = cutover_in_scope(&ctx, &caller, &pid).await?;
    let (trained, cover, risk) = assess(&ctx, &c).await?;
    format::json(json!({
        "cutover": cutover_json(&c),
        "readiness": rules::overall(&[&trained, &cover, &risk]).as_str(),
        "checks": {
            "trained_staff": check_json(&trained),
            "hypercare_cover": check_json(&cover),
            "key_person_risk": check_json(&risk),
        },
        "basis": "The people side only. Trained staff are people of the role, employed on the first day \
                  of the window and recorded at the site, who completed the named course on or before \
                  that day and whose certificate lasts through the window. A shortfall that people with no \
                  site recorded could close is unknown, not at risk. Hypercare is the on-call rota over \
                  the window: a day with nobody on call is at risk. Key-person risk is read at the site, from \
                  declared skill levels, for the skills of the listed pools that are critical on a role \
                  profile; holders with no site recorded make a risk unknown. A site is matched by its \
                  name and aliases, without case or punctuation. An unknown is never ready. Whether to go live is a human \
                  decision outside this service.",
    }))
}

/// The cutover routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/cutovers", get(list))
        .add("/cutovers", post(create))
        .add("/cutovers/{pid}", delete(remove))
        .add("/cutovers/{pid}/roles", post(add_role))
        .add("/cutovers/{pid}/pools", post(add_pool))
        .add("/cutovers/{pid}/readiness", get(readiness))
}
