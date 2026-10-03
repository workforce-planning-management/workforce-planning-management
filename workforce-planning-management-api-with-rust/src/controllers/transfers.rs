//! **Transfers**: moving a worker to another organization. A worker's
//! organization is not editable through `PUT /api/workers/{pid}`; a transfer
//! is its own act, because it has consequences — chiefly, the **group
//! memberships that no longer fit end** (kept as past membership, not
//! deleted).

use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, TransactionTrait};
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::{ensure_valid, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{group_members, groups, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::rules::groups as rules;
use crate::validation::Problems;

/// `POST /api/workers/{pid}/transfer` body.
#[derive(Debug, Deserialize)]
struct TransferPayload {
    /// The organization the worker is moving to.
    organization_ref: String,
}

/// `POST /api/workers/{pid}/transfer` — move a worker to another organization.
/// A write to the worker's record, and the target must be an organization the
/// caller can read. Group memberships in groups not open to the new
/// organization end (they are closed, not deleted); ones that still fit —
/// including confederation groups covering both — stay. The solid-line manager
/// is **not** changed; the response says if they are in another organization.
#[debug_handler]
async fn transfer(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<TransferPayload>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let mut problems = Problems::new();
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        &payload.organization_ref,
    );
    ensure_valid(&problems.into_vec())?;
    if payload.organization_ref == worker.organization_ref {
        return Err(unprocessable("the worker is already in that organization"));
    }
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.contains(&payload.organization_ref)
    {
        return Err(unprocessable(
            "you cannot move a worker to that organization",
        ));
    }
    let graph = crate::models::confederations::live_edges(&ctx.db).await?;
    let txn = ctx.db.begin().await?;
    let open = group_members::Entity::find()
        .filter(group_members::Column::WorkerPid.eq(worker.pid))
        .filter(group_members::Column::LeftAt.is_null())
        .all(&txn)
        .await?;
    let by_group: BTreeMap<Uuid, groups::Model> = groups::Entity::find()
        .all(&txn)
        .await?
        .into_iter()
        .map(|g| (g.pid, g))
        .collect();
    let fits: Vec<(Uuid, &str, &str)> = open
        .iter()
        .filter_map(|m| {
            by_group
                .get(&m.group_pid)
                .map(|g| (m.pid, g.scope.as_str(), g.organization_ref.as_str()))
        })
        .collect();
    let to_end = rules::memberships_to_end(&payload.organization_ref, &fits, &graph);
    let mut ended = Vec::new();
    for membership in open.into_iter().filter(|m| to_end.contains(&m.pid)) {
        let name = by_group.get(&membership.group_pid).map(|g| g.name.clone());
        let mut active: group_members::ActiveModel = membership.into();
        active.left_at = ActiveValue::set(Some(Utc::now().into()));
        active.update(&txn).await?;
        ended.push(name);
    }
    let manager_elsewhere = match worker.manager_pid {
        Some(manager) => workers::Entity::find()
            .filter(workers::Column::Pid.eq(manager))
            .one(&txn)
            .await?
            .is_some_and(|m| m.organization_ref != payload.organization_ref),
        None => false,
    };
    let previous = worker.organization_ref.clone();
    let mut active: workers::ActiveModel = worker.into();
    active.organization_ref = ActiveValue::set(payload.organization_ref.clone());
    let row = active.update(&txn).await?;
    Audit::record(
        &txn,
        "worker",
        row.pid,
        "transferred",
        caller.actor(),
        Some(serde_json::json!({
            "from": previous,
            "to": payload.organization_ref,
            "group_memberships_ended": ended.len(),
        })),
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({
        "worker_pid": row.pid,
        "from": previous,
        "to": row.organization_ref,
        "ended_group_memberships": ended,
        "manager_in_other_organization": manager_elsewhere,
    }))
}

/// The transfer routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/transfer", post(transfer))
}
