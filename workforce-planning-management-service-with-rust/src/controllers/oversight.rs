//! **Oversight** (WPM-R65, WPM-R66, WPM-D48, WPM-D49): an organization's own scale of levels, the
//! positions it has recorded, the exit criteria of each level, the measures behind them and the
//! readings entered for reported measures. `GET /api/oversight/status` reports, for the criteria of
//! the level the organization is at, whether each sustain rule is met (pure rules in
//! [`crate::rules::oversight`]).
//!
//! Nothing here changes a level: a position is recorded by a person with the deciding body, the date
//! and the published reason. No level, criterion or threshold ships in code; the seed task adds a
//! scale labelled synthetic. Everything is scoped to the organization and every write is audited.

use std::collections::BTreeMap;

use chrono::Datelike;
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::Deserialize;
use serde_json::json;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    oversight_criteria as criteria, oversight_levels as levels, oversight_measures as measures,
    oversight_positions as positions, oversight_readings as readings,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::rules::oversight as rules;
use crate::validation::Problems;

async fn ensure_org_in_scope(ctx: &AppContext, caller: &MaybeAuthUser, org: &str) -> Result<()> {
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.iter().any(|r| r == org)
    {
        return Err(Error::NotFound);
    }
    Ok(())
}

fn org_problems(org: &str) -> Result<()> {
    let mut problems = Problems::new();
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        org,
    );
    ensure_valid(&problems.into_vec())
}

async fn audit(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    kind: &str,
    pid: Uuid,
    action: &str,
) -> Result<()> {
    Audit::record(&ctx.db, kind, pid, action, caller.actor(), None).await?;
    Ok(())
}

async fn level_in_scope(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: Uuid,
) -> Result<levels::Model> {
    let level = levels::Entity::find()
        .filter(levels::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &level.organization_ref).await?;
    Ok(level)
}

async fn criterion_in_scope(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: Uuid,
) -> Result<criteria::Model> {
    let c = criteria::Entity::find()
        .filter(criteria::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    level_in_scope(ctx, caller, c.level_pid).await?;
    Ok(c)
}

async fn measure_in_scope(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: Uuid,
) -> Result<measures::Model> {
    let m = measures::Entity::find()
        .filter(measures::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    criterion_in_scope(ctx, caller, m.criterion_pid).await?;
    Ok(m)
}

// ─── Levels ─────────────────────────────────────────────────────────────────

/// `POST /api/oversight/levels` body.
#[derive(Debug, Deserialize)]
struct LevelPayload {
    organization_ref: String,
    ordinal: i32,
    name: String,
}

/// `POST /api/oversight/levels` — add a level to the organization's scale.
#[debug_handler]
async fn create_level(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<LevelPayload>,
) -> Result<Response> {
    org_problems(&p.organization_ref)?;
    let mut problems = Problems::new();
    problems.require_text("name", &p.name);
    ensure_valid(&problems.into_vec())?;
    if !(1..=20).contains(&p.ordinal) {
        return Err(unprocessable("ordinal must be 1 to 20"));
    }
    ensure_org_in_scope(&ctx, &caller, &p.organization_ref).await?;
    let taken = levels::Entity::find()
        .filter(levels::Column::OrganizationRef.eq(&p.organization_ref))
        .filter(levels::Column::Ordinal.eq(p.ordinal))
        .one(&ctx.db)
        .await?;
    if taken.is_some() {
        return Err(unprocessable("that ordinal is already on the scale"));
    }
    let row = levels::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        organization_ref: ActiveValue::set(p.organization_ref),
        ordinal: ActiveValue::set(p.ordinal),
        name: ActiveValue::set(p.name),
        synthetic: ActiveValue::set(false),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    audit(&ctx, &caller, "oversight_level", row.pid, "created").await?;
    format::json(level_json(&row))
}

fn level_json(l: &levels::Model) -> serde_json::Value {
    json!({ "pid": l.pid, "organization_ref": l.organization_ref, "ordinal": l.ordinal, "name": l.name, "synthetic": l.synthetic })
}

/// `DELETE /api/oversight/levels/{pid}` — only a level with no recorded position and no criterion.
#[debug_handler]
async fn delete_level(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let level = level_in_scope(&ctx, &caller, records::parse_pid(&pid)?).await?;
    let used = positions::Entity::find()
        .filter(positions::Column::LevelPid.eq(level.pid))
        .one(&ctx.db)
        .await?
        .is_some()
        || criteria::Entity::find()
            .filter(criteria::Column::LevelPid.eq(level.pid))
            .one(&ctx.db)
            .await?
            .is_some();
    if used {
        return Err(unprocessable(
            "a level with a recorded position or a criterion cannot be removed; remove the criteria first, and a recorded position stays on the record",
        ));
    }
    levels::Entity::delete_by_id(level.id).exec(&ctx.db).await?;
    audit(&ctx, &caller, "oversight_level", level.pid, "removed").await?;
    format::json(json!({ "removed": level.pid }))
}

// ─── Positions ──────────────────────────────────────────────────────────────

/// `POST /api/oversight/positions` body.
#[derive(Debug, Deserialize)]
struct PositionPayload {
    organization_ref: String,
    level_pid: Uuid,
    effective_on: chrono::NaiveDate,
    deciding_body: String,
    reason: String,
}

/// `POST /api/oversight/positions` — record the organization's position. A person records it; the
/// service never does.
#[debug_handler]
async fn record_position(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<PositionPayload>,
) -> Result<Response> {
    org_problems(&p.organization_ref)?;
    let mut problems = Problems::new();
    problems.require_text("deciding_body", &p.deciding_body);
    problems.require_text("reason", &p.reason);
    ensure_valid(&problems.into_vec())?;
    ensure_org_in_scope(&ctx, &caller, &p.organization_ref).await?;
    let level = level_in_scope(&ctx, &caller, p.level_pid).await?;
    if level.organization_ref != p.organization_ref {
        return Err(unprocessable(
            "that level is on another organization's scale",
        ));
    }
    let row = positions::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        organization_ref: ActiveValue::set(p.organization_ref),
        level_pid: ActiveValue::set(level.pid),
        effective_on: ActiveValue::set(p.effective_on),
        deciding_body: ActiveValue::set(p.deciding_body),
        reason: ActiveValue::set(p.reason),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    audit(&ctx, &caller, "oversight_position", row.pid, "recorded").await?;
    format::json(position_json(&row, Some(&level)))
}

fn position_json(p: &positions::Model, level: Option<&levels::Model>) -> serde_json::Value {
    json!({
        "pid": p.pid, "level_pid": p.level_pid, "level": level.map(|l| l.name.clone()),
        "ordinal": level.map(|l| l.ordinal), "effective_on": p.effective_on,
        "deciding_body": p.deciding_body, "reason": p.reason,
    })
}

// ─── Criteria, measures, readings ───────────────────────────────────────────

/// `POST /api/oversight/criteria` body.
#[derive(Debug, Deserialize)]
struct CriterionPayload {
    level_pid: Uuid,
    name: String,
}

/// `POST /api/oversight/criteria` — add an exit criterion to a level.
#[debug_handler]
async fn create_criterion(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<CriterionPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("name", &p.name);
    ensure_valid(&problems.into_vec())?;
    let level = level_in_scope(&ctx, &caller, p.level_pid).await?;
    let row = criteria::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        level_pid: ActiveValue::set(level.pid),
        name: ActiveValue::set(p.name),
        synthetic: ActiveValue::set(false),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    audit(&ctx, &caller, "oversight_criterion", row.pid, "created").await?;
    format::json(json!({ "pid": row.pid, "level_pid": row.level_pid, "name": row.name }))
}

/// `DELETE /api/oversight/criteria/{pid}` — with its measures and their readings.
#[debug_handler]
async fn delete_criterion(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let c = criterion_in_scope(&ctx, &caller, records::parse_pid(&pid)?).await?;
    let ms = measures::Entity::find()
        .filter(measures::Column::CriterionPid.eq(c.pid))
        .all(&ctx.db)
        .await?;
    for m in &ms {
        readings::Entity::delete_many()
            .filter(readings::Column::MeasurePid.eq(m.pid))
            .exec(&ctx.db)
            .await?;
    }
    measures::Entity::delete_many()
        .filter(measures::Column::CriterionPid.eq(c.pid))
        .exec(&ctx.db)
        .await?;
    criteria::Entity::delete_by_id(c.id).exec(&ctx.db).await?;
    audit(&ctx, &caller, "oversight_criterion", c.pid, "removed").await?;
    format::json(json!({ "removed": c.pid }))
}

/// `POST /api/oversight/measures` body.
#[derive(Debug, Deserialize)]
struct MeasurePayload {
    criterion_pid: Uuid,
    name: String,
    origin: String,
    unit: String,
    threshold: i64,
    direction: String,
    sustain: i64,
}

/// `POST /api/oversight/measures` — a measure behind a criterion, with its threshold, direction and
/// sustain rule.
#[debug_handler]
async fn create_measure(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<MeasurePayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("name", &p.name);
    problems.require_text("unit", &p.unit);
    ensure_valid(&problems.into_vec())?;
    rules::validate_spec(&p.direction, p.sustain).map_err(|e| unprocessable(&e))?;
    if !rules::AVAILABLE_ORIGINS.contains(&p.origin.as_str()) {
        return Err(unprocessable(&format!(
            "origin must be one of {:?} (derived measures arrive with their computation)",
            rules::AVAILABLE_ORIGINS
        )));
    }
    let c = criterion_in_scope(&ctx, &caller, p.criterion_pid).await?;
    let row = measures::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        criterion_pid: ActiveValue::set(c.pid),
        name: ActiveValue::set(p.name),
        origin: ActiveValue::set(p.origin),
        unit: ActiveValue::set(p.unit),
        threshold: ActiveValue::set(p.threshold),
        direction: ActiveValue::set(p.direction),
        sustain: ActiveValue::set(i32::try_from(p.sustain).unwrap_or(1)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    audit(&ctx, &caller, "oversight_measure", row.pid, "created").await?;
    format::json(measure_json(&row))
}

fn measure_json(m: &measures::Model) -> serde_json::Value {
    json!({
        "pid": m.pid, "criterion_pid": m.criterion_pid, "name": m.name, "origin": m.origin, "unit": m.unit,
        "threshold": m.threshold, "direction": m.direction, "sustain": m.sustain,
    })
}

/// `DELETE /api/oversight/measures/{pid}` — with its readings.
#[debug_handler]
async fn delete_measure(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let m = measure_in_scope(&ctx, &caller, records::parse_pid(&pid)?).await?;
    readings::Entity::delete_many()
        .filter(readings::Column::MeasurePid.eq(m.pid))
        .exec(&ctx.db)
        .await?;
    measures::Entity::delete_by_id(m.id).exec(&ctx.db).await?;
    audit(&ctx, &caller, "oversight_measure", m.pid, "removed").await?;
    format::json(json!({ "removed": m.pid }))
}

/// `PUT /api/oversight/measures/{pid}/readings` body.
#[derive(Debug, Deserialize)]
struct ReadingPayload {
    /// Any day of the month; the reading is for that whole month.
    period_on: chrono::NaiveDate,
    /// `null` says the data is missing for the month. It is never read as a pass.
    value: Option<i64>,
    evidence: Option<String>,
    source_document: Option<String>,
}

/// `PUT /api/oversight/measures/{pid}/readings` — enter or replace a month's reading. A value needs
/// an evidence note and a named source document.
#[debug_handler]
async fn put_reading(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(p): Json<ReadingPayload>,
) -> Result<Response> {
    let m = measure_in_scope(&ctx, &caller, records::parse_pid(&pid)?).await?;
    if p.value.is_some() {
        rules::validate_reported(
            p.evidence.as_deref().unwrap_or(""),
            p.source_document.as_deref().unwrap_or(""),
        )
        .map_err(|e| unprocessable(&e))?;
    }
    let mut problems = Problems::new();
    problems.cap_opt("evidence", p.evidence.as_deref());
    problems.cap_opt("source_document", p.source_document.as_deref());
    ensure_valid(&problems.into_vec())?;
    let month = p
        .period_on
        .with_day(1)
        .ok_or_else(|| unprocessable("period_on is not a date"))?;
    let by = caller.actor().map(ToString::to_string);
    let existing = readings::Entity::find()
        .filter(readings::Column::MeasurePid.eq(m.pid))
        .filter(readings::Column::PeriodOn.eq(month))
        .one(&ctx.db)
        .await?;
    let row = if let Some(row) = existing {
        let mut a: readings::ActiveModel = row.into();
        a.value = ActiveValue::set(p.value);
        a.evidence = ActiveValue::set(p.evidence);
        a.source_document = ActiveValue::set(p.source_document);
        a.recorded_by = ActiveValue::set(by);
        a.update(&ctx.db).await?
    } else {
        readings::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            measure_pid: ActiveValue::set(m.pid),
            period_on: ActiveValue::set(month),
            value: ActiveValue::set(p.value),
            evidence: ActiveValue::set(p.evidence),
            source_document: ActiveValue::set(p.source_document),
            recorded_by: ActiveValue::set(by),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?
    };
    audit(&ctx, &caller, "oversight_reading", row.pid, "entered").await?;
    format::json(json!({
        "pid": row.pid, "measure_pid": row.measure_pid, "period_on": row.period_on, "value": row.value,
        "evidence": row.evidence, "source_document": row.source_document,
    }))
}

// ─── The status ─────────────────────────────────────────────────────────────

/// Query for the status and the scale.
#[derive(Debug, Deserialize)]
pub struct OrgQuery {
    organization: Option<String>,
}

/// `GET /api/oversight/status?organization=` — the scale, the recorded positions (newest first) and,
/// for the criteria of the level the organization is at, whether each sustain rule is met as of the
/// last complete month. The service never changes the level.
#[debug_handler]
async fn status(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(q): axum::extract::Query<OrgQuery>,
) -> Result<Response> {
    let org = q
        .organization
        .ok_or_else(|| unprocessable("organization is required"))?;
    ensure_org_in_scope(&ctx, &caller, &org).await?;
    let today = chrono::Utc::now().date_naive();
    let scale = levels::Entity::find()
        .filter(levels::Column::OrganizationRef.eq(&org))
        .order_by_asc(levels::Column::Ordinal)
        .all(&ctx.db)
        .await?;
    let by_pid: BTreeMap<Uuid, &levels::Model> = scale.iter().map(|l| (l.pid, l)).collect();
    let history = positions::Entity::find()
        .filter(positions::Column::OrganizationRef.eq(&org))
        .order_by_desc(positions::Column::EffectiveOn)
        .order_by_desc(positions::Column::Id)
        .all(&ctx.db)
        .await?;
    let current = history.iter().find(|p| p.effective_on <= today);
    let latest = rules::month_period(today.year(), today.month()) - 1;
    let mut verdicts = Vec::new();
    if let Some(level) = current.and_then(|p| by_pid.get(&p.level_pid)) {
        let cs = criteria::Entity::find()
            .filter(criteria::Column::LevelPid.eq(level.pid))
            .order_by_asc(criteria::Column::Id)
            .all(&ctx.db)
            .await?;
        for c in &cs {
            let ms = measures::Entity::find()
                .filter(measures::Column::CriterionPid.eq(c.pid))
                .order_by_asc(measures::Column::Id)
                .all(&ctx.db)
                .await?;
            let mut input = Vec::new();
            for m in &ms {
                let rs: Vec<rules::Reading> = readings::Entity::find()
                    .filter(readings::Column::MeasurePid.eq(m.pid))
                    .all(&ctx.db)
                    .await?
                    .into_iter()
                    .map(|r| rules::Reading {
                        period: rules::month_period(r.period_on.year(), r.period_on.month()),
                        value: r.value,
                        origin: rules::Origin::Reported,
                    })
                    .collect();
                let direction =
                    rules::Direction::parse(&m.direction).unwrap_or(rules::Direction::AtMost);
                let spec = rules::Spec {
                    threshold: m.threshold,
                    direction,
                    sustain: u32::try_from(m.sustain).unwrap_or(1),
                };
                input.push((spec, rs));
            }
            let verdict = rules::evaluate_criterion(&input, latest);
            verdicts.push(json!({
                "criterion_pid": c.pid,
                "criterion": c.name,
                "status": verdict.status.as_str(),
                "measures": ms.iter().zip(&verdict.measures).map(|(m, v)| json!({
                    "measure_pid": m.pid, "name": m.name, "origin": m.origin, "unit": m.unit,
                    "threshold": m.threshold, "direction": m.direction,
                    "status": v.status.as_str(), "run": v.run, "required": v.required,
                    "stopped_by": reason_json(v.reason),
                    "reported_periods": v.reported_periods,
                })).collect::<Vec<_>>(),
            }));
        }
    }
    format::json(json!({
        "organization_ref": org,
        "as_of_month": month_label(latest),
        "scale": scale.iter().map(level_json).collect::<Vec<_>>(),
        "current": current.map(|p| position_json(p, by_pid.get(&p.level_pid).copied())),
        "positions": history.iter().map(|p| position_json(p, by_pid.get(&p.level_pid).copied())).collect::<Vec<_>>(),
        "criteria": verdicts,
        "basis": "The criteria shown are the exit criteria of the level the organization is at. A criterion is \
                  met when every measure has met its threshold for its sustain rule of consecutive months, \
                  up to the last complete month. A missing or null reading is never a pass. This is evidence: \
                  the service never changes a level, and a person records any change with the deciding body.",
    }))
}

fn month_label(period: i64) -> String {
    format!(
        "{:04}-{:02}",
        period.div_euclid(12),
        period.rem_euclid(12) + 1
    )
}

fn reason_json(reason: rules::Reason) -> serde_json::Value {
    match reason {
        rules::Reason::Sustained => json!({ "reason": "sustained" }),
        rules::Reason::Breach(p) => json!({ "reason": "breach", "month": month_label(p) }),
        rules::Reason::NoReading(p) => json!({ "reason": "no_reading", "month": month_label(p) }),
        rules::Reason::NullReading(p) => {
            json!({ "reason": "null_reading", "month": month_label(p) })
        }
        rules::Reason::NoMeasures => json!({ "reason": "no_measures" }),
    }
}

/// The oversight routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/oversight")
        .add("/levels", post(create_level))
        .add("/levels/{pid}", delete(delete_level))
        .add("/positions", post(record_position))
        .add("/criteria", post(create_criterion))
        .add("/criteria/{pid}", delete(delete_criterion))
        .add("/measures", post(create_measure))
        .add("/measures/{pid}", delete(delete_measure))
        .add("/measures/{pid}/readings", put(put_reading))
        .add("/status", get(status))
}
