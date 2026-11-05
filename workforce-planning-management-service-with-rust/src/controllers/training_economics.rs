//! **What learning costs and returns** ([`crate::rules::training_economics`], WPM-R227 to WPM-R230).
//!
//! - `GET /api/development-plans/{pid}/cost`: the plan's recorded fees per currency beside the cost of its hours.
//! - `GET /api/positions/{pid}/train-or-hire?candidate=`: both totals and every input. Advisory.
//! - `POST /api/development-plans/{pid}/benefits` and its `/api/me` twin: a stated yearly benefit with its source.
//! - `GET /api/development-plans/{pid}/viability`: net present value and payback, on the fees and with the hours.
//! - `GET` and `PUT /api/learning-budgets`: recorded fees against a department's budget.
//!
//! Plans are for the person and HR only, not a line manager, because the cost of hours rests on pay. Anything worked from a
//! person's pay follows the mask: a caller whose read is masked gets the fees and hours and `pay_withheld`, so the masked
//! salary cannot be read back out of an hourly cost. An unmasked read of someone else's plan records a `salary_read`.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use axa::prelude::*;
use chrono::{Datelike, NaiveDate, Utc};
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;

use super::jurisdictions::Scope;
use super::me::me_worker;
use super::training_plan::load_catalogue;
use super::{ensure_org_in_scope, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{
    benchmarks, development_plan_benefits, development_plan_items, development_plans,
    learning_budgets, plan_demand_lines, positions, requisitions, role_skill_requirements,
    skill_courses, skills, training_enrollments, worker_skills, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::cost as cost_rules;
use crate::rules::jurisdiction::Resolution;
use crate::rules::metrics;
use crate::rules::skill_gap::{self as gap_rules, Need, Source, Status};
use crate::rules::training::DEFAULT_HOURS_PER_LEVEL;
use crate::rules::training_economics::{
    self as rules, Benefit, Fee, HireSide, HoursCost, Step, TrainSide,
};

/// What it means to read a plan's figures.
struct Reader {
    /// The caller is not the plan's own worker.
    on_behalf: bool,
    /// Figures worked from the worker's pay may be shown.
    pay_visible: bool,
}

/// The worker themself, or HR in the worker's organization; a line manager is refused, as a stranger is.
async fn plan_reader(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
) -> Result<Reader> {
    let attrs = auth::worker_resource_attrs(worker);
    let obligations = auth::authorize_record(caller, authentication_verifier::Action::Read, &attrs)
        .map_err(record_rejection)?;
    let on_behalf = auth::acting_for_other(caller, &worker.person_ref);
    if on_behalf {
        auth::authorize_record(caller, authentication_verifier::Action::Write, &attrs)
            .map_err(record_rejection)?;
        ensure_org_in_scope(ctx, caller, &worker.organization_ref).await?;
    }
    Ok(Reader {
        on_behalf,
        pay_visible: !obligations.iter().any(|o| o == "mask"),
    })
}

/// An unmasked figure from someone else's pay is a salary read, audited as the worker read is (WPM-D7).
async fn audit_salary_read(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
) -> Result<()> {
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "salary_read",
        caller.actor(),
        Some(serde_json::json!({ "department": worker.department })),
    )
    .await?;
    Ok(())
}

/// A live plan and its worker.
async fn plan_and_worker(
    ctx: &AppContext,
    pid: &str,
) -> Result<(development_plans::Model, workers::Model)> {
    let plan = records::find_development_plan(&ctx.db, records::parse_pid(pid)?).await?;
    let worker = records::find_worker(&ctx.db, plan.worker_pid).await?;
    Ok((plan, worker))
}

/// The fee a catalogue row states.
fn fee_of(course: &skill_courses::Model) -> Fee {
    match (course.fee_minor, &course.fee_currency) {
        (Some(minor), Some(currency)) => Fee::Known {
            minor,
            currency: currency.clone(),
        },
        _ => Fee::Unknown,
    }
}

/// The steps of a plan, leaving out what was abandoned. A course step reads the catalogue; a step with no catalogue row counts
/// as a course with no fee, at the skill's per-level figure. A step that is not a course has no fee to know.
async fn plan_steps(ctx: &AppContext, plan: &development_plans::Model) -> Result<Vec<Step>> {
    let items = development_plan_items::Entity::find()
        .filter(development_plan_items::Column::PlanPid.eq(plan.pid))
        .order_by_asc(development_plan_items::Column::Id)
        .all(&ctx.db)
        .await?;
    let per_level: HashMap<Uuid, i32> = skills::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter_map(|s| s.hours_per_level.map(|h| (s.pid, h)))
        .collect();
    let courses: HashMap<(Uuid, String), skill_courses::Model> = skill_courses::Entity::find()
        .filter(skill_courses::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|c| ((c.skill_pid, c.course_ref.clone()), c))
        .collect();
    Ok(items
        .iter()
        .filter(|item| item.status != "abandoned")
        .map(|item| {
            let levels = u32::try_from(item.target_level - item.current_level).unwrap_or(0);
            let each = per_level
                .get(&item.skill_pid)
                .and_then(|h| u32::try_from(*h).ok())
                .unwrap_or(DEFAULT_HOURS_PER_LEVEL);
            let estimate = levels.saturating_mul(each);
            let course = item
                .course_ref
                .as_ref()
                .and_then(|r| courses.get(&(item.skill_pid, r.clone())));
            match (item.method.as_str(), course) {
                ("course", Some(course)) => Step {
                    fee: fee_of(course),
                    hours: u32::try_from(course.hours).unwrap_or(0),
                    estimated_hours: false,
                },
                ("course", None) => Step {
                    fee: Fee::Unknown,
                    hours: estimate,
                    estimated_hours: true,
                },
                _ => Step {
                    fee: Fee::NotACourse,
                    hours: estimate,
                    estimated_hours: true,
                },
            }
        })
        .collect())
}

/// The worker's pay as the costing needs it.
fn pay_of(worker: &workers::Model) -> Option<(i64, &str)> {
    Some((worker.salary_minor?, worker.salary_currency.as_deref()?))
}

/// The cost of `hours` for this reader.
fn hours_for(
    reader_sees_pay: bool,
    hours: u32,
    worker: &workers::Model,
    scope: &Scope,
    today: NaiveDate,
) -> HoursCost {
    if !reader_sees_pay {
        return HoursCost::Unresolved {
            missing: vec!["pay_withheld"],
        };
    }
    rules::hours_cost(
        hours,
        pay_of(worker),
        scope.resolve("learning.on_cost_bp", today).integer(),
        scope
            .resolve("learning.full_time_annual_hours", today)
            .integer(),
    )
}

/// `GET /api/development-plans/{pid}/cost` — the recorded fees per currency, and the hours at the worker's loaded rate, beside
/// the fees and never added to them.
#[debug_handler]
async fn plan_cost(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (plan, worker) = plan_and_worker(&ctx, &pid).await?;
    let reader = plan_reader(&ctx, &caller, &worker).await?;
    let today = Utc::now().date_naive();
    let scope = Scope::for_worker(&ctx.db, &worker).await?;
    let cost = rules::plan_cost(&plan_steps(&ctx, &plan).await?);
    let hours = hours_for(reader.pay_visible, cost.hours, &worker, &scope, today);
    if reader.on_behalf && hours.resolved().is_some() {
        audit_salary_read(&ctx, &caller, &worker).await?;
    }
    format::json(serde_json::json!({
        "plan_pid": plan.pid,
        "worker_pid": plan.worker_pid,
        "cost": cost,
        "hours_cost": hours,
        "derivation": "Fees are the recorded catalogue fees of the plan's courses, per currency, never converted; a course \
                       with no fee is counted, not priced at zero. Hours are a course's stated hours, or the skill's \
                       per-level planning figure for any other step. The cost of hours is pay plus on-cost over a full-time \
                       year, shown beside the fees and never added to them. Abandoned steps are left out. An estimate.",
    }))
}

/// `POST .../benefits` body.
#[derive(Debug, Deserialize)]
pub(crate) struct BenefitPayload {
    /// The year it falls in; 1 is the first year after now.
    year: i32,
    amount_minor: i64,
    currency: String,
    /// Who or what the estimate rests on.
    source: String,
}

/// State a benefit on `plan` (WPM-D173). One a year per plan; the entry in the audit log names the year and no amount.
async fn state_benefit(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    plan: &development_plans::Model,
    payload: &BenefitPayload,
) -> Result<Response> {
    rules::valid_benefit(
        payload.year,
        payload.amount_minor,
        &payload.currency,
        &payload.source,
    )
    .map_err(|e| unprocessable(&e))?;
    let taken = development_plan_benefits::Entity::find()
        .filter(development_plan_benefits::Column::PlanPid.eq(plan.pid))
        .filter(development_plan_benefits::Column::Year.eq(payload.year))
        .filter(development_plan_benefits::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .is_some();
    if taken {
        return Err(unprocessable(
            "this plan already has a benefit for that year; remove it first",
        ));
    }
    let txn = ctx.db.begin().await?;
    let row = development_plan_benefits::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        plan_pid: ActiveValue::set(plan.pid),
        worker_pid: ActiveValue::set(plan.worker_pid),
        year: ActiveValue::set(payload.year),
        amount_minor: ActiveValue::set(payload.amount_minor),
        currency: ActiveValue::set(payload.currency.clone()),
        source: ActiveValue::set(payload.source.trim().to_string()),
        stated_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        stated_on: ActiveValue::set(Utc::now().date_naive()),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "development_plan_benefit",
        row.pid,
        "stated",
        caller.actor(),
        Some(serde_json::json!({ "plan_pid": plan.pid, "year": row.year })),
    )
    .await?;
    txn.commit().await?;
    format::json(benefit_json(&row))
}

/// Remove a live benefit.
async fn remove_benefit(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    row: development_plan_benefits::Model,
) -> Result<Response> {
    let pid = row.pid;
    let mut active: development_plan_benefits::ActiveModel = row.into();
    active.deleted_at = ActiveValue::set(Some(Utc::now().into()));
    let txn = ctx.db.begin().await?;
    active.update(&txn).await?;
    Audit::record(
        &txn,
        "development_plan_benefit",
        pid,
        "removed",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": pid }))
}

/// A live benefit by its pid.
async fn find_benefit(ctx: &AppContext, pid: &str) -> Result<development_plan_benefits::Model> {
    development_plan_benefits::Entity::find()
        .filter(development_plan_benefits::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(development_plan_benefits::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

/// `POST /api/development-plans/{pid}/benefits` — HR states a benefit for a plan in their organization.
#[debug_handler]
async fn add_benefit(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<BenefitPayload>,
) -> Result<Response> {
    let (plan, worker) = plan_and_worker(&ctx, &pid).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    ensure_org_in_scope(&ctx, &caller, &worker.organization_ref).await?;
    state_benefit(&ctx, &caller, &plan, &payload).await
}

/// `DELETE /api/development-plan-benefits/{pid}` — an administrator removes a benefit. The reference policy gives HR no delete, so
/// this route is not HR's; the person removes their own under `/api/me`.
#[debug_handler]
async fn delete_benefit(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = find_benefit(&ctx, &pid).await?;
    let worker = records::find_worker(&ctx.db, row.worker_pid).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    ensure_org_in_scope(&ctx, &caller, &worker.organization_ref).await?;
    remove_benefit(&ctx, &caller, row).await
}

/// `POST /api/me/development-plans/{pid}/benefits` — the plan's own worker states a benefit.
#[debug_handler]
pub(crate) async fn add_my_benefit(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<BenefitPayload>,
) -> Result<Response> {
    let me = me_worker(&ctx, &caller).await?;
    let (plan, _) = plan_and_worker(&ctx, &pid).await?;
    if plan.worker_pid != me.pid {
        return Err(Error::NotFound);
    }
    state_benefit(&ctx, &caller, &plan, &payload).await
}

/// `DELETE /api/me/development-plan-benefits/{pid}` — the plan's own worker removes a benefit.
#[debug_handler]
pub(crate) async fn delete_my_benefit(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let me = me_worker(&ctx, &caller).await?;
    let row = find_benefit(&ctx, &pid).await?;
    if row.worker_pid != me.pid {
        return Err(Error::NotFound);
    }
    remove_benefit(&ctx, &caller, row).await
}

fn benefit_json(row: &development_plan_benefits::Model) -> serde_json::Value {
    serde_json::json!({
        "pid": row.pid,
        "year": row.year,
        "amount_minor": row.amount_minor,
        "currency": row.currency,
        "source": row.source,
        "stated_by": row.stated_by,
        "stated_on": row.stated_on,
    })
}

/// `GET /api/development-plans/{pid}/viability` — payback and net present value at the organization's discount rate, on the
/// fees alone and with the hours. No stated benefit is unmeasured, never not viable.
#[debug_handler]
async fn plan_viability(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (plan, worker) = plan_and_worker(&ctx, &pid).await?;
    let reader = plan_reader(&ctx, &caller, &worker).await?;
    let today = Utc::now().date_naive();
    let scope = Scope::for_worker(&ctx.db, &worker).await?;
    let cost = rules::plan_cost(&plan_steps(&ctx, &plan).await?);
    let hours = hours_for(reader.pay_visible, cost.hours, &worker, &scope, today);
    let rows = development_plan_benefits::Entity::find()
        .filter(development_plan_benefits::Column::PlanPid.eq(plan.pid))
        .filter(development_plan_benefits::Column::DeletedAt.is_null())
        .order_by_asc(development_plan_benefits::Column::Year)
        .all(&ctx.db)
        .await?;
    let benefits: Vec<Benefit> = rows
        .iter()
        .filter_map(|b| {
            Some(Benefit {
                year: u32::try_from(b.year).ok()?,
                amount_minor: b.amount_minor,
                currency: b.currency.clone(),
            })
        })
        .collect();
    let rate = scope
        .resolve("learning.discount_rate_bp", today)
        .integer()
        .and_then(|r| u32::try_from(r).ok());
    let viability = rules::plan_viability(&cost, &hours, &benefits, rate);
    if reader.on_behalf && viability.with_hours.is_some() {
        audit_salary_read(&ctx, &caller, &worker).await?;
    }
    format::json(serde_json::json!({
        "plan_pid": plan.pid,
        "viability": viability,
        "benefits": rows.iter().map(benefit_json).collect::<Vec<_>>(),
        "derivation": "Fees and hours fall in year 0 and each stated benefit in its year. A flow is discounted year by year \
                       at the rate shown, rounding half away from zero at each step. Payback is linear inside a year. A \
                       benefit is someone's stated estimate with its source: WPM never turns a skill level, a completion or \
                       a score into money. A fee that is unknown leaves the investment unknown.",
    }))
}

/// Query for train or hire.
#[derive(Debug, Deserialize)]
struct TrainOrHireQuery {
    /// The internal candidate's worker pid.
    candidate: String,
}

/// The skills a seat needs: its demand line's role profile. A seat with neither has none on record, which is unknown.
async fn seat_needs(ctx: &AppContext, position: &positions::Model) -> Result<Option<Vec<Need>>> {
    let Some(line) = position.demand_line_pid else {
        return Ok(None);
    };
    let Some(role) = plan_demand_lines::Entity::find()
        .filter(plan_demand_lines::Column::Pid.eq(line))
        .one(&ctx.db)
        .await?
        .and_then(|l| l.role_profile_pid)
    else {
        return Ok(None);
    };
    let rows = role_skill_requirements::Entity::find()
        .filter(role_skill_requirements::Column::RoleProfilePid.eq(role))
        .all(&ctx.db)
        .await?;
    if rows.is_empty() {
        return Ok(None);
    }
    Ok(Some(
        rows.into_iter()
            .map(|r| Need {
                skill: r.skill_pid,
                required: r.min_proficiency,
                importance: r.importance,
                source: Source::Role,
            })
            .collect(),
    ))
}

/// The training that would close the candidate's gaps for this seat, or `None` while a needed skill is undeclared or the seat
/// has no skills on record. Undeclared is a reason to assess, not a gap of any size.
async fn candidate_steps(
    ctx: &AppContext,
    candidate: &workers::Model,
    needs: &[Need],
) -> Result<(Option<Vec<Step>>, usize)> {
    let declared: BTreeMap<Uuid, i32> = worker_skills::Entity::find()
        .filter(worker_skills::Column::WorkerPid.eq(candidate.pid))
        .filter(worker_skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|r| (r.skill_pid, r.proficiency))
        .collect();
    let gaps = gap_rules::merge(needs, &declared);
    let undeclared = gaps
        .iter()
        .filter(|g| g.status == Status::Undeclared)
        .count();
    if undeclared > 0 {
        return Ok((None, undeclared));
    }
    let catalogue = load_catalogue(ctx, &[candidate.pid]).await?;
    let fees: HashMap<(Uuid, String), Fee> = skill_courses::Entity::find()
        .filter(skill_courses::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|c| ((c.skill_pid, c.course_ref.clone()), fee_of(&c)))
        .collect();
    let mut steps = Vec::new();
    for gap in gaps.iter().filter(|g| g.status == Status::Below) {
        let plan = catalogue.recommend(candidate.pid, gap);
        let course_hours: u32 = plan.courses.iter().map(|c| c.hours).sum();
        for course in &plan.courses {
            steps.push(Step {
                fee: fees
                    .get(&(gap.skill, course.course_ref.clone()))
                    .cloned()
                    .unwrap_or(Fee::Unknown),
                hours: course.hours,
                estimated_hours: false,
            });
        }
        if plan.hours > course_hours {
            steps.push(Step {
                fee: Fee::NotACourse,
                hours: plan.hours - course_hours,
                estimated_hours: true,
            });
        }
    }
    Ok((Some(steps), 0))
}

/// The seat's annual unit cost in `currency`: its title's benchmark, else the average of the department its cost centre names
/// when the cohort is large enough to publish. Never a guess.
async fn seat_unit(
    ctx: &AppContext,
    position: &positions::Model,
    currency: &str,
) -> Result<Option<(i64, cost_rules::CostSource)>> {
    let title = position.title.trim().to_lowercase();
    let benchmark = benchmarks::Entity::find()
        .filter(benchmarks::Column::DeletedAt.is_null())
        .filter(benchmarks::Column::Currency.eq(currency))
        .all(&ctx.db)
        .await?
        .into_iter()
        .find(|b| b.job_title.trim().to_lowercase() == title)
        .map(|b| b.median_minor);
    let salaries: Vec<i64> = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&position.organization_ref))
        .filter(workers::Column::Department.eq(&position.cost_centre))
        .filter(workers::Column::SalaryCurrency.eq(currency))
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::TerminatedOn.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter_map(|w| w.salary_minor)
        .collect();
    Ok(cost_rules::pick_unit_cost(
        benchmark,
        cost_rules::cohort_average_minor(&salaries),
    ))
}

/// An annual figure with on-cost, at the seat's budgeted FTE.
fn loaded(annual: i64, on_cost_bp: i64, fte_percent: i32) -> i64 {
    let with_cost = cost_rules::with_on_cost(annual, i32::try_from(on_cost_bp).unwrap_or(0));
    let scaled = i128::from(with_cost) * i128::from(fte_percent);
    i64::try_from((scaled + 50) / 100).unwrap_or(i64::MAX)
}

/// The recruitment cost when it is stated in `currency`; it is never converted.
fn recruitment_in(scope: &Scope, currency: &str, today: NaiveDate) -> Option<i64> {
    let stated = match scope.resolve("recruitment.currency", today) {
        Resolution::Resolved { value, .. } => value.as_names()?,
        _ => return None,
    };
    if stated.first().map(String::as_str) != Some(currency) {
        return None;
    }
    scope.resolve("recruitment.cost_minor", today).integer()
}

/// The organization's median days to fill, and how many filled requisitions it rests on.
async fn vacancy_days(ctx: &AppContext, organization: &str) -> Result<(Option<i64>, usize)> {
    let days: Vec<i64> = requisitions::Entity::find()
        .filter(requisitions::Column::OrganizationRef.eq(organization))
        .filter(requisitions::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .iter()
        .filter_map(|r| metrics::time_to_fill_days(r.opened_on, r.filled_on))
        .collect();
    #[allow(clippy::cast_possible_truncation)] // a median of day counts
    Ok(match metrics::fill_time_summary(&days) {
        Some(fill) => (Some(fill.median.round() as i64), fill.count),
        None => (None, 0),
    })
}

/// `GET /api/positions/{pid}/train-or-hire?candidate=` — training this candidate for the seat against an external hire. Both
/// totals and every input; a missing input answers `unknown`. Advisory: WPM decides nothing. Parameters resolve at the
/// candidate's place of work, because a seat records none. A masked reader gets the inputs that do not rest on the candidate's
/// pay and no totals.
#[debug_handler]
#[allow(clippy::too_many_lines)] // the two sides, each input named once, in order
async fn train_or_hire(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axum::extract::Query(query): axum::extract::Query<TrainOrHireQuery>,
) -> Result<Response> {
    let position = positions::Entity::find()
        .filter(positions::Column::Pid.eq(records::parse_pid(&pid)?))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    ensure_org_in_scope(&ctx, &caller, &position.organization_ref).await?;
    let candidate = records::find_worker(&ctx.db, records::parse_pid(&query.candidate)?).await?;
    if candidate.organization_ref != position.organization_ref {
        return Err(Error::NotFound);
    }
    let obligations = auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &auth::worker_resource_attrs(&candidate),
    )
    .map_err(record_rejection)?;
    let pay_visible = !obligations.iter().any(|o| o == "mask");

    let today = Utc::now().date_naive();
    let scope = Scope::for_worker(&ctx.db, &candidate).await?;
    let Some(currency) = candidate.salary_currency.clone() else {
        return format::json(serde_json::json!({
            "position_pid": position.pid,
            "candidate_pid": candidate.pid,
            "answer": rules::Answer::Unknown,
            "missing": ["candidate_pay"],
        }));
    };
    let on_cost = scope.resolve("learning.on_cost_bp", today).integer();

    let needs = seat_needs(&ctx, &position).await?;
    let (steps, undeclared) = match &needs {
        Some(needs) => candidate_steps(&ctx, &candidate, needs).await?,
        None => (None, 0),
    };
    let cost = steps.as_deref().map(rules::plan_cost);
    let hours = cost
        .as_ref()
        .map(|c| hours_for(pay_visible, c.hours, &candidate, &scope, today));
    let hours_minor = hours
        .as_ref()
        .and_then(HoursCost::resolved)
        .filter(|(_, code)| *code == currency)
        .map(|(minor, _)| minor);

    let unit = seat_unit(&ctx, &position, &currency).await?;
    let seat_annual = unit
        .zip(on_cost)
        .map(|((annual, _), bp)| loaded(annual, bp, position.budget_fte_percent));
    let candidate_annual = candidate
        .salary_minor
        .filter(|_| pay_visible)
        .zip(on_cost)
        .map(|(annual, bp)| loaded(annual, bp, position.budget_fte_percent));
    let (vacancy, filled) = vacancy_days(&ctx, &position.organization_ref).await?;

    let train = TrainSide {
        fees_minor: cost.as_ref().and_then(|c| c.single_fee(&currency)),
        hours_minor,
        ramp_up_days: scope.resolve("learning.ramp_up_days", today).integer(),
        annual_minor: candidate_annual,
    };
    let hire = HireSide {
        recruitment_minor: recruitment_in(&scope, &currency, today),
        vacancy_days: vacancy,
        ramp_up_days: scope.resolve("recruitment.ramp_up_days", today).integer(),
        annual_minor: seat_annual,
    };
    let comparison = rules::train_or_hire(train, hire);
    let mut missing: Vec<&str> = comparison.missing.clone();
    if needs.is_none() {
        missing.push("seat_skills");
    }
    if undeclared > 0 {
        missing.push("declared_skills");
    }
    if !pay_visible {
        missing.push("pay_withheld");
    }
    if on_cost.is_none() {
        missing.push("on_cost");
    }
    if pay_visible {
        audit_salary_read(&ctx, &caller, &candidate).await?;
    }
    format::json(serde_json::json!({
        "position_pid": position.pid,
        "candidate_pid": candidate.pid,
        "estimate": true,
        "currency": currency,
        "salary_visible": pay_visible,
        "answer": comparison.answer,
        "similar_margin_bp": comparison.similar_margin_bp,
        "missing": missing,
        "on_cost_bp": on_cost,
        "train": {
            "total_minor": comparison.train_total_minor,
            "fees": cost.as_ref().map(|c| &c.fees),
            "fees_minor": train.fees_minor,
            "courses_without_fee": cost.as_ref().map(|c| c.courses_without_fee),
            "hours": cost.as_ref().map(|c| c.hours),
            "estimated_hours": cost.as_ref().map(|c| c.estimated_hours),
            "hours_cost_minor": train.hours_minor,
            "ramp_up_days": train.ramp_up_days,
            "ramp_up_minor": comparison.train_ramp_up_minor,
            "undeclared_skills": undeclared,
        },
        "hire": {
            "total_minor": comparison.hire_total_minor,
            "recruitment_minor": hire.recruitment_minor,
            "vacancy_days": hire.vacancy_days,
            "filled_requisitions": filled,
            "vacancy_minor": comparison.vacancy_minor,
            "ramp_up_days": hire.ramp_up_days,
            "ramp_up_minor": comparison.hire_ramp_up_minor,
            "seat_annual_minor": hire.annual_minor,
            "seat_cost_source": unit.map(|(_, source)| source.as_str()),
            "pay_difference_minor": comparison.pay_difference_minor,
        },
        "derivation": "Training: the courses that would close the candidate's gaps for the seat's skills, their fees, the \
                       hours at the candidate's loaded rate, and a ramp-up at the candidate's daily cost. Hiring: the \
                       recruitment cost, the organization's median days to fill and a ramp-up at the seat's daily cost, \
                       and one year of the seat's cost less the candidate's. The seat's cost is its title's benchmark, or \
                       its department's average for a cohort large enough to publish, with on-cost at the seat's budgeted \
                       FTE. A daily cost is a 365th of the annual one. Each side's total adds cash and time, so read the \
                       parts as well. An estimate for a person to weigh; WPM decides nothing.",
    }))
}

/// Query for the budget view.
#[derive(Debug, Deserialize)]
struct BudgetQuery {
    organization: String,
    /// The calendar year; default this one.
    year: Option<i32>,
}

/// The one fee a course reference carries, or `None` when it has none or its catalogue rows disagree.
fn fees_by_ref(courses: Vec<skill_courses::Model>) -> HashMap<String, (i64, String)> {
    let mut seen: HashMap<String, BTreeSet<Option<(i64, String)>>> = HashMap::new();
    for c in courses {
        seen.entry(c.course_ref)
            .or_default()
            .insert(c.fee_minor.zip(c.fee_currency));
    }
    seen.into_iter()
        .filter_map(|(course, fees)| {
            let mut fees = fees.into_iter();
            match (fees.next(), fees.next()) {
                (Some(Some(fee)), None) => Some((course, fee)),
                _ => None,
            }
        })
        .collect()
}

/// `GET /api/learning-budgets?organization=&year=` — recorded fees against each department's budget, per currency. Spend is a
/// completed enrollment at its course's catalogue fee, in the year it was completed; a completion with no single fee is
/// counted, not priced, and no other actual is invented.
#[debug_handler]
async fn learning_budget(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<BudgetQuery>,
) -> Result<Response> {
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let year = query.year.unwrap_or_else(|| Utc::now().year());
    let (Some(start), Some(end)) = (
        NaiveDate::from_ymd_opt(year, 1, 1),
        NaiveDate::from_ymd_opt(year, 12, 31),
    ) else {
        return Err(unprocessable("year is out of range"));
    };
    let mut budgets: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();
    for row in learning_budgets::Entity::find()
        .filter(learning_budgets::Column::OrganizationRef.eq(&query.organization))
        .filter(learning_budgets::Column::Year.eq(year))
        .all(&ctx.db)
        .await?
    {
        budgets
            .entry(row.department)
            .or_default()
            .insert(row.currency, row.amount_minor);
    }
    let staff = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&query.organization))
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let pids: Vec<Uuid> = staff.iter().map(|w| w.pid).collect();
    let department: HashMap<Uuid, String> =
        staff.into_iter().map(|w| (w.pid, w.department)).collect();
    let fees = fees_by_ref(
        skill_courses::Entity::find()
            .filter(skill_courses::Column::DeletedAt.is_null())
            .all(&ctx.db)
            .await?,
    );
    let completions = super::find_in(
        &ctx,
        training_enrollments::Entity::find()
            .filter(training_enrollments::Column::DeletedAt.is_null())
            .filter(training_enrollments::Column::Status.eq("completed"))
            .filter(training_enrollments::Column::CompletedOn.gte(start))
            .filter(training_enrollments::Column::CompletedOn.lte(end)),
        training_enrollments::Column::WorkerPid,
        &pids,
    )
    .await?;
    let mut spent: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();
    let mut unpriced: BTreeMap<String, usize> = BTreeMap::new();
    for completion in completions {
        let Some(name) = department.get(&completion.worker_pid) else {
            continue;
        };
        match fees.get(&completion.course_ref) {
            Some((minor, currency)) => {
                let total = spent
                    .entry(name.clone())
                    .or_default()
                    .entry(currency.clone())
                    .or_insert(0);
                *total = total.saturating_add(*minor);
            }
            None => *unpriced.entry(name.clone()).or_insert(0) += 1,
        }
    }
    let names: BTreeSet<&String> = budgets
        .keys()
        .chain(spent.keys())
        .chain(unpriced.keys())
        .collect();
    let empty = BTreeMap::new();
    let departments: Vec<serde_json::Value> = names
        .into_iter()
        .map(|name| {
            serde_json::json!({
                "department": name,
                "lines": rules::budget_lines(
                    budgets.get(name).unwrap_or(&empty),
                    spent.get(name).unwrap_or(&empty),
                ),
                "completions_without_fee": unpriced.get(name).copied().unwrap_or(0),
            })
        })
        .collect();
    format::json(serde_json::json!({
        "organization": query.organization,
        "year": year,
        "departments": departments,
        "derivation": "Spend is each completed enrollment in the year at its course's current catalogue fee, in the person's \
                       current department, per currency and never converted. A completion whose course has no single fee is \
                       counted, not priced. This is recorded spend only; no actual is invented.",
    }))
}

/// `PUT /api/learning-budgets` body.
#[derive(Debug, Deserialize)]
struct BudgetPayload {
    organization_ref: String,
    department: String,
    year: i32,
    amount_minor: i64,
    currency: String,
}

/// `PUT /api/learning-budgets` — set a department's budget for a year and currency.
#[debug_handler]
async fn set_budget(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<BudgetPayload>,
) -> Result<Response> {
    ensure_org_in_scope(&ctx, &caller, &payload.organization_ref).await?;
    let department = payload.department.trim().to_string();
    if department.is_empty() || department.chars().count() > 200 {
        return Err(unprocessable(
            "department is required (up to 200 characters)",
        ));
    }
    if !(2000..=2200).contains(&payload.year) {
        return Err(unprocessable("year must be between 2000 and 2200"));
    }
    if payload.amount_minor < 0 {
        return Err(unprocessable("amount_minor must be at least 0"));
    }
    if !cost_rules::valid_currency(&payload.currency) {
        return Err(unprocessable("currency must be an ISO-4217 code, e.g. GBP"));
    }
    let existing = learning_budgets::Entity::find()
        .filter(learning_budgets::Column::OrganizationRef.eq(&payload.organization_ref))
        .filter(learning_budgets::Column::Department.eq(&department))
        .filter(learning_budgets::Column::Year.eq(payload.year))
        .filter(learning_budgets::Column::Currency.eq(&payload.currency))
        .one(&ctx.db)
        .await?;
    let actor = caller.actor().map(ToString::to_string);
    let txn = ctx.db.begin().await?;
    let row = if let Some(row) = existing {
        let mut active: learning_budgets::ActiveModel = row.into();
        active.amount_minor = ActiveValue::set(payload.amount_minor);
        active.recorded_by = ActiveValue::set(actor);
        active.update(&txn).await?
    } else {
        learning_budgets::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            organization_ref: ActiveValue::set(payload.organization_ref.clone()),
            department: ActiveValue::set(department),
            year: ActiveValue::set(payload.year),
            amount_minor: ActiveValue::set(payload.amount_minor),
            currency: ActiveValue::set(payload.currency.clone()),
            recorded_by: ActiveValue::set(actor),
            ..Default::default()
        }
        .insert(&txn)
        .await?
    };
    Audit::record(
        &txn,
        "learning_budget",
        row.pid,
        "set",
        caller.actor(),
        Some(serde_json::json!({
            "department": row.department,
            "year": row.year,
            "amount_minor": row.amount_minor,
            "currency": row.currency,
        })),
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": row.pid }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/development-plans/{pid}/cost", get(plan_cost))
        .add("/development-plans/{pid}/viability", get(plan_viability))
        .add("/development-plans/{pid}/benefits", post(add_benefit))
        .add("/development-plan-benefits/{pid}", delete(delete_benefit))
        .add("/positions/{pid}/train-or-hire", get(train_or_hire))
        .add("/learning-budgets", get(learning_budget))
        .add("/learning-budgets", put(set_budget))
}
