//! The view (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

/// Everything the view and the start check work from, for one organization.
pub(crate) struct OrgCapacity {
    pub(crate) months: Vec<NaiveDate>,
    pub(crate) pools: Vec<skill_pools::Model>,
    pub(crate) views: BTreeMap<String, rules::PoolView>,
    pub(crate) demand: BTreeMap<String, Vec<rules::Demand>>,
    pub(crate) member_counts: BTreeMap<String, usize>,
}

pub(super) fn parse_demand_rows(rows: &[programme_demands::Model]) -> Vec<rules::Demand> {
    rows.iter()
        .filter_map(|r| {
            Some(rules::Demand {
                programme: r.programme_ref.clone(),
                month: r.month,
                fte_centi: r.fte_centi,
                status: rules::DemandStatus::parse(&r.status)?,
            })
        })
        .collect()
}

/// How many workers are read and worked out at a time (WPM-D90). The view's memory and each query's size
/// depend on this, not on the size of the organization.
const WORKER_BATCH: u64 = 2_000;

/// Role profiles by trimmed lower-case job title: a worker whose title matches one holds it.
async fn titles_to_roles(ctx: &AppContext) -> Result<HashMap<String, Uuid>> {
    Ok(role_profiles::Entity::find()
        .filter(role_profiles::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|p| (p.job_title.trim().to_lowercase(), p.pid))
        .collect())
}

/// The next batch of the organization's workers who are employed at some point in the months, as
/// capacity facts, with the `id` to continue after. An empty batch is the end.
async fn worker_facts_batch(
    ctx: &AppContext,
    org: &str,
    months: &[NaiveDate],
    wanted_skills: &BTreeSet<Uuid>,
    titles: &HashMap<String, Uuid>,
    after_id: i32,
) -> Result<(Vec<rules::WorkerFacts>, i32)> {
    let first = months.first().copied().unwrap_or_else(today);
    let last_month = months.last().copied().unwrap_or(first);
    let last_day = last_month
        .checked_add_days(chrono::Days::new(
            u64::try_from(rules::days_in_month(last_month)).unwrap_or(31),
        ))
        .unwrap_or(last_month);
    let staff: Vec<workers::Model> = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::OrganizationRef.eq(org))
        .filter(workers::Column::HiredOn.lt(last_day))
        .filter(
            Condition::any()
                .add(workers::Column::TerminatedOn.is_null())
                .add(workers::Column::TerminatedOn.gt(first)),
        )
        .filter(workers::Column::Id.gt(after_id))
        .order_by_asc(workers::Column::Id)
        .limit(WORKER_BATCH)
        .all(&ctx.db)
        .await?;
    let Some(last_id) = staff.last().map(|w| w.id) else {
        return Ok((Vec::new(), after_id));
    };
    let pids: Vec<Uuid> = staff.iter().map(|w| w.pid).collect();
    let mut chosen: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for row in worker_framework_roles::Entity::find()
        .filter(worker_framework_roles::Column::WorkerPid.is_in(pids.clone()))
        .filter(worker_framework_roles::Column::EndedAt.is_null())
        .all(&ctx.db)
        .await?
    {
        if let Some(role) = row.role_profile_pid {
            chosen.entry(row.worker_pid).or_default().push(role);
        }
    }
    let mut skill_levels: HashMap<Uuid, Vec<(Uuid, i32)>> = HashMap::new();
    if !wanted_skills.is_empty() {
        for row in worker_skills::Entity::find()
            .filter(worker_skills::Column::WorkerPid.is_in(pids.clone()))
            .filter(worker_skills::Column::SkillPid.is_in(wanted_skills.iter().copied()))
            .filter(worker_skills::Column::DeletedAt.is_null())
            .all(&ctx.db)
            .await?
        {
            skill_levels
                .entry(row.worker_pid)
                .or_default()
                .push((row.skill_pid, row.proficiency));
        }
    }
    let mut leave: HashMap<Uuid, Vec<(NaiveDate, NaiveDate)>> = HashMap::new();
    for row in leave_requests::Entity::find()
        .filter(leave_requests::Column::WorkerPid.is_in(pids))
        .filter(leave_requests::Column::Status.eq("approved"))
        .filter(leave_requests::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
    {
        leave
            .entry(row.worker_pid)
            .or_default()
            .push((row.start_on, row.end_on));
    }
    let facts = staff
        .into_iter()
        .map(|w| {
            let mut roles = chosen.remove(&w.pid).unwrap_or_default();
            if let Some(role) = titles.get(&w.job_title.trim().to_lowercase()) {
                roles.push(*role);
            }
            rules::WorkerFacts {
                fte_percent: w.fte_percent,
                hired_on: w.hired_on,
                terminated_on: w.terminated_on,
                role_profiles: roles,
                skills: skill_levels.remove(&w.pid).unwrap_or_default(),
                approved_leave: leave.remove(&w.pid).unwrap_or_default(),
            }
        })
        .collect();
    Ok((facts, last_id))
}

/// For each pool and month, the summed [`rules::pool_weighted_days`] over the organization's workers.
/// Supply adds up across workers, so they are read in batches and only the sums are kept.
async fn weighted_days(
    ctx: &AppContext,
    org: &str,
    months: &[NaiveDate],
    wanted: &BTreeSet<Uuid>,
    members_by_pool: &[Vec<rules::Member>],
) -> Result<Vec<Vec<i64>>> {
    let titles = titles_to_roles(ctx).await?;
    let mut weighted = vec![vec![0_i64; months.len()]; members_by_pool.len()];
    let mut after_id = 0;
    loop {
        let (batch, last_id) =
            worker_facts_batch(ctx, org, months, wanted, &titles, after_id).await?;
        if batch.is_empty() {
            return Ok(weighted);
        }
        for (members, sums) in members_by_pool.iter().zip(weighted.iter_mut()) {
            for (sum, month) in sums.iter_mut().zip(months) {
                *sum += rules::pool_weighted_days(members, &batch, *month);
            }
        }
        after_id = last_id;
    }
}

pub(crate) async fn load_org_capacity(
    ctx: &AppContext,
    org: &str,
    from: NaiveDate,
    count: u32,
) -> Result<OrgCapacity> {
    let months = rules::months(from, count).map_err(|e| unprocessable(&e))?;
    let pools = skill_pools::Entity::find()
        .filter(skill_pools::Column::OrganizationRef.eq(org))
        .filter(skill_pools::Column::DeletedAt.is_null())
        .order_by_asc(skill_pools::Column::Name)
        .all(&ctx.db)
        .await?;
    let pool_pids: Vec<Uuid> = pools.iter().map(|p| p.pid).collect();
    let member_rows = skill_pool_members::Entity::find()
        .filter(skill_pool_members::Column::PoolPid.is_in(pool_pids.clone()))
        .filter(skill_pool_members::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let wanted: BTreeSet<Uuid> = member_rows.iter().filter_map(|m| m.skill_pid).collect();
    let demand_rows = programme_demands::Entity::find()
        .filter(programme_demands::Column::PoolPid.is_in(pool_pids.clone()))
        .filter(programme_demands::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let commitment_rows = partner_commitments::Entity::find()
        .filter(partner_commitments::Column::PoolPid.is_in(pool_pids))
        .filter(partner_commitments::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let members_by_pool: Vec<Vec<rules::Member>> = pools
        .iter()
        .map(|pool| {
            member_rows
                .iter()
                .filter(|m| m.pool_pid == pool.pid)
                .filter_map(
                    |m| match (m.role_profile_pid, m.skill_pid, m.min_proficiency) {
                        (Some(role), None, _) => Some(rules::Member::RoleProfile(role)),
                        (None, Some(skill), Some(min_proficiency)) => Some(rules::Member::Skill {
                            skill,
                            min_proficiency,
                        }),
                        _ => None,
                    },
                )
                .collect()
        })
        .collect();
    let weighted = weighted_days(ctx, org, &months, &wanted, &members_by_pool).await?;
    let mut views = BTreeMap::new();
    let mut demand_by_pool = BTreeMap::new();
    let mut member_counts = BTreeMap::new();
    for ((pool, members), sums) in pools.iter().zip(&members_by_pool).zip(&weighted) {
        let supply: Vec<i64> = months
            .iter()
            .zip(sums)
            .map(|(month, total)| {
                rules::supply_from_weighted(*total, pool.operations_reservation_bp, *month)
            })
            .collect();
        let demand = parse_demand_rows(
            &demand_rows
                .iter()
                .filter(|d| d.pool_pid == pool.pid)
                .cloned()
                .collect::<Vec<_>>(),
        );
        let commitments: Vec<rules::Commitment> = commitment_rows
            .iter()
            .filter(|c| c.pool_pid == pool.pid)
            .filter_map(|c| {
                Some(rules::Commitment {
                    partner: c.partner_ref.clone(),
                    month: c.month,
                    fte_centi: c.fte_centi,
                    status: rules::CommitmentStatus::parse(&c.status)?,
                })
            })
            .collect();
        let key = pool.pid.to_string();
        views.insert(
            key.clone(),
            rules::build_pool(&months, &supply, &demand, &commitments),
        );
        member_counts.insert(key.clone(), members.len());
        demand_by_pool.insert(key, demand);
    }
    Ok(OrgCapacity {
        months,
        pools,
        views,
        demand: demand_by_pool,
        member_counts,
    })
}

pub(super) fn constraint_of(capacity: &OrgCapacity) -> Option<String> {
    let pairs: Vec<(&str, &rules::PoolView)> = capacity
        .views
        .iter()
        .map(|(k, v)| (k.as_str(), v))
        .collect();
    rules::constraint_pool(&pairs).map(ToString::to_string)
}

pub(super) fn cell_json(cell: &rules::Cell) -> serde_json::Value {
    json!({
        "month": cell.month,
        "supply_centi": cell.supply_centi,
        "demand_centi": cell.demand_centi,
        "proposed_centi": cell.proposed_centi,
        "own_balance_centi": cell.own_balance_centi,
        "firm_partner_centi": cell.firm_partner_centi,
        "remaining_shortfall_centi": cell.remaining_shortfall_centi,
        "status": cell.status.as_str(),
        "partners": cell.partners.iter().map(|p| json!({
            "partner": p.partner,
            "status": p.commitment.map(|(s, _)| s.as_str()),
            "fte_centi": p.commitment.map(|(_, f)| f),
        })).collect::<Vec<_>>(),
    })
}

/// Query for the view.
#[derive(Debug, Deserialize)]
pub(super) struct ViewQuery {
    organization: Option<String>,
    from: Option<NaiveDate>,
    pub(super) months: Option<u32>,
}

/// `GET /api/capacity?organization=&from=&months=` — pool × month supply against demand.
/// The capacity view for the monthly pack: the pool × month result and nothing finer. A pool is named
/// (a pool is a skill, not a person); no member, no partner and no programme appears.
pub(crate) async fn pack_view(
    ctx: &AppContext,
    org: &str,
    from: NaiveDate,
) -> Result<serde_json::Value> {
    let capacity = load_org_capacity(ctx, org, from, DEFAULT_MONTHS).await?;
    let constraint = constraint_of(&capacity).and_then(|key| {
        capacity
            .pools
            .iter()
            .find(|p| p.pid.to_string() == key)
            .map(|p| p.name.clone())
    });
    let pools: Vec<serde_json::Value> = capacity
        .pools
        .iter()
        .filter_map(|pool| {
            let view = capacity.views.get(&pool.pid.to_string())?;
            Some(json!({
                "name": pool.name,
                "over_months": view.over_months,
                "unknown_months": view.unknown_months,
                "cells": view.cells.iter().map(|c| json!({
                    "month": c.month,
                    "supply_centi": c.supply_centi,
                    "demand_centi": c.demand_centi,
                    "remaining_shortfall_centi": c.remaining_shortfall_centi,
                    "status": c.status.as_str(),
                })).collect::<Vec<_>>(),
            }))
        })
        .collect();
    Ok(json!({
        "from": capacity.months.first(),
        "months": capacity.months,
        "pools": pools,
        "constraint_pool": constraint,
        "derivation": rules::DERIVATION,
    }))
}

#[debug_handler]
pub(super) async fn capacity_view(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ViewQuery>,
) -> Result<Response> {
    let org = query
        .organization
        .ok_or_else(|| unprocessable("organization is required"))?;
    ensure_org_in_scope(&ctx, &caller, &org).await?;
    let from = query.from.unwrap_or_else(today);
    let capacity =
        load_org_capacity(&ctx, &org, from, query.months.unwrap_or(DEFAULT_MONTHS)).await?;
    let constraint = constraint_of(&capacity);
    let wip_limit = wip_limit_of(&ctx, &org).await?;
    let wip = constraint.as_ref().map(|key| {
        let active =
            rules::programmes_drawing(capacity.demand.get(key).map_or(&[][..], Vec::as_slice));
        json!({ "pool_pid": key, "active": active.len(), "limit": wip_limit })
    });
    let pools: Vec<serde_json::Value> = capacity
        .pools
        .iter()
        .filter_map(|pool| {
            let key = pool.pid.to_string();
            let view = capacity.views.get(&key)?;
            Some(json!({
                "pid": pool.pid,
                "name": pool.name,
                "operations_reservation_bp": pool.operations_reservation_bp,
                "members": capacity.member_counts.get(&key).copied().unwrap_or(0),
                "over_months": view.over_months,
                "unknown_months": view.unknown_months,
                "over_total_centi": view.over_total_centi,
                "cells": view.cells.iter().map(cell_json).collect::<Vec<_>>(),
            }))
        })
        .collect();
    format::json(json!({
        "organization_ref": org,
        "from": capacity.months.first(),
        "months": capacity.months,
        "pools": pools,
        "constraint_pool": constraint,
        "wip": wip,
        "derivation": rules::DERIVATION,
    }))
}
