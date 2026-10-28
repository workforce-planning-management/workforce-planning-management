//! Assignment category vocabularies (WPM-R162, WPM-D116): the codes an organization uses for site, area of work, job role, staff
//! group and occupation code, with the national standard each comes from. Configuration, imported by the deployer; none is shipped.
//! Reading is open to a signed-in caller; adding is HR's. Spec: `spec/staff-records-and-pay.md`.

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, forbidden, row_json, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::category_vocabulary;
use crate::models::audit_logs::Model as Audit;
use crate::rules::assignment as rules;

/// The staff group the vocabulary gives a job role: `None` where the role is not listed or says none.
pub(crate) async fn staff_group_of_role(
    ctx: &AppContext,
    organization: &str,
    role: &str,
) -> Result<Option<String>> {
    Ok(category_vocabulary::Entity::find()
        .filter(category_vocabulary::Column::OrganizationRef.eq(organization))
        .filter(category_vocabulary::Column::Kind.eq("job_role"))
        .filter(category_vocabulary::Column::Code.eq(role))
        .one(&ctx.db)
        .await?
        .and_then(|e| e.staff_group))
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    organization: String,
    kind: Option<String>,
}

/// `GET /api/category-vocabulary?organization=&kind=` — the entries, by kind then code.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(q): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&q.organization))?;
    ensure_org_in_scope(&ctx, &caller, &q.organization).await?;
    let mut find = category_vocabulary::Entity::find()
        .filter(category_vocabulary::Column::OrganizationRef.eq(&q.organization));
    if let Some(kind) = &q.kind {
        ensure_valid(&rules::entry_problems(kind, "x", "x", None, None, None))?;
        find = find.filter(category_vocabulary::Column::Kind.eq(kind));
    }
    let rows = find
        .order_by_asc(category_vocabulary::Column::Kind)
        .order_by_asc(category_vocabulary::Column::Code)
        .all(&ctx.db)
        .await?;
    let mut out = Vec::new();
    for row in &rows {
        let mut v = row_json(row)?;
        if let Some(o) = v.as_object_mut() {
            o.remove("recorded_by");
        }
        out.push(v);
    }
    format::json(json!({ "entries": out }))
}

#[derive(Debug, Deserialize)]
struct EntryPayload {
    organization_ref: String,
    kind: String,
    code: String,
    label: String,
    standard: Option<String>,
    standard_code: Option<String>,
    staff_group: Option<String>,
}

/// `POST /api/category-vocabulary` — add an entry. A code is unique within an organization and kind. HR.
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<EntryPayload>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&p.organization_ref))?;
    ensure_org_in_scope(&ctx, &caller, &p.organization_ref).await?;
    if !super::policies::privileged(&ctx, &caller, &p.organization_ref).await? {
        return Err(forbidden("the vocabularies are kept by HR"));
    }
    ensure_valid(&rules::entry_problems(
        &p.kind,
        &p.code,
        &p.label,
        p.standard.as_deref(),
        p.standard_code.as_deref(),
        p.staff_group.as_deref(),
    ))?;
    let code = p.code.trim().to_string();
    let clash = category_vocabulary::Entity::find()
        .filter(category_vocabulary::Column::OrganizationRef.eq(&p.organization_ref))
        .filter(category_vocabulary::Column::Kind.eq(&p.kind))
        .filter(category_vocabulary::Column::Code.eq(&code))
        .count(&ctx.db)
        .await?;
    if clash > 0 {
        return Err(unprocessable("that code is already in this vocabulary"));
    }
    let trim = |v: Option<String>| v.map(|s| s.trim().to_string());
    let pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    category_vocabulary::ActiveModel {
        pid: ActiveValue::set(pid),
        organization_ref: ActiveValue::set(p.organization_ref),
        kind: ActiveValue::set(p.kind.clone()),
        code: ActiveValue::set(code),
        label: ActiveValue::set(p.label.trim().to_string()),
        standard: ActiveValue::set(trim(p.standard)),
        standard_code: ActiveValue::set(trim(p.standard_code)),
        staff_group: ActiveValue::set(trim(p.staff_group)),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "category_vocabulary",
        pid,
        "vocabulary_entry_added",
        caller.actor(),
        Some(json!({ "kind": p.kind })),
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/category-vocabulary", get(list))
        .add("/category-vocabulary", post(create))
}
