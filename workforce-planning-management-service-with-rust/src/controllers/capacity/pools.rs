//! Pools (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

/// `POST /api/skill-pools` and `PUT /api/skill-pools/{pid}` body.
#[derive(Debug, Deserialize)]
pub(super) struct PoolPayload {
    #[serde(default)]
    organization_ref: Option<String>,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    operations_reservation_bp: Option<i32>,
}

pub(super) fn check_pool(payload: &PoolPayload) -> Result<()> {
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
pub(super) async fn pool_name_taken(
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
pub(super) async fn create_pool(
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
    let txn = ctx.db.begin().await?;
    let row = skill_pools::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        organization_ref: ActiveValue::set(org),
        name: ActiveValue::set(payload.name.trim().to_string()),
        description: ActiveValue::set(payload.description.clone()),
        operations_reservation_bp: ActiveValue::set(payload.operations_reservation_bp.unwrap_or(0)),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(&txn, "skill_pool", row.pid, "created", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(pid_ref(row.pid))
}

/// `PUT /api/skill-pools/{pid}` — rename or change the reservation.
#[debug_handler]
pub(super) async fn update_pool(
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
    let txn = ctx.db.begin().await?;
    let row = active.update(&txn).await?;
    Audit::record(&txn, "skill_pool", row.pid, "updated", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(pid_ref(row.pid))
}

/// `DELETE /api/skill-pools/{pid}` — soft delete. The pool leaves every view; its
/// demand and commitments are kept but no longer appear.
#[debug_handler]
pub(super) async fn delete_pool(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let pool = live_pool(&ctx, &caller, &pid).await?;
    let mut active = pool.into_active_model();
    active.deleted_at = ActiveValue::set(Some(Utc::now().into()));
    let txn = ctx.db.begin().await?;
    let row = active.update(&txn).await?;
    Audit::record(&txn, "skill_pool", row.pid, "deleted", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(pid_ref(row.pid))
}

pub(super) async fn members_of(
    ctx: &AppContext,
    pool_pid: Uuid,
) -> Result<Vec<skill_pool_members::Model>> {
    Ok(skill_pool_members::Entity::find()
        .filter(skill_pool_members::Column::PoolPid.eq(pool_pid))
        .filter(skill_pool_members::Column::DeletedAt.is_null())
        .order_by_asc(skill_pool_members::Column::Id)
        .all(&ctx.db)
        .await?)
}

pub(super) fn pool_json(
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
pub(super) struct OrgQuery {
    pub(super) organization: Option<String>,
}

/// `GET /api/skill-pools?organization=`.
#[debug_handler]
pub(super) async fn list_pools(
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
pub(super) async fn get_pool(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let pool = live_pool(&ctx, &caller, &pid).await?;
    format::json(pool_json(&pool, &members_of(&ctx, pool.pid).await?))
}

/// `POST /api/skill-pools/{pid}/members` body: a role profile **or** a skill.
#[derive(Debug, Deserialize)]
pub(super) struct MemberPayload {
    #[serde(default)]
    role_profile_pid: Option<Uuid>,
    #[serde(default)]
    skill_pid: Option<Uuid>,
    #[serde(default)]
    min_proficiency: Option<i32>,
}

/// `POST /api/skill-pools/{pid}/members`.
#[debug_handler]
pub(super) async fn add_member(
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
    let txn = ctx.db.begin().await?;
    let row = skill_pool_members::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        pool_pid: ActiveValue::set(pool.pid),
        role_profile_pid: ActiveValue::set(payload.role_profile_pid),
        skill_pid: ActiveValue::set(payload.skill_pid),
        min_proficiency: ActiveValue::set(payload.min_proficiency),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "skill_pool",
        pool.pid,
        "member_added",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(pid_ref(row.pid))
}

/// `DELETE /api/skill-pools/{pid}/members/{member_pid}` — soft delete.
#[debug_handler]
pub(super) async fn remove_member(
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
    let txn = ctx.db.begin().await?;
    let row = active.update(&txn).await?;
    Audit::record(
        &txn,
        "skill_pool",
        pool.pid,
        "member_removed",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(pid_ref(row.pid))
}
