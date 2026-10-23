//! **Post funding** (WPM-R62, WPM-R98): where a worker's post is funded from and when that ends,
//! shown beside the contract end so a contract that outlasts its money, or money that outlasts the
//! contract, is flagged. Pure rules in [`crate::rules::funding`].
//!
//! A kind and a date only, never an amount. HR writes and reads it; it is exported with the worker and
//! erased with them. Nothing here changes an engagement.

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, ExprTrait, QuerySelect, Select, TransactionTrait};
use serde::Deserialize;
use serde_json::json;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    engagement_decisions, engagement_extensions, headcount_snapshots, post_fundings, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::need_to_know::facts_for;
use crate::rules::engagement::{
    ContingentCounts, ContingentEngagement, EXTENSION_THRESHOLD, LISTS, LONG_RUNNING_MONTHS,
    contingent_counts, on_list,
};
use crate::rules::equality_monitoring::DEFAULT_FLOOR;
use crate::rules::funding as rules;
use crate::validation::Problems;

/// How far ahead the funding horizon looks.
const HORIZON_MONTHS: u32 = 12;

async fn find(ctx: &AppContext, worker_pid: uuid::Uuid) -> Result<Option<post_fundings::Model>> {
    Ok(post_fundings::Entity::find()
        .filter(post_fundings::Column::WorkerPid.eq(worker_pid))
        .one(&ctx.db)
        .await?)
}

fn view(worker: &workers::Model, row: Option<&post_fundings::Model>) -> serde_json::Value {
    let alignment = rules::align(worker.engagement_ends_on, row.map(|r| r.ends_on));
    json!({
        "funding": row.map(|r| json!({
            "kind": r.kind, "programme_ref": r.programme_ref, "ends_on": r.ends_on,
        })),
        "contract_ends_on": worker.engagement_ends_on,
        "alignment": alignment.as_str(),
        "needs_attention": alignment.needs_attention(),
    })
}

async fn scoped(ctx: &AppContext, caller: &MaybeAuthUser, pid: &str) -> Result<workers::Model> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(pid)?).await?;
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.iter().any(|r| r == &worker.organization_ref)
    {
        return Err(Error::NotFound);
    }
    Ok(worker)
}

/// `GET /api/workers/{pid}/funding` — the funding beside the contract end, and how they compare.
/// HR only.
#[debug_handler]
async fn get_funding(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = scoped(&ctx, &caller, &pid).await?;
    let row = find(&ctx, worker.pid).await?;
    format::json(view(&worker, row.as_ref()))
}

/// Funding is HR's. The policy decides by action only, so a deployment that lets managers write
/// would otherwise let them write this; the handler asks for itself.
async fn ensure_privileged(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
) -> Result<()> {
    if !crate::auth::require_auth() {
        return Ok(());
    }
    let claims = caller
        .claims()
        .ok_or(Error::Unauthorized("sign in".to_string()))?;
    if facts_for(ctx, claims, worker).await?.privileged {
        Ok(())
    } else {
        Err(Error::CustomError(
            axum::http::StatusCode::FORBIDDEN,
            loco_rs::controller::ErrorDetail::new("forbidden", "only HR records a post's funding"),
        ))
    }
}

/// `PUT /api/workers/{pid}/funding` body.
#[derive(Debug, Deserialize)]
struct FundingPayload {
    kind: String,
    programme_ref: Option<String>,
    ends_on: Option<chrono::NaiveDate>,
}

/// `PUT /api/workers/{pid}/funding` — set or change it.
#[debug_handler]
async fn set_funding(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<FundingPayload>,
) -> Result<Response> {
    let worker = super::engagements::writable_worker(&ctx, &caller, &pid).await?;
    ensure_privileged(&ctx, &caller, &worker).await?;
    let mut problems = Problems::new();
    problems.ref_opt(
        "programme_ref",
        entity_ref::EntityType::Thing,
        payload.programme_ref.as_deref(),
    );
    ensure_valid(&problems.into_vec())?;
    rules::validate(&payload.kind, payload.ends_on, worker.hired_on)
        .map_err(|e| unprocessable(&e))?;
    let by = caller.actor().map(ToString::to_string);
    let existing = find(&ctx, worker.pid).await?;
    let txn = ctx.db.begin().await?;
    let saved = if let Some(row) = existing {
        let mut active: post_fundings::ActiveModel = row.into();
        active.kind = ActiveValue::set(payload.kind);
        active.programme_ref = ActiveValue::set(payload.programme_ref);
        active.ends_on = ActiveValue::set(payload.ends_on);
        active.recorded_by = ActiveValue::set(by);
        active.update(&txn).await?
    } else {
        post_fundings::ActiveModel {
            worker_pid: ActiveValue::set(worker.pid),
            kind: ActiveValue::set(payload.kind),
            programme_ref: ActiveValue::set(payload.programme_ref),
            ends_on: ActiveValue::set(payload.ends_on),
            recorded_by: ActiveValue::set(by),
            ..Default::default()
        }
        .insert(&txn)
        .await?
    };
    Audit::record(
        &txn,
        "worker",
        worker.pid,
        "post_funding_set",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(view(&worker, Some(&saved)))
}

/// `DELETE /api/workers/{pid}/funding` — clear it (none recorded is a `404`).
#[debug_handler]
async fn clear_funding(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = super::engagements::writable_worker(&ctx, &caller, &pid).await?;
    ensure_privileged(&ctx, &caller, &worker).await?;
    let txn = ctx.db.begin().await?;
    let removed = post_fundings::Entity::delete_many()
        .filter(post_fundings::Column::WorkerPid.eq(worker.pid))
        .exec(&txn)
        .await?
        .rows_affected;
    if removed == 0 {
        return Err(Error::NotFound);
    }
    Audit::record(
        &txn,
        "worker",
        worker.pid,
        "post_funding_cleared",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(view(&worker, None))
}

/// The people employed on `today` in the caller's scope (`None` is everyone).
pub(super) async fn employed_in_scope(
    ctx: &AppContext,
    scope: Option<&[String]>,
    today: chrono::NaiveDate,
) -> Result<Vec<workers::Model>> {
    Ok(employed_in_scope_query(scope, today).all(&ctx.db).await?)
}

/// The people among `pids` who are employed on `today` in the caller's scope, by id.
pub(super) async fn employed_among(
    ctx: &AppContext,
    scope: Option<&[String]>,
    today: chrono::NaiveDate,
    pids: &[uuid::Uuid],
) -> Result<std::collections::HashMap<uuid::Uuid, workers::Model>> {
    Ok(super::find_in(
        ctx,
        employed_in_scope_query(scope, today),
        workers::Column::Pid,
        pids,
    )
    .await?
    .into_iter()
    .map(|w| (w.pid, w))
    .collect())
}

/// The same people as a query, to filter, count or use as a subquery in the database
/// ([`crate::rules::metrics::is_employed_on`], in SQL).
pub(super) fn employed_in_scope_query(
    scope: Option<&[String]>,
    today: chrono::NaiveDate,
) -> Select<workers::Entity> {
    let mut find = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::HiredOn.lte(today))
        .filter(
            Condition::any()
                .add(workers::Column::TerminatedOn.is_null())
                .add(workers::Column::TerminatedOn.gt(today)),
        );
    if let Some(refs) = scope {
        find = find.filter(workers::Column::OrganizationRef.is_in(refs.to_vec()));
    }
    find
}

/// Each contractor and fixed-term person among `staff`, as the engagement rules see them.
async fn engagements_of<'a>(
    ctx: &AppContext,
    staff: &'a [workers::Model],
) -> Result<Vec<(&'a workers::Model, ContingentEngagement)>> {
    let contingent_staff: Vec<&workers::Model> = staff
        .iter()
        .filter(|w| matches!(w.employment_type.as_str(), "contractor" | "fixed_term"))
        .collect();
    let contingent_pids: Vec<uuid::Uuid> = contingent_staff.iter().map(|w| w.pid).collect();
    let decisions = super::find_in(
        ctx,
        engagement_decisions::Entity::find(),
        engagement_decisions::Column::WorkerPid,
        &contingent_pids,
    )
    .await?;
    let extensions = super::find_in(
        ctx,
        engagement_extensions::Entity::find(),
        engagement_extensions::Column::WorkerPid,
        &contingent_pids,
    )
    .await?;
    Ok(contingent_staff
        .into_iter()
        .map(|w| {
            let engagement = ContingentEngagement {
                contractor: w.employment_type == "contractor",
                hired_on: w.hired_on,
                ends_on: w.engagement_ends_on,
                decided: w.engagement_ends_on.is_some_and(|end| {
                    decisions
                        .iter()
                        .any(|d| d.worker_pid == w.pid && d.ends_on == end)
                }),
                extensions: extensions.iter().filter(|x| x.worker_pid == w.pid).count(),
            };
            (w, engagement)
        })
        .collect())
}

/// The engagement figures of the contingent view (WPM-R86) for the people employed today.
async fn engagement_counts(
    ctx: &AppContext,
    staff: &[workers::Model],
    today: chrono::NaiveDate,
    floor: usize,
) -> Result<ContingentCounts> {
    let engagements: Vec<ContingentEngagement> = engagements_of(ctx, staff)
        .await?
        .into_iter()
        .map(|(_, e)| e)
        .collect();
    Ok(contingent_counts(&engagements, today, floor))
}

/// Query for the named list.
#[derive(Debug, Deserialize)]
struct ListQuery {
    list: Option<String>,
}

/// `GET /api/engagements/contingent-list?list=` — the people behind one figure of the contingent
/// view (WPM-R86), by worker number, for HR to act on. The tokens are `ending_within_30`,
/// `ending_31_to_60`, `ending_61_to_90`, `past_end_undecided`, `much_extended` and
/// `long_running_contractors`. HR only, **audited**, and no floor: HR is entitled to know who.
/// Shows no pay and no name, only the worker number, department, basis and dates.
#[debug_handler]
async fn contingent_list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    let list = query.list.unwrap_or_default();
    if !LISTS.contains(&list.as_str()) {
        return Err(unprocessable(&format!(
            "list must be one of {}",
            LISTS.join(", ")
        )));
    }
    let today = chrono::Utc::now().date_naive();
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let staff = employed_in_scope(&ctx, scope.as_deref(), today).await?;
    let mut found: Vec<_> = engagements_of(&ctx, &staff)
        .await?
        .into_iter()
        .filter(|(_, e)| on_list(&list, e, today) == Some(true))
        .collect();
    found.sort_by(|a, b| a.0.worker_number.cmp(&b.0.worker_number));
    Audit::record(
        &ctx.db,
        "contingent_workforce",
        uuid::Uuid::nil(),
        "contingent_list_read",
        caller.actor(),
        None,
    )
    .await?;
    format::json(json!({
        "list": list,
        "people": found.iter().map(|(w, e)| json!({
            "worker_pid": w.pid,
            "worker_number": w.worker_number,
            "department": w.department,
            "employment_type": w.employment_type,
            "hired_on": w.hired_on,
            "engagement_ends_on": w.engagement_ends_on,
            "extensions": e.extensions,
            "decision_recorded": e.decided,
        })).collect::<Vec<_>>(),
    }))
}

/// The trend of the contingent share from the headcount snapshots in the caller's scope.
async fn snapshot_trend(
    ctx: &AppContext,
    scope: Option<&[String]>,
    floor: usize,
) -> Result<rules::ContingentTrend> {
    let mut select = headcount_snapshots::Entity::find();
    if let Some(refs) = scope {
        select = select.filter(headcount_snapshots::Column::OrganizationRef.is_in(refs.to_vec()));
    }
    let rows: Vec<rules::SplitRow> = select
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|r| rules::SplitRow {
            as_of: r.as_of,
            headcount: i64::from(r.headcount),
            fte: r.fte_percent_total,
            contractor_fte: r.contractor_fte_percent_total,
            fixed_term_fte: r.fixed_term_fte_percent_total,
        })
        .collect();
    Ok(rules::contingent_trend(&rows, floor))
}

/// `GET /api/workforce-intelligence/contingent` — the share of FTE that is contractor and that is
/// fixed-term (each apart), and the funding horizon: who is left when each month's money runs out
/// (WPM-R62). Aggregate only, names no one, small groups withheld. HR only.
#[debug_handler]
#[allow(clippy::too_many_lines)] // loads, groups, then one JSON body, linearly
async fn contingent(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let today = chrono::Utc::now().date_naive();
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    // Grouped by the database: department, basis and FTE with the number of people who share them.
    let groups: Vec<(String, String, i32, usize)> =
        employed_in_scope_query(scope.as_deref(), today)
            .select_only()
            .column(workers::Column::Department)
            .column(workers::Column::EmploymentType)
            .column(workers::Column::FtePercent)
            .column_as(
                sea_orm::sea_query::Expr::col(workers::Column::Id).count(),
                "people",
            )
            .group_by(workers::Column::Department)
            .group_by(workers::Column::EmploymentType)
            .group_by(workers::Column::FtePercent)
            .into_tuple::<(String, String, i32, i64)>()
            .all(&ctx.db)
            .await?
            .into_iter()
            .map(|(department, basis, fte, people)| {
                (department, basis, fte, usize::try_from(people).unwrap_or(0))
            })
            .collect();
    let floor = DEFAULT_FLOOR;
    let share = rules::contingent_share_counted(&groups, floor);
    // Only the contractors and fixed-term people are read whole: the engagement figures are about them.
    let contingent_staff: Vec<workers::Model> = employed_in_scope_query(scope.as_deref(), today)
        .filter(workers::Column::EmploymentType.is_in(["contractor", "fixed_term"]))
        .all(&ctx.db)
        .await?;
    // The funding ends of employed people, with their FTE.
    let funded = post_fundings::Entity::find()
        .filter(post_fundings::Column::EndsOn.is_not_null())
        .all(&ctx.db)
        .await?;
    let funded_pids: Vec<uuid::Uuid> = funded
        .iter()
        .map(|f| f.worker_pid)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let employed = employed_among(&ctx, scope.as_deref(), today, &funded_pids).await?;
    let ends: Vec<(chrono::NaiveDate, i32)> = funded
        .into_iter()
        .filter_map(|f| Some((f.ends_on?, employed.get(&f.worker_pid)?.fte_percent)))
        .collect();
    let horizon = rules::funding_horizon(&ends, today, HORIZON_MONTHS, floor);
    let counts = engagement_counts(&ctx, &contingent_staff, today, floor).await?;
    let trend = snapshot_trend(&ctx, scope.as_deref(), floor).await?;
    Audit::record(
        &ctx.db,
        "contingent_workforce",
        uuid::Uuid::nil(),
        "contingent_workforce_read",
        caller.actor(),
        None,
    )
    .await?;
    format::json(json!({
        "small_group_floor": floor,
        "headcount": share.headcount,
        "fte": share.fte,
        "contractor_percent_of_fte": share.contractor_percent,
        "fixed_term_percent_of_fte": share.fixed_term_percent,
        "departments_shown": share.departments_shown,
        "by_department": share.departments.iter().map(|d| json!({
            "department": d.department,
            "headcount": d.headcount,
            "contractor_percent_of_fte": d.contractor_percent,
            "fixed_term_percent_of_fte": d.fixed_term_percent,
        })).collect::<Vec<_>>(),
        "funding_ends": horizon.iter().map(|m| json!({
            "month": m.month, "people": m.people, "fte": m.fte,
        })).collect::<Vec<_>>(),
        "trend": {
            "status": trend.status,
            "points": trend.points.iter().map(|p| json!({
                "as_of": p.as_of,
                "contractor_percent_of_fte": p.contractor_percent,
                "fixed_term_percent_of_fte": p.fixed_term_percent,
            })).collect::<Vec<_>>(),
            "dates_left_out_without_split": trend.dates_without_split,
            "dates_left_out_below_floor": trend.dates_below_floor,
        },
        "engagements": {
            "ending_within_30_calendar_days": counts.ending_within_30,
            "ending_31_to_60_calendar_days": counts.ending_31_to_60,
            "ending_61_to_90_calendar_days": counts.ending_61_to_90,
            "past_end_with_no_decision": counts.past_end_undecided,
            "extended_more_than": { "times": EXTENSION_THRESHOLD, "people": counts.much_extended },
            "contractors_engaged_longer_than": {
                "calendar_months": LONG_RUNNING_MONTHS,
                "people": counts.long_running_contractors,
            },
            "note": "The ending figures are separate bands, not running totals, and no total is given, so a \
                     withheld one cannot be worked out. A count of one to nine is withheld; zero is shown. \
                     An engagement with no end date is in none of the ending figures. The two thresholds are \
                     fixed in the code, not yet configurable.",
        },
        "derivation": "Contractor and fixed-term are each a share of the FTE of the people employed today, so \
                       part-time work counts for what it is. The department breakdown shows only when every \
                       department reaches the floor, so a withheld one cannot be worked out from the others. \
                       Funding ends counts people employed today whose recorded funding ends in that month, \
                       or already has; a month with some but fewer than the floor is withheld, and no total is \
                       given, so it cannot be worked out. A worker with no funding recorded is not counted. \
                       The trend uses only snapshots recorded with the split by basis; older snapshots are unknown, not zero, and with fewer than three usable dates over at least 60 calendar days it says insufficient_history.",
    }))
}

/// The funding routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/funding", get(get_funding))
        .add("/workers/{pid}/funding", put(set_funding))
        .add("/workers/{pid}/funding", delete(clear_funding))
        .add("/workforce-intelligence/contingent", get(contingent))
        .add("/engagements/contingent-list", get(contingent_list))
}
