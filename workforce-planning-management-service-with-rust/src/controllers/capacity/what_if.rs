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
    /// Where the change moves the programme's end date, if it does: the contingent effect is worked out against it (WPM-R97).
    #[serde(default)]
    programme_new_end: Option<NaiveDate>,
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
    let contingent = match payload.programme_new_end {
        Some(new_end) => {
            Some(contingent_effect(&ctx, &payload.organization_ref, &capacity, new_end).await?)
        }
        None => None,
    };
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
    let mut body = answer_json(&result, &names, from);
    body["programme_new_end"] = json!(payload.programme_new_end);
    body["contingent_ending"] = json!(contingent.map(|counts| {
        counts
        .into_iter()
        .map(|(pool, count)| json!({ "pool_pid": pool, "pool": names.get(&pool), "count": count }))
        .collect::<Vec<_>>()
    }));
    format::json(body)
}

/// Per pool, how many fixed-term and contractor engagements end before the programme's new end: counts, floored, no name and no total.
/// The dates are read for the count and go no further.
async fn contingent_effect(
    ctx: &AppContext,
    org: &str,
    capacity: &OrgCapacity,
    new_end: NaiveDate,
) -> Result<Vec<(String, Option<usize>)>> {
    let mut per_pool = Vec::new();
    for pool in &capacity.pools {
        let key = pool.pid.to_string();
        let members = capacity.members.get(&key).map_or(&[][..], Vec::as_slice);
        let mut ends = Vec::new();
        if !members.is_empty() {
            let sql = format!(
                "SELECT w.engagement_ends_on AS ends FROM workers w
                 WHERE w.deleted_at IS NULL AND w.organization_ref = $1 AND w.terminated_on IS NULL
                   AND w.employment_type IN ('fixed_term', 'contractor') AND w.engagement_ends_on IS NOT NULL
                   AND ({})",
                holds_sql(members)
            );
            for row in ctx
                .db
                .query_all_raw(sea_orm::Statement::from_sql_and_values(
                    sea_orm::DbBackend::Postgres,
                    sql,
                    [org.into()],
                ))
                .await?
            {
                ends.push(row.try_get::<NaiveDate>("", "ends")?);
            }
        }
        per_pool.push((key, ends));
    }
    Ok(rules::ending_counts(
        &per_pool,
        today(),
        new_end,
        crate::rules::pulse::K_ANONYMITY,
    ))
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
