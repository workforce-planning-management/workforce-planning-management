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

use super::ensure_org_in_scope;
use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::Datelike;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::{ensure_valid, oversight_derived, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::models::_entities::{
    oversight_criteria as criteria, oversight_levels as levels, oversight_measures as measures,
    oversight_positions as positions,
};
use crate::models::records;
use crate::rules::derived;
use crate::rules::oversight as rules;
use crate::validation::Problems;

fn org_problems(org: &str) -> Result<()> {
    let mut problems = Problems::new();
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        org,
    );
    ensure_valid(&problems.into_vec())
}

async fn level_in_scope(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: Uuid,
) -> Result<levels::Model> {
    let level = sqlx::query_as!(
        levels::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, ordinal, name,
                  synthetic
             FROM oversight_levels WHERE pid = $1"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
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
    let c = sqlx::query_as!(
        criteria::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, level_pid, name, synthetic
             FROM oversight_criteria WHERE pid = $1"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
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
    let m = sqlx::query_as!(
        measures::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, criterion_pid, name, origin,
                  derivation, unit, threshold, direction, sustain
             FROM oversight_measures WHERE pid = $1"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    criterion_in_scope(ctx, caller, m.criterion_pid).await?;
    Ok(m)
}

/// The measures behind one criterion, or behind any of several, in the order they were added.
async fn measures_of(ctx: &AppContext, criteria: &[Uuid]) -> Result<Vec<measures::Model>> {
    Ok(sqlx::query_as!(
        measures::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, criterion_pid, name, origin,
                  derivation, unit, threshold, direction, sustain
             FROM oversight_measures WHERE criterion_pid = ANY($1) ORDER BY id"#,
        criteria,
    )
    .fetch_all(db::pool(ctx))
    .await?)
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
    let taken = sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM oversight_levels WHERE organization_ref = $1 AND ordinal = $2 LIMIT 1"#,
        p.organization_ref,
        p.ordinal,
    )
    .fetch_optional(db::pool(&ctx))
    .await?;
    if taken.is_some() {
        return Err(unprocessable("that ordinal is already on the scale"));
    }
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        levels::Model,
        r#"INSERT INTO oversight_levels (pid, organization_ref, ordinal, name, synthetic) VALUES ($1, $2, $3, $4, false)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, ordinal, name,
                     synthetic"#,
        Uuid::new_v4(),
        p.organization_ref,
        p.ordinal,
        p.name,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "oversight_level",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let used = sqlx::query_scalar!(
        r#"SELECT (EXISTS (SELECT 1 FROM oversight_positions WHERE level_pid = $1)
                OR EXISTS (SELECT 1 FROM oversight_criteria WHERE level_pid = $1)) AS "used!""#,
        level.pid,
    )
    .fetch_one(db::pool(&ctx))
    .await?;
    if used {
        return Err(unprocessable(
            "a level with a recorded position or a criterion cannot be removed; remove the criteria first, and a recorded position stays on the record",
        ));
    }
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!("DELETE FROM oversight_levels WHERE id = $1", level.id)
        .execute(&mut *tx)
        .await?;
    db::audit(
        &mut tx,
        "oversight_level",
        level.pid,
        "removed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        positions::Model,
        r#"INSERT INTO oversight_positions (pid, organization_ref, level_pid, effective_on, deciding_body, reason, recorded_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, level_pid,
                     effective_on, deciding_body, reason, recorded_by"#,
        Uuid::new_v4(),
        p.organization_ref,
        level.pid,
        p.effective_on,
        p.deciding_body,
        p.reason,
        caller.actor(),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "oversight_position",
        row.pid,
        "recorded",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO oversight_criteria (pid, level_pid, name, synthetic) VALUES ($1, $2, $3, false)",
        pid,
        level.pid,
        p.name,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "oversight_criterion",
        pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid, "level_pid": level.pid, "name": p.name }))
}

/// `DELETE /api/oversight/criteria/{pid}` — with its measures and their readings.
#[debug_handler]
async fn delete_criterion(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let c = criterion_in_scope(&ctx, &caller, records::parse_pid(&pid)?).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "DELETE FROM oversight_readings
          WHERE measure_pid IN (SELECT pid FROM oversight_measures WHERE criterion_pid = $1)",
        c.pid,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "DELETE FROM oversight_measures WHERE criterion_pid = $1",
        c.pid
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!("DELETE FROM oversight_criteria WHERE id = $1", c.id)
        .execute(&mut *tx)
        .await?;
    db::audit(
        &mut tx,
        "oversight_criterion",
        c.pid,
        "removed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "removed": c.pid }))
}

/// `POST /api/oversight/measures` body.
#[derive(Debug, Deserialize)]
struct MeasurePayload {
    criterion_pid: Uuid,
    name: String,
    origin: String,
    /// Which computation supplies the readings; required for `derived`, refused for `reported`.
    derivation: Option<String>,
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
            "origin must be one of {:?}",
            rules::AVAILABLE_ORIGINS
        )));
    }
    check_derivation(&p)?;
    let c = criterion_in_scope(&ctx, &caller, p.criterion_pid).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        measures::Model,
        r#"INSERT INTO oversight_measures (pid, criterion_pid, name, origin, derivation, unit, threshold, direction, sustain)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, criterion_pid, name, origin,
                     derivation, unit, threshold, direction, sustain"#,
        Uuid::new_v4(),
        c.pid,
        p.name,
        p.origin,
        p.derivation,
        p.unit,
        p.threshold,
        p.direction,
        i32::try_from(p.sustain).unwrap_or(1),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "oversight_measure",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(measure_json(&row))
}

/// A derived measure names a known computation and takes its unit, so a threshold cannot be read in
/// another; a reported measure names none.
fn check_derivation(p: &MeasurePayload) -> Result<()> {
    match (p.origin.as_str(), p.derivation.as_deref()) {
        ("derived", Some(key)) => {
            let d = derived::find(key).ok_or_else(|| {
                unprocessable(&format!(
                    "derivation must be one of {:?}",
                    derived::DERIVATIONS
                        .iter()
                        .map(|d| d.key)
                        .collect::<Vec<_>>()
                ))
            })?;
            if p.unit != d.unit {
                return Err(unprocessable(&format!(
                    "the unit of {key} is \"{}\"",
                    d.unit
                )));
            }
            Ok(())
        }
        ("derived", None) => Err(unprocessable("a derived measure needs a derivation")),
        (_, Some(_)) => Err(unprocessable("only a derived measure has a derivation")),
        _ => Ok(()),
    }
}

/// `GET /api/oversight/derivations` — the computations a derived measure may name, each with its unit
/// and its definition (what is counted, what is left out, the period, when it is absent).
#[debug_handler]
async fn list_derivations() -> Result<Response> {
    format::json(json!({
        "derivations": derived::DERIVATIONS.iter().map(|d| json!({
            "key": d.key, "unit": d.unit, "definition": d.definition,
        })).collect::<Vec<_>>(),
    }))
}

fn measure_json(m: &measures::Model) -> serde_json::Value {
    json!({
        "pid": m.pid, "criterion_pid": m.criterion_pid, "name": m.name, "origin": m.origin,
        "derivation": m.derivation, "unit": m.unit,
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
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "DELETE FROM oversight_readings WHERE measure_pid = $1",
        m.pid
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!("DELETE FROM oversight_measures WHERE id = $1", m.id)
        .execute(&mut *tx)
        .await?;
    db::audit(
        &mut tx,
        "oversight_measure",
        m.pid,
        "removed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    if m.origin == "derived" {
        return Err(unprocessable(
            "a derived measure is computed from the data; it takes no entered reading",
        ));
    }
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
    let mut tx = db::pool(&ctx).begin().await?;
    // One reading per measure and month: a second one replaces the first and keeps its pid.
    let row = sqlx::query!(
        "INSERT INTO oversight_readings (pid, measure_pid, period_on, value, evidence, source_document, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (measure_pid, period_on)
         DO UPDATE SET value = EXCLUDED.value, evidence = EXCLUDED.evidence, source_document = EXCLUDED.source_document,
                       recorded_by = EXCLUDED.recorded_by
         RETURNING pid, measure_pid, period_on, value, evidence, source_document",
        Uuid::new_v4(),
        m.pid,
        month,
        p.value,
        p.evidence,
        p.source_document,
        caller.actor(),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "oversight_reading",
        row.pid,
        "entered",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({
        "pid": row.pid, "measure_pid": row.measure_pid, "period_on": row.period_on, "value": row.value,
        "evidence": row.evidence, "source_document": row.source_document,
    }))
}

/// The threshold, direction and sustain rule of a stored measure.
pub(super) fn spec_of(m: &measures::Model) -> rules::Spec {
    rules::Spec {
        threshold: m.threshold,
        direction: rules::Direction::parse(&m.direction).unwrap_or(rules::Direction::AtMost),
        sustain: u32::try_from(m.sustain).unwrap_or(1),
    }
}

/// The last complete month, as a period number.
pub(super) fn last_complete_month(today: chrono::NaiveDate) -> i64 {
    rules::month_period(today.year(), today.month()) - 1
}

/// The readings a measure is judged on: entered ones for a reported measure; for a derived one, the
/// `sustain` months a run needs, computed now.
pub(super) async fn measure_readings(
    ctx: &AppContext,
    org: &str,
    m: &measures::Model,
    latest: i64,
) -> Result<Vec<rules::Reading>> {
    if let Some(key) = m.derivation.as_deref() {
        let first = latest - i64::from(m.sustain) + 1;
        let periods: Vec<i64> = (first..=latest).collect();
        return oversight_derived::readings(ctx, org, key, &periods).await;
    }
    Ok(sqlx::query!(
        "SELECT period_on, value FROM oversight_readings WHERE measure_pid = $1 ORDER BY id",
        m.pid,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
    .map(|r| rules::Reading {
        period: rules::month_period(r.period_on.year(), r.period_on.month()),
        value: r.value,
        origin: rules::Origin::Reported,
    })
    .collect())
}

/// The measures behind the criteria of the level the organization is at on `today` (none when no
/// position is recorded).
pub(super) async fn current_measures(
    ctx: &AppContext,
    org: &str,
    today: chrono::NaiveDate,
) -> Result<Vec<measures::Model>> {
    let current = sqlx::query_scalar!(
        "SELECT level_pid FROM oversight_positions WHERE organization_ref = $1 AND effective_on <= $2
          ORDER BY effective_on DESC, id DESC LIMIT 1",
        org,
        today,
    )
    .fetch_optional(db::pool(ctx))
    .await?;
    let Some(level_pid) = current else {
        return Ok(Vec::new());
    };
    let pids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT pid FROM oversight_criteria WHERE level_pid = $1",
        level_pid,
    )
    .fetch_all(db::pool(ctx))
    .await?;
    measures_of(ctx, &pids).await
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
    axa::extract::Query(q): axa::extract::Query<OrgQuery>,
) -> Result<Response> {
    let org = q
        .organization
        .ok_or_else(|| unprocessable("organization is required"))?;
    ensure_org_in_scope(&ctx, &caller, &org).await?;
    let today = chrono::Utc::now().date_naive();
    format::json(status_json(&ctx, &org, today).await?)
}

/// The status of the organization as of `today`: the same payload the status route returns and the
/// monthly pack freezes.
pub(super) async fn status_json(
    ctx: &AppContext,
    org: &str,
    today: chrono::NaiveDate,
) -> Result<serde_json::Value> {
    // The scale by ordinal; an ordinal is unique on an organization's scale, and the id breaks any tie.
    let scale = sqlx::query_as!(
        levels::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, ordinal, name,
                  synthetic
             FROM oversight_levels WHERE organization_ref = $1 ORDER BY ordinal, id"#,
        org,
    )
    .fetch_all(db::pool(ctx))
    .await?;
    let by_pid: BTreeMap<Uuid, &levels::Model> = scale.iter().map(|l| (l.pid, l)).collect();
    let history = sqlx::query_as!(
        positions::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, level_pid,
                  effective_on, deciding_body, reason, recorded_by
             FROM oversight_positions WHERE organization_ref = $1 ORDER BY effective_on DESC, id DESC"#,
        org,
    )
    .fetch_all(db::pool(ctx))
    .await?;
    let current = history.iter().find(|p| p.effective_on <= today);
    let latest = last_complete_month(today);
    let mut verdicts = Vec::new();
    if let Some(level) = current.and_then(|p| by_pid.get(&p.level_pid)) {
        let cs = sqlx::query!(
            "SELECT pid, name FROM oversight_criteria WHERE level_pid = $1 ORDER BY id",
            level.pid,
        )
        .fetch_all(db::pool(ctx))
        .await?;
        for c in &cs {
            let ms = measures_of(ctx, &[c.pid]).await?;
            let mut input = Vec::new();
            let mut shown: Vec<serde_json::Value> = Vec::new();
            for m in &ms {
                let rs = measure_readings(ctx, org, m, latest).await?;
                shown.push(if m.derivation.is_some() {
                    json!(
                        rs.iter()
                            .map(|r| json!({
                                "month": month_label(r.period), "value": r.value,
                            }))
                            .collect::<Vec<_>>()
                    )
                } else {
                    json!(null)
                });
                input.push((spec_of(m), rs));
            }
            let verdict = rules::evaluate_criterion(&input, latest);
            verdicts.push(json!({
                "criterion_pid": c.pid,
                "criterion": c.name,
                "status": verdict.status.as_str(),
                "measures": ms.iter().zip(&verdict.measures).zip(&shown).map(|((m, v), values)| json!({
                    "measure_pid": m.pid, "name": m.name, "origin": m.origin,
                    "derivation": m.derivation, "unit": m.unit, "derived_values": values,
                    "threshold": m.threshold, "direction": m.direction,
                    "status": v.status.as_str(), "run": v.run, "required": v.required,
                    "stopped_by": reason_json(v.reason),
                    "reported_periods": v.reported_periods,
                })).collect::<Vec<_>>(),
            }));
        }
    }
    Ok(json!({
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
        .add("/derivations", get(list_derivations))
        .add("/status", get(status))
}
