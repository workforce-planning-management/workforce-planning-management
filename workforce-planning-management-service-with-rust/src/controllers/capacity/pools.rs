//! Pools (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;
use axa::extract::{Json, Path};

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
    Ok(pools_of(ctx, org)
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
    let pid = Uuid::new_v4();
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO skill_pools (pid, organization_ref, name, description, operations_reservation_bp)
         VALUES ($1, $2, $3, $4, $5)",
        pid,
        org,
        payload.name.trim(),
        payload.description,
        payload.operations_reservation_bp.unwrap_or(0),
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(&mut tx, "skill_pool", pid, "created", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(pid_ref(pid))
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
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE skill_pools SET name = $2, description = $3, operations_reservation_bp = $4 WHERE id = $1",
        pool.id,
        payload.name.trim(),
        payload.description,
        payload.operations_reservation_bp.unwrap_or(0),
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "skill_pool",
        pool.pid,
        "updated",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(pid_ref(pool.pid))
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
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE skill_pools SET deleted_at = $2 WHERE id = $1",
        pool.id,
        Utc::now()
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "skill_pool",
        pool.pid,
        "deleted",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(pid_ref(pool.pid))
}

pub(super) async fn members_of(
    ctx: &AppContext,
    pool_pid: Uuid,
) -> Result<Vec<skill_pool_members::Model>> {
    members_in(ctx, &[pool_pid]).await
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
        "peer_pool_ref": pool.peer_pool_ref,
        "demand_source": rules::counted_source(pool.peer_pool_ref.is_some()),
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
    axa::extract::Query(query): axa::extract::Query<OrgQuery>,
) -> Result<Response> {
    let refs = scope(&ctx, &caller).await?;
    let pools = sqlx::query_as!(
        skill_pools::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name,
                  description, operations_reservation_bp, peer_pool_ref, deleted_at AS "deleted_at: _"
             FROM skill_pools
            WHERE deleted_at IS NULL AND ($1::text IS NULL OR organization_ref = $1)
              AND ($2::text[] IS NULL OR organization_ref = ANY($2))
            ORDER BY name"#,
        query.organization,
        refs.as_deref(),
    )
    .fetch_all(crate::db::pool(&ctx))
    .await?;
    // Every pool's members in one read, where it was one read per pool.
    let members = members_in(&ctx, &pools.iter().map(|p| p.pid).collect::<Vec<_>>()).await?;
    let out: Vec<serde_json::Value> = pools
        .iter()
        .map(|pool| {
            let mine: Vec<skill_pool_members::Model> = members
                .iter()
                .filter(|m| m.pool_pid == pool.pid)
                .cloned()
                .collect();
            pool_json(pool, &mine)
        })
        .collect();
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
    let db = crate::db::pool(&ctx);
    if let Some(role) = payload.role_profile_pid {
        sqlx::query_scalar!(
            "SELECT id FROM role_profiles WHERE pid = $1 AND deleted_at IS NULL",
            role
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| unprocessable("role_profile_pid does not name a role profile"))?;
    }
    if let Some(skill) = payload.skill_pid {
        sqlx::query_scalar!(
            "SELECT id FROM skills WHERE pid = $1 AND deleted_at IS NULL",
            skill
        )
        .fetch_optional(db)
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
    let member = Uuid::new_v4();
    let mut tx = db.begin().await?;
    sqlx::query!(
        "INSERT INTO skill_pool_members (pid, pool_pid, role_profile_pid, skill_pid, min_proficiency)
         VALUES ($1, $2, $3, $4, $5)",
        member,
        pool.pid,
        payload.role_profile_pid,
        payload.skill_pid,
        payload.min_proficiency,
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "skill_pool",
        pool.pid,
        "member_added",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(pid_ref(member))
}

/// `DELETE /api/skill-pools/{pid}/members/{member_pid}` — soft delete.
#[debug_handler]
pub(super) async fn remove_member(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, member_pid)): Path<(String, String)>,
) -> Result<Response> {
    let pool = live_pool(&ctx, &caller, &pid).await?;
    let mut tx = crate::db::pool(&ctx).begin().await?;
    let member = sqlx::query_scalar!(
        "UPDATE skill_pool_members SET deleted_at = $3 WHERE pid = $1 AND pool_pid = $2 AND deleted_at IS NULL
         RETURNING pid",
        records::parse_pid(&member_pid)?,
        pool.pid,
        Utc::now(),
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;
    crate::db::audit(
        &mut tx,
        "skill_pool",
        pool.pid,
        "member_removed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(pid_ref(member))
}
