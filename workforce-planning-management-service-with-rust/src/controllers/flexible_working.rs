//! **Flexible working requests** (WPM-R126, WPM-D77): a worker asks for a different working
//! arrangement; a person decides. Pure rules in [`crate::rules::flexible_working`].
//!
//! The worker's side (`/api/me/flexible-working`) is in [`super::me`]; this module holds the shared
//! logic and the side of whoever decides. The software computes the **decide-by date**, flags a
//! request that is overdue, limits how many a worker may make, and keeps the record. It never
//! decides, and an approved change of hours is a *proposal* for HR to apply to the contract.
//!
//! Who decides: the worker's line manager (anywhere up the chain) or a privileged caller, and
//! **never the worker themself**. An appeal is decided by someone other than who refused. A decision
//! is a write, so under the reference policy only HR can make one; a deployer who wants managers to
//! decide grants them write access in their policy.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;

use super::unprocessable;
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{flexible_working_requests as requests, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::notifications::Model as Notification;
use crate::models::{memberships, records};
use crate::need_to_know::facts_for;
use crate::rules::access::Relation;
use crate::rules::flexible_working::{self as rules, Action, Status};
use crate::validation::Problems;

// ─── Configuration (the deployer's, with starting-point defaults) ────────────

fn setting(name: &str) -> Option<String> {
    crate::compat::env_var(name).filter(|v| !v.trim().is_empty())
}

fn number(name: &str, default: u32, max: u32) -> u32 {
    match setting(name) {
        None => default,
        Some(text) => match text.trim().parse::<u32>() {
            Ok(n) if n <= max => n,
            _ => {
                tracing::warn!("{name} is not 0 to {max}; using the default");
                default
            }
        },
    }
}

/// Calendar months a decision may take: `WPM_FLEXIBLE_DECISION_MONTHS`, default 2, at most 12.
#[must_use]
pub fn decision_months() -> u32 {
    number(
        "WPM_FLEXIBLE_DECISION_MONTHS",
        rules::DEFAULT_DECISION_MONTHS,
        12,
    )
    .max(1)
}

/// Requests a worker may make in 12 calendar months: `WPM_FLEXIBLE_REQUESTS_PER_YEAR`, default 2,
/// `0` for no limit.
#[must_use]
pub fn requests_per_year() -> u32 {
    number(
        "WPM_FLEXIBLE_REQUESTS_PER_YEAR",
        rules::DEFAULT_REQUESTS_PER_YEAR,
        100,
    )
}

/// Calendar days to appeal a refusal: `WPM_FLEXIBLE_APPEAL_CALENDAR_DAYS`, default 14.
#[must_use]
pub fn appeal_calendar_days() -> u32 {
    number(
        "WPM_FLEXIBLE_APPEAL_CALENDAR_DAYS",
        rules::DEFAULT_APPEAL_CALENDAR_DAYS,
        366,
    )
}

/// The allowed refusal reasons: `WPM_FLEXIBLE_REFUSAL_REASONS` (comma-separated lowercase tokens)
/// or the defaults.
#[must_use]
pub fn refusal_reasons() -> Vec<String> {
    if let Some(text) = setting("WPM_FLEXIBLE_REFUSAL_REASONS") {
        match rules::parse_reasons(&text) {
            Ok(list) => return list,
            Err(error) => {
                tracing::warn!(%error, "WPM_FLEXIBLE_REFUSAL_REASONS ignored; using the defaults");
            }
        }
    }
    rules::DEFAULT_REFUSAL_REASONS
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

// ─── Views ──────────────────────────────────────────────────────────────────

fn status_of(row: &requests::Model) -> Result<Status> {
    Status::parse(&row.status).ok_or_else(|| Error::string("a request has an unknown status"))
}

fn view(row: &requests::Model, today: NaiveDate) -> serde_json::Value {
    let status = Status::parse(&row.status);
    let overdue = status.is_some_and(|s| rules::is_overdue(s, row.decide_by, today));
    json!({
        "pid": row.pid,
        "kind": row.kind,
        "requested_on": row.requested_on,
        "proposed_start": row.proposed_start,
        "proposed_fte_percent": row.proposed_fte_percent,
        "effect_on_team": row.effect_on_team,
        "how_to_manage": row.how_to_manage,
        "status": row.status,
        "decide_by": row.decide_by,
        "overdue": overdue,
        "decided_on": row.decided_on,
        "decision_reason": row.decision_reason,
        "decision_note": row.decision_note,
        "counter_note": row.counter_note,
        "counter_fte_percent": row.counter_fte_percent,
        "trial_until": row.trial_until,
        "appealed_on": row.appealed_on,
        "appeal_note": row.appeal_note,
        "can_appeal": status == Some(Status::Refused)
            && row.decided_on.is_some_and(|on| rules::can_appeal(on, today, appeal_calendar_days())),
        // The software never changes a contract: an agreed change of hours is for HR to apply.
        "hr_to_apply_fte_percent": if status.is_some_and(Status::is_agreed) {
            if status == Some(Status::Accepted) { row.counter_fte_percent.or(row.proposed_fte_percent) } else { row.proposed_fte_percent }
        } else {
            None
        },
    })
}

// ─── The worker's side (called from /api/me) ─────────────────────────────────

/// `POST /api/me/flexible-working` body.
#[derive(Debug, Deserialize)]
pub(crate) struct RequestPayload {
    kind: String,
    proposed_start: NaiveDate,
    #[serde(default)]
    proposed_fte_percent: Option<i32>,
    /// The worker's own view of the effect on the team. Their words; not for health detail.
    #[serde(default)]
    effect_on_team: Option<String>,
    /// How they think it might be managed.
    #[serde(default)]
    how_to_manage: Option<String>,
}

fn clean(value: Option<&String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Make a request. Shared with `/api/me/flexible-working`.
pub(crate) async fn create_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
    payload: RequestPayload,
) -> Result<Response> {
    let today = Utc::now().date_naive();
    let mut problems = Problems::new();
    if let Some(t) = &payload.effect_on_team {
        problems.cap_text("effect_on_team", t);
    }
    if let Some(t) = &payload.how_to_manage {
        problems.cap_text("how_to_manage", t);
    }
    super::ensure_valid(&problems.into_vec())?;
    rules::validate_request(
        &payload.kind,
        payload.proposed_start,
        payload.proposed_fte_percent,
        today,
    )
    .map_err(|e| unprocessable(&e))?;
    let previous: Vec<(NaiveDate, Status)> = requests::Entity::find()
        .filter(requests::Column::WorkerPid.eq(worker.pid))
        .all(&ctx.db)
        .await?
        .iter()
        .filter_map(|r| Some((r.requested_on, Status::parse(&r.status)?)))
        .collect();
    let limit = requests_per_year();
    if !rules::within_limit(&previous, today, limit) {
        return Err(unprocessable(&format!(
            "you have already made {limit} requests in the last 12 calendar months"
        )));
    }
    let decide_by = rules::decide_by(today, decision_months())
        .ok_or_else(|| unprocessable("the decision date runs past the supported calendar"))?;
    let txn = ctx.db.begin().await?;
    let row = requests::ActiveModel {
        pid: ActiveValue::set(uuid::Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        kind: ActiveValue::set(payload.kind.clone()),
        requested_on: ActiveValue::set(today),
        proposed_start: ActiveValue::set(payload.proposed_start),
        proposed_fte_percent: ActiveValue::set(payload.proposed_fte_percent),
        effect_on_team: ActiveValue::set(clean(payload.effect_on_team.as_ref())),
        how_to_manage: ActiveValue::set(clean(payload.how_to_manage.as_ref())),
        status: ActiveValue::set(Status::Requested.as_str().to_string()),
        decide_by: ActiveValue::set(decide_by),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    if let Some(manager) = worker.manager_pid {
        Notification::push(
            &txn,
            manager,
            "flexible_working_requested",
            &format!(
                "{} asked for a flexible working arrangement.",
                worker.display_name
            ),
            json!({ "worker_pid": worker.pid, "request_pid": row.pid, "decide_by": decide_by }),
        )
        .await?;
    }
    Audit::record(
        &txn,
        "worker",
        worker.pid,
        "flexible_working_requested",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": row.pid, "decide_by": decide_by }))
}

/// The worker's own requests, newest first. Shared with `/api/me/flexible-working`.
pub(crate) async fn list_for(ctx: &AppContext, worker: &workers::Model) -> Result<Response> {
    let today = Utc::now().date_naive();
    let rows = requests::Entity::find()
        .filter(requests::Column::WorkerPid.eq(worker.pid))
        .order_by_desc(requests::Column::Id)
        .all(&ctx.db)
        .await?;
    format::json(rows.iter().map(|r| view(r, today)).collect::<Vec<_>>())
}

async fn own_request(
    ctx: &AppContext,
    worker: &workers::Model,
    pid: &str,
) -> Result<requests::Model> {
    requests::Entity::find()
        .filter(requests::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(requests::Column::WorkerPid.eq(worker.pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

async fn move_to(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
    row: requests::Model,
    action: Action,
    event: &str,
    apply: impl FnOnce(&mut requests::ActiveModel),
) -> Result<requests::Model> {
    let to = rules::next(status_of(&row)?, action).map_err(|e| unprocessable(&e))?;
    let mut active: requests::ActiveModel = row.into();
    active.status = ActiveValue::set(to.as_str().to_string());
    apply(&mut active);
    let saved = active.update(&ctx.db).await?;
    Audit::record(&ctx.db, "worker", worker.pid, event, caller.actor(), None).await?;
    Ok(saved)
}

/// Withdraw. Shared with `/api/me/flexible-working/{pid}/withdraw`.
pub(crate) async fn withdraw_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
    pid: &str,
) -> Result<Response> {
    let row = own_request(ctx, worker, pid).await?;
    let saved = move_to(
        ctx,
        caller,
        worker,
        row,
        Action::Withdraw,
        "flexible_working_withdrawn",
        |_| {},
    )
    .await?;
    format::json(view(&saved, Utc::now().date_naive()))
}

/// `POST /api/me/flexible-working/{pid}/respond` body.
#[derive(Debug, Deserialize)]
pub(crate) struct RespondPayload {
    /// `accept` or `decline`.
    response: String,
}

/// Answer a counter-proposal. Shared with `/api/me/flexible-working/{pid}/respond`.
pub(crate) async fn respond_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
    pid: &str,
    payload: RespondPayload,
) -> Result<Response> {
    let action = match payload.response.as_str() {
        "accept" => Action::Accept,
        "decline" => Action::Decline,
        _ => return Err(unprocessable("response must be accept or decline")),
    };
    let row = own_request(ctx, worker, pid).await?;
    let event = if action == Action::Accept {
        "flexible_working_accepted"
    } else {
        "flexible_working_declined"
    };
    let saved = move_to(ctx, caller, worker, row, action, event, |_| {}).await?;
    format::json(view(&saved, Utc::now().date_naive()))
}

/// `POST /api/me/flexible-working/{pid}/appeal` body.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct AppealPayload {
    #[serde(default)]
    note: Option<String>,
}

/// Appeal a refusal, once, within the window. Shared with `/api/me/flexible-working/{pid}/appeal`.
pub(crate) async fn appeal_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
    pid: &str,
    payload: AppealPayload,
) -> Result<Response> {
    let mut problems = Problems::new();
    if let Some(t) = &payload.note {
        problems.cap_text("note", t);
    }
    super::ensure_valid(&problems.into_vec())?;
    let row = own_request(ctx, worker, pid).await?;
    let today = Utc::now().date_naive();
    if status_of(&row)? == Status::Refused
        && !row
            .decided_on
            .is_some_and(|on| rules::can_appeal(on, today, appeal_calendar_days()))
    {
        return Err(unprocessable(&format!(
            "an appeal must be made within {} calendar days of the refusal",
            appeal_calendar_days()
        )));
    }
    let note = clean(payload.note.as_ref());
    let saved = move_to(
        ctx,
        caller,
        worker,
        row,
        Action::Appeal,
        "flexible_working_appealed",
        |a| {
            a.appealed_on = ActiveValue::set(Some(today));
            a.appeal_note = ActiveValue::set(note);
        },
    )
    .await?;
    if let Some(manager) = worker.manager_pid {
        Notification::push(
            &ctx.db,
            manager,
            "flexible_working_appeal",
            &format!(
                "{} appealed a flexible working decision.",
                worker.display_name
            ),
            json!({ "worker_pid": worker.pid, "request_pid": saved.pid }),
        )
        .await?;
    }
    format::json(view(&saved, today))
}

// ─── The deciding side ──────────────────────────────────────────────────────

/// May this caller decide on this worker's request? A line manager (anywhere up the chain) or a
/// privileged caller, and never the worker themself.
pub(crate) async fn ensure_decider(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
) -> Result<()> {
    if !auth::require_auth() {
        return Ok(());
    }
    let claims = caller
        .claims()
        .ok_or(Error::Unauthorized("sign in".to_string()))?;
    let facts = facts_for(ctx, claims, worker).await?;
    if facts.relation == Relation::Own {
        return Err(unprocessable("nobody decides their own request"));
    }
    if facts.privileged || facts.relation == Relation::Manager {
        Ok(())
    } else {
        Err(Error::CustomError(
            axum::http::StatusCode::FORBIDDEN,
            loco_rs::controller::ErrorDetail::new(
                "forbidden",
                "only the worker's line manager or HR decides a request",
            ),
        ))
    }
}

async fn request_and_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<(requests::Model, workers::Model)> {
    let row = requests::Entity::find()
        .filter(requests::Column::Pid.eq(records::parse_pid(pid)?))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let worker = records::find_worker(&ctx.db, row.worker_pid).await?;
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.iter().any(|r| r == &worker.organization_ref)
    {
        return Err(Error::NotFound);
    }
    Ok((row, worker))
}

/// `POST /api/flexible-working/{pid}/decide` body.
#[derive(Debug, Deserialize)]
struct DecidePayload {
    /// `approve`, `refuse` or `counter`.
    outcome: String,
    /// For a refusal: one of the allowed reasons.
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    note: Option<String>,
    /// For an approval: a trial that ends on this day.
    #[serde(default)]
    trial_until: Option<NaiveDate>,
    /// For a counter-proposal: what is offered instead.
    #[serde(default)]
    counter_note: Option<String>,
    #[serde(default)]
    counter_fte_percent: Option<i32>,
}

/// `POST /api/flexible-working/{pid}/decide` — approve, refuse (with a reason from the list) or
/// offer something different. Never the worker's own request. An agreed change of hours is a
/// proposal for HR to apply; the contract is not changed here.
#[debug_handler]
async fn decide(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<DecidePayload>,
) -> Result<Response> {
    let (row, worker) = request_and_worker(&ctx, &caller, &pid).await?;
    ensure_decider(&ctx, &caller, &worker).await?;
    let mut problems = Problems::new();
    for (field, text) in [
        ("note", &payload.note),
        ("counter_note", &payload.counter_note),
    ] {
        if let Some(t) = text {
            problems.cap_text(field, t);
        }
    }
    super::ensure_valid(&problems.into_vec())?;
    let today = Utc::now().date_naive();
    let (action, event) = match payload.outcome.as_str() {
        "approve" => (Action::Approve, "flexible_working_approved"),
        "refuse" => (Action::Refuse, "flexible_working_refused"),
        "counter" => (Action::Counter, "flexible_working_counter_proposed"),
        _ => return Err(unprocessable("outcome must be approve, refuse or counter")),
    };
    let allowed = refusal_reasons();
    let allowed_refs: Vec<&str> = allowed.iter().map(String::as_str).collect();
    match action {
        Action::Refuse => {
            let reason = payload
                .reason
                .as_deref()
                .ok_or_else(|| unprocessable("a refusal needs a reason from the list"))?;
            rules::validate_refusal_reason(reason, &allowed_refs).map_err(|e| unprocessable(&e))?;
        }
        Action::Counter => {
            if clean(payload.counter_note.as_ref()).is_none() {
                return Err(unprocessable(
                    "a counter-proposal says what is offered (counter_note)",
                ));
            }
            if payload
                .counter_fte_percent
                .is_some_and(|p| !(1..=100).contains(&p))
            {
                return Err(unprocessable(
                    "counter_fte_percent must be between 1 and 100",
                ));
            }
        }
        _ => {}
    }
    if action == Action::Approve && payload.trial_until.is_some_and(|t| t < row.proposed_start) {
        return Err(unprocessable(
            "a trial cannot end before the arrangement starts",
        ));
    }
    let actor = caller.actor().map(ToString::to_string);
    let (reason, note, counter_note, counter_fte, trial) = (
        payload.reason.clone().filter(|_| action == Action::Refuse),
        clean(payload.note.as_ref()),
        clean(payload.counter_note.as_ref()).filter(|_| action == Action::Counter),
        payload
            .counter_fte_percent
            .filter(|_| action == Action::Counter),
        payload.trial_until.filter(|_| action == Action::Approve),
    );
    let saved = move_to(&ctx, &caller, &worker, row, action, event, |a| {
        a.decided_on = ActiveValue::set(Some(today));
        a.decided_by = ActiveValue::set(actor);
        a.decision_reason = ActiveValue::set(reason);
        a.decision_note = ActiveValue::set(note);
        a.counter_note = ActiveValue::set(counter_note);
        a.counter_fte_percent = ActiveValue::set(counter_fte);
        a.trial_until = ActiveValue::set(trial);
    })
    .await?;
    Notification::push(
        &ctx.db,
        worker.pid,
        "flexible_working_decided",
        "Your flexible working request has an answer.",
        json!({ "request_pid": saved.pid, "outcome": saved.status }),
    )
    .await?;
    format::json(view(&saved, today))
}

/// `POST /api/flexible-working/{pid}/appeal-decision` body.
#[derive(Debug, Deserialize)]
struct AppealDecisionPayload {
    /// `uphold` (the request is approved) or `dismiss`.
    outcome: String,
    #[serde(default)]
    note: Option<String>,
}

/// `POST /api/flexible-working/{pid}/appeal-decision` — decide an appeal. Made by someone other
/// than who refused the request.
#[debug_handler]
async fn decide_appeal(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<AppealDecisionPayload>,
) -> Result<Response> {
    let (row, worker) = request_and_worker(&ctx, &caller, &pid).await?;
    ensure_decider(&ctx, &caller, &worker).await?;
    let (action, event) = match payload.outcome.as_str() {
        "uphold" => (Action::Uphold, "flexible_working_appeal_upheld"),
        "dismiss" => (Action::Dismiss, "flexible_working_appeal_dismissed"),
        _ => return Err(unprocessable("outcome must be uphold or dismiss")),
    };
    let actor = caller.actor().map(ToString::to_string);
    if actor.is_some() && actor == row.decided_by {
        return Err(unprocessable(
            "an appeal is decided by someone other than who refused the request",
        ));
    }
    let mut problems = Problems::new();
    if let Some(t) = &payload.note {
        problems.cap_text("note", t);
    }
    super::ensure_valid(&problems.into_vec())?;
    let note = clean(payload.note.as_ref());
    let today = Utc::now().date_naive();
    let saved = move_to(&ctx, &caller, &worker, row, action, event, |a| {
        a.appeal_decided_by = ActiveValue::set(actor);
        if note.is_some() {
            a.decision_note = ActiveValue::set(note);
        }
    })
    .await?;
    Notification::push(
        &ctx.db,
        worker.pid,
        "flexible_working_decided",
        "Your flexible working appeal has an answer.",
        json!({ "request_pid": saved.pid, "outcome": saved.status }),
    )
    .await?;
    format::json(view(&saved, today))
}

/// Query for the list.
#[derive(Debug, Deserialize)]
struct ListQuery {
    status: Option<String>,
    /// `true` for only those past their decide-by date.
    overdue: Option<bool>,
}

/// `GET /api/flexible-working?status=&overdue=` — requests the caller may decide: for a manager,
/// their reports' (anywhere down the chain); for a privileged caller, those in their organizations.
/// Soonest decide-by date first. Overdue ones are flagged.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    if let Some(s) = &query.status
        && Status::parse(s).is_none()
    {
        return Err(unprocessable("unknown status"));
    }
    let today = Utc::now().date_naive();
    let mut select = requests::Entity::find().order_by_asc(requests::Column::DecideBy);
    if let Some(s) = &query.status {
        select = select.filter(requests::Column::Status.eq(s));
    }
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let mut out = Vec::new();
    for row in select.all(&ctx.db).await? {
        let Some(worker) = workers::Entity::find()
            .filter(workers::Column::Pid.eq(row.worker_pid))
            .filter(workers::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?
        else {
            continue;
        };
        if scope
            .as_ref()
            .is_some_and(|refs| !refs.iter().any(|r| r == &worker.organization_ref))
        {
            continue;
        }
        if auth::require_auth() {
            let Some(claims) = caller.claims() else {
                continue;
            };
            let facts = facts_for(&ctx, claims, &worker).await?;
            if !(facts.privileged || facts.relation == Relation::Manager) {
                continue;
            }
        }
        let mut item = view(&row, today);
        if query.overdue == Some(true) && item["overdue"] != json!(true) {
            continue;
        }
        item["worker_pid"] = json!(worker.pid);
        item["worker_number"] = json!(worker.worker_number);
        item["department"] = json!(worker.department);
        out.push(item);
    }
    format::json(out)
}

/// `GET /api/workers/{pid}/flexible-working` — the worker's requests, for the worker, their line
/// managers and HR.
#[debug_handler]
async fn worker_requests(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    list_for(&ctx, &worker).await
}

/// The flexible-working routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/flexible-working", get(list))
        .add("/flexible-working/{pid}/decide", post(decide))
        .add(
            "/flexible-working/{pid}/appeal-decision",
            post(decide_appeal),
        )
        .add("/workers/{pid}/flexible-working", get(worker_requests))
}
