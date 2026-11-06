//! Decision gates (WPM-R108, WPM-D68): a list of the kinds of change that need consultation with a recognised body, each with the body to
//! consult. When someone makes such a change in WPM (today: records a scoring feature in the AI register), the response points at the
//! gate and asks for the consultation reference. It **advises and never blocks**: the law and the agreements decide, not this tool.
//! HR writes; the pointer is shown to whoever makes the change. Spec: `spec/worker-unions.md`.

use axa::prelude::*;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::bargaining::find_body;
use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db::{self, pool};
use crate::rules::decision_gate as rules;

/// Where the equality impact template says to record the consultation for a score.
const IMPACT_TEMPLATE: &str = "spec/equality-impact/template.md, section 11 (Consultation)";

/// The gate pointer for a `change`: which bodies are to be consulted (in `organization`, or in every organization when `None`), and whether a
/// consultation reference came with the change. The reference is checked to be a real consultation record, not judged.
///
/// # Errors
///
/// A database error, or a `422` for a reference that is no consultation record.
pub(crate) async fn pointer(
    ctx: &AppContext,
    organization: Option<&str>,
    change: &str,
    consultation_pid: Option<Uuid>,
) -> Result<serde_json::Value> {
    let gates = sqlx::query!(
        "SELECT organization_ref, change_kind, body_pid, note FROM decision_gates
         WHERE change_kind = $1 AND ($2::text IS NULL OR organization_ref = $2)
         ORDER BY organization_ref",
        change,
        organization,
    )
    .fetch_all(pool(ctx))
    .await?;
    if let Some(reference) = consultation_pid {
        let known = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM consultations WHERE pid = $1) AS "known!""#,
            reference,
        )
        .fetch_one(pool(ctx))
        .await?;
        if !known {
            return Err(unprocessable(
                "consultation_pid is not a consultation record",
            ));
        }
    }
    let ids: Vec<String> = gates.iter().map(|g| g.body_pid.to_string()).collect();
    let as_gates: Vec<rules::Gate<'_>> = gates
        .iter()
        .zip(&ids)
        .map(|(g, id)| rules::Gate {
            change_kind: &g.change_kind,
            body: id,
        })
        .collect();
    let bodies = rules::bodies_for(&as_gates, change).len();
    Ok(json!({
        "change": change,
        "standing": rules::standing(bodies, consultation_pid.is_some()),
        "consult": gates.iter().map(|g| json!({ "organization_ref": g.organization_ref, "body_pid": g.body_pid, "note": g.note })).collect::<Vec<_>>(),
        "consultation_pid": consultation_pid,
        "equality_impact_template": if change == "new_or_changed_score" { Some(IMPACT_TEMPLATE) } else { None },
        "note": "advice, not a block: record the consultation reference with the change; whether consultation is required is for the law and your agreements to say",
    }))
}

#[derive(Debug, Deserialize)]
pub(crate) struct OrgQuery {
    organization: String,
}

/// `GET /api/decision-gates?organization=` — the changes that need consultation, and with which body. HR.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<OrgQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let rows = sqlx::query!(
        "SELECT pid, change_kind, body_pid, note FROM decision_gates
         WHERE organization_ref = $1 ORDER BY change_kind",
        query.organization,
    )
    .fetch_all(pool(&ctx))
    .await?;
    format::json(json!({
        "changes": rules::CHANGES,
        "gates": rows.iter().map(|g| json!({ "pid": g.pid, "change_kind": g.change_kind, "body_pid": g.body_pid, "note": g.note })).collect::<Vec<_>>(),
    }))
}

#[derive(Debug, Deserialize)]
struct GatePayload {
    organization_ref: String,
    change_kind: String,
    body_pid: Uuid,
    note: Option<String>,
}

/// `POST /api/decision-gates` — say that a kind of change needs consultation with a body of the organization.
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<GatePayload>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&payload.organization_ref))?;
    ensure_org_in_scope(&ctx, &caller, &payload.organization_ref).await?;
    ensure_valid(&rules::gate_problems(
        &payload.change_kind,
        payload.note.as_deref(),
    ))?;
    let body = find_body(&ctx, &caller, payload.body_pid).await?;
    if body.organization_ref != payload.organization_ref {
        return Err(unprocessable("body_pid is not a body of this organization"));
    }
    let clash = sqlx::query_scalar!(
        r#"SELECT EXISTS (
               SELECT 1 FROM decision_gates WHERE organization_ref = $1 AND change_kind = $2 AND body_pid = $3
           ) AS "clash!""#,
        payload.organization_ref,
        payload.change_kind,
        payload.body_pid,
    )
    .fetch_one(pool(&ctx))
    .await?;
    if clash {
        return Err(unprocessable(
            "this change already has a gate for that body",
        ));
    }
    let pid = Uuid::new_v4();
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO decision_gates (pid, organization_ref, change_kind, body_pid, note, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6)",
        pid,
        payload.organization_ref,
        payload.change_kind,
        payload.body_pid,
        payload
            .note
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty()),
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "decision_gate",
        pid,
        "decision_gate_created",
        caller.actor(),
        Some(json!({ "change_kind": payload.change_kind })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

#[derive(Debug, Deserialize)]
struct CheckQuery {
    organization: String,
    change: String,
    consultation_pid: Option<Uuid>,
}

/// `GET /api/decision-gates/check?organization=&change=&consultation_pid=` — what a change would be pointed at: the bodies to consult and
/// whether a consultation reference is still needed. For a screen to ask before someone makes the change. HR.
#[debug_handler]
async fn check_change(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<CheckQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    ensure_valid(&rules::gate_problems(&query.change, None))?;
    format::json(
        pointer(
            &ctx,
            Some(&query.organization),
            &query.change,
            query.consultation_pid,
        )
        .await?,
    )
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/decision-gates", get(list))
        .add("/decision-gates", post(create))
        .add("/decision-gates/check", get(check_change))
}
