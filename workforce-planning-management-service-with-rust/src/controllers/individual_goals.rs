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
// `SeaORM` stays only for the three helpers that padr and development call inside their own `SeaORM` transactions
// (`goal_snapshot`, `objectives_of`, `freeze_review_goals`); every handler here is on `sqlx` (WPM-T466).
use sea_orm::{ActiveValue, ConnectionTrait, QueryOrder};
use serde::Deserialize;
use uuid::Uuid;

use super::{ensure_org_in_scope, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{
    goal_check_ins, goal_measures, review_cycles, review_goal_outcomes, reviews, worker_goals,
    workers,
};
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
    let pid = Uuid::parse_str(id).ok();
    Ok(sqlx::query_scalar!(
        "SELECT person_ref FROM workers
          WHERE (upstream_worker_ref = $1 OR ($2::uuid IS NOT NULL AND pid = $2)) AND deleted_at IS NULL",
        review.reviewer_ref,
        pid,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .iter()
    .any(|person_ref| is_self(&claims.sub, person_ref)))
}

/// A live goal and its worker.
async fn goal_and_worker(
    ctx: &AppContext,
    pid: &str,
) -> Result<(worker_goals::Model, workers::Model)> {
    let goal = sqlx::query_as!(
        worker_goals::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, organization_ref, title,
                  description, period_start, period_end, status, proposed_by, agreed_on, skill_pid, skill_target_level,
                  plan_item_pid, aligns_to, deleted_at AS "deleted_at: _"
             FROM worker_goals WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
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
    if goals.is_empty() {
        return Ok(());
    }
    // A read changes nothing, so its entries go on a connection of their own.
    let mut connection = db::pool(ctx).acquire().await?;
    for goal in goals {
        db::audit(
            &mut connection,
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
    for row in sqlx::query_as!(
        goal_measures::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, goal_pid, worker_pid, name, kind,
                  unit, start_value, target_value, current_value, done
             FROM goal_measures WHERE goal_pid = ANY($1) ORDER BY id"#,
        &pids,
    )
    .fetch_all(db::pool(ctx))
    .await?
    {
        measures.entry(row.goal_pid).or_default().push(row);
    }
    let mut check_ins: HashMap<Uuid, Vec<goal_check_ins::Model>> = HashMap::new();
    for row in sqlx::query_as!(
        goal_check_ins::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, goal_pid, worker_pid, checked_on,
                  status, note, recorded_by
             FROM goal_check_ins WHERE goal_pid = ANY($1) ORDER BY id DESC"#,
        &pids,
    )
    .fetch_all(db::pool(ctx))
    .await?
    {
        check_ins.entry(row.goal_pid).or_default().push(row);
    }
    let (skill_names, plan_steps) = link_names(ctx, goals).await?;
    let worker_pids: Vec<Uuid> = goals.iter().map(|g| g.worker_pid).collect();
    let declared: HashMap<(Uuid, Uuid), i32> = sqlx::query!(
        "SELECT worker_pid, skill_pid, proficiency FROM worker_skills
          WHERE worker_pid = ANY($1) AND deleted_at IS NULL ORDER BY id",
        &worker_pids,
    )
    .fetch_all(db::pool(ctx))
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
    let items = sqlx::query!(
        "SELECT pid, skill_pid FROM development_plan_items WHERE pid = ANY($1) ORDER BY id",
        &item_pids,
    )
    .fetch_all(db::pool(ctx))
    .await?;
    let skill_pids: Vec<Uuid> = goals
        .iter()
        .filter_map(|g| g.skill_pid)
        .chain(items.iter().map(|item| item.skill_pid))
        .collect();
    let names: HashMap<Uuid, String> = sqlx::query!(
        "SELECT pid, name FROM skills WHERE pid = ANY($1) AND deleted_at IS NULL",
        &skill_pids,
    )
    .fetch_all(db::pool(ctx))
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
    let plan = sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM development_plan_items i
             JOIN development_plans p ON p.pid = i.plan_pid AND p.worker_pid = $2 AND p.deleted_at IS NULL
            WHERE i.pid = $1
            LIMIT 1"#,
        item,
        worker.pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?;
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
        sqlx::query_scalar!(
            r#"SELECT 1 AS "one!" FROM skills WHERE pid = $1 AND deleted_at IS NULL LIMIT 1"#,
            skill
        )
        .fetch_optional(db::pool(ctx))
        .await?
        .ok_or_else(|| unprocessable("that skill is not in the catalogue"))?;
    }
    if let Some(item) = payload.plan_item_pid {
        check_plan_item(ctx, worker, item).await?;
    }
    Ok(())
}

/// Tell whoever a proposal now waits for. The notification says that and nothing else (WPM-R232, WPM-D23).
async fn tell_waiting(
    tx: &mut sqlx::PgConnection,
    goal: Uuid,
    worker: &workers::Model,
    by: Party,
) -> Result<()> {
    if let Some(recipient) = rules::waiting_on(by, worker.pid, worker.manager_pid) {
        db::notify(
            tx,
            recipient,
            "goal_waiting",
            "A goal is waiting for your agreement.",
            serde_json::json!({ "goal_pid": goal }),
        )
        .await?;
    }
    Ok(())
}

/// Everyone below the subject in the reporting line, still employed.
pub(crate) async fn reports_of(ctx: &AppContext, sub: &str) -> Result<Vec<workers::Model>> {
    let staff = sqlx::query_as!(
        workers::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, person_ref,
                  upstream_worker_ref, organization_ref, worker_number, display_name, status, employment_type,
                  fte_percent, department, job_title, location, work_jurisdiction, manager_pid, salary_minor,
                  salary_currency, hired_on, terminated_on, engagement_ends_on, deleted_at AS "deleted_at: _"
             FROM workers
            WHERE deleted_at IS NULL AND terminated_on IS NULL AND manager_pid IS NOT NULL
            ORDER BY display_name"#,
    )
    .fetch_all(db::pool(ctx))
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
    Ok(sqlx::query_as!(
        worker_goals::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, organization_ref, title,
                  description, period_start, period_end, status, proposed_by, agreed_on, skill_pid, skill_target_level,
                  plan_item_pid, aligns_to, deleted_at AS "deleted_at: _"
             FROM worker_goals WHERE worker_pid = $1 AND deleted_at IS NULL
            ORDER BY period_start DESC, id"#,
        worker,
    )
    .fetch_all(db::pool(ctx))
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
async fn write_measures(
    tx: &mut sqlx::PgConnection,
    goal: Uuid,
    worker: Uuid,
    measures: &[MeasurePayload],
) -> Result<()> {
    sqlx::query!("DELETE FROM goal_measures WHERE goal_pid = $1", goal)
        .execute(&mut *tx)
        .await?;
    for measure in measures {
        sqlx::query!(
            "INSERT INTO goal_measures (pid, goal_pid, worker_pid, name, kind, unit, start_value, target_value)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            Uuid::new_v4(),
            goal,
            worker,
            measure.name.trim(),
            measure.kind,
            measure.unit.as_deref().map(str::trim),
            measure.start_value,
            measure.target_value,
        )
        .execute(&mut *tx)
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
    let pid = Uuid::new_v4();
    let mut tx = db::pool(ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO worker_goals (pid, worker_pid, organization_ref, title, description, period_start, period_end, status,
                                   proposed_by, skill_pid, skill_target_level, plan_item_pid, aligns_to)
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'proposed', $8, $9, $10, $11, $12)",
        pid,
        worker.pid,
        worker.organization_ref,
        payload.title.trim(),
        description,
        payload.period_start,
        payload.period_end,
        by.as_str(),
        payload.skill_pid,
        payload.skill_target_level,
        payload.plan_item_pid,
        payload.aligns_to,
    )
    .execute(&mut *tx)
    .await?;
    write_measures(&mut tx, pid, worker.pid, &payload.measures).await?;
    db::audit(&mut tx, "goal", pid, "proposed", caller.actor(), None).await?;
    tell_waiting(&mut tx, pid, worker, by).await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": pid }))
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
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE worker_goals SET proposed_by = $2, title = $3, description = $4, period_start = $5, period_end = $6,
                skill_pid = $7, skill_target_level = $8, plan_item_pid = $9, aligns_to = $10
          WHERE id = $1",
        goal.id,
        who.as_str(),
        payload.title.trim(),
        description,
        payload.period_start,
        payload.period_end,
        payload.skill_pid,
        skill_target_level,
        payload.plan_item_pid,
        payload.aligns_to,
    )
    .execute(&mut *tx)
    .await?;
    write_measures(&mut tx, goal.pid, goal.worker_pid, &payload.measures).await?;
    db::audit(&mut tx, "goal", goal.pid, "edited", caller.actor(), None).await?;
    if counter_proposal {
        tell_waiting(&mut tx, goal.pid, &worker, who).await?;
    }
    tx.commit().await?;
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
    let mut tx = db::pool(&ctx).begin().await?;
    let status = sqlx::query_scalar!(
        "UPDATE worker_goals SET status = 'agreed', agreed_on = $2 WHERE id = $1 RETURNING status",
        goal.id,
        Utc::now().date_naive(),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(&mut tx, "goal", goal.pid, "agreed", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": goal.pid, "status": status }))
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
    let mut tx = db::pool(&ctx).begin().await?;
    let status = sqlx::query_scalar!(
        "UPDATE worker_goals SET status = $2 WHERE id = $1 RETURNING status",
        goal.id,
        payload.to,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "goal",
        goal.pid,
        "status_changed",
        caller.actor(),
        Some(serde_json::json!({ "from": from, "to": payload.to })),
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": goal.pid, "status": status }))
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
            sqlx::query!(
                "SELECT id, name, kind FROM goal_measures WHERE pid = $1 AND goal_pid = $2 LIMIT 1",
                pid,
                goal.pid,
            )
            .fetch_optional(db::pool(&ctx))
            .await?
            .ok_or(Error::NotFound)?,
        ),
        None => None,
    };
    let mut tx = db::pool(&ctx).begin().await?;
    if let Some(measure) = measure {
        rules::valid_measure(MeasureShape {
            name: &measure.name,
            kind: &measure.kind,
            has_number: payload.current_value.is_some(),
            has_done: payload.done.is_some(),
        })
        .map_err(|e| unprocessable(&e))?;
        // What the check-in leaves out stays as recorded.
        sqlx::query!(
            "UPDATE goal_measures SET current_value = COALESCE($2, current_value), done = COALESCE($3, done) WHERE id = $1",
            measure.id,
            payload.current_value,
            payload.done,
        )
        .execute(&mut *tx)
        .await?;
    }
    let pid = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO goal_check_ins (pid, goal_pid, worker_pid, checked_on, status, note, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        pid,
        goal.pid,
        goal.worker_pid,
        Utc::now().date_naive(),
        payload.status,
        note,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "goal",
        goal.pid,
        "checked_in",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": pid }))
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
    let goals = sqlx::query!(
        "SELECT worker_pid, status FROM worker_goals WHERE organization_ref = $1 AND deleted_at IS NULL ORDER BY id",
        query.organization,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let department: HashMap<Uuid, String> = sqlx::query!(
        "SELECT pid, department FROM workers WHERE organization_ref = $1",
        query.organization,
    )
    .fetch_all(db::pool(&ctx))
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
    // A live performance review: a PADR answers only on its own routes (WPM-D176).
    let review = sqlx::query_as!(
        reviews::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, cycle_pid, kind, worker_pid,
                  reviewer_ref, status, rating, content, deleted_at AS "deleted_at: _"
             FROM reviews WHERE pid = $1 AND kind = 'performance' AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let cycle = sqlx::query_as!(
        review_cycles::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, period_start, period_end,
                  status, deleted_at AS "deleted_at: _"
             FROM review_cycles WHERE pid = $1 LIMIT 1"#,
        review.cycle_pid.ok_or(Error::NotFound)?,
    )
    .fetch_optional(db::pool(ctx))
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
    let outcomes = sqlx::query_as!(
        review_goal_outcomes::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, review_pid, goal_pid, worker_pid,
                  reviewer_outcome, reviewee_outcome, snapshot
             FROM review_goal_outcomes WHERE review_pid = $1 ORDER BY id"#,
        review.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    if review.status == "shared" {
        let mut connection = db::pool(&ctx).acquire().await?;
        for outcome in &outcomes {
            db::audit(
                &mut connection,
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
    let existing = sqlx::query_scalar!(
        "SELECT id FROM review_goal_outcomes WHERE review_pid = $1 AND goal_pid = $2 LIMIT 1",
        review.pid,
        goal.pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?;
    // Each party writes only their own outcome; the other's stays as recorded.
    let reviewer_says = the_reviewer.then(|| payload.outcome.clone());
    let person_says = the_person.then(|| payload.outcome.clone());
    let mut tx = db::pool(&ctx).begin().await?;
    if let Some(id) = existing {
        sqlx::query!(
            "UPDATE review_goal_outcomes
                SET reviewer_outcome = COALESCE($2, reviewer_outcome), reviewee_outcome = COALESCE($3, reviewee_outcome)
              WHERE id = $1",
            id,
            reviewer_says,
            person_says,
        )
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query!(
            "INSERT INTO review_goal_outcomes (pid, review_pid, goal_pid, worker_pid, reviewer_outcome, reviewee_outcome)
             VALUES ($1, $2, $3, $4, $5, $6)",
            Uuid::new_v4(),
            review.pid,
            goal.pid,
            goal.worker_pid,
            reviewer_says,
            person_says,
        )
        .execute(&mut *tx)
        .await?;
    }
    db::audit(
        &mut tx,
        "goal",
        goal.pid,
        "outcome_recorded",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
