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
    /// The figure, in hundredths of an FTE; `0` withdraws the peer's claim (WPM-D146).
    fte_centi: i64,
    /// The peer's id for the event that carried this figure; the same id again changes nothing.
    peer_event_id: String,
    /// When the peer sent the event (RFC 3339). Optional; when it and the last applied event both carry one, an older event is ignored.
    #[serde(default)]
    sent_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}

/// Write what [`rules::peer_demand_outcome`] decided: a new claim, or the existing one changed. Returns the row and whether it had been
/// active before.
async fn write_peer_claim(
    txn: &sea_orm::DatabaseTransaction,
    pool: &skill_pools::Model,
    payload: &PeerDemandPayload,
    event: &str,
    sent_at: Option<chrono::DateTime<Utc>>,
    existing: Option<programme_demands::Model>,
    outcome: rules::PeerDemandOutcome,
) -> Result<(programme_demands::Model, bool)> {
    let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
    if let Some(row) = existing {
        let was_active = row.status == "active";
        let mut active = row.into_active_model();
        match outcome {
            rules::PeerDemandOutcome::Withdrawn => {
                // The last figure is kept (the column refuses zero); the claim is closed and stops counting.
                active.status = ActiveValue::set("closed".to_string());
                active.peer_withdrawn_at = ActiveValue::set(Some(now));
            }
            rules::PeerDemandOutcome::Reopened => {
                active.fte_centi = ActiveValue::set(payload.fte_centi);
                active.status = ActiveValue::set("proposed".to_string());
                active.peer_withdrawn_at = ActiveValue::set(None);
            }
            rules::PeerDemandOutcome::Updated => {
                active.fte_centi = ActiveValue::set(payload.fte_centi);
            }
            _ => {}
        }
        active.peer_event_id = ActiveValue::set(Some(event.to_string()));
        active.received_at = ActiveValue::set(Some(now));
        if sent_at.is_some() {
            active.peer_sent_at = ActiveValue::set(payload.sent_at);
        }
        return Ok((active.update(txn).await?, was_active));
    }
    let row = programme_demands::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        programme_ref: ActiveValue::set(payload.programme_ref.clone()),
        pool_pid: ActiveValue::set(pool.pid),
        month: ActiveValue::set(payload.month),
        fte_centi: ActiveValue::set(payload.fte_centi),
        status: ActiveValue::set("proposed".to_string()),
        source: ActiveValue::set("ppm".to_string()),
        peer_event_id: ActiveValue::set(Some(event.to_string())),
        received_at: ActiveValue::set(Some(now)),
        peer_sent_at: ActiveValue::set(payload.sent_at),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(txn)
    .await?;
    Ok((row, false))
}

/// `POST /api/capacity/peer-demand` — take a programme's demand on a linked pool for a month from the peer. It is stored as a claim with
/// source `ppm`, `proposed` (a claim becomes active only through a start decision, WPM-D50), beside any claim entered here and never
/// added to it. A pool that is not linked refuses it. **`fte_centi: 0` withdraws the claim** (WPM-R215, WPM-D146): it is `closed` and
/// marked withdrawn by the peer, an active one stops counting, and the start decision that activated it is kept as it was; zero for a
/// claim that does not exist changes nothing and says so. An event older (`sent_at`) than the last one applied is ignored. The answer
/// says what happened (`outcome`, [`rules::PeerDemandOutcome`]).
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
    rules::validate_peer_fte_centi(payload.fte_centi).map_err(|e| unprocessable(&e))?;
    let sent_at = payload.sent_at.map(|t| t.with_timezone(&Utc));
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
        return format::json(json!({
            "pid": row.pid, "duplicate": true, "outcome": "duplicate", "status": row.status,
        }));
    }
    let existing = programme_demands::Entity::find()
        .filter(programme_demands::Column::ProgrammeRef.eq(&payload.programme_ref))
        .filter(programme_demands::Column::PoolPid.eq(pool.pid))
        .filter(programme_demands::Column::Month.eq(payload.month))
        .filter(programme_demands::Column::Source.eq("ppm"))
        .filter(programme_demands::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?;
    let outcome = rules::peer_demand_outcome(
        payload.fte_centi,
        sent_at,
        existing.as_ref().map(|row| rules::PeerClaim {
            withdrawn: row.status == "closed",
            last_sent_at: row.peer_sent_at.map(|t| t.with_timezone(&Utc)),
        }),
    );
    if !outcome.writes() {
        return format::json(json!({
            "pid": existing.as_ref().map(|row| row.pid),
            "duplicate": false,
            "outcome": outcome.as_str(),
            "status": existing.as_ref().map(|row| row.status.clone()),
        }));
    }
    let txn = ctx.db.begin().await?;
    let (row, was_active) =
        write_peer_claim(&txn, &pool, &payload, event, sent_at, existing, outcome).await?;
    let action = match outcome {
        rules::PeerDemandOutcome::Withdrawn | rules::PeerDemandOutcome::AlreadyWithdrawn => {
            "peer_demand_withdrawn"
        }
        _ => "peer_demand_received",
    };
    // The audit entry says whether a withdrawn claim had been active; no figure.
    let snapshot = (outcome == rules::PeerDemandOutcome::Withdrawn)
        .then(|| json!({ "was_active": was_active }));
    Audit::record(
        &txn,
        "programme_demand",
        row.pid,
        action,
        caller.actor(),
        snapshot,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({
        "pid": row.pid, "duplicate": false, "outcome": outcome.as_str(), "status": row.status,
    }))
}

/// Query for `GET /api/capacity/peer-supply`.
#[derive(Debug, Deserialize)]
pub(super) struct PeerSupplyQuery {
    pool: Uuid,
    from: Option<NaiveDate>,
    months: Option<u32>,
}

/// `GET /api/capacity/peer-supply?pool=&from=&months=` — what a linked pool gives its peer: the pool's supply by month, as FTE in
/// hundredths, with a month **hidden** (`null`) where fewer than five workers make it up. It names no worker, gives no total, and is
/// answered under the caller's own token (WPM-D135): a caller who may not see the pool is refused as on the capacity view. A pool that is
/// not linked has nothing to give (WPM-D138, WPM-R213).
#[debug_handler]
pub(super) async fn peer_supply_view(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<PeerSupplyQuery>,
) -> Result<Response> {
    let pool = live_pool(&ctx, &caller, &query.pool.to_string()).await?;
    let Some(peer_pool_ref) = pool.peer_pool_ref.clone() else {
        return Err(unprocessable(
            "the pool is not linked to a peer pool, so it gives the peer nothing",
        ));
    };
    let from = query.from.unwrap_or_else(today);
    let capacity = load_org_capacity(
        &ctx,
        &pool.organization_ref,
        from,
        query.months.unwrap_or(DEFAULT_MONTHS),
    )
    .await?;
    let key = pool.pid.to_string();
    let (Some(view), Some(workers)) = (capacity.views.get(&key), capacity.workers.get(&key)) else {
        return Err(Error::NotFound);
    };
    let supply: Vec<i64> = view.cells.iter().map(|c| c.supply_centi).collect();
    let shown = rules::peer_supply(&supply, workers, crate::rules::pulse::K_ANONYMITY);
    format::json(json!({
        "pool_pid": pool.pid,
        "peer_pool_ref": peer_pool_ref,
        "source": "wpm",
        "as_of": Utc::now(),
        "floor": crate::rules::pulse::K_ANONYMITY,
        "months": capacity.months.iter().zip(&shown).map(|(month, centi)| json!({
            "month": month,
            "supply_centi": centi,
            "hidden": centi.is_none(),
        })).collect::<Vec<_>>(),
        "derivation": rules::DERIVATION,
    }))
}
