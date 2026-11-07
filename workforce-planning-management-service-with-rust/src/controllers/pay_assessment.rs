//! The joint pay assessment signal (WPM-R146, WPM-D99): whether a pay report makes an assessment with workers'
//! representatives due for a category of workers, and the record a person keeps of what was justified, remedied and
//! assessed. It reports; a person decides and records. Spec: `spec/pay-transparency.md`.

use axa::extract::Json;
use axa::prelude::*;
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::{ReportQuery, build};
use super::{ensure_org_in_scope, ensure_valid, row_json, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db::{self, pool};
use crate::models::_entities::pay_assessment_records;
use crate::rules::joint_assessment::{self as rules, Config, Record};
use crate::validation::Problems;

use super::jurisdictions::Scope;
use super::statutory_defaults as statutory;

/// The joint assessment's threshold and period in force, and where each came from.
pub(crate) struct Configured {
    /// The threshold and the period.
    pub config: Config,
    /// Where the threshold came from (WPM-D153).
    pub threshold_from: serde_json::Value,
    /// Where the period came from.
    pub months_from: serde_json::Value,
}

/// The threshold (percent, a whole number) and the period (calendar months): the organization's
/// profile over its home jurisdiction's chain (`pay_transparency.joint_assessment_percent`,
/// `pay_transparency.joint_assessment_months`), else the environment, else the Directive's. A pair
/// that does not check together falls back to the defaults with a warning.
pub(crate) fn configured(scope: Option<&Scope>, as_of: NaiveDate) -> Configured {
    let threshold_setting = crate::compat::env_var("WPM_JOINT_ASSESSMENT_THRESHOLD_PERCENT")
        .and_then(|v| v.trim().parse::<i64>().ok());
    let months_setting = crate::compat::env_var("WPM_JOINT_ASSESSMENT_MONTHS")
        .and_then(|v| v.trim().parse::<u32>().ok());
    let percent = statutory::integer(
        scope,
        "WPM_JOINT_ASSESSMENT_THRESHOLD_PERCENT",
        as_of,
        threshold_setting,
        rules::DEFAULT_THRESHOLD_BP / 100,
    );
    let months = statutory::integer(
        scope,
        "WPM_JOINT_ASSESSMENT_MONTHS",
        as_of,
        months_setting,
        rules::DEFAULT_REMEDY_MONTHS,
    );
    match rules::config(percent.value * 100, months.value) {
        Ok(config) => Configured {
            config,
            threshold_from: percent.came_from,
            months_from: months.came_from,
        },
        Err(why) => {
            tracing::warn!("joint assessment settings ignored: {why}");
            let default = |setting: &str| {
                serde_json::json!({ "kind": "default", "setting": setting,
                    "default_from": crate::statutory_defaults::find(setting).map(|d| d.origin) })
            };
            Configured {
                config: Config::default(),
                threshold_from: default("WPM_JOINT_ASSESSMENT_THRESHOLD_PERCENT"),
                months_from: default("WPM_JOINT_ASSESSMENT_MONTHS"),
            }
        }
    }
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
    axa::extract::Query(query): axa::extract::Query<SignalQuery>,
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
    let configured = configured(
        Some(&Scope::for_organization(&ctx.db, &query.organization).await?),
        chrono::Utc::now().date_naive(),
    );
    let config = configured.config;
    let records = sqlx::query_as!(
        pay_assessment_records::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, organization_ref, category, reported_on, justified, justification,
                  remedied_on, assessed_on, outcome, representatives, recorded_by
           FROM pay_assessment_records
           WHERE organization_ref = $1
           ORDER BY reported_on DESC"#,
        query.organization,
    )
    .fetch_all(pool(&ctx))
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
    db::audit(
        &mut *pool(&ctx).acquire().await?,
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
        // Whose threshold and period these are (WPM-T384).
        "threshold_basis": statutory::basis("WPM_JOINT_ASSESSMENT_THRESHOLD_PERCENT", &configured.threshold_from),
        "remedy_basis": statutory::basis("WPM_JOINT_ASSESSMENT_MONTHS", &configured.months_from),
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
    axa::extract::Query(query): axa::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let rows = sqlx::query_as!(
        pay_assessment_records::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, organization_ref, category, reported_on, justified, justification,
                  remedied_on, assessed_on, outcome, representatives, recorded_by
           FROM pay_assessment_records
           WHERE organization_ref = $1
           ORDER BY reported_on DESC, category ASC"#,
        query.organization,
    )
    .fetch_all(pool(&ctx))
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

/// The trimmed category of a record that passes every check.
fn checked_category(payload: &RecordPayload) -> Result<String> {
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
    Ok(category)
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
    let category = checked_category(&payload)?;
    let existing = sqlx::query!(
        "SELECT pid, category FROM pay_assessment_records
         WHERE organization_ref = $1 AND reported_on = $2",
        payload.organization_ref,
        payload.reported_on,
    )
    .fetch_all(pool(&ctx))
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
    let mut tx = pool(&ctx).begin().await?;
    let pid = if let Some(row) = existing {
        sqlx::query!(
            "UPDATE pay_assessment_records
             SET category = $2, justified = $3, justification = $4, remedied_on = $5,
                 assessed_on = $6, outcome = $7, representatives = $8, recorded_by = $9
             WHERE pid = $1",
            row.pid,
            category,
            payload.justified,
            justification,
            payload.remedied_on,
            payload.assessed_on,
            payload.outcome,
            representatives,
            caller.actor(),
        )
        .execute(&mut *tx)
        .await?;
        row.pid
    } else {
        let pid = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO pay_assessment_records
                 (pid, organization_ref, category, reported_on, justified, justification,
                  remedied_on, assessed_on, outcome, representatives, recorded_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            pid,
            payload.organization_ref,
            category,
            payload.reported_on,
            payload.justified,
            justification,
            payload.remedied_on,
            payload.assessed_on,
            payload.outcome,
            representatives,
            caller.actor(),
        )
        .execute(&mut *tx)
        .await?;
        pid
    };
    db::audit(
        &mut tx,
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
    tx.commit().await?;
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
