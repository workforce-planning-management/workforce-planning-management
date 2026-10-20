//! Cost + affordability (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

/// Query for the cost view.
#[derive(Debug, Default, Deserialize)]
pub(super) struct CostQuery {
    /// ISO-4217 currency to cost in; default the plan's budget currency,
    /// else GBP. Totals never mix currencies.
    currency: Option<String>,
}

/// `GET /api/workforce-plans/{pid}/cost?currency=` — what closing the
/// headcount gaps by **hiring** would cost per year, and whether it fits the
/// plan's budget.
///
/// Salary-derived, so it is shown only to a caller with unmasked read access;
/// otherwise the hires needed are given and the money is withheld
/// (`salary_visible: false`). Unit costs come from a role's salary benchmark
/// when every line has one, else from a department average published only
/// for a cohort of at least [`cost_rules::MIN_COHORT`]. Each department is
/// costed at its latest target date. Annual run-rate of salary plus the
/// plan's stated on-cost — not recruitment or onboarding.
#[debug_handler]
pub(super) async fn cost(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axum::extract::Query(query): axum::extract::Query<CostQuery>,
) -> Result<Response> {
    format::json(cost_value(&ctx, &caller, &pid, query).await?)
}

#[allow(clippy::too_many_lines)] // groups → hires → unit cost → totals, linearly
pub(super) async fn cost_value(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
    query: CostQuery,
) -> Result<serde_json::Value> {
    let plan = scoped_plan(ctx, caller, pid).await?;
    let obligations = auth::authorize_record(
        caller,
        authentication_verifier::Action::Read,
        &BTreeMap::new(),
    )
    .map_err(record_rejection)?;
    let salary_visible = !obligations.iter().any(|o| o == "mask");
    let currency = query
        .currency
        .or_else(|| plan.budget_currency.clone())
        .unwrap_or_else(|| "GBP".to_string());
    if !cost_rules::valid_currency(&currency) {
        return Err(unprocessable("currency must be an ISO-4217 code, e.g. GBP"));
    }
    let on_cost_bp = plan.on_cost_bp.unwrap_or(0);
    let today = chrono::Utc::now().date_naive();
    let (attrition, source) = resolve_attrition(ctx, &plan).await?;
    let lines = plan_lines(ctx, plan.pid).await?;
    let employed =
        employed_in_departments(ctx, &plan.organization_ref, &departments_of(&lines)).await?;
    let titles = role_titles(ctx).await?;
    let medians: BTreeMap<String, i64> = benchmarks::Entity::find()
        .filter(benchmarks::Column::DeletedAt.is_null())
        .filter(benchmarks::Column::Currency.eq(&currency))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|b| (b.job_title.trim().to_lowercase(), b.median_minor))
        .collect();

    let mut groups: BTreeMap<(String, chrono::NaiveDate), Vec<&plan_demand_lines::Model>> =
        BTreeMap::new();
    for line in &lines {
        groups
            .entry((line.department.clone(), line.target_on))
            .or_default()
            .push(line);
    }
    // department → (target date, annual cost) at the latest costed date.
    let mut latest: BTreeMap<String, (chrono::NaiveDate, i64)> = BTreeMap::new();
    let mut uncosted = 0usize;
    let rows: Vec<serde_json::Value> = groups
        .iter()
        .map(|((department, target_on), group)| {
            let in_dept: Vec<&workers::Model> = employed
                .iter()
                .filter(|w| &w.department == department)
                .collect();
            let demand: i64 = group.iter().map(|l| i64::from(l.target_headcount)).sum();
            let days = (*target_on - today).num_days().max(0);
            let gap = attrition.map(|bp| {
                rules::headcount_gap(demand, rules::project_supply(in_dept.len(), bp, days))
            });
            let hires = gap.map(cost_rules::hires_to_close);
            let benchmark_parts: Vec<(i64, usize)> = group
                .iter()
                .filter_map(|l| {
                    let title = l.role_profile_pid.and_then(|r| titles.get(&r))?;
                    let median = medians.get(&title.trim().to_lowercase())?;
                    Some((*median, usize::try_from(l.target_headcount).unwrap_or(0)))
                })
                .collect();
            let benchmark = (benchmark_parts.len() == group.len())
                .then(|| cost_rules::weighted_unit_cost(&benchmark_parts))
                .flatten();
            let salaries: Vec<i64> = in_dept
                .iter()
                .filter(|w| w.salary_currency.as_deref() == Some(currency.as_str()))
                .filter_map(|w| w.salary_minor)
                .collect();
            let unit =
                cost_rules::pick_unit_cost(benchmark, cost_rules::cohort_average_minor(&salaries));
            let (annual, reason) = match (hires, unit, salary_visible) {
                (_, _, false) => (None, Some("salary_not_visible")),
                (None, _, _) => (None, Some("insufficient_history")),
                (Some(_), None, _) => (None, Some("no_unit_cost")),
                (Some(h), Some((unit_cost, _)), true) => (
                    Some(cost_rules::annual_cost(h, unit_cost, on_cost_bp)),
                    None,
                ),
            };
            match annual {
                Some(amount) => {
                    let entry = latest
                        .entry(department.clone())
                        .or_insert((*target_on, amount));
                    if *target_on >= entry.0 {
                        *entry = (*target_on, amount);
                    }
                }
                None => uncosted += 1,
            }
            serde_json::json!({
                "department": department,
                "target_on": target_on,
                "hires_needed": hires,
                "unit_cost_minor": if salary_visible { unit.map(|u| u.0) } else { None },
                "unit_cost_source": if salary_visible { unit.map(|u| u.1.as_str()) } else { None },
                "annual_cost_minor": annual,
                "reason": reason,
            })
        })
        .collect();

    let total: i64 = latest.values().map(|(_, amount)| amount).sum();
    let budget = match (plan.budget_minor, plan.budget_currency.as_deref()) {
        (Some(minor), Some(code)) if code == currency => Some(minor),
        _ => None,
    };
    let affordability = if salary_visible {
        budget.map(|b| {
            let (remaining, within) = cost_rules::against_budget(total, b);
            serde_json::json!({ "budget_minor": b, "remaining_minor": remaining, "within_budget": within })
        })
    } else {
        None
    };
    Ok(serde_json::json!({
        "derivation": "annual run-rate of salary for the hires needed to close each headcount gap \
                       (gap from the forecast: demand − projected supply, no hires assumed), plus \
                       the plan's stated on-cost. Unit cost: a salary benchmark median when every \
                       demand line has one, else a department average published only for a cohort \
                       of at least 5 salaried workers. Each department is costed at its latest \
                       target date. Excludes recruitment and onboarding. Salary-derived: withheld \
                       from callers without unmasked read access. Totals never mix currencies.",
        "plan": { "pid": plan.pid, "name": plan.name, "status": plan.status },
        "currency": currency,
        "salary_visible": salary_visible,
        "assumptions": {
            "attrition_bp": attrition,
            "attrition_source": source,
            "on_cost_bp": on_cost_bp,
            "min_cohort": cost_rules::MIN_COHORT,
        },
        "groups": rows,
        "total_annual_cost_minor": if salary_visible { Some(total) } else { None },
        "uncosted_groups": uncosted,
        "affordability": affordability,
    }))
}
