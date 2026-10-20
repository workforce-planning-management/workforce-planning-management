//! Forecast + gap analysis (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

/// Workers per skills query, well under the database's bind-parameter limit.
const DECLARED_CHUNK: usize = 5_000;

/// Employed workers of the plan's organization in the given departments, today. Only the planned
/// departments are read: a plan about two of twenty departments does not load the other eighteen.
pub(super) async fn employed_in_departments(
    ctx: &AppContext,
    org: &str,
    departments: &BTreeSet<String>,
) -> Result<Vec<workers::Model>> {
    let today = chrono::Utc::now().date_naive();
    let scope = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::OrganizationRef.eq(org))
        .filter(workers::Column::Department.is_in(departments.iter().cloned()))
        .filter(workers::Column::HiredOn.lte(today))
        .filter(
            Condition::any()
                .add(workers::Column::TerminatedOn.is_null())
                .add(workers::Column::TerminatedOn.gt(today)),
        );
    crate::controllers::ensure_scale(ctx, scope.clone()).await?;
    Ok(scope
        .all(&ctx.db)
        .await?
        .into_iter()
        // the rule is the authority on "employed today"; SQL only narrows what is read
        .filter(|w| is_employed_on(today, w.hired_on, w.terminated_on))
        .collect())
}

/// The departments a plan's demand lines name.
pub(super) fn departments_of(lines: &[plan_demand_lines::Model]) -> BTreeSet<String> {
    lines.iter().map(|l| l.department.clone()).collect()
}

/// The attrition assumption in force: the plan's own, else the observed
/// rate from the organization's snapshots, else none.
pub(super) async fn resolve_attrition(
    ctx: &AppContext,
    plan: &workforce_plans::Model,
) -> Result<(Option<i32>, &'static str)> {
    if let Some(bp) = plan.attrition_bp {
        return Ok((Some(bp), "plan_assumption"));
    }
    let rows = headcount_snapshots::Entity::find()
        .filter(headcount_snapshots::Column::OrganizationRef.eq(&plan.organization_ref))
        .all(&ctx.db)
        .await?;
    let mut by_date: BTreeMap<chrono::NaiveDate, (i64, Option<i64>)> = BTreeMap::new();
    for row in &rows {
        let entry = by_date.entry(row.as_of).or_insert((0, Some(0)));
        entry.0 += i64::from(row.headcount);
        entry.1 = match (entry.1, row.leavers) {
            (Some(total), Some(n)) => Some(total + i64::from(n)),
            _ => None,
        };
    }
    let snaps: Vec<rules::Snap> = by_date
        .into_iter()
        .map(|(as_of, (headcount, leavers))| rules::Snap {
            as_of,
            headcount,
            leavers,
        })
        .collect();
    Ok(match rules::observed_attrition_bp(&snaps) {
        Some(bp) => (Some(bp), "observed_snapshots"),
        None => (None, "insufficient_history"),
    })
}

/// Hundredths of a person as a number of people.
#[allow(clippy::cast_precision_loss)]
pub(super) fn centi(hundredths: i64) -> f64 {
    hundredths as f64 / 100.0
}

pub(super) fn supply_json(part: &rules::BasisSupply) -> serde_json::Value {
    serde_json::json!({
        "headcount": part.headcount,
        "fte": part.fte_centi.map(centi),
        "no_end_date_counted_as_continuing": part.no_end_date,
        "leaving_before_the_date": part.leaving_before,
    })
}

/// The demand lines of one group, as the rules see them.
pub(super) fn demand_lines_of(group: &[&plan_demand_lines::Model]) -> Vec<rules::DemandLine> {
    group
        .iter()
        .map(|l| rules::DemandLine {
            programme: l.programme_ref.clone(),
            basis: l.basis.clone(),
            headcount: i64::from(l.target_headcount),
            fte_percent: l.target_fte_percent.map(i64::from),
        })
        .collect()
}

/// `GET /api/workforce-plans/{pid}/forecast` — supply projection and gaps
/// per department per target date, with the assumptions and derivation.
///
/// Headcount: opening employed workers, projected after expected leavers
/// (no hires assumed), against the planned demand. Competency: for each
/// demand line with a role profile, how many employed workers in that
/// department are already proficient in each required skill. Aggregate
/// only; levers are suggestions.
#[debug_handler]
#[allow(clippy::too_many_lines)] // headcount gap + competency gap in one pass
pub(super) async fn forecast(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    let today = chrono::Utc::now().date_naive();
    let (attrition, source) = resolve_attrition(&ctx, &plan).await?;
    let lines = plan_lines(&ctx, plan.pid).await?;
    let employed =
        employed_in_departments(&ctx, &plan.organization_ref, &departments_of(&lines)).await?;
    let titles = role_titles(&ctx).await?;
    let requirements = role_skill_requirements::Entity::find().all(&ctx.db).await?;
    let skill_names: BTreeMap<Uuid, String> = crate::models::_entities::skills::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|s| (s.pid, s.name))
        .collect();
    // Only the planned departments' workers and only the skills the lines' roles ask for are read.
    let needed_skills: BTreeSet<Uuid> = requirements.iter().map(|r| r.skill_pid).collect();
    let mut declared: BTreeMap<(Uuid, Uuid), i32> = BTreeMap::new();
    for chunk in employed.chunks(DECLARED_CHUNK) {
        for d in worker_skills::Entity::find()
            .filter(worker_skills::Column::DeletedAt.is_null())
            .filter(worker_skills::Column::WorkerPid.is_in(chunk.iter().map(|w| w.pid)))
            .filter(worker_skills::Column::SkillPid.is_in(needed_skills.iter().copied()))
            .all(&ctx.db)
            .await?
        {
            declared.insert((d.worker_pid, d.skill_pid), d.proficiency);
        }
    }

    // Open plans to convert, by the status and FTE of the worker they are for. A plan names a worker, so
    // only counts leave here, and they are read against the department the worker is in today.
    let open_conversions: BTreeMap<Uuid, String> = conversion_plans::Entity::find()
        .filter(conversion_plans::Column::Intent.eq("convert"))
        .filter(conversion_plans::Column::Status.is_in(["proposed", "approved"]))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|c| (c.worker_pid, c.status))
        .collect();

    // Group lines by (department, target date).
    let mut groups: BTreeMap<(String, chrono::NaiveDate), Vec<&plan_demand_lines::Model>> =
        BTreeMap::new();
    for line in &lines {
        groups
            .entry((line.department.clone(), line.target_on))
            .or_default()
            .push(line);
    }
    let departments: Vec<serde_json::Value> = groups
        .iter()
        .map(|((department, target_on), group)| {
            let in_dept: Vec<&workers::Model> = employed
                .iter()
                .filter(|w| &w.department == department)
                .collect();
            let opening = in_dept.len();
            let demand_lines = demand_lines_of(group);
            let rules::Breakdown {
                total: demand_total,
                by_basis,
                by_programme,
            } = rules::demand_breakdown(&demand_lines);
            let demand = demand_total.headcount;
            let days = (*target_on - today).num_days().max(0);
            let people: Vec<rules::Person> = in_dept
                .iter()
                .map(|w| rules::Person {
                    basis: crate::rules::engagement::Basis::parse(&w.employment_type)
                        .unwrap_or(crate::rules::engagement::Basis::Permanent),
                    fte_percent: w.fte_percent,
                    ends_on: w.engagement_ends_on,
                })
                .collect();
            let projection =
                rules::project_by_basis(&people, today, *target_on, attrition, source);
            let supply = projection.total_headcount;
            let gap = supply.map(|s| rules::headcount_gap(demand, s));
            let contract = rules::contract_fte(&people);
            let conversions = rules::conversion_cover(
                &in_dept
                    .iter()
                    .filter_map(|w| {
                        open_conversions
                            .get(&w.pid)
                            .map(|status| (status.as_str(), w.fte_percent))
                    })
                    .collect::<Vec<_>>(),
            );
            let gap_fte = demand_total
                .fte_percent
                .zip(projection.total_fte_centi)
                .map(|(d, s)| rules::headcount_gap(d, s));
            let competency: Vec<serde_json::Value> = group
                .iter()
                .filter_map(|line| line.role_profile_pid.map(|role| (line, role)))
                .flat_map(|(line, role)| {
                    let needed = usize::try_from(line.target_headcount).unwrap_or(0);
                    requirements
                        .iter()
                        .filter(move |r| r.role_profile_pid == role)
                        .map(|r| {
                            let (mut proficient, mut reskill_pool) = (0usize, 0usize);
                            for worker in &in_dept {
                                match declared.get(&(worker.pid, r.skill_pid)) {
                                    Some(level) if *level >= r.min_proficiency => proficient += 1,
                                    Some(_) => reskill_pool += 1,
                                    None => {}
                                }
                            }
                            serde_json::json!({
                                "job_title": titles.get(&role),
                                "skill": skill_names.get(&r.skill_pid),
                                "importance": r.importance,
                                "min_proficiency": r.min_proficiency,
                                "needed": needed,
                                "proficient_now": proficient,
                                "shortfall": rules::competency_shortfall(needed, proficient),
                                "reskill_pool": reskill_pool,
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .collect();
            serde_json::json!({
                "department": department,
                "target_on": target_on,
                "days_ahead": days,
                "opening_headcount": opening,
                "planned_demand": demand,
                "planned_demand_fte": demand_total.fte_percent.map(centi),
                "demand_by_basis": by_basis.iter().map(|(basis, t)| serde_json::json!({
                    "basis": basis, "headcount": t.headcount, "fte": t.fte_percent.map(centi),
                })).collect::<Vec<_>>(),
                "demand_by_programme": by_programme.iter().map(|(programme, t)| serde_json::json!({
                    "programme_ref": programme, "headcount": t.headcount, "fte": t.fte_percent.map(centi),
                })).collect::<Vec<_>>(),
                "projected_supply": supply,
                "projected_supply_fte": projection.total_fte_centi.map(centi),
                "supply_by_basis": {
                    "permanent": supply_json(&projection.permanent),
                    "fixed_term": supply_json(&projection.fixed_term),
                    "contractor": supply_json(&projection.contractor),
                    "intern": supply_json(&projection.intern),
                },
                "supply_assumptions": projection.assumptions,
                "headcount_gap": gap,
                "fte_gap": gap_fte.map(centi),
                "levers": gap.map(|g| rules::suggested_levers(g, &contract)),
                "contract_fte_now": {
                    "contractor": { "headcount": contract.contractor_headcount, "fte": centi(contract.contractor_fte_centi) },
                    "fixed_term": { "headcount": contract.fixed_term_headcount, "fte": centi(contract.fixed_term_fte_centi) },
                },
                "conversions": {
                    "agreed": { "headcount": conversions.agreed_headcount, "fte": centi(conversions.agreed_fte_centi) },
                    "hoped": { "headcount": conversions.hoped_headcount, "fte": centi(conversions.hoped_fte_centi) },
                },
                "competency_gaps": competency,
            })
        })
        .collect();

    // Demand by programme and date across the whole plan.
    let mut dates: BTreeMap<chrono::NaiveDate, Vec<&plan_demand_lines::Model>> = BTreeMap::new();
    for line in &lines {
        dates.entry(line.target_on).or_default().push(line);
    }
    let programmes: Vec<serde_json::Value> = dates
        .iter()
        .flat_map(|(date, group)| {
            let by_programme = rules::demand_breakdown(&demand_lines_of(group)).by_programme;
            by_programme.into_iter().map(move |(programme, t)| {
                serde_json::json!({
                    "target_on": date, "programme_ref": programme,
                    "headcount": t.headcount, "fte": t.fte_percent.map(centi),
                })
            })
        })
        .collect();
    format::json(serde_json::json!({
        "derivation": "opening = employed workers in the department today; projected supply = \
                       each basis on its own terms: permanent staff leave at the stated attrition, \
                       fixed-term, contractor and intern engagements on their recorded end dates \
                       (no hires assumed); \
                       gap = planned demand − projected supply (positive = shortfall). Lines for \
                       the same department and date add up. Competency counts EMPLOYED workers in \
                       the department whose DECLARED proficiency meets the requirement; the \
                       reskill pool is those who declared it below the bar. Conversions count \
                       the open plans to convert a department's people to permanent: agreed \
                       (approved by a person) and hoped (only proposed), apart; neither changes \
                       the projected supply. Aggregate only; levers are suggestions, not \
                       decisions.",
        "plan": { "pid": plan.pid, "name": plan.name, "status": plan.status },
        "as_of": today,
        "assumptions": {
            "attrition_bp": attrition,
            "attrition_source": source,
            "hires_assumed": 0,
        },
        "departments": departments,
        "programmes": programmes,
    }))
}
