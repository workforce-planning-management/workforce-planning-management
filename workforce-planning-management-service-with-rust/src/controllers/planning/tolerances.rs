//! Tolerances (part of [`super`]; WPM-R140, WPM-D93): the largest deviation a plan may have in time,
//! cost, resources and risk, and where it stands now, as red, amber or green.
//!
//! The deviations are read off the plan's own forecast, cost and alignment views, so there is one
//! definition of each figure. Nothing here decides or escalates: it measures and says what it rests on.

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;
use crate::models::_entities::plan_tolerances;
use crate::rules::tolerance as tol;

/// `PUT /api/workforce-plans/{pid}/tolerances/{dimension}` body.
#[derive(Debug, Deserialize)]
pub(super) struct TolerancePayload {
    /// The largest deviation allowed, in the dimension's unit.
    limit: i64,
    /// The share of the limit at which green turns amber; default 75.
    #[serde(default)]
    amber_percent: Option<i32>,
}

/// What each dimension's deviation is, to show beside it.
fn basis(dimension: tol::Dimension) -> &'static str {
    match dimension {
        tol::Dimension::Time => {
            "the most calendar days any demand line with a headcount gap is past its target date; 0 when none is"
        }
        tol::Dimension::Cost => {
            "the projected annual cost of closing the gaps above the plan's budget, in basis points of the budget; 0 when within it"
        }
        tol::Dimension::Resources => {
            "the largest headcount gap on any demand line, in basis points of that line's planned demand"
        }
        tol::Dimension::Risk => {
            "the critical roles in the plan's departments with no ready successor (the shared single-point-of-failure rule)"
        }
    }
}

/// Why a deviation could not be worked out.
fn reason(dimension: tol::Dimension) -> &'static str {
    match dimension {
        tol::Dimension::Time | tol::Dimension::Resources => {
            "no demand line has a known gap: the plan needs an attrition assumption and a demand line with planned headcount"
        }
        tol::Dimension::Cost => {
            "cost needs a budget in the currency of the costing, salaries you may see, a known gap, and a unit cost for every group (or an overspend already)"
        }
        tol::Dimension::Risk => "there is no succession plan in the plan's departments to look at",
    }
}

/// The forecast's departments as the lines the tolerance rules read.
fn lines_of(forecast: &serde_json::Value) -> Vec<tol::Line> {
    forecast["departments"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|d| {
            Some(tol::Line {
                target_on: d["target_on"].as_str()?.parse().ok()?,
                planned: d["planned_demand"].as_i64()?,
                gap: d["headcount_gap"].as_i64(),
            })
        })
        .collect()
}

/// `GET /api/workforce-plans/{pid}/tolerances` — every dimension with its deviation, limit and status.
#[debug_handler]
pub(super) async fn list_tolerances(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    let set: BTreeMap<String, plan_tolerances::Model> = plan_tolerances::Entity::find()
        .filter(plan_tolerances::Column::PlanPid.eq(plan.pid))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|t| (t.dimension.clone(), t))
        .collect();
    let today = chrono::Utc::now().date_naive();
    let forecast = forecast_value(&ctx, &caller, &pid).await?;
    let cost = cost_value(&ctx, &caller, &pid, CostQuery::default()).await?;
    let alignment = alignment_value(&ctx, &caller, &pid).await?;
    let lines = lines_of(&forecast);
    let bench = &alignment["critical_roles_without_bench"];
    let usize_of = |v: &serde_json::Value| usize::try_from(v.as_u64().unwrap_or(0)).unwrap_or(0);

    let mut statuses = Vec::new();
    let dimensions: Vec<serde_json::Value> = tol::Dimension::ALL
        .into_iter()
        .map(|dimension| {
            let deviation = match dimension {
                tol::Dimension::Time => tol::time_deviation(&lines, today),
                tol::Dimension::Resources => tol::resources_deviation(&lines),
                tol::Dimension::Cost => tol::cost_deviation(
                    cost["total_annual_cost_minor"].as_i64(),
                    cost["affordability"]["budget_minor"].as_i64(),
                    cost["uncosted_groups"].as_u64() == Some(0),
                ),
                tol::Dimension::Risk => tol::risk_deviation(
                    usize_of(&bench["succession_plans_in_plan_departments"]),
                    usize_of(&bench["in_plan_departments"]),
                ),
            };
            let row = set.get(dimension.as_str());
            let rag = row.map_or(tol::Rag::NotSet, |t| {
                tol::assess(deviation, t.limit_value, t.amber_percent)
            });
            if row.is_some() {
                statuses.push(rag);
            }
            serde_json::json!({
                "dimension": dimension,
                "unit": dimension.unit(),
                "set": row.is_some(),
                "limit": row.map(|t| t.limit_value),
                "amber_percent": row.map(|t| t.amber_percent),
                "deviation": deviation,
                "rag": rag,
                "basis": basis(dimension),
                "reason": if deviation.is_none() { Some(reason(dimension)) } else { None },
            })
        })
        .collect();
    format::json(serde_json::json!({
        "derivation": "a tolerance is the largest deviation the plan may have; green is under amber_percent of it, amber is amber_percent of it up to and including the limit, red is over the limit. \
                       A deviation that cannot be worked out is unknown, never green. The overall status is the worst of the dimensions that have a tolerance. \
                       Nothing is escalated or blocked: this measures and shows its working.",
        "plan": { "pid": plan.pid, "name": plan.name, "status": plan.status },
        "as_of": today,
        "overall": tol::overall(&statuses),
        "dimensions": dimensions,
    }))
}

/// Read the `{dimension}` path segment.
fn dimension_of(text: &str) -> Result<tol::Dimension> {
    tol::Dimension::parse(text)
        .ok_or_else(|| unprocessable("dimension must be one of time, cost, resources, risk"))
}

/// `PUT /api/workforce-plans/{pid}/tolerances/{dimension}` — set or change a tolerance.
#[debug_handler]
pub(super) async fn set_tolerance(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, dimension)): Path<(String, String)>,
    Json(payload): Json<TolerancePayload>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    let dimension = dimension_of(&dimension)?;
    let amber = payload.amber_percent.unwrap_or(tol::DEFAULT_AMBER_PERCENT);
    tol::validate(payload.limit, amber).map_err(|e| unprocessable(&e))?;
    let existing = plan_tolerances::Entity::find()
        .filter(plan_tolerances::Column::PlanPid.eq(plan.pid))
        .filter(plan_tolerances::Column::Dimension.eq(dimension.as_str()))
        .one(&ctx.db)
        .await?;
    let verb = if let Some(row) = existing {
        let mut active: plan_tolerances::ActiveModel = row.into();
        active.limit_value = ActiveValue::set(payload.limit);
        active.amber_percent = ActiveValue::set(amber);
        active.update(&ctx.db).await?;
        "tolerance_changed"
    } else {
        plan_tolerances::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            plan_pid: ActiveValue::set(plan.pid),
            dimension: ActiveValue::set(dimension.as_str().to_string()),
            limit_value: ActiveValue::set(payload.limit),
            amber_percent: ActiveValue::set(amber),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?;
        "tolerance_set"
    };
    Audit::record(
        &ctx.db,
        "workforce_plan",
        plan.pid,
        verb,
        caller.actor(),
        Some(serde_json::json!({
            "dimension": dimension, "limit": payload.limit, "amber_percent": amber,
        })),
    )
    .await?;
    list_tolerances(State(ctx), caller, Path(pid)).await
}

/// `DELETE /api/workforce-plans/{pid}/tolerances/{dimension}` — take a tolerance away.
#[debug_handler]
pub(super) async fn delete_tolerance(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, dimension)): Path<(String, String)>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    let dimension = dimension_of(&dimension)?;
    let removed = plan_tolerances::Entity::delete_many()
        .filter(plan_tolerances::Column::PlanPid.eq(plan.pid))
        .filter(plan_tolerances::Column::Dimension.eq(dimension.as_str()))
        .exec(&ctx.db)
        .await?;
    if removed.rows_affected == 0 {
        return Err(Error::NotFound);
    }
    Audit::record(
        &ctx.db,
        "workforce_plan",
        plan.pid,
        "tolerance_removed",
        caller.actor(),
        Some(serde_json::json!({ "dimension": dimension })),
    )
    .await?;
    format::empty_json()
}
