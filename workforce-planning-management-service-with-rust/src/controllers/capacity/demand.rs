//! Demand and commitments (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

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
pub(super) async fn list_demand(
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
pub(super) async fn delete_demand(
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
pub(super) async fn list_commitments(
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
pub(super) async fn delete_commitment(
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
