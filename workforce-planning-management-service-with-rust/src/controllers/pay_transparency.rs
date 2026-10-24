//! Pay transparency (WPM-R144, WPM-D99): the categories of workers an employer reports pay gaps by, and the report
//! itself. Spec: `spec/pay-transparency.md`.
//!
//! - **Categories** are the employer's own grouping of roles (job titles) that do the same work or work of equal
//!   value, with the basis stated. They hold no personal data.
//! - **The report** is an aggregate: the seven indicators of Article 9(1) with small cells withheld, and the
//!   workers left out counted. It names no one and carries no head count. It reads salaries, so it needs the same
//!   unmasked read as the plan cost view, and it reads the sex a worker chose to declare under equality monitoring
//!   (off until a deployer records a lawful basis), so it does not exist where that is off.
//! - **Pay components beyond salary are not held**, so the variable-pay indicators are `unknown` until a source exists.

use std::collections::BTreeMap;

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::{
    ensure_org_in_scope, ensure_valid, equality_monitoring, forbidden, record_rejection, row_json,
    unprocessable,
};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{equality_declarations, pay_categories, workers};
use crate::models::audit_logs::Model as Audit;
use crate::rules::cost as cost_rules;
use crate::rules::csv as csv_rules;
use crate::rules::metrics::is_employed_on;
use crate::rules::pay_category as category_rules;
use crate::rules::pay_gap::{
    self, Declared, Excluded, Figure, Gap, Report, Variable, VariableBasis, WorkerPay,
};
use crate::validation::Problems;

/// The equality category that holds a worker's sex; the deployer may name another.
const DEFAULT_SEX_CATEGORY: &str = "sex";

/// What the pay figures are made of, said in every report.
const PAY_BASIS: &str = "Annual full-time-equivalent basic salary as recorded on the worker record, in one currency. \
     Standard hours are taken as equal across workers, so this is comparable with hourly pay. Part-time pay is not scaled down.";
/// What is not held.
const VARIABLE_NOTE: &str = "WPM holds no bonus, allowance or payment in kind for a worker, so every variable component is \
     unknown, and the indicators that need one are unknown rather than zero.";

#[derive(Debug, Deserialize)]
pub(crate) struct CategoryPayload {
    organization_ref: String,
    name: String,
    basis: String,
    job_titles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListQuery {
    organization: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReportQuery {
    pub(crate) organization: String,
    /// The day the workforce is taken on; default today.
    pub(crate) as_of: Option<NaiveDate>,
    /// ISO-4217 currency; default the most common among workers with a salary.
    pub(crate) currency: Option<String>,
    /// `all_known` (default) or `recipients_only`: who a gap in variable pay is taken over.
    pub(crate) variable: Option<String>,
}

pub(crate) fn organization_problems(value: &str) -> Vec<String> {
    let mut problems = Problems::new();
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        value,
    );
    problems.into_vec()
}

async fn live_categories(
    ctx: &AppContext,
    organization: &str,
) -> Result<Vec<pay_categories::Model>> {
    Ok(pay_categories::Entity::find()
        .filter(pay_categories::Column::OrganizationRef.eq(organization))
        .filter(pay_categories::Column::DeletedAt.is_null())
        .order_by_asc(pay_categories::Column::Name)
        .all(&ctx.db)
        .await?)
}

fn titles_of(row: &pay_categories::Model) -> Vec<String> {
    serde_json::from_value(row.job_titles.clone()).unwrap_or_default()
}

/// The checks both writes share: valid fields, and no job title another category already holds.
async fn checked(
    ctx: &AppContext,
    payload: &CategoryPayload,
    except: Option<Uuid>,
) -> Result<Vec<String>> {
    ensure_valid(&organization_problems(&payload.organization_ref))?;
    let titles = category_rules::validate(&payload.name, &payload.basis, &payload.job_titles)
        .map_err(|e| unprocessable(&e))?;
    let others: Vec<pay_categories::Model> = live_categories(ctx, &payload.organization_ref)
        .await?
        .into_iter()
        .filter(|c| Some(c.pid) != except)
        .collect();
    if others
        .iter()
        .any(|c| c.name.trim().to_lowercase() == payload.name.trim().to_lowercase())
    {
        return Err(unprocessable(
            "another category in this organization has that name",
        ));
    }
    let held: Vec<(String, Vec<String>)> = others
        .iter()
        .map(|c| (c.name.clone(), titles_of(c)))
        .collect();
    if let Some((title, category)) = category_rules::clash(&held, &titles) {
        return Err(unprocessable(&format!(
            "the job title \"{title}\" is already in the category \"{category}\"; a title is in one category"
        )));
    }
    Ok(titles)
}

/// `GET /api/pay-categories?organization=` — the categories of workers, with their stated basis.
#[debug_handler]
pub(crate) async fn list_categories(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let rows = live_categories(&ctx, &query.organization).await?;
    format::json(
        rows.iter()
            .map(row_json)
            .collect::<Result<Vec<serde_json::Value>>>()?,
    )
}

/// `POST /api/pay-categories` — set up a category.
#[debug_handler]
pub(crate) async fn create_category(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<CategoryPayload>,
) -> Result<Response> {
    ensure_org_in_scope(&ctx, &caller, &payload.organization_ref).await?;
    let titles = checked(&ctx, &payload, None).await?;
    let pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    pay_categories::ActiveModel {
        pid: ActiveValue::set(pid),
        organization_ref: ActiveValue::set(payload.organization_ref.clone()),
        name: ActiveValue::set(payload.name.trim().to_string()),
        basis: ActiveValue::set(payload.basis.trim().to_string()),
        job_titles: ActiveValue::set(json!(titles)),
        set_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "pay_category",
        pid,
        "pay_category_set",
        caller.actor(),
        Some(json!({ "titles": titles.len() })),
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

async fn find_category(ctx: &AppContext, pid: &str) -> Result<pay_categories::Model> {
    let pid = Uuid::parse_str(pid).map_err(|_| Error::NotFound)?;
    pay_categories::Entity::find()
        .filter(pay_categories::Column::Pid.eq(pid))
        .filter(pay_categories::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

/// `PUT /api/pay-categories/{pid}` — replace a category's name, basis and job titles. The organization stays.
#[debug_handler]
pub(crate) async fn change_category(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<CategoryPayload>,
) -> Result<Response> {
    let row = find_category(&ctx, &pid).await?;
    ensure_org_in_scope(&ctx, &caller, &row.organization_ref).await?;
    if payload.organization_ref != row.organization_ref {
        return Err(unprocessable(
            "a category cannot move to another organization",
        ));
    }
    let titles = checked(&ctx, &payload, Some(row.pid)).await?;
    let row_pid = row.pid;
    let txn = ctx.db.begin().await?;
    let mut active: pay_categories::ActiveModel = row.into();
    active.name = ActiveValue::set(payload.name.trim().to_string());
    active.basis = ActiveValue::set(payload.basis.trim().to_string());
    active.job_titles = ActiveValue::set(json!(titles));
    active.set_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    active.update(&txn).await?;
    Audit::record(
        &txn,
        "pay_category",
        row_pid,
        "pay_category_changed",
        caller.actor(),
        Some(json!({ "titles": titles.len() })),
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": row_pid }))
}

/// `DELETE /api/pay-categories/{pid}` — take a category away. Reports read afterwards count its workers as
/// uncategorised.
#[debug_handler]
pub(crate) async fn remove_category(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = find_category(&ctx, &pid).await?;
    ensure_org_in_scope(&ctx, &caller, &row.organization_ref).await?;
    let row_pid = row.pid;
    let txn = ctx.db.begin().await?;
    let mut active: pay_categories::ActiveModel = row.into();
    active.deleted_at = ActiveValue::set(Some(Utc::now().into()));
    active.update(&txn).await?;
    Audit::record(
        &txn,
        "pay_category",
        row_pid,
        "pay_category_removed",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::empty_json()
}

/// What the report and its CSV share.
pub(crate) struct Built {
    pub(crate) organization: String,
    pub(crate) as_of: NaiveDate,
    pub(crate) currency: Option<String>,
    pub(crate) variable: VariableBasis,
    pub(crate) report: Report,
    pub(crate) headcount: usize,
    pub(crate) category_count: usize,
}

fn declared(value: Option<&String>) -> Option<Declared> {
    value.map(|v| match v.as_str() {
        "female" => Declared::Female,
        "male" => Declared::Male,
        _ => Declared::Other,
    })
}

pub(crate) async fn build(ctx: &AppContext, caller: &MaybeAuthUser, query: &ReportQuery) -> Result<Built> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(ctx, caller, &query.organization).await?;
    // The same rule as the plan cost view: a figure made of salaries is for a caller who may read salaries.
    let obligations = auth::authorize_record(
        caller,
        authentication_verifier::Action::Read,
        &BTreeMap::new(),
    )
    .map_err(record_rejection)?;
    if obligations.iter().any(|o| o == "mask") {
        return Err(forbidden(
            "a pay gap report is made of salaries and needs payroll read access",
        ));
    }
    let variable = match query.variable.as_deref() {
        None | Some("all_known") => VariableBasis::AllKnown,
        Some("recipients_only") => VariableBasis::RecipientsOnly,
        Some(_) => {
            return Err(unprocessable(
                "variable must be all_known or recipients_only",
            ));
        }
    };
    // Sex is what a worker chose to declare, and only where the deployer has recorded a basis for asking.
    let (_, config, floor) = equality_monitoring::on().map_err(|_| {
        unprocessable("equality monitoring is off on this deployment, so no sex is recorded and there is nothing to compare")
    })?;
    let sex_category = crate::compat::env_var("WPM_PAY_GAP_SEX_CATEGORY")
        .unwrap_or_else(|| DEFAULT_SEX_CATEGORY.to_string());
    if !config.has_category(&sex_category) {
        return Err(unprocessable(
            "the sex category is not monitored on this deployment (WPM_PAY_GAP_SEX_CATEGORY names it)",
        ));
    }
    let as_of = query.as_of.unwrap_or_else(|| Utc::now().date_naive());
    let staff: Vec<workers::Model> = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&query.organization))
        .filter(workers::Column::DeletedAt.is_null())
        .order_by_asc(workers::Column::Id)
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| is_employed_on(as_of, w.hired_on, w.terminated_on))
        .collect();
    let answers: BTreeMap<Uuid, String> = equality_declarations::Entity::find()
        .filter(equality_declarations::Column::Category.eq(&sex_category))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|a| (a.worker_pid, a.value))
        .collect();
    let categories = live_categories(ctx, &query.organization).await?;
    let held: Vec<(String, Vec<String>)> = categories
        .iter()
        .map(|c| (c.name.clone(), titles_of(c)))
        .collect();
    let by_title = category_rules::index(&held);

    let currency = match &query.currency {
        Some(code) if cost_rules::valid_currency(code) => Some(code.clone()),
        Some(_) => return Err(unprocessable("currency must be an ISO-4217 code, e.g. GBP")),
        None => most_common_currency(&staff),
    };
    let pay: Vec<WorkerPay> = staff
        .iter()
        .map(|w| WorkerPay {
            key: w.pid.to_string(),
            sex: declared(answers.get(&w.pid)),
            category: by_title
                .get(&category_rules::title_key(&w.job_title))
                .cloned(),
            // Pay in another currency is not comparable: it counts as no pay recorded.
            basic_minor: w
                .salary_minor
                .filter(|_| currency.is_some() && w.salary_currency == currency),
            variable: Variable::Unknown,
        })
        .collect();
    let mut report = pay_gap::report(&pay, floor, variable).map_err(|e| unprocessable(&e))?;
    // A category the employer set up that no eligible worker is in is still listed, as having nobody, not left out.
    for c in &categories {
        if !report.categories.iter().any(|g| g.category == c.name) {
            report.categories.push(pay_gap::CategoryGap {
                category: c.name.clone(),
                basic: Figure::Unknown(pay_gap::Reason::NoWorkers),
                variable: Figure::Unknown(pay_gap::Reason::NoWorkers),
            });
        }
    }
    report.categories.sort_by(|a, b| a.category.cmp(&b.category));
    Ok(Built {
        organization: query.organization.clone(),
        as_of,
        currency,
        variable,
        report,
        headcount: staff.len(),
        category_count: categories.len(),
    })
}

fn most_common_currency(staff: &[workers::Model]) -> Option<String> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for w in staff {
        if w.salary_minor.is_some()
            && let Some(code) = &w.salary_currency
        {
            *counts.entry(code.as_str()).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(a.0)))
        .map(|(code, _)| code.to_string())
}

/// The audit row for a read of the report (`exported` for the CSV). It names the day and how many categories, no figure.
async fn audit_read(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    built: &Built,
    exported: bool,
) -> Result<()> {
    let snapshot = Some(json!({ "as_of": built.as_of, "categories": built.category_count }));
    if exported {
        Audit::record(
            &ctx.db,
            "pay_gap_report",
            Uuid::nil(),
            "pay_gap_report_exported",
            caller.actor(),
            snapshot,
        )
        .await?;
    } else {
        Audit::record(
            &ctx.db,
            "pay_gap_report",
            Uuid::nil(),
            "pay_gap_report_read",
            caller.actor(),
            snapshot,
        )
        .await?;
    }
    Ok(())
}

/// `GET /api/pay-transparency/gap-report?organization=&as_of=&currency=&variable=` — the seven indicators.
/// Aggregates with small cells withheld; no worker is named and no head count is given. The read is audited.
#[debug_handler]
pub(crate) async fn gap_report(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ReportQuery>,
) -> Result<Response> {
    let built = build(&ctx, &caller, &query).await?;
    audit_read(&ctx, &caller, &built, false).await?;
    let tiers = pay_gap::directive_tiers();
    let tier = pay_gap::tier_for(built.headcount, &tiers);
    format::json(json!({
        "organization": built.organization,
        "as_of": built.as_of,
        "currency": built.currency,
        "basis": {
            "pay": PAY_BASIS,
            "variable": VARIABLE_NOTE,
            "variable_gap_taken_over": match built.variable {
                VariableBasis::AllKnown => "all_known",
                VariableBasis::RecipientsOnly => "recipients_only",
            },
        },
        "reporting": tier.map(|t| json!({
            "every_years": t.every_years,
            "first_report_on": t.first_report_on,
            "next_report_on": pay_gap::next_due(t, built.as_of),
        })),
        "report": built.report,
        "derivation": "Counts and means come from the pay on the worker record and the sex each worker chose to declare. \
                       A figure shows only when both sexes reach the floor in it; a side with nobody is unknown; no figure \
                       is a head count. Workers left out are counted by reason, and a count under the floor is not exact.",
    }))
}

fn gap_row(indicator: &str, scope: &str, figure: &Figure<Gap>) -> String {
    let (status, reason, female, male, gap) = match figure {
        Figure::Shown(g) => (
            "shown",
            String::new(),
            g.female_minor.to_string(),
            g.male_minor.to_string(),
            g.gap_bp.to_string(),
        ),
        Figure::Withheld => (
            "withheld",
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ),
        Figure::Unknown(reason) => (
            "unknown",
            reason_text(*reason),
            String::new(),
            String::new(),
            String::new(),
        ),
    };
    csv_rules::row(&[
        csv_rules::field(indicator),
        csv_rules::field(scope),
        status.into(),
        reason,
        female,
        male,
        gap,
        String::new(),
        "minor_units_per_year".into(),
    ])
}

fn reason_text(reason: pay_gap::Reason) -> String {
    serde_json::to_value(reason)
        .ok()
        .and_then(|v| v.as_str().map(ToString::to_string))
        .unwrap_or_default()
}

fn composition_row(
    indicator: &str,
    status: &str,
    reason: String,
    female: String,
    male: String,
) -> String {
    csv_rules::row(&[
        csv_rules::field(indicator),
        "organization".into(),
        status.into(),
        reason,
        female,
        male,
        String::new(),
        String::new(),
        "basis_points_of_group".into(),
    ])
}

fn exclusion_row(reason: &str, count: Excluded) -> String {
    let (status, value) = match count {
        Excluded::Exactly(n) => ("shown", n.to_string()),
        Excluded::UnderFloor => ("under_floor", String::new()),
    };
    csv_rules::row(&[
        csv_rules::field(&format!("excluded: {reason}")),
        "organization".into(),
        status.into(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        value,
        "workers".into(),
    ])
}

/// Rows (e) and (f): the share receiving a variable component, and the four quartile compositions.
fn composition_rows(r: &Report) -> String {
    let mut out = String::new();
    let name = "(e) share receiving a variable component";
    out.push_str(&match &r.receiving_variable {
        Figure::Shown(s) => composition_row(
            name,
            "shown",
            String::new(),
            s.female_bp.to_string(),
            s.male_bp.to_string(),
        ),
        Figure::Withheld => composition_row(
            name,
            "withheld",
            String::new(),
            String::new(),
            String::new(),
        ),
        Figure::Unknown(reason) => composition_row(
            name,
            "unknown",
            reason_text(*reason),
            String::new(),
            String::new(),
        ),
    });
    for (i, band) in r.quartiles.iter().enumerate() {
        let name = format!("(f) quartile {} composition", i + 1);
        out.push_str(&match band {
            Figure::Shown(c) => composition_row(
                &name,
                "shown",
                String::new(),
                c.female_bp.to_string(),
                c.male_bp.to_string(),
            ),
            Figure::Withheld => composition_row(
                &name,
                "withheld",
                String::new(),
                String::new(),
                String::new(),
            ),
            Figure::Unknown(reason) => composition_row(
                &name,
                "unknown",
                reason_text(*reason),
                String::new(),
                String::new(),
            ),
        });
    }
    out
}

/// The report as CSV, one row per figure with the two figures the gap comes from (the workings).
fn to_csv(built: &Built) -> String {
    let r = &built.report;
    let mut out = csv_rules::row(
        &[
            "indicator",
            "scope",
            "status",
            "reason",
            "female",
            "male",
            "gap_bp",
            "value",
            "unit",
        ]
        .map(String::from),
    );
    out.push_str(&gap_row(
        "(a) mean gender pay gap",
        "organization",
        &r.mean_gap,
    ));
    out.push_str(&gap_row(
        "(b) mean gap in variable components",
        "organization",
        &r.mean_variable_gap,
    ));
    out.push_str(&gap_row(
        "(c) median gender pay gap",
        "organization",
        &r.median_gap,
    ));
    out.push_str(&gap_row(
        "(d) median gap in variable components",
        "organization",
        &r.median_variable_gap,
    ));
    out.push_str(&composition_rows(r));
    for c in &r.categories {
        out.push_str(&gap_row("(g) gap in basic pay", &c.category, &c.basic));
        out.push_str(&gap_row(
            "(g) gap in variable components",
            &c.category,
            &c.variable,
        ));
    }
    let e = &r.exclusions;
    for (reason, count) in [
        ("no sex declared", e.no_sex_declared),
        ("sex not female or male", e.sex_not_female_or_male),
        ("no pay recorded in this currency", e.no_pay_recorded),
        ("variable component not recorded", e.variable_unknown),
        ("in no category", e.uncategorised),
    ] {
        out.push_str(&exclusion_row(reason, count));
    }
    out
}

/// `GET /api/pay-transparency/gap-report/csv?…` — the same report as CSV, with the workings. Audited.
#[debug_handler]
pub(crate) async fn gap_report_csv(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ReportQuery>,
) -> Result<Response> {
    let built = build(&ctx, &caller, &query).await?;
    audit_read(&ctx, &caller, &built, true).await?;
    axum::response::Response::builder()
        .header("content-type", "text/csv; charset=utf-8")
        .header(
            "content-disposition",
            "attachment; filename=\"pay-gap-report.csv\"",
        )
        .body(axum::body::Body::from(to_csv(&built)))
        .map_err(|e| super::internal(&e.to_string()))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/pay-categories", get(list_categories))
        .add("/pay-categories", post(create_category))
        .add("/pay-categories/{pid}", put(change_category))
        .add("/pay-categories/{pid}", delete(remove_category))
        .add("/pay-transparency/gap-report", get(gap_report))
        .add("/pay-transparency/gap-report/csv", get(gap_report_csv))
}
