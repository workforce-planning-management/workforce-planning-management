//! Start check and decision (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;
use axa::extract::Json;

/// One line of a what-if.
#[derive(Debug, Deserialize)]
pub(super) struct WishLine {
    pool_pid: Uuid,
    month: NaiveDate,
    fte_centi: i64,
}

/// `POST /api/capacity/start-check` and `/start-decisions` body.
#[derive(Debug, Deserialize)]
pub(super) struct StartPayload {
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

pub(super) struct Checked {
    check: rules::StartCheck,
    capacity: OrgCapacity,
    constraint: Option<String>,
    proposed: Vec<programme_demands::Model>,
    from: NaiveDate,
}

pub(super) async fn run_start_check(
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
    let pool_rows = pools_of(ctx, &payload.organization_ref).await?;
    let org_pools: Vec<Uuid> = pool_rows.iter().map(|p| p.pid).collect();
    let linked: std::collections::HashSet<Uuid> = pool_rows
        .iter()
        .filter(|p| p.peer_pool_ref.is_some())
        .map(|p| p.pid)
        .collect();
    let proposed: Vec<programme_demands::Model> =
        demands_of_programme(ctx, &payload.programme_ref, true, Some(&org_pools))
            .await?
            .into_iter()
            .filter(|r| counts_here(&r.source, linked.contains(&r.pool_pid)))
            .collect();
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

pub(super) fn check_json(checked: &Checked) -> serde_json::Value {
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
pub(super) async fn start_check(
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
pub(super) async fn record_start_decision(
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
    let decision_pid = Uuid::new_v4();
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO start_decisions (pid, organization_ref, programme_ref, decision, reason, fit, earliest_shift_months,
                                      wip_exceeds, decided_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        decision_pid,
        payload.organization_ref,
        payload.programme_ref,
        decision,
        reason.trim(),
        checked.check.fit.as_str(),
        checked
            .check
            .earliest_shift_months
            .and_then(|n| i32::try_from(n).ok()),
        checked.check.wip.as_ref().map(|w| w.exceeds),
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    if decision == "proceed" {
        // The claims the check was made on become active, in one statement.
        let claims: Vec<i32> = checked.proposed.iter().map(|c| c.id).collect();
        sqlx::query!(
            "UPDATE programme_demands SET status = 'active' WHERE id = ANY($1)",
            &claims
        )
        .execute(&mut *tx)
        .await?;
    }
    // The audit entry names the decision, not the reason text.
    crate::db::audit(
        &mut tx,
        "start_decision",
        decision_pid,
        if decision == "proceed" {
            "proceeded"
        } else {
            "deferred"
        },
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(
        json!({ "pid": decision_pid, "decision": decision, "check": check_json(&checked) }),
    )
}

/// Query for the decision list.
#[derive(Debug, Deserialize)]
pub(super) struct DecisionQuery {
    programme: Option<String>,
    organization: Option<String>,
}

/// `GET /api/capacity/start-decisions?programme=&organization=`.
#[debug_handler]
pub(super) async fn list_start_decisions(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(query): axa::extract::Query<DecisionQuery>,
) -> Result<Response> {
    let refs = scope(&ctx, &caller).await?;
    let rows = sqlx::query!(
        "SELECT pid, organization_ref, programme_ref, decision, reason, fit, earliest_shift_months, wip_exceeds,
                decided_by, created_at
           FROM start_decisions
          WHERE ($1::text IS NULL OR programme_ref = $1) AND ($2::text IS NULL OR organization_ref = $2)
            AND ($3::text[] IS NULL OR organization_ref = ANY($3))
          ORDER BY id DESC",
        query.programme,
        query.organization,
        refs.as_deref(),
    )
    .fetch_all(crate::db::pool(&ctx))
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
