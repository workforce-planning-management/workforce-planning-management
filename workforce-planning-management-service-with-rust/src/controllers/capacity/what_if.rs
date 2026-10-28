//! The what-if answer (part of [`super`]): what a proposed change to a programme's demand does to the plan (WPM-R97, WPM-R209).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

/// The most changes one proposal may carry.
const MAX_CHANGES: usize = 200;

/// One proposed change to the programme's active claim on a pool in a month.
#[derive(Debug, Deserialize)]
pub(super) struct ChangeLine {
    pool_pid: Uuid,
    month: NaiveDate,
    /// Centi-FTE; negative gives capacity back.
    fte_centi: i64,
}

/// `POST /api/capacity/what-if` body.
#[derive(Debug, Deserialize)]
pub(super) struct WhatIfPayload {
    organization_ref: String,
    programme_ref: String,
    changes: Vec<ChangeLine>,
}

/// `POST /api/capacity/what-if` — what a proposed change to a programme's demand does: which pool-months become over-committed or stop
/// being so, the constraint pool before and after, and the earliest start that fits. It reads the capacity view, computes, and **stores
/// nothing**. It names no one: a pool is a skill, and no worker, member, partner or other programme appears. Where the model has no pools,
/// or a partner has not answered, the answer is `unknown`, never "no impact" (WPM-R97).
#[debug_handler]
pub(super) async fn what_if(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<WhatIfPayload>,
) -> Result<Response> {
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
    let mut problems = problems.into_vec();
    if payload.changes.len() > MAX_CHANGES {
        problems.push(format!("a proposal has at most {MAX_CHANGES} changes"));
    }
    for line in &payload.changes {
        if rules::validate_month(line.month).is_err() {
            problems.push("every month must be the first day of a month".to_string());
            break;
        }
    }
    ensure_valid(&problems)?;
    ensure_org_in_scope(&ctx, &caller, &payload.organization_ref).await?;
    let from = payload
        .changes
        .iter()
        .map(|c| c.month)
        .min()
        .unwrap_or_else(today);
    let capacity = load_org_capacity(&ctx, &payload.organization_ref, from, CHECK_MONTHS).await?;
    if capacity.pools.is_empty() {
        return answer_unknown("the organization has no skill pools, so capacity cannot be said");
    }
    let names: BTreeMap<String, String> = capacity
        .pools
        .iter()
        .map(|p| (p.pid.to_string(), p.name.clone()))
        .collect();
    let changes: Vec<rules::Change> = payload
        .changes
        .iter()
        .map(|c| rules::Change {
            pool: c.pool_pid.to_string(),
            month: c.month,
            fte_centi: c.fte_centi,
        })
        .collect();
    ensure_valid(&rules::change_problems(
        &capacity.views,
        &capacity.months,
        &changes,
    ))?;
    // What this programme already holds, by pool and month: a change cannot take back more.
    let mut held: BTreeMap<(String, NaiveDate), i64> = BTreeMap::new();
    for (pool, demand) in &capacity.demand {
        for d in demand.iter().filter(|d| {
            d.programme == payload.programme_ref && d.status == rules::DemandStatus::Active
        }) {
            *held.entry((pool.clone(), d.month)).or_insert(0) += d.fte_centi;
        }
    }
    let result = rules::what_if(&capacity.views, &held, &changes);
    let txn = ctx.db.begin().await?;
    Audit::record(
        &txn,
        "capacity",
        Uuid::new_v4(),
        "capacity_what_if_asked",
        caller.actor(),
        Some(json!({
            "changes": changes.len(),
            "answer": result.answer.as_str(),
            "becomes_over": result.becomes_over,
        })),
    )
    .await?;
    txn.commit().await?;
    format::json(answer_json(&result, &names, from))
}

/// The answer as the API shows it. A pool is named (a skill, not a person); nothing finer appears.
fn answer_json(
    result: &rules::WhatIf,
    names: &BTreeMap<String, String>,
    from: NaiveDate,
) -> serde_json::Value {
    let name = |key: &Option<String>| {
        key.as_ref()
            .map(|k| json!({ "pool_pid": k, "name": names.get(k) }))
    };
    let earliest_start = result
        .earliest_shift_months
        .and_then(|k| rules::shift_month(from, k));
    json!({
        "answer": result.answer.as_str(),
        "becomes_over": result.becomes_over,
        "stops_over": result.stops_over,
        "unknown": result.unknown,
        "constraint_pool_before": name(&result.constraint_before),
        "constraint_pool_after": name(&result.constraint_after),
        "earliest_shift_months": result.earliest_shift_months,
        "earliest_start": earliest_start,
        "cells": result.cells.iter().map(|c| json!({
            "pool_pid": c.pool,
            "pool": names.get(&c.pool),
            "month": c.month,
            "before": c.before.as_str(),
            "after": c.after.as_str(),
            "before_balance_centi": c.before_balance_centi,
            "after_balance_centi": c.after_balance_centi,
        })).collect::<Vec<_>>(),
        "stored": false,
        "derivation": rules::DERIVATION,
    })
}

/// The answer when the model cannot say anything: `unknown`, with the reason, and nothing else.
fn answer_unknown(reason: &str) -> Result<Response> {
    format::json(json!({
        "answer": "unknown",
        "reason": reason,
        "cells": [],
        "stored": false,
        "derivation": rules::DERIVATION,
    }))
}
