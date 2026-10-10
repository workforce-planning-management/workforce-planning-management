//! **Engagements** (WPM-R79, WPM-R80, WPM-D56, WPM-D58): the end of a fixed-term or contractor
//! engagement, its extensions, a contractor's supplier, route and rate, and an optional
//! employment-status assessment. Pure rules in [`crate::rules::engagement`].
//!
//! An end date is the **last day** of the engagement. It moves only by an extension, which is
//! kept as a dated entry, so the history shows how often an engagement was extended. A rate is
//! personal and sensitive: it is masked like salary, and no audit entry carries it. Everything
//! here is about one person, so it is exported with the worker and erased with them.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;

use super::{ensure_valid, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{
    engagement_decisions, engagement_extensions, engagement_status_assessments,
    worker_contractor_details, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::rules::engagement as rules;
use crate::validation::Problems;

/// A worker the caller may change: HR, or the worker themself.
async fn writable_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<workers::Model> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(pid)?).await?;
    auth::authorize_record(
        caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    Ok(worker)
}

fn extension_json(row: &engagement_extensions::Model) -> serde_json::Value {
    json!({
        "pid": row.pid,
        "previous_end": row.previous_end,
        "new_end": row.new_end,
        "reason": row.reason,
        "decided_by": row.decided_by,
    })
}

async fn extensions_of(
    ctx: &AppContext,
    worker_pid: uuid::Uuid,
) -> Result<Vec<engagement_extensions::Model>> {
    Ok(engagement_extensions::Entity::find()
        .filter(engagement_extensions::Column::WorkerPid.eq(worker_pid))
        .order_by_asc(engagement_extensions::Column::Id)
        .all(&ctx.db)
        .await?)
}

/// `GET /api/workers/{pid}/engagement` — the basis, the end date, where it stands, and the
/// extensions so far. Seen by the worker and HR.
#[debug_handler]
async fn get_engagement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let today = Utc::now().date_naive();
    let extensions = extensions_of(&ctx, worker.pid).await?;
    let decisions = decisions_of(&ctx, worker.pid).await?;
    // A decision settles the end date it was made for; an extension gives a new date and so a new question.
    let current = worker
        .engagement_ends_on
        .and_then(|end| decisions.iter().rev().find(|d| d.ends_on == end));
    let standing = rules::Basis::parse(&worker.employment_type).map(|basis| {
        rules::standing(
            basis,
            worker.engagement_ends_on,
            today,
            rules::DEFAULT_WINDOW_CALENDAR_DAYS,
            current.is_some(),
        )
        .as_str()
    });
    format::json(json!({
        "employment_type": worker.employment_type,
        "hired_on": worker.hired_on,
        "engagement_ends_on": worker.engagement_ends_on,
        "standing": standing,
        "window_calendar_days": rules::DEFAULT_WINDOW_CALENDAR_DAYS,
        "extension_count": extensions.len(),
        "extensions": extensions.iter().map(extension_json).collect::<Vec<_>>(),
        "decision": current.map(|d| d.decision.clone()),
        "decisions": decisions.iter().map(|d| json!({
            "pid": d.pid, "ends_on": d.ends_on, "decision": d.decision,
            "decided_by": d.decided_by, "decided_on": d.decided_on,
        })).collect::<Vec<_>>(),
    }))
}

async fn decisions_of(
    ctx: &AppContext,
    worker_pid: uuid::Uuid,
) -> Result<Vec<engagement_decisions::Model>> {
    Ok(engagement_decisions::Entity::find()
        .filter(engagement_decisions::Column::WorkerPid.eq(worker_pid))
        .order_by_asc(engagement_decisions::Column::Id)
        .all(&ctx.db)
        .await?)
}

/// `POST …/engagement/decision` body.
#[derive(Debug, Deserialize)]
struct DecisionPayload {
    /// `extend`, `convert` or `end`.
    decision: String,
}

/// `POST /api/workers/{pid}/engagement/decision` — record what a person decides about an engagement
/// that is ending, against the end date it settles (WPM-R81, WPM-D66). The worker's line manager or
/// a privileged caller, never the worker. **It records a decision and changes nothing else:** to
/// extend, record the extension; to convert or end, do it through the processes that exist. Silence
/// is never read as continuing.
#[debug_handler]
async fn decide(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<DecisionPayload>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    super::flexible_working::ensure_decider(&ctx, &caller, &worker).await?;
    rules::validate_decision(&payload.decision).map_err(|e| unprocessable(&e))?;
    let ends_on = worker
        .engagement_ends_on
        .ok_or_else(|| unprocessable("the engagement has no end date to decide about"))?;
    if worker.terminated_on.is_some() {
        return Err(unprocessable("the worker's employment has ended"));
    }
    let row = engagement_decisions::ActiveModel {
        pid: ActiveValue::set(uuid::Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        ends_on: ActiveValue::set(ends_on),
        decision: ActiveValue::set(payload.decision.clone()),
        decided_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        decided_on: ActiveValue::set(Utc::now().date_naive()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    // The audit entry names the event, not the decision.
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "engagement_decision_recorded",
        caller.actor(),
        None,
    )
    .await?;
    format::json(json!({ "pid": row.pid, "ends_on": row.ends_on, "decision": row.decision }))
}

/// `POST …/engagement/extensions` body.
#[derive(Debug, Deserialize)]
struct ExtensionPayload {
    new_end: NaiveDate,
    #[serde(default)]
    reason: Option<String>,
}

/// `POST /api/workers/{pid}/engagement/extensions` — move the end later, keeping the history.
#[debug_handler]
async fn extend(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ExtensionPayload>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let mut problems = Problems::new();
    if let Some(text) = &payload.reason {
        problems.cap_text("reason", text);
    }
    ensure_valid(&problems.into_vec())?;
    if worker.terminated_on.is_some() {
        return Err(unprocessable("the worker's employment has ended"));
    }
    let previous = worker.engagement_ends_on.ok_or_else(|| {
        unprocessable("the engagement has no end date to extend; record one first")
    })?;
    rules::validate_extension(previous, payload.new_end).map_err(|e| unprocessable(&e))?;
    let txn = ctx.db.begin().await?;
    let row = engagement_extensions::ActiveModel {
        pid: ActiveValue::set(uuid::Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        previous_end: ActiveValue::set(previous),
        new_end: ActiveValue::set(payload.new_end),
        reason: ActiveValue::set(payload.reason.clone().filter(|r| !r.trim().is_empty())),
        decided_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    let mut active: workers::ActiveModel = worker.clone().into();
    active.engagement_ends_on = ActiveValue::set(Some(payload.new_end));
    active.update(&txn).await?;
    // The audit entry names the event, not the reason text.
    Audit::record(
        &txn,
        "worker",
        worker.pid,
        "engagement_extended",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(extension_json(&row))
}

// ─── Contractor details ─────────────────────────────────────────────────────

/// `PUT …/contractor-details` body.
#[derive(Debug, Deserialize)]
struct DetailsPayload {
    /// The supplier (agency, or the contractor's own company): an `organization:` URN.
    #[serde(default)]
    supplier_ref: Option<String>,
    #[serde(default)]
    route: Option<String>,
    #[serde(default)]
    rate_minor: Option<i64>,
    #[serde(default)]
    rate_currency: Option<String>,
    #[serde(default)]
    rate_basis: Option<String>,
}

async fn details_of(
    ctx: &AppContext,
    worker_pid: uuid::Uuid,
) -> Result<Option<worker_contractor_details::Model>> {
    Ok(worker_contractor_details::Entity::find()
        .filter(worker_contractor_details::Column::WorkerPid.eq(worker_pid))
        .one(&ctx.db)
        .await?)
}

/// The details as shown. A masked caller sees that a rate exists, not what it is.
fn details_json(row: &worker_contractor_details::Model, masked: bool) -> serde_json::Value {
    json!({
        "supplier_ref": row.supplier_ref,
        "route": row.route,
        "rate_minor": if masked { None } else { row.rate_minor },
        "rate_currency": if masked { None } else { row.rate_currency.clone() },
        "rate_basis": row.rate_basis,
        "rate_recorded": row.rate_minor.is_some(),
        "rate_masked": masked && row.rate_minor.is_some(),
    })
}

/// `GET /api/workers/{pid}/contractor-details` — or `null`. The rate is masked like salary.
#[debug_handler]
async fn get_details(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let obligations = auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let masked = obligations.iter().any(|o| o == "mask");
    let row = details_of(&ctx, worker.pid).await?;
    if row.as_ref().is_some_and(|r| r.rate_minor.is_some()) && !masked {
        Audit::record(
            &ctx.db,
            "worker",
            worker.pid,
            "rate_read",
            caller.actor(),
            None,
        )
        .await?;
    }
    format::json(json!({ "contractor_details": row.map(|r| details_json(&r, masked)) }))
}

/// `PUT /api/workers/{pid}/contractor-details` — set or change them. Only a contractor has
/// them. The response and the audit entry carry no rate.
#[debug_handler]
async fn set_details(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<DetailsPayload>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    if worker.employment_type != "contractor" {
        return Err(unprocessable("only a contractor has contractor details"));
    }
    let mut problems = Problems::new();
    problems.ref_opt(
        "supplier_ref",
        entity_ref::EntityType::Organization,
        payload.supplier_ref.as_deref(),
    );
    ensure_valid(&problems.into_vec())?;
    rules::validate_contractor_details(
        payload.route.as_deref(),
        payload.rate_minor,
        payload.rate_currency.as_deref(),
        payload.rate_basis.as_deref(),
    )
    .map_err(|e| unprocessable(&e))?;
    let recorded_by = caller.actor().map(ToString::to_string);
    let saved = if let Some(row) = details_of(&ctx, worker.pid).await? {
        let mut active: worker_contractor_details::ActiveModel = row.into();
        active.supplier_ref = ActiveValue::set(payload.supplier_ref.clone());
        active.route = ActiveValue::set(payload.route.clone());
        active.rate_minor = ActiveValue::set(payload.rate_minor);
        active.rate_currency = ActiveValue::set(payload.rate_currency.clone());
        active.rate_basis = ActiveValue::set(payload.rate_basis.clone());
        active.recorded_by = ActiveValue::set(recorded_by);
        active.update(&ctx.db).await?
    } else {
        worker_contractor_details::ActiveModel {
            worker_pid: ActiveValue::set(worker.pid),
            supplier_ref: ActiveValue::set(payload.supplier_ref.clone()),
            route: ActiveValue::set(payload.route.clone()),
            rate_minor: ActiveValue::set(payload.rate_minor),
            rate_currency: ActiveValue::set(payload.rate_currency.clone()),
            rate_basis: ActiveValue::set(payload.rate_basis.clone()),
            recorded_by: ActiveValue::set(recorded_by),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?
    };
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "contractor_details_set",
        caller.actor(),
        None,
    )
    .await?;
    format::json(json!({ "contractor_details": details_json(&saved, true) }))
}

/// `DELETE /api/workers/{pid}/contractor-details` — clear them (none recorded is a 404).
#[debug_handler]
async fn clear_details(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let row = details_of(&ctx, worker.pid).await?.ok_or(Error::NotFound)?;
    worker_contractor_details::Entity::delete_by_id(row.id)
        .exec(&ctx.db)
        .await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "contractor_details_cleared",
        caller.actor(),
        None,
    )
    .await?;
    format::empty()
}

// ─── Employment-status assessments ──────────────────────────────────────────

/// `POST …/engagement/status-assessments` body.
#[derive(Debug, Deserialize)]
struct AssessmentPayload {
    outcome: String,
    /// Default today; not in the future.
    #[serde(default)]
    assessed_on: Option<NaiveDate>,
}

/// `POST /api/workers/{pid}/engagement/status-assessments` — record an assessment of a
/// contractor's employment status. The reviewer is the caller. Whether a deployment requires
/// one, and what the outcome means, is its own jurisdiction's question.
#[debug_handler]
async fn record_assessment(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<AssessmentPayload>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    if worker.employment_type != "contractor" {
        return Err(unprocessable(
            "only a contractor has an employment-status assessment",
        ));
    }
    let mut problems = Problems::new();
    problems.require_token("outcome", rules::STATUS_OUTCOMES, &payload.outcome);
    ensure_valid(&problems.into_vec())?;
    let today = Utc::now().date_naive();
    let on = payload.assessed_on.unwrap_or(today);
    if on > today {
        return Err(unprocessable("assessed_on must not be in the future"));
    }
    let row = engagement_status_assessments::ActiveModel {
        pid: ActiveValue::set(uuid::Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        outcome: ActiveValue::set(payload.outcome.clone()),
        assessed_on: ActiveValue::set(on),
        reviewed_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "status_assessed",
        caller.actor(),
        None,
    )
    .await?;
    format::json(json!({ "pid": row.pid }))
}

/// `GET /api/workers/{pid}/engagement/status-assessments` — newest first.
#[debug_handler]
async fn list_assessments(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let rows = engagement_status_assessments::Entity::find()
        .filter(engagement_status_assessments::Column::WorkerPid.eq(worker.pid))
        .order_by_desc(engagement_status_assessments::Column::Id)
        .all(&ctx.db)
        .await?;
    format::json(
        rows.iter()
            .map(|r| {
                json!({
                    "pid": r.pid, "outcome": r.outcome, "assessed_on": r.assessed_on,
                    "reviewed_by": r.reviewed_by,
                })
            })
            .collect::<Vec<_>>(),
    )
}

// ─── Missing end dates ──────────────────────────────────────────────────────

/// Query for the missing-end-date list.
#[derive(Debug, Deserialize)]
struct MissingQuery {
    organization: Option<String>,
}

/// `GET /api/engagements/missing-end-date?organization=` — fixed-term and contractor workers
/// whose end date is not recorded, until HR records one. No date is ever invented for them.
/// Names no pay: worker number, department, basis and start only.
#[debug_handler]
async fn missing_end_date(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<MissingQuery>,
) -> Result<Response> {
    let mut select = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::TerminatedOn.is_null())
        .filter(workers::Column::EngagementEndsOn.is_null())
        .filter(workers::Column::EmploymentType.is_in(["fixed_term", "contractor"]));
    if let Some(org) = &query.organization {
        select = select.filter(workers::Column::OrganizationRef.eq(org));
    }
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await? {
        select = select.filter(workers::Column::OrganizationRef.is_in(refs));
    }
    let rows = select
        .order_by_asc(workers::Column::WorkerNumber)
        .all(&ctx.db)
        .await?;
    format::json(
        rows.iter()
            .map(|w| {
                json!({
                    "worker_pid": w.pid, "worker_number": w.worker_number,
                    "department": w.department, "employment_type": w.employment_type,
                    "hired_on": w.hired_on,
                })
            })
            .collect::<Vec<_>>(),
    )
}

/// The engagement routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/engagement", get(get_engagement))
        .add("/workers/{pid}/engagement/extensions", post(extend))
        .add("/workers/{pid}/engagement/decision", post(decide))
        .add(
            "/workers/{pid}/engagement/status-assessments",
            post(record_assessment),
        )
        .add(
            "/workers/{pid}/engagement/status-assessments",
            get(list_assessments),
        )
        .add("/workers/{pid}/contractor-details", get(get_details))
        .add("/workers/{pid}/contractor-details", put(set_details))
        .add("/workers/{pid}/contractor-details", delete(clear_details))
        .add("/engagements/missing-end-date", get(missing_end_date))
}
