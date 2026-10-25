//! The organization now and as a plan intends it (WPM-R153): a plan's target made of roles, and the comparison with the live
//! reporting structure. Aggregate by role; it names no one and lists no one a restructure would move.

#[allow(clippy::wildcard_imports)] // the planning sections share their parent's imports
use super::*;
use crate::models::_entities::plan_org_nodes;
use crate::rules::equality_monitoring::DEFAULT_FLOOR;
use crate::rules::org_shape::{self as shape, Node, Person};
use sea_orm::TransactionTrait;

#[derive(Debug, Deserialize)]
pub(super) struct TargetNode {
    key: String,
    parent: Option<String>,
    department: String,
    title: String,
    headcount: i32,
}

#[derive(Debug, Deserialize)]
pub(super) struct TargetPayload {
    nodes: Vec<TargetNode>,
}

fn node_of(n: &TargetNode) -> Node {
    Node {
        key: n.key.trim().to_string(),
        parent: n
            .parent
            .as_ref()
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty()),
        department: n.department.trim().to_string(),
        title: n.title.trim().to_string(),
        headcount: n.headcount,
    }
}

async fn target_of(ctx: &AppContext, plan: Uuid) -> Result<Vec<Node>> {
    Ok(plan_org_nodes::Entity::find()
        .filter(plan_org_nodes::Column::PlanPid.eq(plan))
        .order_by_asc(plan_org_nodes::Column::Id)
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|n| Node {
            key: n.node_key,
            parent: n.parent_key,
            department: n.department,
            title: n.title,
            headcount: n.headcount,
        })
        .collect())
}

/// `GET /api/workforce-plans/{pid}/target-organization` — the roles the plan intends: key, parent, department, title and headcount.
#[debug_handler]
pub(super) async fn get_target(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    let nodes = target_of(&ctx, plan.pid).await?;
    format::json(serde_json::json!({
        "nodes": nodes.iter().map(|n| serde_json::json!({
            "key": n.key, "parent": n.parent, "department": n.department, "title": n.title, "headcount": n.headcount,
        })).collect::<Vec<_>>(),
    }))
}

/// `PUT /api/workforce-plans/{pid}/target-organization` — replace the whole target. Roles with a headcount and a parent role;
/// checked for unique keys, present parents and no loop. There is no way to name a person.
#[debug_handler]
pub(super) async fn set_target(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<TargetPayload>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    ensure_editable(&plan)?;
    let nodes: Vec<Node> = payload.nodes.iter().map(node_of).collect();
    shape::validate_target(&nodes).map_err(|e| unprocessable(&e.join("; ")))?;
    let txn = ctx.db.begin().await?;
    plan_org_nodes::Entity::delete_many()
        .filter(plan_org_nodes::Column::PlanPid.eq(plan.pid))
        .exec(&txn)
        .await?;
    for n in &nodes {
        plan_org_nodes::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            plan_pid: ActiveValue::set(plan.pid),
            node_key: ActiveValue::set(n.key.clone()),
            parent_key: ActiveValue::set(n.parent.clone()),
            department: ActiveValue::set(n.department.clone()),
            title: ActiveValue::set(n.title.clone()),
            headcount: ActiveValue::set(n.headcount),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
    }
    Audit::record(
        &txn,
        "workforce_plan",
        plan.pid,
        "plan_target_organization_set",
        caller.actor(),
        Some(serde_json::json!({ "roles": nodes.len() })),
    )
    .await?;
    txn.commit().await?;
    format::json(PidRef {
        pid: plan.pid.to_string(),
    })
}

/// `GET /api/workforce-plans/{pid}/organization-diff` — the live structure of the plan's organization against its target:
/// layers, span of control, and each role added, removed, moved or resized. Counts of people under the floor are withheld; no
/// person is named.
#[debug_handler]
pub(super) async fn organization_diff(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    let target = target_of(&ctx, plan.pid).await?;
    let today = chrono::Utc::now().date_naive();
    let people: Vec<Person> = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&plan.organization_ref))
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::HiredOn.lte(today))
        .filter(
            sea_orm::Condition::any()
                .add(workers::Column::TerminatedOn.is_null())
                .add(workers::Column::TerminatedOn.gt(today)),
        )
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| Person {
            key: w.pid.to_string(),
            manager: w.manager_pid.map(|m| m.to_string()),
            department: w.department,
            title: w.job_title,
        })
        .collect();
    if target.is_empty() {
        return format::json(serde_json::json!({
            "plan": plan.pid, "target_set": false,
            "derivation": "This plan has no target organization yet; PUT one to compare.",
        }));
    }
    let diff = shape::diff(&people, &target, DEFAULT_FLOOR);
    format::json(serde_json::json!({
        "plan": plan.pid,
        "target_set": true,
        "small_group_floor": DEFAULT_FLOOR,
        "diff": diff,
        "derivation": "The live side is the workers employed today in the plan's organization, reduced to a manager link, a department and a title before anything is \
                       compared. The target side is the plan's roles and their headcounts. Roles match on title, ignoring case. A count of people under the floor \
                       is withheld, and so is the change that would give it back. Span of control is reports per manager seat, rounded up for the widest. No person is \
                       named, and cost and skills are not compared here.",
    }))
}
