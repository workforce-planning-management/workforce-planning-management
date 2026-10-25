//! Pay information (WPM-R145, WPM-D99): a worker asks what they are paid and what their category of workers is paid on
//! average by sex; HR answers; and a vacancy states its pay before it is opened. Spec: `spec/pay-transparency.md`.
//!
//! - The request holds dates only. The **answer is computed when it is given** from the pay on the worker records and
//!   the sex each worker declared, and is **not stored**.
//! - Working out an answer reads salaries, so it needs the unmasked read of the plan cost view (payroll,
//!   administrators); **recording** that it was given is a write (HR). Neither is the worker's.
//! - An average is withheld under the floor for each sex on its own.

use std::collections::BTreeMap;

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::{equality_monitoring, forbidden, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{
    equality_declarations, pay_categories, pay_information_requests, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::rules::equality_monitoring::{DEFAULT_FLOOR, Gate};
use crate::rules::metrics::is_employed_on;
use crate::rules::pay_category as category_rules;
use crate::rules::pay_gap::Reason;
use crate::rules::pay_information as rules;

/// Calendar months an answer may take: `WPM_PAY_INFORMATION_MONTHS`, default 2 (the Directive's), 1 to 12.
fn answer_months() -> u32 {
    crate::compat::env_var("WPM_PAY_INFORMATION_MONTHS")
        .and_then(|v| v.trim().parse::<u32>().ok())
        .filter(|n| (1..=rules::MAX_ANSWER_MONTHS).contains(n))
        .unwrap_or(rules::DEFAULT_ANSWER_MONTHS)
}

/// Whether a vacancy must state its pay range before it is opened: on unless `WPM_REQUIRE_PAY_RANGE` is off. A deployment
/// outside the Directive's reach may turn it off; the default follows Article 5.
#[must_use]
pub fn pay_range_required() -> bool {
    !matches!(
        crate::compat::env_var("WPM_REQUIRE_PAY_RANGE")
            .map(|v| v.trim().to_lowercase())
            .as_deref(),
        Some("0" | "false" | "no" | "off")
    )
}

fn view(row: &pay_information_requests::Model, today: NaiveDate) -> serde_json::Value {
    json!({
        "pid": row.pid,
        "asked_on": row.asked_on,
        "due_on": row.due_on,
        "answered_on": row.answered_on,
        "standing": rules::standing(row.answered_on, row.due_on, today),
    })
}

/// The worker's own requests, newest first, each with its due date and standing. Shared with
/// `GET /api/me/pay-information-requests`.
pub(crate) async fn list_for(ctx: &AppContext, worker: &workers::Model) -> Result<Response> {
    let today = Utc::now().date_naive();
    let rows = pay_information_requests::Entity::find()
        .filter(pay_information_requests::Column::WorkerPid.eq(worker.pid))
        .filter(pay_information_requests::Column::DeletedAt.is_null())
        .order_by_desc(pay_information_requests::Column::Id)
        .all(&ctx.db)
        .await?;
    format::json(rows.iter().map(|r| view(r, today)).collect::<Vec<_>>())
}

/// Ask for your own pay level and the average pay, by sex, of your category of workers. No body. The due date is
/// computed (calendar months, default 2); one request waits at a time. Shared with `POST /api/me/pay-information-requests`.
pub(crate) async fn ask_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
) -> Result<Response> {
    let open = pay_information_requests::Entity::find()
        .filter(pay_information_requests::Column::WorkerPid.eq(worker.pid))
        .filter(pay_information_requests::Column::DeletedAt.is_null())
        .filter(pay_information_requests::Column::AnsweredOn.is_null())
        .count(&ctx.db)
        .await?
        > 0;
    rules::may_ask(open).map_err(|e| unprocessable(&e))?;
    let today = Utc::now().date_naive();
    let due_on = rules::due_on(today, answer_months())
        .ok_or_else(|| unprocessable("the due date runs past the supported calendar"))?;
    let txn = ctx.db.begin().await?;
    let row = pay_information_requests::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        asked_on: ActiveValue::set(today),
        due_on: ActiveValue::set(due_on),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "worker",
        worker.pid,
        "pay_information_requested",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": row.pid, "due_on": due_on }))
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListQuery {
    organization: Option<String>,
    /// `open`, `overdue` or `answered`.
    status: Option<String>,
}

/// `GET /api/pay-information-requests?organization=&status=` — requests to answer, oldest due first, with who asked (HR
/// needs to know) and no figure.
#[debug_handler]
pub(crate) async fn list_requests(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    if let Some(wanted) = &query.status
        && !["open", "overdue", "answered"].contains(&wanted.as_str())
    {
        return Err(unprocessable("status is open, overdue or answered"));
    }
    let scope =
        crate::models::memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let today = Utc::now().date_naive();
    let rows = pay_information_requests::Entity::find()
        .filter(pay_information_requests::Column::DeletedAt.is_null())
        .order_by_asc(pay_information_requests::Column::DueOn)
        .order_by_asc(pay_information_requests::Column::Id)
        .all(&ctx.db)
        .await?;
    let people: BTreeMap<Uuid, workers::Model> = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| (w.pid, w))
        .collect();
    let mut out = Vec::new();
    for row in &rows {
        let Some(w) = people.get(&row.worker_pid) else {
            continue;
        };
        if query
            .organization
            .as_ref()
            .is_some_and(|o| o != &w.organization_ref)
            || scope
                .as_ref()
                .is_some_and(|refs| !refs.iter().any(|r| r == &w.organization_ref))
        {
            continue;
        }
        let standing = rules::standing(row.answered_on, row.due_on, today);
        let label = serde_json::to_value(standing)
            .ok()
            .and_then(|v| v.as_str().map(ToString::to_string));
        if query
            .status
            .as_ref()
            .is_some_and(|s| Some(s) != label.as_ref())
        {
            continue;
        }
        let mut item = view(row, today);
        item["worker_pid"] = json!(w.pid);
        item["worker_number"] = json!(w.worker_number);
        item["display_name"] = json!(w.display_name);
        item["organization_ref"] = json!(w.organization_ref);
        out.push(item);
    }
    format::json(out)
}

async fn request_and_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<(pay_information_requests::Model, workers::Model)> {
    let pid = Uuid::parse_str(pid).map_err(|_| Error::NotFound)?;
    let row = pay_information_requests::Entity::find()
        .filter(pay_information_requests::Column::Pid.eq(pid))
        .filter(pay_information_requests::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let worker = workers::Entity::find()
        .filter(workers::Column::Pid.eq(row.worker_pid))
        .filter(workers::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    super::ensure_org_in_scope(ctx, caller, &worker.organization_ref).await?;
    Ok((row, worker))
}

/// The pay of the workers of each sex in the asker's category, or why there is none, with the floor to use.
async fn pools_for(
    ctx: &AppContext,
    staff: &[workers::Model],
    by_title: &BTreeMap<String, String>,
    category: Option<&String>,
    currency: Option<&String>,
) -> Result<(usize, std::result::Result<(Vec<i64>, Vec<i64>), Reason>)> {
    let Some(name) = category else {
        return Ok((DEFAULT_FLOOR, Err(Reason::NoCategory)));
    };
    let Ok(Gate::On { config, floor, .. }) = equality_monitoring::gate_from_env() else {
        return Ok((DEFAULT_FLOOR, Err(Reason::NoSexRecorded)));
    };
    let sex_category =
        crate::compat::env_var("WPM_PAY_GAP_SEX_CATEGORY").unwrap_or_else(|| "sex".to_string());
    if !config.has_category(&sex_category) {
        return Ok((floor, Err(Reason::NoSexRecorded)));
    }
    let answers: BTreeMap<Uuid, String> = equality_declarations::Entity::find()
        .filter(equality_declarations::Column::Category.eq(&sex_category))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|a| (a.worker_pid, a.value))
        .collect();
    let (mut female, mut male) = (Vec::new(), Vec::new());
    for w in staff {
        if by_title.get(&category_rules::title_key(&w.job_title)) != Some(name) {
            continue;
        }
        let Some(pay) = w
            .salary_minor
            .filter(|_| w.salary_currency.as_ref() == currency)
        else {
            continue;
        };
        match answers.get(&w.pid).map(String::as_str) {
            Some("female") => female.push(pay),
            Some("male") => male.push(pay),
            _ => {}
        }
    }
    Ok((floor, Ok((female, male))))
}

/// `GET /api/pay-information-requests/{pid}/answer` — work out the answer to give: the worker's own pay level and the
/// average pay of their category by sex, each average withheld under the floor. **Not stored.** Needs the unmasked salary
/// read; the read is audited with no figure.
#[debug_handler]
pub(crate) async fn answer(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (row, worker) = request_and_worker(&ctx, &caller, &pid).await?;
    let obligations = auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &BTreeMap::new(),
    )
    .map_err(record_rejection)?;
    if obligations.iter().any(|o| o == "mask") {
        return Err(forbidden(
            "an answer is made of salaries and needs payroll read access",
        ));
    }
    let today = Utc::now().date_naive();
    let staff: Vec<workers::Model> = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&worker.organization_ref))
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| is_employed_on(today, w.hired_on, w.terminated_on))
        .collect();
    let categories = pay_categories::Entity::find()
        .filter(pay_categories::Column::OrganizationRef.eq(&worker.organization_ref))
        .filter(pay_categories::Column::DeletedAt.is_null())
        .order_by_asc(pay_categories::Column::Name)
        .all(&ctx.db)
        .await?;
    let held: Vec<(String, Vec<String>)> = categories
        .iter()
        .map(|c| {
            (
                c.name.clone(),
                serde_json::from_value(c.job_titles.clone()).unwrap_or_default(),
            )
        })
        .collect();
    let by_title = category_rules::index(&held);
    let key = category_rules::title_key(&worker.job_title);
    let category = by_title.get(&key).cloned();
    let currency = worker.salary_currency.clone();
    let (floor, pools) = pools_for(
        &ctx,
        &staff,
        &by_title,
        category.as_ref(),
        currency.as_ref(),
    )
    .await?;
    let given = rules::answer(
        worker.salary_minor,
        pools
            .as_ref()
            .map(|(f, m)| (f.as_slice(), m.as_slice()))
            .map_err(|r| *r),
        floor,
    );
    Audit::record(
        &ctx.db,
        "pay_information_request",
        row.pid,
        "pay_information_answer_read",
        caller.actor(),
        None,
    )
    .await?;
    let basis = categories
        .iter()
        .find(|c| Some(&c.name) == category.as_ref())
        .map(|c| json!({ "name": c.name, "basis": c.basis }));
    format::json(json!({
        "request_pid": row.pid,
        "asked_on": row.asked_on,
        "due_on": row.due_on,
        "standing": rules::standing(row.answered_on, row.due_on, today),
        "currency": currency,
        "category": basis,
        "answer": given,
        "small_group_floor": floor,
        "derivation": "Annual full-time-equivalent basic salary on the worker record. An average is of the workers in the same \
                       category who declared female or male, in the same currency, and is withheld for a sex with fewer \
                       than the floor. Nothing here is stored.",
    }))
}

/// `POST /api/pay-information-requests/{pid}/answered` — record that the answer was given, and when. HR; the figures are
/// not recorded.
#[debug_handler]
pub(crate) async fn mark_answered(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (row, _) = request_and_worker(&ctx, &caller, &pid).await?;
    if row.answered_on.is_some() {
        return Err(unprocessable("this request is already answered"));
    }
    let row_pid = row.pid;
    let today = Utc::now().date_naive();
    let txn = ctx.db.begin().await?;
    let mut active: pay_information_requests::ActiveModel = row.into();
    active.answered_on = ActiveValue::set(Some(today));
    active.answered_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    let saved = active.update(&txn).await?;
    Audit::record(
        &txn,
        "pay_information_request",
        row_pid,
        "pay_information_answered",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(view(&saved, today))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/pay-information-requests", get(list_requests))
        .add("/pay-information-requests/{pid}/answer", get(answer))
        .add(
            "/pay-information-requests/{pid}/answered",
            post(mark_answered),
        )
}
