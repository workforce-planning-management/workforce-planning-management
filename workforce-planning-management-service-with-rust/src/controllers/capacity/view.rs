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
    /// What makes a worker count towards each pool, by pool.
    pub(crate) members: BTreeMap<String, Vec<rules::Member>>,
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

/// For each pool and month, the summed [`rules::pool_weighted_days`] over the organization's workers,
/// worked out by the database (WPM-D95): one query per pool returns a row per month, so neither the
/// time nor the memory of the view depends on how many workers there are. The definitions are the
/// rules': a worker counts if they hold one of the pool's role profiles (chosen, or by job title) or
/// a skill at its minimum; the days are the employed days of the month less the merged approved
/// leave, clipped to employment (`rules::available_days`, which the test walking every day pins).
async fn weighted_days(
    ctx: &AppContext,
    org: &str,
    months: &[NaiveDate],
    members_by_pool: &[Vec<rules::Member>],
) -> Result<Vec<Vec<i64>>> {
    let mut weighted = vec![vec![0_i64; months.len()]; members_by_pool.len()];
    for (members, sums) in members_by_pool.iter().zip(weighted.iter_mut()) {
        if members.is_empty() || months.is_empty() {
            continue;
        }
        let rows = ctx
            .db
            .query_all_raw(sea_orm::Statement::from_sql_and_values(
                sea_orm::DbBackend::Postgres,
                pool_supply_sql(members, months),
                [org.into()],
            ))
            .await?;
        for row in rows {
            let first: NaiveDate = row.try_get("", "first")?;
            let total: i64 = row.try_get("", "total")?;
            if let Some(i) = months.iter().position(|m| *m == first) {
                sums[i] = total;
            }
        }
    }
    Ok(weighted)
}

/// The SQL condition (on `workers w`) that a worker counts towards a pool: one of its role profiles or skills. Only UUIDs and integers
/// from typed values are written into it.
pub(super) fn holds_sql(members: &[rules::Member]) -> String {
    members
        .iter()
        .map(|member| match member {
            rules::Member::RoleProfile(role) => format!(
                "EXISTS (SELECT 1 FROM worker_framework_roles r WHERE r.worker_pid = w.pid
                         AND r.ended_at IS NULL AND r.role_profile_pid = '{role}')
                 OR lower(btrim(w.job_title)) = (SELECT lower(btrim(p.job_title)) FROM role_profiles p
                                                 WHERE p.pid = '{role}' AND p.deleted_at IS NULL)"
            ),
            rules::Member::Skill {
                skill,
                min_proficiency,
            } => format!(
                "EXISTS (SELECT 1 FROM worker_skills k WHERE k.worker_pid = w.pid AND k.deleted_at IS NULL
                         AND k.skill_pid = '{skill}' AND k.proficiency >= {min_proficiency})"
            ),
        })
        .collect::<Vec<_>>()
        .join(" OR ")
}

/// The SQL of [`weighted_days`] for one pool. Only dates, UUIDs and integers are written into it (all
/// from typed values, never from text); the organization is a bound parameter.
fn pool_supply_sql(members: &[rules::Member], months: &[NaiveDate]) -> String {
    let month_rows = months
        .iter()
        .map(|m| {
            let first = rules::month_start(*m);
            let last = first
                .checked_add_days(chrono::Days::new(
                    u64::try_from(rules::days_in_month(first) - 1).unwrap_or(30),
                ))
                .unwrap_or(first);
            format!("(DATE '{first}', DATE '{last}')")
        })
        .collect::<Vec<_>>()
        .join(", ");
    let holds = holds_sql(members);
    format!(
        "WITH m(first, last) AS (VALUES {month_rows}),
         span AS (
           SELECT w.pid, greatest(w.fte_percent, 0) AS fte, m.first,
                  greatest(m.first, w.hired_on) AS f,
                  least(m.last, coalesce(w.terminated_on - 1, m.last)) AS t
           FROM workers w CROSS JOIN m
           WHERE w.deleted_at IS NULL AND w.organization_ref = $1 AND ({holds})
             AND greatest(m.first, w.hired_on) <= least(m.last, coalesce(w.terminated_on - 1, m.last))),
         clipped AS (
           SELECT s.pid, s.first, s.f, greatest(l.start_on, s.f) AS ls, least(l.end_on, s.t) AS le
           FROM span s JOIN leave_requests l
             ON l.worker_pid = s.pid AND l.status = 'approved' AND l.deleted_at IS NULL
            AND l.start_on <= s.t AND l.end_on >= s.f),
         away AS (
           SELECT pid, first, sum(greatest(0, le - greatest(ls, coalesce(covered, f - 1) + 1) + 1)) AS days
           FROM (SELECT c.*, max(le) OVER (PARTITION BY pid, first ORDER BY ls, le
                                           ROWS BETWEEN UNBOUNDED PRECEDING AND 1 PRECEDING) AS covered
                 FROM clipped c) c
           GROUP BY pid, first)
         SELECT s.first AS first,
                coalesce(sum(s.fte::bigint * ((s.t - s.f + 1) - coalesce(a.days, 0))), 0)::bigint AS total
         FROM span s LEFT JOIN away a ON a.pid = s.pid AND a.first = s.first
         GROUP BY s.first"
    )
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
    let weighted = weighted_days(ctx, org, &months, &members_by_pool).await?;
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
    let members = pools
        .iter()
        .map(|p| p.pid.to_string())
        .zip(members_by_pool)
        .collect();
    Ok(OrgCapacity {
        months,
        pools,
        views,
        demand: demand_by_pool,
        member_counts,
        members,
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
