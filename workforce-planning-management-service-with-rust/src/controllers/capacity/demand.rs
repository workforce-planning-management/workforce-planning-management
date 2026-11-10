//! Demand and commitments (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;
use axa::extract::{Json, Path};

/// `PUT /api/programme-demands` body.
#[derive(Debug, Deserialize)]
pub(super) struct DemandPayload {
    programme_ref: String,
    pool_pid: Uuid,
    month: NaiveDate,
    fte_centi: i64,
    /// `proposed` or `closed`; omit to keep the current status (`proposed` for a new
    /// claim). A claim becomes `active` only through a start decision.
    #[serde(default)]
    status: Option<String>,
}

pub(super) async fn live_pool_by_pid(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pool_pid: Uuid,
) -> Result<skill_pools::Model> {
    live_pool(ctx, caller, &pool_pid.to_string()).await
}

/// `PUT /api/programme-demands` — set a programme's claim on a pool for a month.
#[debug_handler]
pub(super) async fn set_demand(
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
    let mut tx = crate::db::pool(&ctx).begin().await?;
    // One upsert on the live claim's key (the local source), where it was a read and then an insert or an update. A
    // status left out keeps the claim's current one; a new claim is `proposed`.
    let row = sqlx::query_scalar!(
        "INSERT INTO programme_demands (pid, programme_ref, pool_pid, month, fte_centi, status)
         VALUES ($1, $2, $3, $4, $5, COALESCE($6, 'proposed'))
         ON CONFLICT (programme_ref, pool_pid, month, source) WHERE deleted_at IS NULL
         DO UPDATE SET fte_centi = EXCLUDED.fte_centi, status = COALESCE($6, programme_demands.status)
         RETURNING pid",
        Uuid::new_v4(),
        payload.programme_ref,
        pool.pid,
        payload.month,
        payload.fte_centi,
        payload.status,
    )
    .fetch_one(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "programme_demand",
        row,
        "set",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(pid_ref(row))
}

/// Query for the demand and commitment lists.
#[derive(Debug, Deserialize)]
pub(super) struct PoolQuery {
    pool: Option<Uuid>,
    programme: Option<String>,
    partner: Option<String>,
}

/// Pools the caller may see, as a pid set.
pub(super) async fn visible_pool_pids(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
) -> Result<Vec<Uuid>> {
    let refs = scope(ctx, caller).await?;
    Ok(sqlx::query_scalar!(
        "SELECT pid FROM skill_pools WHERE deleted_at IS NULL AND ($1::text[] IS NULL OR organization_ref = ANY($1))",
        refs.as_deref(),
    )
    .fetch_all(crate::db::pool(ctx))
    .await?)
}

/// `GET /api/programme-demands?pool=&programme=`.
#[debug_handler]
pub(super) async fn list_demand(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(query): axa::extract::Query<PoolQuery>,
) -> Result<Response> {
    let visible = visible_pool_pids(&ctx, &caller).await?;
    let rows = sqlx::query!(
        "SELECT pid, programme_ref, pool_pid, month, fte_centi, status, source, received_at, peer_sent_at,
                peer_withdrawn_at
           FROM programme_demands
          WHERE deleted_at IS NULL AND pool_pid = ANY($1) AND ($2::uuid IS NULL OR pool_pid = $2)
            AND ($3::text IS NULL OR programme_ref = $3)
          ORDER BY month, id",
        &visible,
        query.pool,
        query.programme,
    )
    .fetch_all(crate::db::pool(&ctx))
    .await?;
    format::json(
        rows.iter()
            .map(|r| {
                json!({
                    "pid": r.pid, "programme_ref": r.programme_ref, "pool_pid": r.pool_pid,
                    "month": r.month, "fte_centi": r.fte_centi, "status": r.status,
                    "source": r.source,
                    // As the answer gave them before: with a `+00:00` offset, not `Z`.
                    "received_at": r.received_at.map(|t| t.fixed_offset()),
                    "peer_sent_at": r.peer_sent_at.map(|t| t.fixed_offset()),
                    "peer_withdrawn_at": r.peer_withdrawn_at.map(|t| t.fixed_offset()),
                })
            })
            .collect::<Vec<_>>(),
    )
}

/// `DELETE /api/programme-demands/{pid}` — soft delete.
#[debug_handler]
pub(super) async fn delete_demand(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = sqlx::query!(
        "SELECT id, pid, pool_pid FROM programme_demands WHERE pid = $1 AND deleted_at IS NULL",
        records::parse_pid(&pid)?,
    )
    .fetch_optional(crate::db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    live_pool_by_pid(&ctx, &caller, row.pool_pid).await?;
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE programme_demands SET deleted_at = $2 WHERE id = $1",
        row.id,
        Utc::now()
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "programme_demand",
        row.pid,
        "deleted",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(pid_ref(row.pid))
}

/// `PUT /api/partner-commitments` body.
#[derive(Debug, Deserialize)]
pub(super) struct CommitmentPayload {
    partner_ref: String,
    pool_pid: Uuid,
    month: NaiveDate,
    fte_centi: i64,
    status: String,
}

/// `PUT /api/partner-commitments` — a partner's aggregate capacity for a pool and month.
#[debug_handler]
pub(super) async fn set_commitment(
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
    let mut tx = crate::db::pool(&ctx).begin().await?;
    // One upsert on the live commitment's key, where it was a read and then an insert or an update.
    let row = sqlx::query_scalar!(
        "INSERT INTO partner_commitments (pid, partner_ref, pool_pid, month, fte_centi, status) VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (partner_ref, pool_pid, month) WHERE deleted_at IS NULL
         DO UPDATE SET fte_centi = EXCLUDED.fte_centi, status = EXCLUDED.status
         RETURNING pid",
        Uuid::new_v4(),
        payload.partner_ref,
        pool.pid,
        payload.month,
        payload.fte_centi,
        payload.status,
    )
    .fetch_one(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "partner_commitment",
        row,
        "set",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(pid_ref(row))
}

/// `GET /api/partner-commitments?pool=&partner=`.
#[debug_handler]
pub(super) async fn list_commitments(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(query): axa::extract::Query<PoolQuery>,
) -> Result<Response> {
    let visible = visible_pool_pids(&ctx, &caller).await?;
    let rows = sqlx::query!(
        "SELECT pid, partner_ref, pool_pid, month, fte_centi, status FROM partner_commitments
          WHERE deleted_at IS NULL AND pool_pid = ANY($1) AND ($2::uuid IS NULL OR pool_pid = $2)
            AND ($3::text IS NULL OR partner_ref = $3)
          ORDER BY month, id",
        &visible,
        query.pool,
        query.partner,
    )
    .fetch_all(crate::db::pool(&ctx))
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
pub(super) async fn delete_commitment(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = sqlx::query!(
        "SELECT id, pid, pool_pid FROM partner_commitments WHERE pid = $1 AND deleted_at IS NULL",
        records::parse_pid(&pid)?,
    )
    .fetch_optional(crate::db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    live_pool_by_pid(&ctx, &caller, row.pool_pid).await?;
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE partner_commitments SET deleted_at = $2 WHERE id = $1",
        row.id,
        Utc::now()
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "partner_commitment",
        row.pid,
        "deleted",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(pid_ref(row.pid))
}
