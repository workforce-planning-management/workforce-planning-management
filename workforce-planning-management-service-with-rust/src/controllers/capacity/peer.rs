//! The link to a peer's pool and demand received from the peer (part of [`super`]): WPM-R213, WPM-D138 to D141.

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

/// The longest event id taken from a peer.
const MAX_EVENT_ID: usize = 128;

/// `PUT /api/skill-pools/{pid}/peer-link` body.
#[derive(Debug, Deserialize)]
pub(super) struct LinkPayload {
    /// The peer's id for the same pool; `null` removes the link.
    peer_pool_ref: Option<String>,
}

/// `PUT /api/skill-pools/{pid}/peer-link` — link the pool to the peer's pool, or remove the link. A link is set by a person, never
/// matched by name (WPM-D139). The pool takes its demand from the peer while linked and from here when not (WPM-D140); nothing is
/// deleted either way.
#[debug_handler]
pub(super) async fn set_peer_link(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<LinkPayload>,
) -> Result<Response> {
    let pool = live_pool(&ctx, &caller, &pid).await?;
    if let Some(text) = &payload.peer_pool_ref
        && !rules::peer_pool_ref_ok(text)
    {
        return Err(unprocessable(
            "peer_pool_ref must be the peer's pool id: a lower-case hyphenated UUID",
        ));
    }
    if let Some(text) = &payload.peer_pool_ref {
        let taken = skill_pools::Entity::find()
            .filter(skill_pools::Column::OrganizationRef.eq(&pool.organization_ref))
            .filter(skill_pools::Column::PeerPoolRef.eq(text))
            .filter(skill_pools::Column::Pid.ne(pool.pid))
            .filter(skill_pools::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?;
        if taken.is_some() {
            return Err(unprocessable(
                "another pool is already linked to that peer pool",
            ));
        }
    }
    let action = if payload.peer_pool_ref.is_some() {
        "peer_linked"
    } else {
        "peer_unlinked"
    };
    let mut active = pool.into_active_model();
    active.peer_pool_ref = ActiveValue::set(payload.peer_pool_ref.clone());
    let txn = ctx.db.begin().await?;
    let row = active.update(&txn).await?;
    Audit::record(&txn, "skill_pool", row.pid, action, caller.actor(), None).await?;
    txn.commit().await?;
    format::json(json!({
        "pid": row.pid,
        "peer_pool_ref": row.peer_pool_ref,
        "demand_source": rules::counted_source(row.peer_pool_ref.is_some()),
    }))
}

/// `POST /api/capacity/peer-demand` body.
#[derive(Debug, Deserialize)]
pub(super) struct PeerDemandPayload {
    pool_pid: Uuid,
    programme_ref: String,
    month: NaiveDate,
    fte_centi: i64,
    /// The peer's id for the event that carried this figure; the same id again changes nothing.
    peer_event_id: String,
}

/// `POST /api/capacity/peer-demand` — take a programme's demand on a linked pool for a month from the peer. It is stored as a claim with
/// source `ppm`, `proposed` (a claim becomes active only through a start decision, WPM-D50), beside any claim entered here and never
/// added to it. A pool that is not linked refuses it. **The route is WPM's own to define; the peer has not been read to call it.**
#[debug_handler]
pub(super) async fn receive_peer_demand(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<PeerDemandPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_ref(
        "programme_ref",
        entity_ref::EntityType::Thing,
        &payload.programme_ref,
    );
    let mut problems = problems.into_vec();
    let event = payload.peer_event_id.trim();
    if event.is_empty() || event.len() > MAX_EVENT_ID {
        problems.push(format!(
            "peer_event_id must be 1 to {MAX_EVENT_ID} characters"
        ));
    }
    ensure_valid(&problems)?;
    rules::validate_month(payload.month).map_err(|e| unprocessable(&e))?;
    rules::validate_fte_centi(payload.fte_centi).map_err(|e| unprocessable(&e))?;
    let pool = live_pool(&ctx, &caller, &payload.pool_pid.to_string()).await?;
    if pool.peer_pool_ref.is_none() {
        return Err(unprocessable(
            "the pool is not linked to a peer pool, so it takes no demand from the peer",
        ));
    }
    let repeated = programme_demands::Entity::find()
        .filter(programme_demands::Column::Source.eq("ppm"))
        .filter(programme_demands::Column::PeerEventId.eq(event))
        .one(&ctx.db)
        .await?;
    if let Some(row) = repeated {
        return format::json(json!({ "pid": row.pid, "duplicate": true }));
    }
    let existing = programme_demands::Entity::find()
        .filter(programme_demands::Column::ProgrammeRef.eq(&payload.programme_ref))
        .filter(programme_demands::Column::PoolPid.eq(pool.pid))
        .filter(programme_demands::Column::Month.eq(payload.month))
        .filter(programme_demands::Column::Source.eq("ppm"))
        .filter(programme_demands::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?;
    let txn = ctx.db.begin().await?;
    let row = if let Some(row) = existing {
        let mut active = row.into_active_model();
        active.fte_centi = ActiveValue::set(payload.fte_centi);
        active.peer_event_id = ActiveValue::set(Some(event.to_string()));
        active.received_at = ActiveValue::set(Some(Utc::now().into()));
        active.update(&txn).await?
    } else {
        programme_demands::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            programme_ref: ActiveValue::set(payload.programme_ref.clone()),
            pool_pid: ActiveValue::set(pool.pid),
            month: ActiveValue::set(payload.month),
            fte_centi: ActiveValue::set(payload.fte_centi),
            status: ActiveValue::set("proposed".to_string()),
            source: ActiveValue::set("ppm".to_string()),
            peer_event_id: ActiveValue::set(Some(event.to_string())),
            received_at: ActiveValue::set(Some(Utc::now().into())),
            deleted_at: ActiveValue::set(None),
            ..Default::default()
        }
        .insert(&txn)
        .await?
    };
    Audit::record(
        &txn,
        "programme_demand",
        row.pid,
        "peer_demand_received",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": row.pid, "duplicate": false }))
}
