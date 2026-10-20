//! Alignment (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

/// `GET /api/workforce-plans/{pid}/alignment` — does the planned headcount
/// serve the strategy? Share of planned headcount tied to an objective,
/// objectives with no demand behind them, demand lines serving no
/// objective, and critical roles with no ready successor.
#[debug_handler]
pub(super) async fn alignment(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    format::json(alignment_value(&ctx, &caller, &pid).await?)
}

pub(super) async fn alignment_value(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<serde_json::Value> {
    let plan = scoped_plan(ctx, caller, pid).await?;
    let lines = plan_lines(ctx, plan.pid).await?;
    let objectives = plan_objectives::Entity::find()
        .filter(plan_objectives::Column::PlanPid.eq(plan.pid))
        .all(&ctx.db)
        .await?;
    let line_pids: BTreeSet<Uuid> = lines.iter().map(|l| l.pid).collect();
    let link_rows: Vec<demand_line_objectives::Model> = demand_line_objectives::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|k| line_pids.contains(&k.line_pid))
        .collect();
    let linked_lines: BTreeSet<Uuid> = link_rows.iter().map(|k| k.line_pid).collect();
    let served_objectives: BTreeSet<Uuid> = link_rows.iter().map(|k| k.objective_pid).collect();

    let total_planned: usize = lines
        .iter()
        .map(|l| usize::try_from(l.target_headcount).unwrap_or(0))
        .sum();
    let aligned_planned: usize = lines
        .iter()
        .filter(|l| linked_lines.contains(&l.pid))
        .map(|l| usize::try_from(l.target_headcount).unwrap_or(0))
        .sum();
    let unresourced: Vec<&str> = objectives
        .iter()
        .filter(|o| !served_objectives.contains(&o.pid))
        .map(|o| o.title.as_str())
        .collect();
    let unaligned: Vec<serde_json::Value> = lines
        .iter()
        .filter(|l| !linked_lines.contains(&l.pid))
        .map(|l| serde_json::json!({ "department": l.department, "target_on": l.target_on, "target_headcount": l.target_headcount }))
        .collect();

    // Critical roles with no ready successor (the shared single-point rule).
    let plan_departments: BTreeSet<&str> = lines.iter().map(|l| l.department.as_str()).collect();
    let succession = succession_plans::Entity::find()
        .filter(succession_plans::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let candidates = succession_candidates::Entity::find()
        .filter(succession_candidates::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let (mut spof_total, mut spof_in_plan, mut plans_in_plan) = (0usize, 0usize, 0usize);
    for s in &succession {
        if plan_departments.contains(s.department.as_str()) {
            plans_in_plan += 1;
        }
        let readiness: Vec<String> = candidates
            .iter()
            .filter(|c| c.plan_pid == s.pid)
            .map(|c| c.readiness.clone())
            .collect();
        let coverage = talent_rules::bench_coverage(&readiness);
        if talent_rules::is_single_point_of_failure(
            s.criticality,
            coverage,
            s.risk_of_loss.as_deref(),
        ) {
            spof_total += 1;
            if plan_departments.contains(s.department.as_str()) {
                spof_in_plan += 1;
            }
        }
    }

    Ok(serde_json::json!({
        "derivation": "planned headcount sums the plan's demand lines; a line is aligned when it \
                       serves at least one objective. Critical roles with no ready successor use \
                       the shared single-point-of-failure rule. Terms-carrying ratios; null when \
                       there is nothing to divide.",
        "plan": { "pid": plan.pid, "name": plan.name, "status": plan.status },
        "planned_headcount": total_planned,
        "aligned_share": ratio_json(talent_rules::ratio(aligned_planned, total_planned)),
        "objectives": objectives.len(),
        "unresourced_objectives": unresourced,
        "unaligned_demand_lines": unaligned,
        "critical_roles_without_bench": {
            "in_plan_departments": spof_in_plan,
            "succession_plans_in_plan_departments": plans_in_plan,
            "all_departments": spof_total,
        },
    }))
}
