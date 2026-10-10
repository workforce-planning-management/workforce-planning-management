//! **Equality and diversity monitoring** (WPM-R125, WPM-D74): voluntary, self-declared protected
//! characteristics, held to monitor the equality of a workforce. Pure rules in
//! [`crate::rules::equality_monitoring`].
//!
//! This is the one deliberate exception to "WPM holds no demographics", and it is bounded:
//!
//! - **Off** until a deployer records the lawful basis they rely on
//!   (`WPM_EQUALITY_MONITORING_BASIS`) and defines the categories (`WPM_EQUALITY_CATEGORIES`).
//!   WPM ships no classification. Until then every route answers that monitoring is off.
//! - **The worker only** reads or changes their own declarations (`/api/me/equality-monitoring`).
//!   No manager, no HR, no payroll and no administrator can read an individual's.
//! - The **only output** is an aggregate with small groups withheld
//!   (`/api/equality-monitoring/summary`, privileged callers), and a completeness figure that names no
//!   value. No audit entry carries a value.
//! - **Never an input to any score, ranking or decision about a person**: a test fails if any other
//!   code names the table.

use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;

use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{equality_declarations, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::memberships;
use crate::rules::equality_monitoring::{self as rules, Config, Gate};

/// The gate, read from the environment each time so a change of setting needs no restart of a
/// test, and a misconfiguration is reported rather than silently ignored.
///
/// # Errors
///
/// A basis recorded with unusable categories or floor.
pub fn gate_from_env() -> std::result::Result<Gate, String> {
    let read = |name: &str| crate::compat::env_var(name);
    rules::gate(
        read("WPM_EQUALITY_MONITORING_BASIS").as_deref(),
        read("WPM_EQUALITY_CATEGORIES").as_deref(),
        read("WPM_EQUALITY_FLOOR").as_deref(),
    )
}

/// The gate for a request: the settings if monitoring is on, else `404`, because the feature does
/// not exist on a deployment that has not recorded a basis.
fn on() -> Result<(String, Config, usize)> {
    match gate_from_env() {
        Ok(Gate::On {
            basis,
            config,
            floor,
        }) => Ok((basis, config, floor)),
        Ok(Gate::Off) => Err(Error::NotFound),
        Err(message) => Err(Error::string(&message)),
    }
}

async fn own_declarations(
    ctx: &AppContext,
    worker: &workers::Model,
) -> Result<BTreeMap<String, String>> {
    Ok(equality_declarations::Entity::find()
        .filter(equality_declarations::Column::WorkerPid.eq(worker.pid))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|d| (d.category, d.value))
        .collect())
}

/// What the worker sees. When monitoring is off they are told so and nothing else.
pub(crate) async fn view_for(ctx: &AppContext, worker: &workers::Model) -> Result<Response> {
    match gate_from_env() {
        Ok(Gate::Off) => format::json(json!({ "enabled": false })),
        Ok(Gate::On {
            basis,
            config,
            floor,
        }) => format::json(json!({
            "enabled": true,
            "lawful_basis": basis,
            "categories": config.offered(),
            "declarations": own_declarations(ctx, worker).await?,
            "small_group_floor": floor,
            "notice": "This is voluntary. Only you can see your answers. They are used only to count, by group, \
                       and a group smaller than the floor is never shown. They are never used to make a \
                       decision about you. You can change an answer or take them all back at any time.",
        })),
        Err(message) => Err(Error::string(&message)),
    }
}

/// `PUT /api/me/equality-monitoring` body.
#[derive(Debug, Deserialize)]
pub(crate) struct DeclarePayload {
    /// Category to value. Only the categories given are changed.
    declarations: BTreeMap<String, String>,
}

/// Record or change the worker's own declarations. Shared with `/api/me/equality-monitoring`.
pub(crate) async fn declare_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
    payload: DeclarePayload,
) -> Result<Response> {
    let (_, config, _) = on()?;
    if payload.declarations.is_empty() {
        return Err(unprocessable("give at least one category to declare"));
    }
    for (category, value) in &payload.declarations {
        config
            .validate(category, value)
            .map_err(|e| unprocessable(&e))?;
    }
    for (category, value) in &payload.declarations {
        let existing = equality_declarations::Entity::find()
            .filter(equality_declarations::Column::WorkerPid.eq(worker.pid))
            .filter(equality_declarations::Column::Category.eq(category))
            .one(&ctx.db)
            .await?;
        if let Some(row) = existing {
            let mut active: equality_declarations::ActiveModel = row.into();
            active.value = ActiveValue::set(value.clone());
            active.update(&ctx.db).await?;
        } else {
            equality_declarations::ActiveModel {
                worker_pid: ActiveValue::set(worker.pid),
                category: ActiveValue::set(category.clone()),
                value: ActiveValue::set(value.clone()),
                ..Default::default()
            }
            .insert(&ctx.db)
            .await?;
        }
    }
    // The audit entry says that declarations changed, never which categories or what values.
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "equality_monitoring_declared",
        caller.actor(),
        None,
    )
    .await?;
    view_for(ctx, worker).await
}

/// Withdraw one category, or all of them (`category` of `None`). Shared with `/api/me`.
pub(crate) async fn withdraw_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
    category: Option<&str>,
) -> Result<Response> {
    on()?;
    let mut delete = equality_declarations::Entity::delete_many()
        .filter(equality_declarations::Column::WorkerPid.eq(worker.pid));
    if let Some(c) = category {
        delete = delete.filter(equality_declarations::Column::Category.eq(c));
    }
    let removed = delete.exec(&ctx.db).await?.rows_affected;
    if removed == 0 {
        return Err(Error::NotFound);
    }
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "equality_monitoring_withdrawn",
        caller.actor(),
        None,
    )
    .await?;
    view_for(ctx, worker).await
}

/// Query for the summary.
#[derive(Debug, Deserialize)]
struct SummaryQuery {
    category: String,
}

/// `GET /api/equality-monitoring/summary?category=` — one category across the organizations the
/// caller belongs to: completeness (no value named), the organization's counts with small cells
/// withheld, and a department breakdown only when every department and every cell reaches the
/// floor. Privileged callers only. The read is audited.
#[debug_handler]
async fn summary(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<SummaryQuery>,
) -> Result<Response> {
    let (_, config, floor) = on()?;
    if !config.has_category(&query.category) {
        return Err(unprocessable(
            "that category is not monitored on this deployment",
        ));
    }
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let today = Utc::now().date_naive();
    let staff: Vec<workers::Model> = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .order_by_asc(workers::Column::Id)
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| crate::rules::metrics::is_employed_on(today, w.hired_on, w.terminated_on))
        .filter(|w| {
            scope
                .as_ref()
                .is_none_or(|refs| refs.iter().any(|r| r == &w.organization_ref))
        })
        .collect();
    let mut headcount: BTreeMap<String, usize> = BTreeMap::new();
    let mut department_of: BTreeMap<uuid::Uuid, String> = BTreeMap::new();
    for w in &staff {
        *headcount.entry(w.department.clone()).or_insert(0) += 1;
        department_of.insert(w.pid, w.department.clone());
    }
    let answers = equality_declarations::Entity::find()
        .filter(equality_declarations::Column::Category.eq(&query.category))
        .all(&ctx.db)
        .await?;
    let rows: Vec<(String, String)> = answers
        .into_iter()
        .filter_map(|a| Some((department_of.get(&a.worker_pid)?.clone(), a.value)))
        .collect();
    let result = rules::summarise(&rows, &headcount, floor);
    Audit::record(
        &ctx.db,
        "equality_monitoring",
        uuid::Uuid::nil(),
        "equality_monitoring_summary_read",
        caller.actor(),
        None,
    )
    .await?;
    format::json(json!({
        "category": query.category,
        "small_group_floor": floor,
        "headcount": result.headcount,
        "responded": result.responded,
        "completeness_percent": result.completeness_percent,
        "organization": result.organization,
        "by_department": result.by_department,
        "derivation": "Counts of people who answered, by value. A value shows only when it reaches the \
                       floor, nothing shows until the answers reach it, and the breakdown by department \
                       shows only when every department and every value in it reaches it, so a withheld \
                       figure cannot be worked out from the others. Completeness is the share of people \
                       employed who answered, and names no value.",
    }))
}

/// The routes. The worker's own are in [`super::me`].
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/equality-monitoring/summary", get(summary))
}
