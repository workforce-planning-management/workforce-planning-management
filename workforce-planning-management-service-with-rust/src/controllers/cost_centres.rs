//! Cost centres (WPM-R163, WPM-D116): an organization's register of codes, each with a name, an optional parent and the days it is
//! valid. Closed, never deleted. A write that names a code not open on its effective day is a `422`. The free-text
//! `positions.cost_centre` values that match nothing in the register are listed for HR, never matched by a guess or created silently.
//! Reading the register is open to a signed-in caller in scope; keeping it is HR's. Spec: `spec/staff-records-and-pay.md`.

use std::collections::BTreeMap;

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, forbidden, row_json, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{cost_centres, positions};
use crate::models::audit_logs::Model as Audit;
use crate::rules::coding as rules;

fn as_rule(c: &cost_centres::Model) -> rules::CostCentre {
    rules::CostCentre {
        code: c.code.clone(),
        parent: c.parent_code.clone(),
        valid_from: c.valid_from,
        valid_to: c.valid_to,
    }
}

async fn register(ctx: &AppContext, organization: &str) -> Result<Vec<cost_centres::Model>> {
    Ok(cost_centres::Entity::find()
        .filter(cost_centres::Column::OrganizationRef.eq(organization))
        .order_by_asc(cost_centres::Column::Code)
        .all(&ctx.db)
        .await?)
}

/// A `422` when `code` is not in the organization's register or not open on `on` (WPM-R163).
pub(crate) async fn ensure_open(
    ctx: &AppContext,
    organization: &str,
    code: &str,
    on: NaiveDate,
) -> Result<()> {
    let rows: Vec<_> = register(ctx, organization)
        .await?
        .iter()
        .map(as_rule)
        .collect();
    rules::check_open(&rows, code, on).map_err(|m| unprocessable(&m))
}

#[derive(Debug, Deserialize)]
struct OrgQuery {
    organization: String,
}

/// `GET /api/cost-centres?organization=` — the register, each with whether it is open today and its parent chain.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(q): axum::extract::Query<OrgQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&q.organization))?;
    ensure_org_in_scope(&ctx, &caller, &q.organization).await?;
    let rows = register(&ctx, &q.organization).await?;
    let all: Vec<_> = rows.iter().map(as_rule).collect();
    let today = Utc::now().date_naive();
    let mut out = Vec::new();
    for (row, rule) in rows.iter().zip(&all) {
        let mut v = row_json(row)?;
        if let Some(o) = v.as_object_mut() {
            o.remove("recorded_by");
            o.insert("open".into(), json!(rule.open_on(today)));
            o.insert("chain".into(), json!(rules::chain(&all, &row.code)));
        }
        out.push(v);
    }
    format::json(json!({ "cost_centres": out }))
}

#[derive(Debug, Deserialize)]
struct CreatePayload {
    organization_ref: String,
    code: String,
    name: String,
    parent_code: Option<String>,
    valid_from: Option<NaiveDate>,
}

async fn hr_only(ctx: &AppContext, caller: &MaybeAuthUser, organization: &str) -> Result<()> {
    ensure_valid(&organization_problems(organization))?;
    ensure_org_in_scope(ctx, caller, organization).await?;
    if !super::policies::privileged(ctx, caller, organization).await? {
        return Err(forbidden("the cost centre register is kept by HR"));
    }
    Ok(())
}

/// `POST /api/cost-centres` — add a cost centre. The code is unique in the organization; a parent must be in the register and open
/// on the first day. HR.
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<CreatePayload>,
) -> Result<Response> {
    hr_only(&ctx, &caller, &p.organization_ref).await?;
    let code = p.code.trim().to_string();
    let parent = p
        .parent_code
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let from = p.valid_from.unwrap_or_else(|| Utc::now().date_naive());
    let mut problems = Vec::new();
    if !(1..=20).contains(&code.chars().count()) {
        problems.push("code must be 1 to 20 characters".to_string());
    }
    if !(1..=120).contains(&p.name.trim().chars().count()) {
        problems.push("name must be 1 to 120 characters".to_string());
    }
    ensure_valid(&problems)?;
    let rows = register(&ctx, &p.organization_ref).await?;
    if rows.iter().any(|c| c.code == code) {
        return Err(unprocessable("that code is already in the register"));
    }
    if let Some(parent) = &parent {
        let all: Vec<_> = rows.iter().map(as_rule).collect();
        rules::check_open(&all, parent, from).map_err(|m| unprocessable(&m))?;
    }
    let pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    cost_centres::ActiveModel {
        pid: ActiveValue::set(pid),
        organization_ref: ActiveValue::set(p.organization_ref),
        code: ActiveValue::set(code),
        name: ActiveValue::set(p.name.trim().to_string()),
        parent_code: ActiveValue::set(parent),
        valid_from: ActiveValue::set(from),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "cost_centre",
        pid,
        "cost_centre_added",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

#[derive(Debug, Deserialize)]
struct ClosePayload {
    valid_to: NaiveDate,
}

/// `POST /api/cost-centres/{pid}/close` — close a cost centre on a day not before it opened. It is never deleted, and a closed one
/// is not reopened here. HR.
#[debug_handler]
async fn close(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(p): Json<ClosePayload>,
) -> Result<Response> {
    let row = cost_centres::Entity::find()
        .filter(cost_centres::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    hr_only(&ctx, &caller, &row.organization_ref).await?;
    if row.valid_to.is_some() {
        return Err(unprocessable("this cost centre is already closed"));
    }
    if p.valid_to < row.valid_from {
        return Err(unprocessable(
            "valid_to must not be before the cost centre opened",
        ));
    }
    let txn = ctx.db.begin().await?;
    let mut active: cost_centres::ActiveModel = row.into();
    active.valid_to = ActiveValue::set(Some(p.valid_to));
    active.update(&txn).await?;
    Audit::record(
        &txn,
        "cost_centre",
        pid,
        "cost_centre_closed",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid, "valid_to": p.valid_to }))
}

/// `GET /api/cost-centres/unmatched?organization=` — the free-text `positions.cost_centre` values that are not a code in the register,
/// each with how many positions carry it. HR resolves them; nothing is matched or created here.
#[debug_handler]
async fn unmatched(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(q): axum::extract::Query<OrgQuery>,
) -> Result<Response> {
    hr_only(&ctx, &caller, &q.organization).await?;
    let codes: Vec<String> = register(&ctx, &q.organization)
        .await?
        .into_iter()
        .map(|c| c.code)
        .collect();
    let mut by: BTreeMap<String, usize> = BTreeMap::new();
    for p in positions::Entity::find()
        .filter(positions::Column::OrganizationRef.eq(&q.organization))
        .all(&ctx.db)
        .await?
    {
        if !codes.contains(&p.cost_centre) {
            *by.entry(p.cost_centre).or_default() += 1;
        }
    }
    let values: Vec<_> = by
        .into_iter()
        .map(|(value, positions)| json!({ "value": value, "positions": positions }))
        .collect();
    format::json(json!({ "organization": q.organization, "unmatched": values }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/cost-centres", get(list))
        .add("/cost-centres", post(create))
        .add("/cost-centres/unmatched", get(unmatched))
        .add("/cost-centres/{pid}/close", post(close))
}
