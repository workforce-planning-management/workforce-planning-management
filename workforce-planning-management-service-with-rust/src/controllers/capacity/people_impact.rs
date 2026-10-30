//! People-impact for a programme (part of [`super`]): WPM-R211, WPM-T361. The pull PPM makes of WPM, under the caller's own token
//! (WPM-D135). The counting is in [`crate::rules::people_impact`].

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

use std::collections::BTreeSet;

use crate::models::_entities::post_fundings;
use crate::rules::people_impact::{self as impact, Sub};

/// Query for `GET /api/capacity/people-impact`.
#[derive(Debug, Deserialize)]
pub(super) struct PeopleImpactQuery {
    /// The programme, `thing:<uuid>` (a PPM plan's pid).
    programme: String,
    /// The programme's end, as its owner gives it: WPM holds none.
    ends_on: Option<NaiveDate>,
    /// How many months, from this one, the over-committed months are looked for in.
    months: Option<u32>,
}

/// `GET /api/capacity/people-impact?programme=thing:<uuid>&ends_on=&months=` — counts only, for one programme: the people committed to
/// it by pool, the fixed-term and contract engagements among them that end before its end, and the over-committed pool-months it holds
/// a claim in. Each is known or unknown with the reason, never a zero for "not known". A count of people under the floor is hidden and
/// no total is given. Answered under the caller's own token and organization scope, and audited (WPM-D135). Stores nothing else.
#[debug_handler]
pub(super) async fn people_impact(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<PeopleImpactQuery>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_ref("programme", entity_ref::EntityType::Thing, &query.programme);
    ensure_valid(&problems.into_vec())?;
    let today = today();
    let months_asked = query.months.unwrap_or(DEFAULT_MONTHS);
    let months = rules::months(today, months_asked).map_err(|e| unprocessable(&e))?;
    let scope = scope(&ctx, &caller).await?;
    let in_scope = |org: &str| {
        scope
            .as_ref()
            .is_none_or(|refs| refs.iter().any(|r| r == org))
    };

    // The people the programme funds, employed today, in scope (WPM-D59).
    let funded = post_fundings::Entity::find()
        .filter(post_fundings::Column::ProgrammeRef.eq(&query.programme))
        .all(&ctx.db)
        .await?;
    let funded_pids: Vec<Uuid> = funded
        .iter()
        .filter(|f| f.ends_on.is_none_or(|e| e >= today))
        .map(|f| f.worker_pid)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let staff =
        crate::controllers::funding::employed_among(&ctx, scope.as_deref(), today, &funded_pids)
            .await?;

    // The pools the programme holds a claim on, in scope.
    let demand_rows = programme_demands::Entity::find()
        .filter(programme_demands::Column::ProgrammeRef.eq(&query.programme))
        .filter(programme_demands::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let claimed_pools = skill_pools::Entity::find()
        .filter(
            skill_pools::Column::Pid
                .is_in(demand_rows.iter().map(|d| d.pool_pid).collect::<Vec<_>>()),
        )
        .filter(skill_pools::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let orgs: BTreeSet<String> = staff
        .values()
        .map(|w| w.organization_ref.clone())
        .chain(claimed_pools.iter().map(|p| p.organization_ref.clone()))
        .filter(|o| in_scope(o))
        .collect();

    let Gathered {
        committed,
        pool_keys,
        names,
        claimed,
    } = gather(&ctx, &orgs, &staff, &demand_rows, today, months_asked).await?;

    let committed_counts = impact::workers_committed(&committed, &pool_keys, impact::FLOOR);
    let ending =
        impact::contingent_ending(&committed, &pool_keys, today, query.ends_on, impact::FLOOR);
    let over = impact::over_committed(&claimed);

    let txn = ctx.db.begin().await?;
    Audit::record(
        &txn,
        "capacity",
        Uuid::new_v4(),
        "people_impact_asked",
        caller.actor(),
        Some(json!({
            "programme_ref": query.programme,
            "organizations": orgs.len(),
            "ends_on_given": query.ends_on.is_some(),
        })),
    )
    .await?;
    txn.commit().await?;

    let (workers_json, ending_json, over_json) =
        parts_json(&committed_counts, &ending, &over, &names);
    format::json(json!({
        "programme_ref": query.programme,
        "source": "wpm",
        "as_of": today,
        "programme_ends_on": query.ends_on,
        "programme_ends_on_source": query.ends_on.map(|_| "caller"),
        "months": months,
        "floor": impact::FLOOR,
        "workers_committed": workers_json,
        "contingent_ending_before_end": ending_json,
        "over_committed_pool_months": over_json,
        "derivation": impact::DERIVATION,
    }))
}

/// What the counting works from, gathered across the organizations involved.
struct Gathered {
    committed: Vec<impact::Committed>,
    pool_keys: Vec<String>,
    names: BTreeMap<String, String>,
    claimed: Vec<impact::ClaimedMonth>,
}

/// For each organization: its pools, which of the committed people count towards each, and the pool-months the programme holds a
/// counted claim in, with what each comes to in the capacity view.
async fn gather(
    ctx: &AppContext,
    orgs: &BTreeSet<String>,
    staff: &std::collections::HashMap<Uuid, crate::models::_entities::workers::Model>,
    demand_rows: &[programme_demands::Model],
    today: NaiveDate,
    months_asked: u32,
) -> Result<Gathered> {
    let mut pool_keys = Vec::new();
    let mut names = BTreeMap::new();
    let mut pools_of: BTreeMap<Uuid, Vec<String>> = BTreeMap::new();
    let mut claimed = Vec::new();
    for org in orgs {
        let capacity = load_org_capacity(ctx, org, today, months_asked).await?;
        let here: Vec<Uuid> = staff
            .values()
            .filter(|w| &w.organization_ref == org)
            .map(|w| w.pid)
            .collect();
        for pool in &capacity.pools {
            let key = pool.pid.to_string();
            pool_keys.push(key.clone());
            names.insert(key.clone(), pool.name.clone());
            let members = capacity.members.get(&key).map_or(&[][..], Vec::as_slice);
            if !members.is_empty() && !here.is_empty() {
                for pid in members_among(ctx, org, members, &here).await? {
                    pools_of.entry(pid).or_default().push(key.clone());
                }
            }
            let Some(view) = capacity.views.get(&key) else {
                continue;
            };
            let linked = pool.peer_pool_ref.is_some();
            for d in demand_rows
                .iter()
                .filter(|d| d.pool_pid == pool.pid && counts_here(&d.source, linked))
            {
                if let Some(cell) = view.cells.iter().find(|c| c.month == d.month) {
                    claimed.push(impact::ClaimedMonth {
                        pool: key.clone(),
                        month: d.month,
                        status: cell.status,
                    });
                }
            }
        }
    }
    let committed: Vec<impact::Committed> = staff
        .values()
        .map(|w| impact::Committed {
            pools: pools_of.get(&w.pid).cloned().unwrap_or_default(),
            contingent: matches!(w.employment_type.as_str(), "fixed_term" | "contractor"),
            engagement_ends: w.engagement_ends_on,
        })
        .collect();

    Ok(Gathered {
        committed,
        pool_keys,
        names,
        claimed,
    })
}

/// The three parts as the API shows them: a pool is named (a skill, not a person); a count of people is `null` with `hidden: true` under
/// the floor; an unknown part has a reason and no figures.
fn parts_json(
    committed_counts: &Sub<impact::ByPool>,
    ending: &Sub<impact::Ending>,
    over: &Sub<impact::Over>,
    names: &BTreeMap<String, String>,
) -> (serde_json::Value, serde_json::Value, serde_json::Value) {
    let pool_json = |pools: &[(String, Option<usize>)]| -> Vec<serde_json::Value> {
        pools
            .iter()
            .map(|(k, n)| json!({ "pool_pid": k, "pool": names.get(k), "count": n, "hidden": n.is_none() }))
            .collect()
    };
    let cell = |n: Option<usize>| json!({ "count": n, "hidden": n.is_none() });
    let workers_json = match committed_counts {
        Sub::Known(c) => json!({
            "known": true, "reason": null,
            "pools": pool_json(&c.pools), "in_no_pool": cell(c.in_no_pool),
        }),
        Sub::Unknown(reason) => {
            json!({ "known": false, "reason": reason, "pools": null, "in_no_pool": null })
        }
    };
    let ending_json = match ending {
        Sub::Known(e) => json!({
            "known": true, "reason": null,
            "pools": pool_json(&e.by_pool.pools), "in_no_pool": cell(e.by_pool.in_no_pool),
            "end_not_recorded": cell(e.end_not_recorded),
        }),
        Sub::Unknown(reason) => json!({
            "known": false, "reason": reason, "pools": null, "in_no_pool": null, "end_not_recorded": null,
        }),
    };
    let over_json = match over {
        Sub::Known(o) => json!({
            "known": true, "reason": null,
            "considered": o.considered, "over": o.over, "unknown": o.unknown,
            "pools": o.pools.iter().map(|(k, over, unknown)| json!({
                "pool_pid": k, "pool": names.get(k), "over_months": over, "unknown_months": unknown,
            })).collect::<Vec<_>>(),
        }),
        Sub::Unknown(reason) => json!({
            "known": false, "reason": reason, "considered": null, "over": null, "unknown": null, "pools": null,
        }),
    };
    (workers_json, ending_json, over_json)
}

/// Which of `among` (workers of `org`) count towards a pool with these members. Only UUIDs and integers from typed values are written
/// into the SQL; the organization is a bound parameter.
async fn members_among(
    ctx: &AppContext,
    org: &str,
    members: &[rules::Member],
    among: &[Uuid],
) -> Result<Vec<Uuid>> {
    let list = among
        .iter()
        .map(|p| format!("'{p}'"))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT w.pid AS pid FROM workers w
         WHERE w.deleted_at IS NULL AND w.organization_ref = $1 AND w.pid IN ({list}) AND ({})",
        holds_sql(members)
    );
    let rows = ctx
        .db
        .query_all_raw(sea_orm::Statement::from_sql_and_values(
            sea_orm::DbBackend::Postgres,
            sql,
            [org.into()],
        ))
        .await?;
    rows.iter()
        .map(|r| r.try_get::<Uuid>("", "pid").map_err(Into::into))
        .collect()
}
