//! The joint pay assessment signal (WPM-R146, WPM-D99): whether a pay report makes an assessment with workers'
//! representatives due for a category of workers, and the record a person keeps of what was justified, remedied and
//! assessed. It reports; a person decides and records. Spec: `spec/pay-transparency.md`.

use chrono::NaiveDate;
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::{ReportQuery, build};
use super::{ensure_org_in_scope, ensure_valid, row_json, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::pay_assessment_records;
use crate::models::audit_logs::Model as Audit;
use crate::rules::joint_assessment::{self as rules, Config, Record};
use crate::validation::Problems;

/// The threshold (percent, a whole number) and the period (calendar months) from the environment, else the Directive's.
fn configured() -> Config {
    let threshold = crate::compat::env_var("WPM_JOINT_ASSESSMENT_THRESHOLD_PERCENT")
        .and_then(|v| v.trim().parse::<i64>().ok())
        .map_or(rules::DEFAULT_THRESHOLD_BP, |p| p * 100);
    let months = crate::compat::env_var("WPM_JOINT_ASSESSMENT_MONTHS")
        .and_then(|v| v.trim().parse::<u32>().ok())
        .unwrap_or(rules::DEFAULT_REMEDY_MONTHS);
    rules::config(threshold, months).unwrap_or_else(|why| {
        tracing::warn!("joint assessment settings ignored: {why}");
        Config::default()
    })
}

fn organization_problems(value: &str) -> Vec<String> {
    let mut problems = Problems::new();
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        value,
    );
    problems.into_vec()
}

fn record_of(row: &pay_assessment_records::Model) -> Record {
    Record {
        justified: row.justified,
        remedied_on: row.remedied_on,
        assessed_on: row.assessed_on,
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SignalQuery {
    organization: String,
    as_of: Option<NaiveDate>,
    currency: Option<String>,
    variable: Option<String>,
    /// The date of reporting; default the latest recorded for each category.
    reported_on: Option<NaiveDate>,
}

/// `GET /api/pay-transparency/joint-assessment?organization=&reported_on=&as_of=&currency=` — for each category, whether a
/// joint pay assessment is due, pending or not due, from the category's gap in basic pay, the threshold and period, and what
/// a person recorded. Needs the same unmasked salary read as the report; the read is audited with no figure.
#[debug_handler]
pub(crate) async fn signals(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<SignalQuery>,
) -> Result<Response> {
    let built = build(
        &ctx,
        &caller,
        &ReportQuery {
            organization: query.organization.clone(),
            as_of: query.as_of,
            currency: query.currency.clone(),
            variable: query.variable.clone(),
        },
    )
    .await?;
    let config = configured();
    let records = pay_assessment_records::Entity::find()
        .filter(pay_assessment_records::Column::OrganizationRef.eq(&query.organization))
        .order_by_desc(pay_assessment_records::Column::ReportedOn)
        .all(&ctx.db)
        .await?;
    let mut out = Vec::new();
    for c in &built.report.categories {
        let name = c.category.to_lowercase();
        let latest = records.iter().find(|r| r.category.to_lowercase() == name);
        let reported_on = query.reported_on.or(latest.map(|r| r.reported_on));
        let record = records
            .iter()
            .find(|r| r.category.to_lowercase() == name && Some(r.reported_on) == reported_on);
        let signal = rules::signal(
            &c.basic,
            record.map(record_of).as_ref(),
            reported_on,
            config,
            built.as_of,
        );
        out.push(json!({
            "category": c.category,
            "gap": c.basic,
            "reported_on": reported_on,
            "signal": signal,
            "record": record.map(row_json).transpose()?,
        }));
    }
    Audit::record(
        &ctx.db,
        "pay_gap_report",
        Uuid::nil(),
        "joint_assessment_read",
        caller.actor(),
        Some(json!({ "as_of": built.as_of, "categories": built.category_count })),
    )
    .await?;
    format::json(json!({
        "organization": built.organization,
        "as_of": built.as_of,
        "threshold_bp": config.threshold_bp,
        "remedy_months": config.remedy_months,
        "categories": out,
        "derivation": "A joint pay assessment is due when a category's average pay gap in basic pay is at or over the threshold, \
                       either way, the employer has not recorded a justification on objective, gender-neutral criteria, and has \
                       not remedied the gap by the last day of the period counted from the date of reporting. This reports; a \
                       person records the justification, the remedy and the assessment, and nothing here says anyone was paid unfairly.",
    }))
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListQuery {
    organization: String,
}

/// `GET /api/pay-transparency/assessment-records?organization=` — what has been recorded, newest report first.
#[debug_handler]
pub(crate) async fn list_records(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let rows = pay_assessment_records::Entity::find()
        .filter(pay_assessment_records::Column::OrganizationRef.eq(&query.organization))
        .order_by_desc(pay_assessment_records::Column::ReportedOn)
        .order_by_asc(pay_assessment_records::Column::Category)
        .all(&ctx.db)
        .await?;
    format::json(
        rows.iter()
            .map(row_json)
            .collect::<Result<Vec<serde_json::Value>>>()?,
    )
}

#[derive(Debug, Deserialize)]
pub(crate) struct RecordPayload {
    organization_ref: String,
    category: String,
    reported_on: NaiveDate,
    #[serde(default)]
    justified: bool,
    justification: Option<String>,
    remedied_on: Option<NaiveDate>,
    assessed_on: Option<NaiveDate>,
    outcome: Option<String>,
    /// The body or group of workers' representatives the assessment was done with, by name (never a person).
    representatives: Option<String>,
}

/// `PUT /api/pay-transparency/assessment-records` — record, for a category and a report date, whether the gap was
/// justified (and on what), when it was remedied, and whether the joint assessment was carried out and how it ended.
/// Replaces what was there. A person records; the service never decides.
#[debug_handler]
pub(crate) async fn put_record(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<RecordPayload>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&payload.organization_ref))?;
    ensure_org_in_scope(&ctx, &caller, &payload.organization_ref).await?;
    let category = payload.category.trim().to_string();
    if category.is_empty() || category.chars().count() > 120 {
        return Err(unprocessable("category is 1 to 120 characters"));
    }
    if payload
        .representatives
        .as_ref()
        .is_some_and(|r| r.chars().count() > 200)
    {
        return Err(unprocessable("representatives is at most 200 characters"));
    }
    rules::validate_record(
        payload.reported_on,
        payload.justified,
        payload.justification.as_deref(),
        payload.remedied_on,
        payload.assessed_on,
        payload.outcome.as_deref(),
    )
    .map_err(|e| unprocessable(&e))?;
    let existing = pay_assessment_records::Entity::find()
        .filter(pay_assessment_records::Column::OrganizationRef.eq(&payload.organization_ref))
        .filter(pay_assessment_records::Column::ReportedOn.eq(payload.reported_on))
        .all(&ctx.db)
        .await?
        .into_iter()
        .find(|r| r.category.to_lowercase() == category.to_lowercase());
    let justification = payload
        .justification
        .as_ref()
        .map(|t| t.trim().to_string())
        .filter(|_| payload.justified);
    let representatives = payload
        .representatives
        .as_ref()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let txn = ctx.db.begin().await?;
    let pid = if let Some(row) = existing {
        let pid = row.pid;
        let mut active: pay_assessment_records::ActiveModel = row.into();
        active.category = ActiveValue::set(category);
        active.justified = ActiveValue::set(payload.justified);
        active.justification = ActiveValue::set(justification);
        active.remedied_on = ActiveValue::set(payload.remedied_on);
        active.assessed_on = ActiveValue::set(payload.assessed_on);
        active.outcome = ActiveValue::set(payload.outcome.clone());
        active.representatives = ActiveValue::set(representatives);
        active.recorded_by = ActiveValue::set(caller.actor().map(ToString::to_string));
        active.update(&txn).await?;
        pid
    } else {
        let pid = Uuid::new_v4();
        pay_assessment_records::ActiveModel {
            pid: ActiveValue::set(pid),
            organization_ref: ActiveValue::set(payload.organization_ref.clone()),
            category: ActiveValue::set(category),
            reported_on: ActiveValue::set(payload.reported_on),
            justified: ActiveValue::set(payload.justified),
            justification: ActiveValue::set(justification),
            remedied_on: ActiveValue::set(payload.remedied_on),
            assessed_on: ActiveValue::set(payload.assessed_on),
            outcome: ActiveValue::set(payload.outcome.clone()),
            representatives: ActiveValue::set(representatives),
            recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
        pid
    };
    Audit::record(
        &txn,
        "pay_assessment",
        pid,
        "pay_assessment_recorded",
        caller.actor(),
        Some(json!({
            "justified": payload.justified,
            "remedied": payload.remedied_on.is_some(),
            "assessed": payload.assessed_on.is_some(),
        })),
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/pay-transparency/joint-assessment", get(signals))
        .add("/pay-transparency/assessment-records", get(list_records))
        .add("/pay-transparency/assessment-records", put(put_record))
}
