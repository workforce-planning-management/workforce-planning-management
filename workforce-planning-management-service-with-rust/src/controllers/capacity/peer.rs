//! The link to a peer's pool and demand received from the peer (part of [`super`]): WPM-R213, WPM-D138 to D141.

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;
use axa::extract::{Json, Path};

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
        let taken = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM skill_pools WHERE organization_ref = $1 AND peer_pool_ref = $2 AND pid <> $3
                                                         AND deleted_at IS NULL) AS "taken!""#,
            pool.organization_ref,
            text,
            pool.pid,
        )
        .fetch_one(crate::db::pool(&ctx))
        .await?;
        if taken {
            return Err(unprocessable(
                "another pool is already linked to that peer pool",
            ));
        }
    }
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE skill_pools SET peer_pool_ref = $2 WHERE id = $1",
        pool.id,
        payload.peer_pool_ref,
    )
    .execute(&mut *tx)
    .await?;
    if payload.peer_pool_ref.is_some() {
        crate::db::audit(
            &mut tx,
            "skill_pool",
            pool.pid,
            "peer_linked",
            caller.actor(),
            None,
        )
        .await?;
    } else {
        crate::db::audit(
            &mut tx,
            "skill_pool",
            pool.pid,
            "peer_unlinked",
            caller.actor(),
            None,
        )
        .await?;
    }
    tx.commit().await?;
    format::json(json!({
        "pid": pool.pid,
        "peer_pool_ref": payload.peer_pool_ref,
        "demand_source": rules::counted_source(payload.peer_pool_ref.is_some()),
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
    /// The peer's id for the event that carried this figure; the same id again for the same pool, programme and month changes nothing.
    peer_event_id: String,
    /// When the peer sent the event (RFC 3339). Optional; when it and the last applied event both carry one, an older event is ignored.
    #[serde(default)]
    sent_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}

/// Write what [`rules::peer_demand_outcome`] decided: a new claim, or the existing one changed. Returns the row and whether it had been
/// active before.
async fn write_peer_claim(
    tx: &mut sqlx::PgConnection,
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
        let (fte_centi, status, withdrawn_at) = match outcome {
            // The last figure is kept (the column refuses zero); the claim is closed and stops counting.
            rules::PeerDemandOutcome::Withdrawn => (row.fte_centi, "closed".to_string(), Some(now)),
            rules::PeerDemandOutcome::Reopened => (payload.fte_centi, "proposed".to_string(), None),
            rules::PeerDemandOutcome::Updated => {
                (payload.fte_centi, row.status.clone(), row.peer_withdrawn_at)
            }
            _ => (row.fte_centi, row.status.clone(), row.peer_withdrawn_at),
        };
        let peer_sent_at = if sent_at.is_some() {
            payload.sent_at
        } else {
            row.peer_sent_at
        };
        let updated = sqlx::query_as!(
            programme_demands::Model,
            r#"UPDATE programme_demands
                  SET fte_centi = $2, status = $3, peer_withdrawn_at = $4, peer_event_id = $5, received_at = $6,
                      peer_sent_at = $7
                WHERE id = $1
                RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, programme_ref, pool_pid, month,
                  fte_centi, status, source, peer_event_id, received_at AS "received_at: _",
                  peer_sent_at AS "peer_sent_at: _", peer_withdrawn_at AS "peer_withdrawn_at: _",
                  deleted_at AS "deleted_at: _""#,
            row.id,
            fte_centi,
            status,
            withdrawn_at,
            event,
            now,
            peer_sent_at,
        )
        .fetch_one(tx)
        .await?;
        return Ok((updated, was_active));
    }
    let row = sqlx::query_as!(
        programme_demands::Model,
        r#"INSERT INTO programme_demands (pid, programme_ref, pool_pid, month, fte_centi, status, source, peer_event_id,
                                          received_at, peer_sent_at)
           VALUES ($1, $2, $3, $4, $5, 'proposed', 'ppm', $6, $7, $8)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, programme_ref, pool_pid, month,
                  fte_centi, status, source, peer_event_id, received_at AS "received_at: _",
                  peer_sent_at AS "peer_sent_at: _", peer_withdrawn_at AS "peer_withdrawn_at: _",
                  deleted_at AS "deleted_at: _""#,
        Uuid::new_v4(),
        payload.programme_ref,
        pool.pid,
        payload.month,
        payload.fte_centi,
        event,
        now,
        payload.sent_at,
    )
    .fetch_one(tx)
    .await?;
    Ok((row, false))
}

/// The audit entry of a peer claim written: whether a withdrawn claim had been active, and no figure. Each action is
/// written out, so the audit guard reads it.
async fn audit_peer_claim(
    connection: &mut sqlx::PgConnection,
    claim: Uuid,
    outcome: rules::PeerDemandOutcome,
    was_active: bool,
    actor: Option<&str>,
) -> Result<()> {
    let snapshot = (outcome == rules::PeerDemandOutcome::Withdrawn)
        .then(|| json!({ "was_active": was_active }));
    if matches!(
        outcome,
        rules::PeerDemandOutcome::Withdrawn | rules::PeerDemandOutcome::AlreadyWithdrawn
    ) {
        crate::db::audit(
            connection,
            "programme_demand",
            claim,
            "peer_demand_withdrawn",
            actor,
            snapshot,
        )
        .await?;
    } else {
        crate::db::audit(
            connection,
            "programme_demand",
            claim,
            "peer_demand_received",
            actor,
            snapshot,
        )
        .await?;
    }
    Ok(())
}

/// `POST /api/capacity/peer-demand` — take a programme's demand on a linked pool for a month from the peer. It is stored as a claim with
/// source `ppm`, `proposed` (a claim becomes active only through a start decision, WPM-D50), beside any claim entered here and never
/// added to it. A pool that is not linked refuses it. **`fte_centi: 0` withdraws the claim** (WPM-R215, WPM-D146): it is `closed` and
/// marked withdrawn by the peer, an active one stops counting, and the start decision that activated it is kept as it was; zero for a
/// claim that does not exist changes nothing and says so. An event older (`sent_at`) than the last one applied is ignored. The answer
/// says what happened (`outcome`, [`rules::PeerDemandOutcome`]). The order is fixed (WPM-D148): the body is checked, then the pool under the
/// caller's scope (unknown or out of scope is `404`, unlinked `422`, exactly as for a fresh event), and only then is a repeat looked for,
/// among the claims of the same pool, programme and month.
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
    // A repeat is looked for only among the claims of this event's own key, once the pool is known to be one the caller may see and
    // is linked (WPM-T397, WPM-D148): the same id applied to a claim of another pool, programme or month, in this organization or
    // another, is a new event here, and the answer never says whether the id exists elsewhere.
    let db = crate::db::pool(&ctx);
    let repeated = sqlx::query!(
        "SELECT pid, status FROM programme_demands
          WHERE pool_pid = $1 AND programme_ref = $2 AND month = $3 AND source = 'ppm' AND peer_event_id = $4
          LIMIT 1",
        pool.pid,
        payload.programme_ref,
        payload.month,
        event,
    )
    .fetch_optional(db)
    .await?;
    if let Some(row) = repeated {
        return format::json(json!({
            "pid": row.pid, "duplicate": true, "outcome": "duplicate", "status": row.status,
        }));
    }
    let existing = sqlx::query_as!(
        programme_demands::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, programme_ref, pool_pid, month,
                  fte_centi, status, source, peer_event_id, received_at AS "received_at: _",
                  peer_sent_at AS "peer_sent_at: _", peer_withdrawn_at AS "peer_withdrawn_at: _",
                  deleted_at AS "deleted_at: _"
             FROM programme_demands
            WHERE programme_ref = $1 AND pool_pid = $2 AND month = $3 AND source = 'ppm' AND deleted_at IS NULL"#,
        payload.programme_ref,
        pool.pid,
        payload.month,
    )
    .fetch_optional(db)
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
    let mut tx = db.begin().await?;
    let (row, was_active) =
        write_peer_claim(&mut tx, &pool, &payload, event, sent_at, existing, outcome).await?;
    audit_peer_claim(&mut tx, row.pid, outcome, was_active, caller.actor()).await?;
    tx.commit().await?;
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
    axa::extract::Query(query): axa::extract::Query<PeerSupplyQuery>,
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
