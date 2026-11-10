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

use super::ensure_org_in_scope;
use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::oversight_packs::ensure_may_freeze;
use super::{ensure_valid, key_person, rotas, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::models::_entities::{cutovers, rotas as rota_table};
use crate::models::records;
use crate::rules::cutover as rules;
use crate::rules::metrics::is_employed_on;
use crate::validation::Problems;

async fn cutover_in_scope(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<cutovers::Model> {
    let row = sqlx::query_as!(
        cutovers::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, programme_ref,
                  name, site, window_start, window_end, rota_pid, site_aliases
             FROM cutovers WHERE pid = $1"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &row.organization_ref).await?;
    Ok(row)
}

/// A live rota by pid.
async fn live_rota(ctx: &AppContext, pid: Uuid) -> Result<Option<rota_table::Model>> {
    Ok(sqlx::query_as!(
        rota_table::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name,
                  description, period_days, starts_on, deleted_at AS "deleted_at: _"
             FROM rotas WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?)
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
        let found = live_rota(&ctx, rota).await?;
        if found.is_none_or(|r| r.organization_ref != p.organization_ref) {
            return Err(unprocessable(
                "that rota is not one of this organization's rotas",
            ));
        }
    }
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        cutovers::Model,
        r#"INSERT INTO cutovers (pid, organization_ref, programme_ref, name, site, window_start, window_end, rota_pid,
                                 site_aliases)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref,
                     programme_ref, name, site, window_start, window_end, rota_pid, site_aliases"#,
        Uuid::new_v4(),
        p.organization_ref,
        p.programme_ref,
        p.name,
        p.site,
        p.window_start,
        p.window_end,
        p.rota_pid,
        json!(aliases),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(&mut tx, "cutover", row.pid, "created", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(cutover_json(&row))
}

/// Whether the learning records know the course: a catalogue entry or any enrollment carries the reference.
async fn course_known(ctx: &AppContext, course_ref: &str) -> Result<bool> {
    Ok(sqlx::query_scalar!(
        r#"SELECT (EXISTS (SELECT 1 FROM skill_courses WHERE course_ref = $1 AND deleted_at IS NULL)
                OR EXISTS (SELECT 1 FROM training_enrollments WHERE course_ref = $1 AND deleted_at IS NULL)) AS "known!""#,
        course_ref,
    )
    .fetch_one(db::pool(ctx))
    .await?)
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
    let profile = sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM role_profiles WHERE pid = $1 AND deleted_at IS NULL"#,
        p.role_profile_pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?;
    if profile.is_none() {
        return Err(unprocessable("that role profile does not exist"));
    }
    let taken = sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM cutover_roles WHERE cutover_pid = $1 AND role_profile_pid = $2"#,
        c.pid,
        p.role_profile_pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .is_some();
    if taken {
        return Err(unprocessable(
            "that role is already listed for this cutover",
        ));
    }
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO cutover_roles (pid, cutover_pid, role_profile_pid, course_ref, required_headcount)
         VALUES ($1, $2, $3, $4, $5)",
        pid,
        c.pid,
        p.role_profile_pid,
        p.course_ref,
        p.required_headcount,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "cutover_role", pid, "added", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(json!({
        "pid": pid, "cutover_pid": c.pid, "role_profile_pid": p.role_profile_pid,
        "course_ref": p.course_ref, "lms_url": super::lms::course_url(&p.course_ref), "required_headcount": p.required_headcount,
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
    let pool_org = sqlx::query_scalar!(
        "SELECT organization_ref FROM skill_pools WHERE pid = $1 AND deleted_at IS NULL",
        p.pool_pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?;
    if pool_org.is_none_or(|org| org != c.organization_ref) {
        return Err(unprocessable(
            "that pool is not one of this organization's pools",
        ));
    }
    let taken = sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM cutover_pools WHERE cutover_pid = $1 AND pool_pid = $2"#,
        c.pid,
        p.pool_pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .is_some();
    if taken {
        return Err(unprocessable(
            "that pool is already listed for this cutover",
        ));
    }
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO cutover_pools (pid, cutover_pid, pool_pid) VALUES ($1, $2, $3)",
        pid,
        c.pid,
        p.pool_pid,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "cutover_pool", pid, "added", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid, "cutover_pid": c.pid, "pool_pid": p.pool_pid }))
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
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!("DELETE FROM cutover_roles WHERE cutover_pid = $1", c.pid)
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM cutover_pools WHERE cutover_pid = $1", c.pid)
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM cutovers WHERE id = $1", c.id)
        .execute(&mut *tx)
        .await?;
    db::audit(&mut tx, "cutover", c.pid, "removed", caller.actor(), None).await?;
    tx.commit().await?;
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
    axa::extract::Query(q): axa::extract::Query<OrgQuery>,
    page: super::Page,
) -> Result<Response> {
    let window = super::window(&page)?;
    let org = q
        .organization
        .ok_or_else(|| unprocessable("organization is required"))?;
    ensure_org_in_scope(&ctx, &caller, &org).await?;
    let (limit, offset) = window.bounds();
    let rows = sqlx::query_as!(
        cutovers::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, programme_ref,
                  name, site, window_start, window_end, rota_pid, site_aliases
             FROM cutovers WHERE organization_ref = $1 ORDER BY window_start, id LIMIT $2 OFFSET $3"#,
        org,
        limit,
        offset,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    Ok(window.respond(rows.iter().map(cutover_json).collect::<Vec<_>>()))
}

/// What the trained-staff check reads of a worker.
struct Staff {
    pid: Uuid,
    job_title: String,
    location: Option<String>,
    hired_on: NaiveDate,
    terminated_on: Option<NaiveDate>,
}

/// The trained-staff check: for each required role, the people of the role at the site who completed
/// the course and hold it through the window, and the people of the role with no site on file.
async fn trained_check(ctx: &AppContext, c: &cutovers::Model) -> Result<rules::Check> {
    let roles = sqlx::query!(
        "SELECT role_profile_pid, course_ref, required_headcount FROM cutover_roles WHERE cutover_pid = $1 ORDER BY id",
        c.pid,
    )
    .fetch_all(db::pool(ctx))
    .await?;
    if roles.is_empty() {
        return Ok(rules::trained_staff(&[]));
    }
    let staff: Vec<Staff> = sqlx::query_as!(
        Staff,
        "SELECT pid, job_title, location, hired_on, terminated_on FROM workers
          WHERE organization_ref = $1 AND deleted_at IS NULL ORDER BY id",
        c.organization_ref,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
    .filter(|w| is_employed_on(c.window_start, w.hired_on, w.terminated_on))
    .collect();
    let pids: Vec<Uuid> = staff.iter().map(|w| w.pid).collect();
    let role_pids: Vec<Uuid> = roles.iter().map(|r| r.role_profile_pid).collect();
    // Every profile a role names, soft-deleted or not, as before.
    let profiles: BTreeMap<Uuid, String> = sqlx::query!(
        "SELECT pid, job_title FROM role_profiles WHERE pid = ANY($1)",
        &role_pids,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
    .map(|p| (p.pid, p.job_title))
    .collect();
    let mut chosen: HashMap<Uuid, BTreeSet<Uuid>> = HashMap::new();
    for row in sqlx::query!(
        "SELECT worker_pid, role_profile_pid FROM worker_framework_roles WHERE worker_pid = ANY($1) AND ended_at IS NULL",
        &pids,
    )
    .fetch_all(db::pool(ctx))
    .await?
    {
        if let Some(role) = row.role_profile_pid {
            chosen.entry(row.worker_pid).or_default().insert(role);
        }
    }
    let names = site_names(c);
    let mut out = Vec::new();
    for role in &roles {
        let Some(job_title) = profiles.get(&role.role_profile_pid) else {
            continue;
        };
        let title = job_title.trim().to_lowercase();
        let of_role: Vec<&Staff> = staff
            .iter()
            .filter(|w| {
                chosen
                    .get(&w.pid)
                    .is_some_and(|set| set.contains(&role.role_profile_pid))
                    || w.job_title.trim().to_lowercase() == title
            })
            .collect();
        let of_role_pids: Vec<Uuid> = of_role.iter().map(|w| w.pid).collect();
        let done: BTreeSet<Uuid> = sqlx::query!(
            "SELECT worker_pid, completed_on, certificate_expires_on FROM training_enrollments
              WHERE course_ref = $1 AND status = 'completed' AND deleted_at IS NULL AND worker_pid = ANY($2)",
            role.course_ref,
            &of_role_pids,
        )
        .fetch_all(db::pool(ctx))
        .await?
        .into_iter()
        .filter(|e| e.completed_on.is_some_and(|d| d <= c.window_start))
        .filter(|e| e.certificate_expires_on.is_none_or(|d| d >= c.window_end))
        .map(|e| e.worker_pid)
        .collect();
        let at = |w: &&Staff| rules::at_site(w.location.as_deref(), &names);
        let unplaced = |w: &&Staff| rules::unplaced(w.location.as_deref());
        let count = |f: &dyn Fn(&&Staff) -> bool| {
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
            job_title.clone(),
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
    let Some(rota) = live_rota(ctx, rota_pid).await? else {
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
    let pools: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT pool_pid FROM cutover_pools WHERE cutover_pid = $1",
        c.pid,
    )
    .fetch_all(db::pool(ctx))
    .await?;
    if pools.is_empty() {
        return Ok(rules::key_person(false, &[]));
    }
    let wanted: BTreeSet<Uuid> = sqlx::query_scalar!(
        "SELECT skill_pid FROM skill_pool_members WHERE pool_pid = ANY($1) AND deleted_at IS NULL",
        &pools,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
    .flatten()
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
    json!({
        "status": c.status.as_str(),
        "reasons": c.reasons,
        "details": c.details.iter().map(|d| json!({
            "code": d.code,
            "params": d.params.iter().map(|(k, v)| (k.to_string(), json!(v))).collect::<serde_json::Map<_, _>>(),
        })).collect::<Vec<_>>(),
    })
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
