//! **Delivery capacity** (WPM-R56–R60, WPM-D45, WPM-D46, WPM-D50): skill pools,
//! programme demand, partner commitments, the capacity view and the start check.
//! The arithmetic is in [`crate::rules::capacity`].
//!
//! Nothing here names a worker. Supply is worked out from workers' FTE, dates, role
//! profiles and skills and leaves only as numbers. Programmes and partners are
//! `EntityRef` URNs owned by other services; WPM keeps no copy of them. A claim becomes
//! **active** only through a recorded start decision (WPM-D50), never by a plain write.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, IntoActiveModel, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    capacity_settings, leave_requests, partner_commitments, programme_demands, role_profiles,
    skill_pool_members, skill_pools, skills, start_decisions, worker_framework_roles,
    worker_skills, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::rules::capacity as rules;
use crate::validation::Problems;

/// The default number of months in the view.
const DEFAULT_MONTHS: u32 = 12;
/// The number of months a start check looks across, so a delay has room to be found.
const CHECK_MONTHS: u32 = 24;

fn today() -> NaiveDate {
    Utc::now().date_naive()
}

/// The organizations the caller may see: `None` when unrestricted.
async fn scope(ctx: &AppContext, caller: &MaybeAuthUser) -> Result<Option<Vec<String>>> {
    memberships::scope_organization_refs(&ctx.db, caller.claims()).await
}

/// `404` for an organization outside the caller's scope, as for every scoped read.
async fn ensure_org_in_scope(ctx: &AppContext, caller: &MaybeAuthUser, org: &str) -> Result<()> {
    if let Some(refs) = scope(ctx, caller).await?
        && !refs.iter().any(|r| r == org)
    {
        return Err(Error::NotFound);
    }
    Ok(())
}

async fn live_pool(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<skill_pools::Model> {
    let pool = skill_pools::Entity::find()
        .filter(skill_pools::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(skill_pools::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &pool.organization_ref).await?;
    Ok(pool)
}

fn pid_ref(pid: Uuid) -> serde_json::Value {
    json!({ "pid": pid.to_string() })
}

// ─── Pools ──────────────────────────────────────────────────────────────────

/// `POST /api/skill-pools` and `PUT /api/skill-pools/{pid}` body.
#[derive(Debug, Deserialize)]
struct PoolPayload {
    #[serde(default)]
    organization_ref: Option<String>,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    operations_reservation_bp: Option<i32>,
}

fn check_pool(payload: &PoolPayload) -> Result<()> {
    let mut problems = Problems::new();
    problems.cap_text("name", &payload.name);
    if payload.name.trim().is_empty() {
        problems.push("name must not be blank".to_string());
    }
    if let Some(text) = &payload.description {
        problems.cap_text("description", text);
    }
    ensure_valid(&problems.into_vec())?;
    rules::validate_reservation_bp(payload.operations_reservation_bp.unwrap_or(0))
        .map_err(|e| unprocessable(&e))
}

/// A unique-name clash on a live pool in the organization.
async fn pool_name_taken(
    ctx: &AppContext,
    org: &str,
    name: &str,
    except: Option<Uuid>,
) -> Result<bool> {
    let wanted = name.trim().to_lowercase();
    Ok(skill_pools::Entity::find()
        .filter(skill_pools::Column::OrganizationRef.eq(org))
        .filter(skill_pools::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .iter()
        .any(|p| p.name.trim().to_lowercase() == wanted && Some(p.pid) != except))
}

/// `POST /api/skill-pools`.
#[debug_handler]
async fn create_pool(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<PoolPayload>,
) -> Result<Response> {
    let Some(org) = payload.organization_ref.clone() else {
        return Err(unprocessable("organization_ref is required"));
    };
    let mut problems = Problems::new();
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        &org,
    );
    ensure_valid(&problems.into_vec())?;
    check_pool(&payload)?;
    ensure_org_in_scope(&ctx, &caller, &org).await?;
    if pool_name_taken(&ctx, &org, &payload.name, None).await? {
        return Err(unprocessable("a pool with that name already exists"));
    }
    let row = skill_pools::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        organization_ref: ActiveValue::set(org),
        name: ActiveValue::set(payload.name.trim().to_string()),
        description: ActiveValue::set(payload.description.clone()),
        operations_reservation_bp: ActiveValue::set(payload.operations_reservation_bp.unwrap_or(0)),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "skill_pool",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    format::json(pid_ref(row.pid))
}

/// `PUT /api/skill-pools/{pid}` — rename or change the reservation.
#[debug_handler]
async fn update_pool(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<PoolPayload>,
) -> Result<Response> {
    let pool = live_pool(&ctx, &caller, &pid).await?;
    check_pool(&payload)?;
    if pool_name_taken(&ctx, &pool.organization_ref, &payload.name, Some(pool.pid)).await? {
        return Err(unprocessable("a pool with that name already exists"));
    }
    let mut active = pool.into_active_model();
    active.name = ActiveValue::set(payload.name.trim().to_string());
    active.description = ActiveValue::set(payload.description.clone());
    active.operations_reservation_bp =
        ActiveValue::set(payload.operations_reservation_bp.unwrap_or(0));
    let row = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "skill_pool",
        row.pid,
        "updated",
        caller.actor(),
        None,
    )
    .await?;
    format::json(pid_ref(row.pid))
}

/// `DELETE /api/skill-pools/{pid}` — soft delete. The pool leaves every view; its
/// demand and commitments are kept but no longer appear.
#[debug_handler]
async fn delete_pool(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let pool = live_pool(&ctx, &caller, &pid).await?;
    let mut active = pool.into_active_model();
    active.deleted_at = ActiveValue::set(Some(Utc::now().into()));
    let row = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "skill_pool",
        row.pid,
        "deleted",
        caller.actor(),
        None,
    )
    .await?;
    format::json(pid_ref(row.pid))
}

async fn members_of(ctx: &AppContext, pool_pid: Uuid) -> Result<Vec<skill_pool_members::Model>> {
    Ok(skill_pool_members::Entity::find()
        .filter(skill_pool_members::Column::PoolPid.eq(pool_pid))
        .filter(skill_pool_members::Column::DeletedAt.is_null())
        .order_by_asc(skill_pool_members::Column::Id)
        .all(&ctx.db)
        .await?)
}

fn pool_json(
    pool: &skill_pools::Model,
    members: &[skill_pool_members::Model],
) -> serde_json::Value {
    json!({
        "pid": pool.pid,
        "organization_ref": pool.organization_ref,
        "name": pool.name,
        "description": pool.description,
        "operations_reservation_bp": pool.operations_reservation_bp,
        "members": members.iter().map(|m| json!({
            "pid": m.pid,
            "role_profile_pid": m.role_profile_pid,
            "skill_pid": m.skill_pid,
            "min_proficiency": m.min_proficiency,
        })).collect::<Vec<_>>(),
    })
}

/// Query for the pool list.
#[derive(Debug, Deserialize)]
struct OrgQuery {
    organization: Option<String>,
}

/// `GET /api/skill-pools?organization=`.
#[debug_handler]
async fn list_pools(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<OrgQuery>,
) -> Result<Response> {
    let mut select = skill_pools::Entity::find().filter(skill_pools::Column::DeletedAt.is_null());
    if let Some(org) = &query.organization {
        select = select.filter(skill_pools::Column::OrganizationRef.eq(org));
    }
    if let Some(refs) = scope(&ctx, &caller).await? {
        select = select.filter(skill_pools::Column::OrganizationRef.is_in(refs));
    }
    let pools = select
        .order_by_asc(skill_pools::Column::Name)
        .all(&ctx.db)
        .await?;
    let mut out = Vec::with_capacity(pools.len());
    for pool in &pools {
        out.push(pool_json(pool, &members_of(&ctx, pool.pid).await?));
    }
    format::json(out)
}

/// `GET /api/skill-pools/{pid}`.
#[debug_handler]
async fn get_pool(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let pool = live_pool(&ctx, &caller, &pid).await?;
    format::json(pool_json(&pool, &members_of(&ctx, pool.pid).await?))
}

/// `POST /api/skill-pools/{pid}/members` body: a role profile **or** a skill.
#[derive(Debug, Deserialize)]
struct MemberPayload {
    #[serde(default)]
    role_profile_pid: Option<Uuid>,
    #[serde(default)]
    skill_pid: Option<Uuid>,
    #[serde(default)]
    min_proficiency: Option<i32>,
}

/// `POST /api/skill-pools/{pid}/members`.
#[debug_handler]
async fn add_member(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<MemberPayload>,
) -> Result<Response> {
    let pool = live_pool(&ctx, &caller, &pid).await?;
    rules::validate_member(
        payload.role_profile_pid,
        payload.skill_pid,
        payload.min_proficiency,
    )
    .map_err(|e| unprocessable(&e))?;
    if let Some(role) = payload.role_profile_pid {
        role_profiles::Entity::find()
            .filter(role_profiles::Column::Pid.eq(role))
            .filter(role_profiles::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?
            .ok_or_else(|| unprocessable("role_profile_pid does not name a role profile"))?;
    }
    if let Some(skill) = payload.skill_pid {
        skills::Entity::find()
            .filter(skills::Column::Pid.eq(skill))
            .filter(skills::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?
            .ok_or_else(|| unprocessable("skill_pid does not name a skill"))?;
    }
    let existing = members_of(&ctx, pool.pid).await?;
    if existing.iter().any(|m| {
        m.role_profile_pid == payload.role_profile_pid
            && m.skill_pid == payload.skill_pid
            && m.min_proficiency == payload.min_proficiency
    }) {
        return Err(unprocessable("that member is already in the pool"));
    }
    let row = skill_pool_members::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        pool_pid: ActiveValue::set(pool.pid),
        role_profile_pid: ActiveValue::set(payload.role_profile_pid),
        skill_pid: ActiveValue::set(payload.skill_pid),
        min_proficiency: ActiveValue::set(payload.min_proficiency),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "skill_pool",
        pool.pid,
        "member_added",
        caller.actor(),
        None,
    )
    .await?;
    format::json(pid_ref(row.pid))
}

/// `DELETE /api/skill-pools/{pid}/members/{member_pid}` — soft delete.
#[debug_handler]
async fn remove_member(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, member_pid)): Path<(String, String)>,
) -> Result<Response> {
    let pool = live_pool(&ctx, &caller, &pid).await?;
    let member = skill_pool_members::Entity::find()
        .filter(skill_pool_members::Column::Pid.eq(records::parse_pid(&member_pid)?))
        .filter(skill_pool_members::Column::PoolPid.eq(pool.pid))
        .filter(skill_pool_members::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let mut active = member.into_active_model();
    active.deleted_at = ActiveValue::set(Some(Utc::now().into()));
    let row = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "skill_pool",
        pool.pid,
        "member_removed",
        caller.actor(),
        None,
    )
    .await?;
    format::json(pid_ref(row.pid))
}

// ─── Demand and commitments ─────────────────────────────────────────────────

/// `PUT /api/programme-demands` body.
#[derive(Debug, Deserialize)]
struct DemandPayload {
    programme_ref: String,
    pool_pid: Uuid,
    month: NaiveDate,
    fte_centi: i64,
    /// `proposed` or `closed`; omit to keep the current status (`proposed` for a new
    /// claim). A claim becomes `active` only through a start decision.
    #[serde(default)]
    status: Option<String>,
}

async fn live_pool_by_pid(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pool_pid: Uuid,
) -> Result<skill_pools::Model> {
    live_pool(ctx, caller, &pool_pid.to_string()).await
}

/// `PUT /api/programme-demands` — set a programme's claim on a pool for a month.
#[debug_handler]
async fn set_demand(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<DemandPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_ref(
        "programme_ref",
        entity_ref::EntityType::Thing,
        &payload.programme_ref,
    );
    ensure_valid(&problems.into_vec())?;
    rules::validate_month(payload.month).map_err(|e| unprocessable(&e))?;
    rules::validate_fte_centi(payload.fte_centi).map_err(|e| unprocessable(&e))?;
    if let Some(status) = &payload.status
        && !matches!(status.as_str(), "proposed" | "closed")
    {
        return Err(unprocessable(
            "status can be set to proposed or closed; a claim becomes active through a start decision",
        ));
    }
    let pool = live_pool_by_pid(&ctx, &caller, payload.pool_pid).await?;
    let existing = programme_demands::Entity::find()
        .filter(programme_demands::Column::ProgrammeRef.eq(&payload.programme_ref))
        .filter(programme_demands::Column::PoolPid.eq(pool.pid))
        .filter(programme_demands::Column::Month.eq(payload.month))
        .filter(programme_demands::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?;
    let row = if let Some(row) = existing {
        let mut active = row.into_active_model();
        active.fte_centi = ActiveValue::set(payload.fte_centi);
        if let Some(status) = &payload.status {
            active.status = ActiveValue::set(status.clone());
        }
        active.update(&ctx.db).await?
    } else {
        programme_demands::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            programme_ref: ActiveValue::set(payload.programme_ref.clone()),
            pool_pid: ActiveValue::set(pool.pid),
            month: ActiveValue::set(payload.month),
            fte_centi: ActiveValue::set(payload.fte_centi),
            status: ActiveValue::set(payload.status.clone().unwrap_or_else(|| "proposed".into())),
            deleted_at: ActiveValue::set(None),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?
    };
    Audit::record(
        &ctx.db,
        "programme_demand",
        row.pid,
        "set",
        caller.actor(),
        None,
    )
    .await?;
    format::json(pid_ref(row.pid))
}

/// Query for the demand and commitment lists.
#[derive(Debug, Deserialize)]
struct PoolQuery {
    pool: Option<Uuid>,
    programme: Option<String>,
    partner: Option<String>,
}

/// Pools the caller may see, as a pid set.
async fn visible_pool_pids(ctx: &AppContext, caller: &MaybeAuthUser) -> Result<Vec<Uuid>> {
    let mut select = skill_pools::Entity::find().filter(skill_pools::Column::DeletedAt.is_null());
    if let Some(refs) = scope(ctx, caller).await? {
        select = select.filter(skill_pools::Column::OrganizationRef.is_in(refs));
    }
    Ok(select
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|p| p.pid)
        .collect())
}

/// `GET /api/programme-demands?pool=&programme=`.
#[debug_handler]
async fn list_demand(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<PoolQuery>,
) -> Result<Response> {
    let visible = visible_pool_pids(&ctx, &caller).await?;
    let mut select = programme_demands::Entity::find()
        .filter(programme_demands::Column::DeletedAt.is_null())
        .filter(programme_demands::Column::PoolPid.is_in(visible));
    if let Some(pool) = query.pool {
        select = select.filter(programme_demands::Column::PoolPid.eq(pool));
    }
    if let Some(programme) = &query.programme {
        select = select.filter(programme_demands::Column::ProgrammeRef.eq(programme));
    }
    let rows = select
        .order_by_asc(programme_demands::Column::Month)
        .all(&ctx.db)
        .await?;
    format::json(
        rows.iter()
            .map(|r| {
                json!({
                    "pid": r.pid, "programme_ref": r.programme_ref, "pool_pid": r.pool_pid,
                    "month": r.month, "fte_centi": r.fte_centi, "status": r.status,
                })
            })
            .collect::<Vec<_>>(),
    )
}

/// `DELETE /api/programme-demands/{pid}` — soft delete.
#[debug_handler]
async fn delete_demand(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = programme_demands::Entity::find()
        .filter(programme_demands::Column::Pid.eq(records::parse_pid(&pid)?))
        .filter(programme_demands::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    live_pool_by_pid(&ctx, &caller, row.pool_pid).await?;
    let mut active = row.into_active_model();
    active.deleted_at = ActiveValue::set(Some(Utc::now().into()));
    let row = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "programme_demand",
        row.pid,
        "deleted",
        caller.actor(),
        None,
    )
    .await?;
    format::json(pid_ref(row.pid))
}

/// `PUT /api/partner-commitments` body.
#[derive(Debug, Deserialize)]
struct CommitmentPayload {
    partner_ref: String,
    pool_pid: Uuid,
    month: NaiveDate,
    fte_centi: i64,
    status: String,
}

/// `PUT /api/partner-commitments` — a partner's aggregate capacity for a pool and month.
#[debug_handler]
async fn set_commitment(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<CommitmentPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_ref(
        "partner_ref",
        entity_ref::EntityType::Organization,
        &payload.partner_ref,
    );
    ensure_valid(&problems.into_vec())?;
    rules::validate_month(payload.month).map_err(|e| unprocessable(&e))?;
    rules::validate_fte_centi(payload.fte_centi).map_err(|e| unprocessable(&e))?;
    if rules::CommitmentStatus::parse(&payload.status).is_none() {
        return Err(unprocessable(&format!(
            "status must be one of {}",
            rules::CommitmentStatus::ALL.join(", ")
        )));
    }
    let pool = live_pool_by_pid(&ctx, &caller, payload.pool_pid).await?;
    let existing = partner_commitments::Entity::find()
        .filter(partner_commitments::Column::PartnerRef.eq(&payload.partner_ref))
        .filter(partner_commitments::Column::PoolPid.eq(pool.pid))
        .filter(partner_commitments::Column::Month.eq(payload.month))
        .filter(partner_commitments::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?;
    let row = if let Some(row) = existing {
        let mut active = row.into_active_model();
        active.fte_centi = ActiveValue::set(payload.fte_centi);
        active.status = ActiveValue::set(payload.status.clone());
        active.update(&ctx.db).await?
    } else {
        partner_commitments::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            partner_ref: ActiveValue::set(payload.partner_ref.clone()),
            pool_pid: ActiveValue::set(pool.pid),
            month: ActiveValue::set(payload.month),
            fte_centi: ActiveValue::set(payload.fte_centi),
            status: ActiveValue::set(payload.status.clone()),
            deleted_at: ActiveValue::set(None),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?
    };
    Audit::record(
        &ctx.db,
        "partner_commitment",
        row.pid,
        "set",
        caller.actor(),
        None,
    )
    .await?;
    format::json(pid_ref(row.pid))
}

/// `GET /api/partner-commitments?pool=&partner=`.
#[debug_handler]
async fn list_commitments(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<PoolQuery>,
) -> Result<Response> {
    let visible = visible_pool_pids(&ctx, &caller).await?;
    let mut select = partner_commitments::Entity::find()
        .filter(partner_commitments::Column::DeletedAt.is_null())
        .filter(partner_commitments::Column::PoolPid.is_in(visible));
    if let Some(pool) = query.pool {
        select = select.filter(partner_commitments::Column::PoolPid.eq(pool));
    }
    if let Some(partner) = &query.partner {
        select = select.filter(partner_commitments::Column::PartnerRef.eq(partner));
    }
    let rows = select
        .order_by_asc(partner_commitments::Column::Month)
        .all(&ctx.db)
        .await?;
    format::json(
        rows.iter()
            .map(|r| {
                json!({
                    "pid": r.pid, "partner_ref": r.partner_ref, "pool_pid": r.pool_pid,
                    "month": r.month, "fte_centi": r.fte_centi, "status": r.status,
                })
            })
            .collect::<Vec<_>>(),
    )
}

/// `DELETE /api/partner-commitments/{pid}` — soft delete.
#[debug_handler]
async fn delete_commitment(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = partner_commitments::Entity::find()
        .filter(partner_commitments::Column::Pid.eq(records::parse_pid(&pid)?))
        .filter(partner_commitments::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    live_pool_by_pid(&ctx, &caller, row.pool_pid).await?;
    let mut active = row.into_active_model();
    active.deleted_at = ActiveValue::set(Some(Utc::now().into()));
    let row = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "partner_commitment",
        row.pid,
        "deleted",
        caller.actor(),
        None,
    )
    .await?;
    format::json(pid_ref(row.pid))
}

// ─── Settings ───────────────────────────────────────────────────────────────

/// `PUT /api/capacity/settings` body.
#[derive(Debug, Deserialize)]
struct SettingsPayload {
    organization_ref: String,
    /// The most programmes that may be active on the constraint pool; absent for no limit.
    #[serde(default)]
    wip_limit: Option<i32>,
}

/// `PUT /api/capacity/settings` — the organization's work-in-progress limit.
#[debug_handler]
async fn set_settings(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<SettingsPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        &payload.organization_ref,
    );
    ensure_valid(&problems.into_vec())?;
    if payload.wip_limit.is_some_and(|n| n < 0) {
        return Err(unprocessable("wip_limit must not be negative"));
    }
    ensure_org_in_scope(&ctx, &caller, &payload.organization_ref).await?;
    let existing = capacity_settings::Entity::find()
        .filter(capacity_settings::Column::OrganizationRef.eq(&payload.organization_ref))
        .one(&ctx.db)
        .await?;
    let row = if let Some(row) = existing {
        let mut active = row.into_active_model();
        active.wip_limit = ActiveValue::set(payload.wip_limit);
        active.update(&ctx.db).await?
    } else {
        capacity_settings::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            organization_ref: ActiveValue::set(payload.organization_ref.clone()),
            wip_limit: ActiveValue::set(payload.wip_limit),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?
    };
    Audit::record(
        &ctx.db,
        "capacity_settings",
        row.pid,
        "set",
        caller.actor(),
        None,
    )
    .await?;
    format::json(json!({ "organization_ref": row.organization_ref, "wip_limit": row.wip_limit }))
}

async fn wip_limit_of(ctx: &AppContext, org: &str) -> Result<Option<i32>> {
    Ok(capacity_settings::Entity::find()
        .filter(capacity_settings::Column::OrganizationRef.eq(org))
        .one(&ctx.db)
        .await?
        .and_then(|s| s.wip_limit))
}

/// `GET /api/capacity/settings?organization=`.
#[debug_handler]
async fn get_settings(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<OrgQuery>,
) -> Result<Response> {
    let org = query
        .organization
        .ok_or_else(|| unprocessable("organization is required"))?;
    ensure_org_in_scope(&ctx, &caller, &org).await?;
    format::json(json!({ "organization_ref": org, "wip_limit": wip_limit_of(&ctx, &org).await? }))
}

// ─── The view ───────────────────────────────────────────────────────────────

/// Everything the view and the start check work from, for one organization.
struct OrgCapacity {
    months: Vec<NaiveDate>,
    pools: Vec<skill_pools::Model>,
    views: BTreeMap<String, rules::PoolView>,
    demand: BTreeMap<String, Vec<rules::Demand>>,
    member_counts: BTreeMap<String, usize>,
}

fn parse_demand_rows(rows: &[programme_demands::Model]) -> Vec<rules::Demand> {
    rows.iter()
        .filter_map(|r| {
            Some(rules::Demand {
                programme: r.programme_ref.clone(),
                month: r.month,
                fte_centi: r.fte_centi,
                status: rules::DemandStatus::parse(&r.status)?,
            })
        })
        .collect()
}

/// Workers of the organization as capacity facts for the given months.
async fn worker_facts(
    ctx: &AppContext,
    org: &str,
    months: &[NaiveDate],
    wanted_skills: &BTreeSet<Uuid>,
) -> Result<Vec<rules::WorkerFacts>> {
    let first = months.first().copied().unwrap_or_else(today);
    let last_month = months.last().copied().unwrap_or(first);
    let last_day = last_month
        .checked_add_days(chrono::Days::new(
            u64::try_from(rules::days_in_month(last_month)).unwrap_or(31),
        ))
        .unwrap_or(last_month);
    let staff: Vec<workers::Model> = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::OrganizationRef.eq(org))
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| w.hired_on < last_day && w.terminated_on.is_none_or(|t| t > first))
        .collect();
    let pids: Vec<Uuid> = staff.iter().map(|w| w.pid).collect();
    let titles: HashMap<String, Uuid> = role_profiles::Entity::find()
        .filter(role_profiles::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|p| (p.job_title.trim().to_lowercase(), p.pid))
        .collect();
    let mut chosen: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for row in worker_framework_roles::Entity::find()
        .filter(worker_framework_roles::Column::WorkerPid.is_in(pids.clone()))
        .filter(worker_framework_roles::Column::EndedAt.is_null())
        .all(&ctx.db)
        .await?
    {
        if let Some(role) = row.role_profile_pid {
            chosen.entry(row.worker_pid).or_default().push(role);
        }
    }
    let mut skill_levels: HashMap<Uuid, Vec<(Uuid, i32)>> = HashMap::new();
    if !wanted_skills.is_empty() {
        for row in worker_skills::Entity::find()
            .filter(worker_skills::Column::WorkerPid.is_in(pids.clone()))
            .filter(worker_skills::Column::SkillPid.is_in(wanted_skills.iter().copied()))
            .filter(worker_skills::Column::DeletedAt.is_null())
            .all(&ctx.db)
            .await?
        {
            skill_levels
                .entry(row.worker_pid)
                .or_default()
                .push((row.skill_pid, row.proficiency));
        }
    }
    let mut leave: HashMap<Uuid, Vec<(NaiveDate, NaiveDate)>> = HashMap::new();
    for row in leave_requests::Entity::find()
        .filter(leave_requests::Column::WorkerPid.is_in(pids))
        .filter(leave_requests::Column::Status.eq("approved"))
        .filter(leave_requests::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
    {
        leave
            .entry(row.worker_pid)
            .or_default()
            .push((row.start_on, row.end_on));
    }
    Ok(staff
        .into_iter()
        .map(|w| {
            let mut roles = chosen.remove(&w.pid).unwrap_or_default();
            if let Some(role) = titles.get(&w.job_title.trim().to_lowercase()) {
                roles.push(*role);
            }
            rules::WorkerFacts {
                fte_percent: w.fte_percent,
                hired_on: w.hired_on,
                terminated_on: w.terminated_on,
                role_profiles: roles,
                skills: skill_levels.remove(&w.pid).unwrap_or_default(),
                approved_leave: leave.remove(&w.pid).unwrap_or_default(),
            }
        })
        .collect())
}

async fn load_org_capacity(
    ctx: &AppContext,
    org: &str,
    from: NaiveDate,
    count: u32,
) -> Result<OrgCapacity> {
    let months = rules::months(from, count).map_err(|e| unprocessable(&e))?;
    let pools = skill_pools::Entity::find()
        .filter(skill_pools::Column::OrganizationRef.eq(org))
        .filter(skill_pools::Column::DeletedAt.is_null())
        .order_by_asc(skill_pools::Column::Name)
        .all(&ctx.db)
        .await?;
    let pool_pids: Vec<Uuid> = pools.iter().map(|p| p.pid).collect();
    let member_rows = skill_pool_members::Entity::find()
        .filter(skill_pool_members::Column::PoolPid.is_in(pool_pids.clone()))
        .filter(skill_pool_members::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let wanted: BTreeSet<Uuid> = member_rows.iter().filter_map(|m| m.skill_pid).collect();
    let staff = worker_facts(ctx, org, &months, &wanted).await?;
    let demand_rows = programme_demands::Entity::find()
        .filter(programme_demands::Column::PoolPid.is_in(pool_pids.clone()))
        .filter(programme_demands::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let commitment_rows = partner_commitments::Entity::find()
        .filter(partner_commitments::Column::PoolPid.is_in(pool_pids))
        .filter(partner_commitments::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let mut views = BTreeMap::new();
    let mut demand_by_pool = BTreeMap::new();
    let mut member_counts = BTreeMap::new();
    for pool in &pools {
        let members: Vec<rules::Member> = member_rows
            .iter()
            .filter(|m| m.pool_pid == pool.pid)
            .filter_map(
                |m| match (m.role_profile_pid, m.skill_pid, m.min_proficiency) {
                    (Some(role), None, _) => Some(rules::Member::RoleProfile(role)),
                    (None, Some(skill), Some(min_proficiency)) => Some(rules::Member::Skill {
                        skill,
                        min_proficiency,
                    }),
                    _ => None,
                },
            )
            .collect();
        let supply: Vec<i64> = months
            .iter()
            .map(|m| rules::pool_supply_centi(&members, pool.operations_reservation_bp, &staff, *m))
            .collect();
        let demand = parse_demand_rows(
            &demand_rows
                .iter()
                .filter(|d| d.pool_pid == pool.pid)
                .cloned()
                .collect::<Vec<_>>(),
        );
        let commitments: Vec<rules::Commitment> = commitment_rows
            .iter()
            .filter(|c| c.pool_pid == pool.pid)
            .filter_map(|c| {
                Some(rules::Commitment {
                    partner: c.partner_ref.clone(),
                    month: c.month,
                    fte_centi: c.fte_centi,
                    status: rules::CommitmentStatus::parse(&c.status)?,
                })
            })
            .collect();
        let key = pool.pid.to_string();
        views.insert(
            key.clone(),
            rules::build_pool(&months, &supply, &demand, &commitments),
        );
        member_counts.insert(key.clone(), members.len());
        demand_by_pool.insert(key, demand);
    }
    Ok(OrgCapacity {
        months,
        pools,
        views,
        demand: demand_by_pool,
        member_counts,
    })
}

fn constraint_of(capacity: &OrgCapacity) -> Option<String> {
    let pairs: Vec<(&str, &rules::PoolView)> = capacity
        .views
        .iter()
        .map(|(k, v)| (k.as_str(), v))
        .collect();
    rules::constraint_pool(&pairs).map(ToString::to_string)
}

fn cell_json(cell: &rules::Cell) -> serde_json::Value {
    json!({
        "month": cell.month,
        "supply_centi": cell.supply_centi,
        "demand_centi": cell.demand_centi,
        "proposed_centi": cell.proposed_centi,
        "own_balance_centi": cell.own_balance_centi,
        "firm_partner_centi": cell.firm_partner_centi,
        "remaining_shortfall_centi": cell.remaining_shortfall_centi,
        "status": cell.status.as_str(),
        "partners": cell.partners.iter().map(|p| json!({
            "partner": p.partner,
            "status": p.commitment.map(|(s, _)| s.as_str()),
            "fte_centi": p.commitment.map(|(_, f)| f),
        })).collect::<Vec<_>>(),
    })
}

/// Query for the view.
#[derive(Debug, Deserialize)]
struct ViewQuery {
    organization: Option<String>,
    from: Option<NaiveDate>,
    months: Option<u32>,
}

/// `GET /api/capacity?organization=&from=&months=` — pool × month supply against demand.
#[debug_handler]
async fn capacity_view(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ViewQuery>,
) -> Result<Response> {
    let org = query
        .organization
        .ok_or_else(|| unprocessable("organization is required"))?;
    ensure_org_in_scope(&ctx, &caller, &org).await?;
    let from = query.from.unwrap_or_else(today);
    let capacity =
        load_org_capacity(&ctx, &org, from, query.months.unwrap_or(DEFAULT_MONTHS)).await?;
    let constraint = constraint_of(&capacity);
    let wip_limit = wip_limit_of(&ctx, &org).await?;
    let wip = constraint.as_ref().map(|key| {
        let active =
            rules::programmes_drawing(capacity.demand.get(key).map_or(&[][..], Vec::as_slice));
        json!({ "pool_pid": key, "active": active.len(), "limit": wip_limit })
    });
    let pools: Vec<serde_json::Value> = capacity
        .pools
        .iter()
        .filter_map(|pool| {
            let key = pool.pid.to_string();
            let view = capacity.views.get(&key)?;
            Some(json!({
                "pid": pool.pid,
                "name": pool.name,
                "operations_reservation_bp": pool.operations_reservation_bp,
                "members": capacity.member_counts.get(&key).copied().unwrap_or(0),
                "over_months": view.over_months,
                "unknown_months": view.unknown_months,
                "over_total_centi": view.over_total_centi,
                "cells": view.cells.iter().map(cell_json).collect::<Vec<_>>(),
            }))
        })
        .collect();
    format::json(json!({
        "organization_ref": org,
        "from": capacity.months.first(),
        "months": capacity.months,
        "pools": pools,
        "constraint_pool": constraint,
        "wip": wip,
        "derivation": rules::DERIVATION,
    }))
}

// ─── Start check and decision ───────────────────────────────────────────────

/// One line of a what-if.
#[derive(Debug, Deserialize)]
struct WishLine {
    pool_pid: Uuid,
    month: NaiveDate,
    fte_centi: i64,
}

/// `POST /api/capacity/start-check` and `/start-decisions` body.
#[derive(Debug, Deserialize)]
struct StartPayload {
    organization_ref: String,
    programme_ref: String,
    /// Check these lines instead of the programme's stored proposed claims. Stores nothing.
    #[serde(default)]
    lines: Option<Vec<WishLine>>,
    /// `proceed` or `defer` (start-decisions only).
    #[serde(default)]
    decision: Option<String>,
    /// Why (start-decisions only; required).
    #[serde(default)]
    reason: Option<String>,
}

struct Checked {
    check: rules::StartCheck,
    capacity: OrgCapacity,
    constraint: Option<String>,
    proposed: Vec<programme_demands::Model>,
    from: NaiveDate,
}

async fn run_start_check(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    payload: &StartPayload,
) -> Result<Checked> {
    let mut problems = Problems::new();
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        &payload.organization_ref,
    );
    problems.require_ref(
        "programme_ref",
        entity_ref::EntityType::Thing,
        &payload.programme_ref,
    );
    ensure_valid(&problems.into_vec())?;
    ensure_org_in_scope(ctx, caller, &payload.organization_ref).await?;
    let org_pools: Vec<Uuid> = skill_pools::Entity::find()
        .filter(skill_pools::Column::OrganizationRef.eq(&payload.organization_ref))
        .filter(skill_pools::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|p| p.pid)
        .collect();
    let proposed: Vec<programme_demands::Model> = programme_demands::Entity::find()
        .filter(programme_demands::Column::ProgrammeRef.eq(&payload.programme_ref))
        .filter(programme_demands::Column::Status.eq("proposed"))
        .filter(programme_demands::Column::DeletedAt.is_null())
        .filter(programme_demands::Column::PoolPid.is_in(org_pools.clone()))
        .all(&ctx.db)
        .await?;
    let lines: Vec<(String, NaiveDate, i64)> = if let Some(wish) = &payload.lines {
        let mut out = Vec::with_capacity(wish.len());
        for line in wish {
            rules::validate_month(line.month).map_err(|e| unprocessable(&e))?;
            rules::validate_fte_centi(line.fte_centi).map_err(|e| unprocessable(&e))?;
            if !org_pools.contains(&line.pool_pid) {
                return Err(unprocessable(
                    "a line names a pool outside the organization",
                ));
            }
            out.push((line.pool_pid.to_string(), line.month, line.fte_centi));
        }
        out
    } else {
        proposed
            .iter()
            .map(|r| (r.pool_pid.to_string(), r.month, r.fte_centi))
            .collect()
    };
    if lines.is_empty() {
        return Err(unprocessable(
            "the programme has no proposed demand to check; set a claim or give lines",
        ));
    }
    let from = lines.iter().map(|(_, m, _)| *m).min().unwrap_or_else(today);
    let capacity = load_org_capacity(ctx, &payload.organization_ref, from, CHECK_MONTHS).await?;
    let constraint = constraint_of(&capacity);
    let active = constraint
        .as_ref()
        .and_then(|key| capacity.demand.get(key))
        .map(|d| rules::programmes_drawing(d))
        .unwrap_or_default();
    let candidate = rules::Candidate {
        programme: payload.programme_ref.clone(),
        lines,
    };
    let check = rules::start_check(
        &candidate,
        &capacity.views,
        wip_limit_of(ctx, &payload.organization_ref).await?,
        constraint.as_deref(),
        &active,
    );
    Ok(Checked {
        check,
        capacity,
        constraint,
        proposed,
        from,
    })
}

fn check_json(checked: &Checked) -> serde_json::Value {
    let names: HashMap<String, &str> = checked
        .capacity
        .pools
        .iter()
        .map(|p| (p.pid.to_string(), p.name.as_str()))
        .collect();
    let earliest_start = checked
        .check
        .earliest_shift_months
        .and_then(|shift| rules::shift_month(checked.from, shift));
    json!({
        "fit": checked.check.fit.as_str(),
        "lines": checked.check.lines.iter().map(|l| json!({
            "pool_pid": l.pool,
            "pool_name": names.get(&l.pool),
            "month": l.month,
            "needed_centi": l.needed_centi,
            "available_centi": l.available_centi,
            "short_by_centi": l.short_by_centi,
            "outcome": l.outcome,
        })).collect::<Vec<_>>(),
        "earliest_shift_months": checked.check.earliest_shift_months,
        "earliest_start_month": earliest_start,
        "constraint_pool": checked.constraint,
        "wip": checked.check.wip.as_ref().map(|w| json!({
            "pool_pid": w.pool, "limit": w.limit, "active": w.active,
            "would_be": w.would_be, "exceeds": w.exceeds,
        })),
        "derivation": checked.check.derivation,
        "method": rules::DERIVATION,
        "advisory": "A suggestion with its evidence. A person decides, and the decision is recorded.",
    })
}

/// `POST /api/capacity/start-check` — can this programme start? Stores nothing.
#[debug_handler]
async fn start_check(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<StartPayload>,
) -> Result<Response> {
    let checked = run_start_check(&ctx, &caller, &payload).await?;
    format::json(check_json(&checked))
}

/// `POST /api/capacity/start-decisions` — record a person's decision to start or defer,
/// with the reason. The check is re-run on the server; a `proceed` makes the
/// programme's proposed claims **active**.
#[debug_handler]
async fn record_start_decision(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<StartPayload>,
) -> Result<Response> {
    let decision = payload.decision.clone().unwrap_or_default();
    if !matches!(decision.as_str(), "proceed" | "defer") {
        return Err(unprocessable("decision must be proceed or defer"));
    }
    let reason = payload.reason.clone().unwrap_or_default();
    let mut problems = Problems::new();
    problems.cap_text("reason", &reason);
    if reason.trim().is_empty() {
        problems.push("reason is required: say why".to_string());
    }
    ensure_valid(&problems.into_vec())?;
    if payload.lines.is_some() {
        return Err(unprocessable(
            "a decision is made on the programme's stored proposed claims, not on what-if lines",
        ));
    }
    let checked = run_start_check(&ctx, &caller, &payload).await?;
    let txn = ctx.db.begin().await?;
    let row = start_decisions::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        organization_ref: ActiveValue::set(payload.organization_ref.clone()),
        programme_ref: ActiveValue::set(payload.programme_ref.clone()),
        decision: ActiveValue::set(decision.clone()),
        reason: ActiveValue::set(reason.trim().to_string()),
        fit: ActiveValue::set(checked.check.fit.as_str().to_string()),
        earliest_shift_months: ActiveValue::set(
            checked
                .check
                .earliest_shift_months
                .and_then(|n| i32::try_from(n).ok()),
        ),
        wip_exceeds: ActiveValue::set(checked.check.wip.as_ref().map(|w| w.exceeds)),
        decided_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    if decision == "proceed" {
        for claim in &checked.proposed {
            let mut active = claim.clone().into_active_model();
            active.status = ActiveValue::set("active".to_string());
            active.update(&txn).await?;
        }
    }
    // The audit entry names the decision, not the reason text.
    Audit::record(
        &txn,
        "start_decision",
        row.pid,
        if decision == "proceed" {
            "proceeded"
        } else {
            "deferred"
        },
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": row.pid, "decision": decision, "check": check_json(&checked) }))
}

/// Query for the decision list.
#[derive(Debug, Deserialize)]
struct DecisionQuery {
    programme: Option<String>,
    organization: Option<String>,
}

/// `GET /api/capacity/start-decisions?programme=&organization=`.
#[debug_handler]
async fn list_start_decisions(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<DecisionQuery>,
) -> Result<Response> {
    let mut select = start_decisions::Entity::find();
    if let Some(programme) = &query.programme {
        select = select.filter(start_decisions::Column::ProgrammeRef.eq(programme));
    }
    if let Some(org) = &query.organization {
        select = select.filter(start_decisions::Column::OrganizationRef.eq(org));
    }
    if let Some(refs) = scope(&ctx, &caller).await? {
        select = select.filter(start_decisions::Column::OrganizationRef.is_in(refs));
    }
    let rows = select
        .order_by_desc(start_decisions::Column::Id)
        .all(&ctx.db)
        .await?;
    format::json(
        rows.iter()
            .map(|r| {
                json!({
                    "pid": r.pid, "organization_ref": r.organization_ref,
                    "programme_ref": r.programme_ref, "decision": r.decision,
                    "reason": r.reason, "fit": r.fit,
                    "earliest_shift_months": r.earliest_shift_months,
                    "wip_exceeds": r.wip_exceeds, "decided_by": r.decided_by,
                    "decided_on": r.created_at.date_naive().to_string(),
                })
            })
            .collect::<Vec<_>>(),
    )
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/skill-pools", post(create_pool))
        .add("/skill-pools", get(list_pools))
        .add("/skill-pools/{pid}", get(get_pool))
        .add("/skill-pools/{pid}", put(update_pool))
        .add("/skill-pools/{pid}", delete(delete_pool))
        .add("/skill-pools/{pid}/members", post(add_member))
        .add(
            "/skill-pools/{pid}/members/{member_pid}",
            delete(remove_member),
        )
        .add("/programme-demands", put(set_demand))
        .add("/programme-demands", get(list_demand))
        .add("/programme-demands/{pid}", delete(delete_demand))
        .add("/partner-commitments", put(set_commitment))
        .add("/partner-commitments", get(list_commitments))
        .add("/partner-commitments/{pid}", delete(delete_commitment))
        .add("/capacity", get(capacity_view))
        .add("/capacity/settings", put(set_settings))
        .add("/capacity/settings", get(get_settings))
        .add("/capacity/start-check", post(start_check))
        .add("/capacity/start-decisions", post(record_start_decision))
        .add("/capacity/start-decisions", get(list_start_decisions))
}
