//! Assignment category vocabularies (WPM-R162, WPM-D116): the codes an organization uses for site, area of work, job role, staff
//! group and occupation code, with the national standard each comes from. Configuration, imported by the deployer; none is shipped.
//! Reading is open to a signed-in caller; adding is HR's. Spec: `spec/staff-records-and-pay.md`.

use axa::extract::Json;
use axa::prelude::*;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, forbidden, row_json, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db::{self, pool};
use crate::models::_entities::category_vocabulary;
use crate::rules::assignment as rules;

/// The staff group the vocabulary gives a job role: `None` where the role is not listed or says none.
pub(crate) async fn staff_group_of_role(
    ctx: &AppContext,
    organization: &str,
    role: &str,
) -> Result<Option<String>> {
    Ok(sqlx::query_scalar!(
        "SELECT staff_group FROM category_vocabulary
         WHERE organization_ref = $1 AND kind = 'job_role' AND code = $2 LIMIT 1",
        organization,
        role,
    )
    .fetch_optional(pool(ctx))
    .await?
    .flatten())
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
    axa::extract::Query(q): axa::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&q.organization))?;
    ensure_org_in_scope(&ctx, &caller, &q.organization).await?;
    if let Some(kind) = &q.kind {
        ensure_valid(&rules::entry_problems(kind, "x", "x", None, None, None))?;
    }
    let rows = sqlx::query_as!(
        category_vocabulary::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, kind, code, label,
                  standard, standard_code, staff_group, recorded_by
           FROM category_vocabulary
           WHERE organization_ref = $1 AND ($2::text IS NULL OR kind = $2)
           ORDER BY kind, code"#,
        q.organization,
        q.kind,
    )
    .fetch_all(pool(&ctx))
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
    let clash = sqlx::query_scalar!(
        r#"SELECT EXISTS (
               SELECT 1 FROM category_vocabulary WHERE organization_ref = $1 AND kind = $2 AND code = $3
           ) AS "clash!""#,
        p.organization_ref,
        p.kind,
        code,
    )
    .fetch_one(pool(&ctx))
    .await?;
    if clash {
        return Err(unprocessable("that code is already in this vocabulary"));
    }
    let trim = |v: Option<String>| v.map(|s| s.trim().to_string());
    let pid = Uuid::new_v4();
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO category_vocabulary
             (pid, organization_ref, kind, code, label, standard, standard_code, staff_group, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        pid,
        p.organization_ref,
        p.kind,
        code,
        p.label.trim(),
        trim(p.standard),
        trim(p.standard_code),
        trim(p.staff_group),
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "category_vocabulary",
        pid,
        "vocabulary_entry_added",
        caller.actor(),
        Some(json!({ "kind": p.kind })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/category-vocabulary", get(list))
        .add("/category-vocabulary", post(create))
}
