//! Bridges to the sibling apps (WPM-R206 to R211, WPM-D134, WPM-D135): references and aggregates, never records. Every call acts
//! under the caller's own token; there is no bridge service account. Spec: `spec/cross-app-bridges.md`.

use std::collections::BTreeSet;

use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::TransactionTrait;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, forbidden};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::workers;
use crate::models::audit_logs::Model as Audit;
use crate::rules::bridge as rules;
use crate::rules::metrics::is_employed_on;

#[derive(Debug, Deserialize)]
struct OverlapPayload {
    organization_ref: String,
    person_refs: Vec<String>,
}

/// `POST /api/bridges/overlap` — of up to 500 `person:` references the caller already holds, which are workers of the organization
/// today? A yes or no for each reference supplied and nothing else: no pid, role, dates or department, and nothing about a person the
/// caller did not name. HR, under their own token; the audit entry carries the count, never the references (WPM-R208).
#[debug_handler]
async fn overlap(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<OverlapPayload>,
) -> Result<Response> {
    let mut problems = organization_problems(&p.organization_ref);
    problems.extend(rules::overlap_problems(&p.person_refs));
    ensure_valid(&problems)?;
    ensure_org_in_scope(&ctx, &caller, &p.organization_ref).await?;
    if !super::policies::privileged(&ctx, &caller, &p.organization_ref).await? {
        return Err(forbidden("the overlap check is for HR"));
    }
    let today = Utc::now().date_naive();
    let employed: BTreeSet<String> = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&p.organization_ref))
        .filter(workers::Column::PersonRef.is_in(p.person_refs.clone()))
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| is_employed_on(today, w.hired_on, w.terminated_on))
        .map(|w| w.person_ref)
        .collect();
    let results = rules::overlap(&employed, &p.person_refs);
    let txn = ctx.db.begin().await?;
    Audit::record(
        &txn,
        "bridge",
        Uuid::new_v4(),
        "bridge_overlap_checked",
        caller.actor(),
        Some(json!({ "asked": results.len(), "workers": results.iter().filter(|(_, w)| *w).count() })),
    )
    .await?;
    txn.commit().await?;
    let results: Vec<_> = results
        .into_iter()
        .map(|(person_ref, worker)| json!({ "person_ref": person_ref, "worker": worker }))
        .collect();
    format::json(json!({ "organization_ref": p.organization_ref, "results": results }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/bridges/overlap", post(overlap))
}
