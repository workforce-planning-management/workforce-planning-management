//! **Individual goals** ([`crate::rules::individual_goals`], WPM-R224 to WPM-R226, WPM-D163 to WPM-D166).
//!
//! A goal is one person's, and the person or a manager in their chain proposes it and the other agrees. HR sees and does not
//! agree, so every write is under `/api/me`, resolved from the verified token: the reference policy lets only HR write, and
//! neither party here is HR. Goals are as confidential as review content: the person, their managers and HR of the organization
//! read them, payroll does not, and each read is audited with the goal's pid and no content.
//!
//! Nothing here computes an attainment, a rate or a rank (WPM-D163), and no pay, progression or employment decision is made from
//! a goal. A review lists the goals of its period, the reviewer and the reviewee each record an outcome, and a review that is
//! shared freezes a snapshot (WPM-D165).

use std::collections::HashMap;

use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::Utc;
use sea_orm::{ActiveValue, Condition, ConnectionTrait, QueryOrder, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;

use super::{ensure_org_in_scope, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{
    development_plan_items, development_plans, goal_check_ins, goal_measures, review_cycles,
    review_goal_outcomes, reviews, skills, worker_goals, worker_skills, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::notifications::Model as Notification;
use crate::models::records;
use crate::need_to_know::is_line_manager;
use crate::rules::career::is_self;
use crate::rules::individual_goals::{self as rules, MeasureShape, Party};

/// The caller's token subject.
pub(crate) fn subject(caller: &MaybeAuthUser) -> Result<&str> {
    caller
        .claims()
        .map(|c| c.sub.as_str())
        .ok_or(Error::CustomError(
            axum::http::StatusCode::UNAUTHORIZED,
            axa::error::ErrorDetail::new("unauthorized", "sign in to use /api/me"),
        ))
}

/// Who the caller is to this worker's goals, if anyone.
async fn party(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
) -> Result<Option<Party>> {
    let sub = subject(caller)?;
    if is_self(sub, &worker.person_ref) {
        return Ok(Some(Party::Worker));
    }
    Ok(is_line_manager(ctx, worker, sub)
        .await?
        .then_some(Party::Manager))
}

/// The party, or a 404: the record is not there for anyone else.
async fn require_party(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
) -> Result<Party> {
    party(ctx, caller, worker).await?.ok_or(Error::NotFound)
}

/// The person, a manager in their chain, or HR of the organization; payroll and everyone else are refused (WPM-D164). With
/// enforcement off there is no identity to compare, so the policy's own answer stands.
async fn ensure_reader(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
) -> Result<()> {
    if auth::require_auth() && party(ctx, caller, worker).await?.is_some() {
        return Ok(());
    }
    auth::authorize_record(
        caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(worker),
    )
    .map_err(record_rejection)?;
    ensure_org_in_scope(ctx, caller, &worker.organization_ref).await
}

/// Whether the caller is this review's reviewer. `reviewer_ref` is a `worker:` reference, to a worker's pid or to the upstream
/// reference that worker carries.
pub(crate) async fn is_reviewer(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    review: &reviews::Model,
) -> Result<bool> {
    let Some(claims) = caller.claims() else {
        return Ok(false);
    };
    let id = review.reviewer_ref.rsplit(':').next().unwrap_or_default();
    let mut matching =
        Condition::any().add(workers::Column::UpstreamWorkerRef.eq(review.reviewer_ref.clone()));
    if let Ok(pid) = Uuid::parse_str(id) {
        matching = matching.add(workers::Column::Pid.eq(pid));
    }
    Ok(workers::Entity::find()
        .filter(matching)
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .iter()
        .any(|w| is_self(&claims.sub, &w.person_ref)))
}

/// A live goal and its worker.
async fn goal_and_worker(
    ctx: &AppContext,
    pid: &str,
) -> Result<(worker_goals::Model, workers::Model)> {
    let goal = worker_goals::Entity::find()
        .filter(worker_goals::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(worker_goals::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let worker = records::worker_by_pid(ctx, goal.worker_pid).await?;
    Ok((goal, worker))
}

/// Each read leaves one entry per goal, with the pid and no content (WPM-D164).
pub(crate) async fn audit_reads(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    goals: &[worker_goals::Model],
) -> Result<()> {
    for goal in goals {
        Audit::record(
            &ctx.db,
            "goal",
            goal.pid,
            "goals_read",
            caller.actor(),
            None,
        )
        .await?;
    }
    Ok(())
}

/// Goals with their measures and check-ins, as a reader sees them.
pub(crate) async fn views(
    ctx: &AppContext,
    goals: &[worker_goals::Model],
) -> Result<Vec<serde_json::Value>> {
    let pids: Vec<Uuid> = goals.iter().map(|g| g.pid).collect();
    let mut measures: HashMap<Uuid, Vec<goal_measures::Model>> = HashMap::new();
    for row in super::find_in(
        ctx,
        goal_measures::Entity::find().order_by_asc(goal_measures::Column::Id),
        goal_measures::Column::GoalPid,
        &pids,
    )
    .await?
    {
        measures.entry(row.goal_pid).or_default().push(row);
    }
    let mut check_ins: HashMap<Uuid, Vec<goal_check_ins::Model>> = HashMap::new();
    for row in super::find_in(
        ctx,
        goal_check_ins::Entity::find().order_by_desc(goal_check_ins::Column::Id),
        goal_check_ins::Column::GoalPid,
        &pids,
    )
    .await?
    {
        check_ins.entry(row.goal_pid).or_default().push(row);
    }
    let (skill_names, plan_steps) = link_names(ctx, goals).await?;
    let worker_pids: Vec<Uuid> = goals.iter().map(|g| g.worker_pid).collect();
    let declared: HashMap<(Uuid, Uuid), i32> = super::find_in(
        ctx,
        worker_skills::Entity::find().filter(worker_skills::Column::DeletedAt.is_null()),
        worker_skills::Column::WorkerPid,
        &worker_pids,
    )
    .await?
    .into_iter()
    .map(|row| ((row.worker_pid, row.skill_pid), row.proficiency))
    .collect();
    Ok(goals
        .iter()
        .map(|goal| {
            serde_json::json!({
                "goal": goal,
                "measures": measures.get(&goal.pid).cloned().unwrap_or_default(),
                "check_ins": check_ins.get(&goal.pid).cloned().unwrap_or_default(),
                "skill_name": goal.skill_pid.and_then(|pid| skill_names.get(&pid)),
                "plan_step": goal.plan_item_pid.and_then(|pid| plan_steps.get(&pid)),
                "skill_evidence": goal.skill_pid.zip(goal.skill_target_level).map(|(skill, target)| {
                    let level = declared.get(&(goal.worker_pid, skill)).copied();
                    serde_json::json!({
                        "target": target,
                        "declared": level,
                        "state": rules::skill_evidence(level, target),
                    })
                }),
            })
        })
        .collect())
}

/// The names a goal's links show: a skill's, and a plan step's, which is the skill it builds. A link whose target has gone shows
/// nothing rather than a pid.
async fn link_names(
    ctx: &AppContext,
    goals: &[worker_goals::Model],
) -> Result<(HashMap<Uuid, String>, HashMap<Uuid, String>)> {
    let item_pids: Vec<Uuid> = goals.iter().filter_map(|g| g.plan_item_pid).collect();
    let items = super::find_in(
        ctx,
        development_plan_items::Entity::find(),
        development_plan_items::Column::Pid,
        &item_pids,
    )
    .await?;
    let skill_pids: Vec<Uuid> = goals
        .iter()
        .filter_map(|g| g.skill_pid)
        .chain(items.iter().map(|item| item.skill_pid))
        .collect();
    let names: HashMap<Uuid, String> = super::find_in(
        ctx,
        skills::Entity::find().filter(skills::Column::DeletedAt.is_null()),
        skills::Column::Pid,
        &skill_pids,
    )
    .await?
    .into_iter()
    .map(|skill| (skill.pid, skill.name))
    .collect();
    let steps = items
        .iter()
        .filter_map(|item| Some((item.pid, names.get(&item.skill_pid)?.clone())))
        .collect();
    Ok((names, steps))
}

/// What a goal links to must exist, and a plan item must be on one of this person's own plans.
pub(crate) async fn check_plan_item(
    ctx: &AppContext,
    worker: &workers::Model,
    item: Uuid,
) -> Result<()> {
    let plan = match development_plan_items::Entity::find()
        .filter(development_plan_items::Column::Pid.eq(item))
        .one(&ctx.db)
        .await?
    {
        Some(row) => {
            development_plans::Entity::find()
                .filter(development_plans::Column::Pid.eq(row.plan_pid))
                .filter(development_plans::Column::WorkerPid.eq(worker.pid))
                .filter(development_plans::Column::DeletedAt.is_null())
                .one(&ctx.db)
                .await?
        }
        None => None,
    };
    plan.map(|_| ())
        .ok_or_else(|| unprocessable("that plan item is not on this person's plans"))
}

/// Check a goal's skill and plan item.
async fn check_links(
    ctx: &AppContext,
    worker: &workers::Model,
    payload: &GoalPayload,
) -> Result<()> {
    if let Some(skill) = payload.skill_pid {
        skills::Entity::find()
            .filter(skills::Column::Pid.eq(skill))
            .filter(skills::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?
            .ok_or_else(|| unprocessable("that skill is not in the catalogue"))?;
    }
    if let Some(item) = payload.plan_item_pid {
        check_plan_item(ctx, worker, item).await?;
    }
    Ok(())
}

/// Tell whoever a proposal now waits for. The notification says that and nothing else (WPM-R232, WPM-D23).
async fn tell_waiting<C: ConnectionTrait>(
    db: &C,
    goal: &worker_goals::Model,
    worker: &workers::Model,
    by: Party,
) -> Result<()> {
    if let Some(recipient) = rules::waiting_on(by, worker.pid, worker.manager_pid) {
        Notification::push(
            db,
            recipient,
            "goal_waiting",
            "A goal is waiting for your agreement.",
            serde_json::json!({ "goal_pid": goal.pid }),
        )
        .await?;
    }
    Ok(())
}

/// Everyone below the subject in the reporting line, still employed.
pub(crate) async fn reports_of(ctx: &AppContext, sub: &str) -> Result<Vec<workers::Model>> {
    let staff = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::TerminatedOn.is_null())
        .filter(workers::Column::ManagerPid.is_not_null())
        .order_by_asc(workers::Column::DisplayName)
        .all(&ctx.db)
        .await?;
    let mut reports = Vec::new();
    for worker in staff {
        if is_line_manager(ctx, &worker, sub).await? {
            reports.push(worker);
        }
    }
    Ok(reports)
}

/// A worker's live goals, newest period first.
async fn goals_of(ctx: &AppContext, worker: Uuid) -> Result<Vec<worker_goals::Model>> {
    Ok(worker_goals::Entity::find()
        .filter(worker_goals::Column::WorkerPid.eq(worker))
        .filter(worker_goals::Column::DeletedAt.is_null())
        .order_by_desc(worker_goals::Column::PeriodStart)
        .order_by_asc(worker_goals::Column::Id)
        .all(&ctx.db)
        .await?)
}

/// `GET /api/workers/{pid}/goals` — the person, a manager in their chain, or HR of the organization. Audited.
#[debug_handler]
async fn worker_goals_view(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    ensure_reader(&ctx, &caller, &worker).await?;
    let goals = goals_of(&ctx, worker.pid).await?;
    audit_reads(&ctx, &caller, &goals).await?;
    format::json(serde_json::json!({
        "worker_pid": worker.pid,
        "has_manager": worker.manager_pid.is_some(),
        "goals": views(&ctx, &goals).await?,
    }))
}

/// `GET /api/me/team-goals` — everyone below the caller in the reporting line, by name, with their goals. Someone with none is
/// listed too, so a first goal has somewhere to go. Audited.
#[debug_handler]
pub(crate) async fn team_goals(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
) -> Result<Response> {
    let sub = subject(&caller)?.to_string();
    let mut team = Vec::new();
    for worker in reports_of(&ctx, &sub).await? {
        let goals = goals_of(&ctx, worker.pid).await?;
        audit_reads(&ctx, &caller, &goals).await?;
        team.push(serde_json::json!({
            "worker_pid": worker.pid,
            "display_name": worker.display_name,
            "goals": views(&ctx, &goals).await?,
        }));
    }
    format::json(serde_json::json!({ "team": team }))
}

/// One measure as proposed.
#[derive(Debug, Deserialize)]
pub(crate) struct MeasurePayload {
    name: String,
    kind: String,
    #[serde(default)]
    unit: Option<String>,
    #[serde(default)]
    start_value: Option<i64>,
    #[serde(default)]
    target_value: Option<i64>,
}

/// A goal as proposed or edited.
#[derive(Debug, Deserialize)]
pub(crate) struct GoalPayload {
    title: String,
    #[serde(default)]
    description: Option<String>,
    period_start: chrono::NaiveDate,
    period_end: chrono::NaiveDate,
    #[serde(default)]
    skill_pid: Option<Uuid>,
    /// The level the goal aims at in that skill.
    #[serde(default)]
    skill_target_level: Option<i32>,
    #[serde(default)]
    plan_item_pid: Option<Uuid>,
    /// An opaque reference to an objective held elsewhere; never resolved (WPM-D166).
    #[serde(default)]
    aligns_to: Option<String>,
    #[serde(default)]
    measures: Vec<MeasurePayload>,
}

/// What is wrong with a goal, if anything; its tidied description.
fn checked(payload: &GoalPayload) -> Result<Option<String>> {
    let reject = |e: String| unprocessable(&e);
    rules::valid_goal(&payload.title, payload.period_start, payload.period_end).map_err(reject)?;
    rules::valid_skill_target(payload.skill_pid.is_some(), payload.skill_target_level)
        .map_err(reject)?;
    if let Some(reference) = &payload.aligns_to {
        rules::valid_alignment(reference).map_err(reject)?;
    }
    for measure in &payload.measures {
        rules::valid_measure(MeasureShape {
            name: &measure.name,
            kind: &measure.kind,
            has_number: measure.start_value.is_some() || measure.target_value.is_some(),
            has_done: false,
        })
        .map_err(reject)?;
        rules::clean_text(measure.unit.as_deref(), rules::MAX_TITLE_CHARS).map_err(reject)?;
    }
    rules::clean_text(payload.description.as_deref(), rules::MAX_NOTE_CHARS).map_err(reject)
}

/// Replace a goal's measures.
async fn write_measures<C: ConnectionTrait>(
    db: &C,
    goal: &worker_goals::Model,
    measures: &[MeasurePayload],
) -> Result<()> {
    goal_measures::Entity::delete_many()
        .filter(goal_measures::Column::GoalPid.eq(goal.pid))
        .exec(db)
        .await?;
    for measure in measures {
        goal_measures::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            goal_pid: ActiveValue::set(goal.pid),
            worker_pid: ActiveValue::set(goal.worker_pid),
            name: ActiveValue::set(measure.name.trim().to_string()),
            kind: ActiveValue::set(measure.kind.clone()),
            unit: ActiveValue::set(measure.unit.as_deref().map(str::trim).map(String::from)),
            start_value: ActiveValue::set(measure.start_value),
            target_value: ActiveValue::set(measure.target_value),
            current_value: ActiveValue::set(None),
            done: ActiveValue::set(None),
            ..Default::default()
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

/// Propose a goal for `worker`.
async fn propose(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
    by: Party,
    payload: &GoalPayload,
) -> Result<Response> {
    let description = checked(payload)?;
    check_links(ctx, worker, payload).await?;
    let txn = ctx.db.begin().await?;
    let goal = worker_goals::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        organization_ref: ActiveValue::set(worker.organization_ref.clone()),
        title: ActiveValue::set(payload.title.trim().to_string()),
        description: ActiveValue::set(description),
        period_start: ActiveValue::set(payload.period_start),
        period_end: ActiveValue::set(payload.period_end),
        status: ActiveValue::set("proposed".to_string()),
        proposed_by: ActiveValue::set(by.as_str().to_string()),
        agreed_on: ActiveValue::set(None),
        skill_pid: ActiveValue::set(payload.skill_pid),
        skill_target_level: ActiveValue::set(payload.skill_target_level),
        plan_item_pid: ActiveValue::set(payload.plan_item_pid),
        aligns_to: ActiveValue::set(payload.aligns_to.clone()),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    write_measures(&txn, &goal, &payload.measures).await?;
    Audit::record(&txn, "goal", goal.pid, "proposed", caller.actor(), None).await?;
    tell_waiting(&txn, &goal, worker, by).await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": goal.pid }))
}

/// `POST /api/me/goals` — propose a goal of your own.
#[debug_handler]
pub(crate) async fn propose_mine(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<GoalPayload>,
) -> Result<Response> {
    let worker = super::me::me_worker(&ctx, &caller).await?;
    propose(&ctx, &caller, &worker, Party::Worker, &payload).await
}

/// `POST /api/me/reports/{pid}/goals` — propose a goal for someone in your reporting line.
#[debug_handler]
pub(crate) async fn propose_for_report(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<GoalPayload>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    if require_party(&ctx, &caller, &worker).await? != Party::Manager {
        return Err(Error::NotFound);
    }
    propose(&ctx, &caller, &worker, Party::Manager, &payload).await
}

/// `PUT /api/me/goals/{pid}` — change a goal until it is agreed; after that it is what both agreed to. A change by the party who did
/// not propose it makes them the proposer, so the other must agree the new words (WPM-D177).
#[debug_handler]
pub(crate) async fn edit(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<GoalPayload>,
) -> Result<Response> {
    let (goal, worker) = goal_and_worker(&ctx, &pid).await?;
    let who = require_party(&ctx, &caller, &worker).await?;
    if !rules::is_editable(&goal.status) {
        return Err(unprocessable(
            "an agreed goal cannot be edited; withdraw it and propose another",
        ));
    }
    let description = checked(&payload)?;
    check_links(&ctx, &worker, &payload).await?;
    let counter_proposal = goal.proposed_by != who.as_str();
    // A form that does not show the target keeps it, as with measures; a different skill drops it.
    let skill_target_level =
        payload
            .skill_target_level
            .or(if payload.skill_pid == goal.skill_pid {
                goal.skill_target_level
            } else {
                None
            });
    let mut active: worker_goals::ActiveModel = goal.into();
    active.proposed_by = ActiveValue::set(who.as_str().to_string());
    active.title = ActiveValue::set(payload.title.trim().to_string());
    active.description = ActiveValue::set(description);
    active.period_start = ActiveValue::set(payload.period_start);
    active.period_end = ActiveValue::set(payload.period_end);
    active.skill_pid = ActiveValue::set(payload.skill_pid);
    active.skill_target_level = ActiveValue::set(skill_target_level);
    active.plan_item_pid = ActiveValue::set(payload.plan_item_pid);
    active.aligns_to = ActiveValue::set(payload.aligns_to.clone());
    let txn = ctx.db.begin().await?;
    let goal = active.update(&txn).await?;
    write_measures(&txn, &goal, &payload.measures).await?;
    Audit::record(&txn, "goal", goal.pid, "edited", caller.actor(), None).await?;
    if counter_proposal {
        tell_waiting(&txn, &goal, &worker, who).await?;
    }
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": goal.pid }))
}

/// `POST /api/me/goals/{pid}/agree` — the other party agrees. HR is neither party.
#[debug_handler]
pub(crate) async fn agree(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (goal, worker) = goal_and_worker(&ctx, &pid).await?;
    let who = require_party(&ctx, &caller, &worker).await?;
    if !rules::may_agree(&goal.status, &goal.proposed_by, who) {
        return Err(unprocessable(
            "only the other party can agree a proposed goal",
        ));
    }
    let mut active: worker_goals::ActiveModel = goal.into();
    active.status = ActiveValue::set("agreed".to_string());
    active.agreed_on = ActiveValue::set(Some(Utc::now().date_naive()));
    let txn = ctx.db.begin().await?;
    let goal = active.update(&txn).await?;
    Audit::record(&txn, "goal", goal.pid, "agreed", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": goal.pid, "status": goal.status }))
}

/// `POST /api/me/goals/{pid}/status` body.
#[derive(Debug, Deserialize)]
pub(crate) struct StatusPayload {
    to: String,
}

/// `POST /api/me/goals/{pid}/status` — move a goal on.
#[debug_handler]
pub(crate) async fn move_status(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<StatusPayload>,
) -> Result<Response> {
    let (goal, worker) = goal_and_worker(&ctx, &pid).await?;
    require_party(&ctx, &caller, &worker).await?;
    if !rules::may_move(&goal.status, &payload.to) {
        return Err(unprocessable(
            "a goal cannot move to that status from this one",
        ));
    }
    let from = goal.status.clone();
    let mut active: worker_goals::ActiveModel = goal.into();
    active.status = ActiveValue::set(payload.to.clone());
    let txn = ctx.db.begin().await?;
    let goal = active.update(&txn).await?;
    Audit::record(
        &txn,
        "goal",
        goal.pid,
        "status_changed",
        caller.actor(),
        Some(serde_json::json!({ "from": from, "to": payload.to })),
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": goal.pid, "status": goal.status }))
}

/// `POST /api/me/goals/{pid}/check-ins` body.
#[derive(Debug, Deserialize)]
pub(crate) struct CheckInPayload {
    status: String,
    #[serde(default)]
    note: Option<String>,
    /// The measure this check-in updates, if any.
    #[serde(default)]
    measure_pid: Option<Uuid>,
    #[serde(default)]
    current_value: Option<i64>,
    #[serde(default)]
    done: Option<bool>,
}

/// `POST /api/me/goals/{pid}/check-ins` — a dated entry on an open goal, optionally updating one of its measures.
#[debug_handler]
pub(crate) async fn check_in(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<CheckInPayload>,
) -> Result<Response> {
    let (goal, worker) = goal_and_worker(&ctx, &pid).await?;
    require_party(&ctx, &caller, &worker).await?;
    if !rules::is_open(&goal.status) {
        return Err(unprocessable(
            "a check-in needs an agreed goal that is not finished",
        ));
    }
    if !rules::CHECK_IN_STATUSES.contains(&payload.status.as_str()) {
        return Err(unprocessable(
            "status must be on_track, at_risk or off_track",
        ));
    }
    let note = rules::clean_text(payload.note.as_deref(), rules::MAX_NOTE_CHARS)
        .map_err(|e| unprocessable(&e))?;
    let measure = match payload.measure_pid {
        Some(pid) => Some(
            goal_measures::Entity::find()
                .filter(goal_measures::Column::Pid.eq(pid))
                .filter(goal_measures::Column::GoalPid.eq(goal.pid))
                .one(&ctx.db)
                .await?
                .ok_or(Error::NotFound)?,
        ),
        None => None,
    };
    let txn = ctx.db.begin().await?;
    if let Some(measure) = measure {
        rules::valid_measure(MeasureShape {
            name: &measure.name,
            kind: &measure.kind,
            has_number: payload.current_value.is_some(),
            has_done: payload.done.is_some(),
        })
        .map_err(|e| unprocessable(&e))?;
        let mut active: goal_measures::ActiveModel = measure.into();
        if payload.current_value.is_some() {
            active.current_value = ActiveValue::set(payload.current_value);
        }
        if payload.done.is_some() {
            active.done = ActiveValue::set(payload.done);
        }
        active.update(&txn).await?;
    }
    let row = goal_check_ins::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        goal_pid: ActiveValue::set(goal.pid),
        worker_pid: ActiveValue::set(goal.worker_pid),
        checked_on: ActiveValue::set(Utc::now().date_naive()),
        status: ActiveValue::set(payload.status.clone()),
        note: ActiveValue::set(note),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(&txn, "goal", goal.pid, "checked_in", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": row.pid }))
}

/// Query for the roll-up.
#[derive(Debug, Deserialize)]
struct RollupQuery {
    organization: String,
}

/// `GET /api/goals/rollup?organization=` — goals by status and department, with the floor. Counts only.
#[debug_handler]
async fn rollup(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(query): axa::extract::Query<RollupQuery>,
) -> Result<Response> {
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let goals = worker_goals::Entity::find()
        .filter(worker_goals::Column::OrganizationRef.eq(&query.organization))
        .filter(worker_goals::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let department: HashMap<Uuid, String> = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&query.organization))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| (w.pid, w.department))
        .collect();
    let rows: Vec<rules::RollupRow> = goals
        .into_iter()
        .filter_map(|g| {
            Some(rules::RollupRow {
                department: department.get(&g.worker_pid)?.clone(),
                worker: g.worker_pid,
                status: g.status,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "organization": query.organization,
        "departments": rules::rollup(&rows),
        "floor": rules::ROLLUP_FLOOR,
        "derivation": "Goals per status in each department. A department with goals from too few people is withheld, and so \
                       is the next smallest when exactly one is. Counts only: no rate, no score, and nobody named.",
    }))
}

/// The review and its period.
async fn review_and_period(
    ctx: &AppContext,
    pid: &str,
) -> Result<(reviews::Model, review_cycles::Model)> {
    let review = records::find_review(&ctx.db, records::parse_pid(pid)?).await?;
    let cycle = review_cycles::Entity::find()
        .filter(review_cycles::Column::Pid.eq(review.cycle_pid.ok_or(Error::NotFound)?))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    Ok((review, cycle))
}

/// The worker's goals overlapping the cycle's period.
fn in_period(
    goals: Vec<worker_goals::Model>,
    cycle: &review_cycles::Model,
) -> Vec<worker_goals::Model> {
    goals
        .into_iter()
        .filter(|g| {
            rules::overlaps(
                (g.period_start, g.period_end),
                (cycle.period_start, cycle.period_end),
            )
        })
        .collect()
}

/// `GET /api/reviews/{pid}/individual-goals` — the goals of the review's period with both outcomes. A shared review answers
/// from what it froze, so a later edit to a goal changes nothing here.
#[debug_handler]
async fn review_goals(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (review, cycle) = review_and_period(&ctx, &pid).await?;
    let worker = records::worker_by_pid(&ctx, review.worker_pid).await?;
    if !is_reviewer(&ctx, &caller, &review).await? {
        ensure_reader(&ctx, &caller, &worker).await?;
    }
    let outcomes = review_goal_outcomes::Entity::find()
        .filter(review_goal_outcomes::Column::ReviewPid.eq(review.pid))
        .all(&ctx.db)
        .await?;
    if review.status == "shared" {
        for outcome in &outcomes {
            Audit::record(
                &ctx.db,
                "goal",
                outcome.goal_pid,
                "goals_read",
                caller.actor(),
                None,
            )
            .await?;
        }
        return format::json(serde_json::json!({
            "review_pid": review.pid,
            "frozen": true,
            "goals": outcomes,
        }));
    }
    let goals = in_period(goals_of(&ctx, review.worker_pid).await?, &cycle);
    audit_reads(&ctx, &caller, &goals).await?;
    let by_goal: HashMap<Uuid, &review_goal_outcomes::Model> =
        outcomes.iter().map(|o| (o.goal_pid, o)).collect();
    let rows = views(&ctx, &goals).await?;
    let goals_json: Vec<serde_json::Value> = goals
        .iter()
        .zip(rows)
        .map(|(goal, view)| {
            let outcome = by_goal.get(&goal.pid);
            serde_json::json!({
                "goal_pid": goal.pid,
                "view": view,
                "reviewer_outcome": outcome.and_then(|o| o.reviewer_outcome.clone()),
                "reviewee_outcome": outcome.and_then(|o| o.reviewee_outcome.clone()),
            })
        })
        .collect();
    format::json(serde_json::json!({
        "review_pid": review.pid,
        "frozen": false,
        "goals": goals_json,
    }))
}

/// `PUT /api/me/reviews/{pid}/goals/{goal}/outcome` body.
#[derive(Debug, Deserialize)]
pub(crate) struct OutcomePayload {
    outcome: String,
}

/// `PUT /api/me/reviews/{pid}/goals/{goal}/outcome` — the reviewer and the reviewee each record their own outcome, and neither
/// can write the other's. A shared review is frozen.
#[debug_handler]
pub(crate) async fn record_outcome(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, goal_pid)): Path<(String, String)>,
    Json(payload): Json<OutcomePayload>,
) -> Result<Response> {
    let (review, cycle) = review_and_period(&ctx, &pid).await?;
    let (goal, worker) = goal_and_worker(&ctx, &goal_pid).await?;
    if goal.worker_pid != review.worker_pid
        || !rules::overlaps(
            (goal.period_start, goal.period_end),
            (cycle.period_start, cycle.period_end),
        )
    {
        return Err(Error::NotFound);
    }
    let sub = subject(&caller)?;
    let the_person = is_self(sub, &worker.person_ref);
    let the_reviewer = is_reviewer(&ctx, &caller, &review).await?;
    if !the_person && !the_reviewer {
        return Err(Error::NotFound);
    }
    if review.status == "shared" {
        return Err(unprocessable(
            "this review is shared, so its goals are frozen",
        ));
    }
    if !rules::OUTCOMES.contains(&payload.outcome.as_str()) {
        return Err(unprocessable(
            "outcome must be achieved, partly_achieved, not_achieved or withdrawn",
        ));
    }
    let existing = review_goal_outcomes::Entity::find()
        .filter(review_goal_outcomes::Column::ReviewPid.eq(review.pid))
        .filter(review_goal_outcomes::Column::GoalPid.eq(goal.pid))
        .one(&ctx.db)
        .await?;
    let txn = ctx.db.begin().await?;
    let mut active = existing.map_or_else(
        || review_goal_outcomes::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            review_pid: ActiveValue::set(review.pid),
            goal_pid: ActiveValue::set(goal.pid),
            worker_pid: ActiveValue::set(goal.worker_pid),
            ..Default::default()
        },
        Into::into,
    );
    if the_reviewer {
        active.reviewer_outcome = ActiveValue::set(Some(payload.outcome.clone()));
    }
    if the_person {
        active.reviewee_outcome = ActiveValue::set(Some(payload.outcome.clone()));
    }
    active.save(&txn).await?;
    Audit::record(
        &txn,
        "goal",
        goal.pid,
        "outcome_recorded",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "review_pid": review.pid, "goal_pid": goal.pid }))
}

/// What a signed review keeps of a goal (WPM-R225).
pub(crate) async fn goal_snapshot<C: ConnectionTrait>(
    db: &C,
    goal: &worker_goals::Model,
) -> Result<serde_json::Value> {
    let measures = goal_measures::Entity::find()
        .filter(goal_measures::Column::GoalPid.eq(goal.pid))
        .order_by_asc(goal_measures::Column::Id)
        .all(db)
        .await?;
    Ok(serde_json::json!({
        "pid": goal.pid,
        "title": goal.title,
        "status": goal.status,
        "period_start": goal.period_start,
        "period_end": goal.period_end,
        "measures": measures,
    }))
}

/// A PADR's objectives (WPM-R225): the person's live goals in its period, withdrawn ones left out.
pub(crate) async fn objectives_of<C: ConnectionTrait>(
    db: &C,
    worker: Uuid,
    period: (chrono::NaiveDate, chrono::NaiveDate),
) -> Result<Vec<worker_goals::Model>> {
    Ok(worker_goals::Entity::find()
        .filter(worker_goals::Column::WorkerPid.eq(worker))
        .filter(worker_goals::Column::DeletedAt.is_null())
        .order_by_asc(worker_goals::Column::Id)
        .all(db)
        .await?
        .into_iter()
        .filter(|g| rules::is_objective(&g.status, (g.period_start, g.period_end), period))
        .collect())
}

/// Freeze the goals of a review as it is shared (WPM-D165): each goal in the period gets a snapshot beside its outcomes, in the
/// transaction that shares it. A later edit to the goal never reaches a shared review.
pub(crate) async fn freeze_review_goals<C: ConnectionTrait>(
    db: &C,
    review: &reviews::Model,
) -> Result<()> {
    let Some(cycle_pid) = review.cycle_pid else {
        return Ok(());
    };
    let Some(cycle) = review_cycles::Entity::find()
        .filter(review_cycles::Column::Pid.eq(cycle_pid))
        .one(db)
        .await?
    else {
        return Ok(());
    };
    let goals = worker_goals::Entity::find()
        .filter(worker_goals::Column::WorkerPid.eq(review.worker_pid))
        .filter(worker_goals::Column::DeletedAt.is_null())
        .all(db)
        .await?;
    for goal in in_period(goals, &cycle) {
        let snapshot = goal_snapshot(db, &goal).await?;
        let existing = review_goal_outcomes::Entity::find()
            .filter(review_goal_outcomes::Column::ReviewPid.eq(review.pid))
            .filter(review_goal_outcomes::Column::GoalPid.eq(goal.pid))
            .one(db)
            .await?;
        let mut active = existing.map_or_else(
            || review_goal_outcomes::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                review_pid: ActiveValue::set(review.pid),
                goal_pid: ActiveValue::set(goal.pid),
                worker_pid: ActiveValue::set(goal.worker_pid),
                ..Default::default()
            },
            Into::into,
        );
        active.snapshot = ActiveValue::set(Some(snapshot));
        active.save(db).await?;
    }
    Ok(())
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/goals", get(worker_goals_view))
        .add("/goals/rollup", get(rollup))
        .add("/reviews/{pid}/individual-goals", get(review_goals))
}
