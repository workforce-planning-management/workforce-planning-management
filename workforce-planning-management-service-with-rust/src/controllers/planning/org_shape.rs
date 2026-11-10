//! The organization now and as a plan intends it (WPM-R153): a plan's target made of roles, and the comparison with the live
//! reporting structure. Aggregate by role; it names no one and lists no one a restructure would move.

#[allow(clippy::wildcard_imports)] // the planning sections share their parent's imports
use super::*;
use crate::db::{self, pool};
use crate::rules::equality_monitoring::DEFAULT_FLOOR;
use crate::rules::org_shape::{self as shape, Node, Person};

use axa::extract::{Json, Path};

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
    Ok(sqlx::query_as!(
        Node,
        r#"SELECT node_key AS key, parent_key AS parent, department, title, headcount
           FROM plan_org_nodes WHERE plan_pid = $1 ORDER BY id"#,
        plan,
    )
    .fetch_all(pool(ctx))
    .await?)
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
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!("DELETE FROM plan_org_nodes WHERE plan_pid = $1", plan.pid)
        .execute(&mut *tx)
        .await?;
    for n in &nodes {
        sqlx::query!(
            "INSERT INTO plan_org_nodes (pid, plan_pid, node_key, parent_key, department, title, headcount)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            Uuid::new_v4(),
            plan.pid,
            n.key,
            n.parent,
            n.department,
            n.title,
            n.headcount,
        )
        .execute(&mut *tx)
        .await?;
    }
    db::audit(
        &mut tx,
        "workforce_plan",
        plan.pid,
        "plan_target_organization_set",
        caller.actor(),
        Some(serde_json::json!({ "roles": nodes.len() })),
    )
    .await?;
    tx.commit().await?;
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
    let people: Vec<Person> = sqlx::query!(
        "SELECT pid, manager_pid, department, job_title FROM workers
         WHERE organization_ref = $1 AND deleted_at IS NULL AND hired_on <= $2
           AND (terminated_on IS NULL OR terminated_on > $2)",
        plan.organization_ref,
        today,
    )
    .fetch_all(pool(&ctx))
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
