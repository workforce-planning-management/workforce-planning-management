//! Pay information (WPM-R145, WPM-D99): a worker asks what they are paid and what their category of workers is paid on
//! average by sex; HR answers; and a vacancy states its pay before it is opened. Spec: `spec/pay-transparency.md`.
//!
//! - The request holds dates only. The **answer is computed when it is given** from the pay on the worker records and
//!   the sex each worker declared, and is **not stored**.
//! - Working out an answer reads salaries, so it needs the unmasked read of the plan cost view (payroll,
//!   administrators); **recording** that it was given is a write (HR). Neither is the worker's.
//! - An average is withheld under the floor for each sex on its own.

use std::collections::BTreeMap;

use axa::extract::Path;
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::{equality_monitoring, forbidden, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{pay_information_requests, workers};
use crate::rules::equality_monitoring::{DEFAULT_FLOOR, Gate};
use crate::rules::metrics::is_employed_on;
use crate::rules::pay_category as category_rules;
use crate::rules::pay_gap::Reason;
use crate::rules::pay_information as rules;

use super::jurisdictions::Scope;
use super::statutory_defaults::{self as statutory, InForce};

/// Calendar months an answer may take: the worker's profile (`pay_transparency.information_months`),
/// else `WPM_PAY_INFORMATION_MONTHS` (1 to 12), else 2 (the Directive's).
pub(crate) fn answer_months(scope: Option<&Scope>, as_of: NaiveDate) -> InForce<u32> {
    let configured = crate::compat::env_var("WPM_PAY_INFORMATION_MONTHS")
        .and_then(|v| v.trim().parse::<u32>().ok())
        .filter(|n| (1..=rules::MAX_ANSWER_MONTHS).contains(n));
    statutory::integer(
        scope,
        "WPM_PAY_INFORMATION_MONTHS",
        as_of,
        configured,
        rules::DEFAULT_ANSWER_MONTHS,
    )
}

/// Whether a vacancy must state its pay range before it is opened: the organization's profile
/// (`pay_transparency.pay_range_required`, over its home jurisdiction's chain), else on unless
/// `WPM_REQUIRE_PAY_RANGE` is off. A deployment outside the Directive's reach may turn it off; the
/// default follows Article 5.
#[must_use]
pub fn pay_range_required(scope: Option<&Scope>, as_of: NaiveDate) -> InForce<bool> {
    let configured = crate::compat::env_var("WPM_REQUIRE_PAY_RANGE")
        .map(|v| v.trim().to_lowercase())
        .filter(|v| !v.is_empty())
        .map(|v| !matches!(v.as_str(), "0" | "false" | "no" | "off"));
    statutory::pick(
        scope,
        "WPM_REQUIRE_PAY_RANGE",
        as_of,
        crate::rules::jurisdiction::Value::as_flag,
        configured,
        true,
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
    let rows = sqlx::query_as!(
        pay_information_requests::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, asked_on, due_on, answered_on,
                  answered_by, deleted_at AS "deleted_at: _"
             FROM pay_information_requests WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY id DESC"#,
        worker.pid,
    )
    .fetch_all(db::pool(ctx))
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
    let open = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM pay_information_requests
                           WHERE worker_pid = $1 AND deleted_at IS NULL AND answered_on IS NULL) AS "open!""#,
        worker.pid,
    )
    .fetch_one(db::pool(ctx))
    .await?;
    rules::may_ask(open).map_err(|e| unprocessable(&e))?;
    let today = Utc::now().date_naive();
    let scope = Scope::for_worker(crate::db::pool(ctx), worker).await?;
    let months = answer_months(Some(&scope), today);
    let due_on = rules::due_on(today, months.value)
        .ok_or_else(|| unprocessable("the due date runs past the supported calendar"))?;
    let request = Uuid::new_v4();
    let mut tx = db::pool(ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO pay_information_requests (pid, worker_pid, asked_on, due_on) VALUES ($1, $2, $3, $4)",
        request,
        worker.pid,
        today,
        due_on,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "pay_information_requested",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({
        "pid": request,
        "due_on": due_on,
        // Where the period came from: the worker's profile, the setting or the default (WPM-D153).
        "due_on_basis": months.came_from,
    }))
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
    axa::extract::Query(query): axa::extract::Query<ListQuery>,
) -> Result<Response> {
    if let Some(wanted) = &query.status
        && !["open", "overdue", "answered"].contains(&wanted.as_str())
    {
        return Err(unprocessable("status is open, overdue or answered"));
    }
    let scope =
        crate::models::memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let today = Utc::now().date_naive();
    // One read with the asker, in the caller's organizations (and the one asked for); a removed worker's is left out.
    let rows = sqlx::query!(
        "SELECT r.pid, r.asked_on, r.due_on, r.answered_on,
                w.pid AS worker_pid, w.worker_number, w.display_name, w.organization_ref
           FROM pay_information_requests r JOIN workers w ON w.pid = r.worker_pid AND w.deleted_at IS NULL
          WHERE r.deleted_at IS NULL AND ($1::text IS NULL OR w.organization_ref = $1)
            AND ($2::text[] IS NULL OR w.organization_ref = ANY($2))
          ORDER BY r.due_on, r.id",
        query.organization,
        scope.as_deref(),
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let mut out = Vec::new();
    for row in &rows {
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
        out.push(json!({
            "pid": row.pid,
            "asked_on": row.asked_on,
            "due_on": row.due_on,
            "answered_on": row.answered_on,
            "standing": standing,
            "worker_pid": row.worker_pid,
            "worker_number": row.worker_number,
            "display_name": row.display_name,
            "organization_ref": row.organization_ref,
        }));
    }
    format::json(out)
}

async fn request_and_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<(pay_information_requests::Model, workers::Model)> {
    let pid = Uuid::parse_str(pid).map_err(|_| Error::NotFound)?;
    let row = sqlx::query_as!(
        pay_information_requests::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, asked_on, due_on, answered_on,
                  answered_by, deleted_at AS "deleted_at: _"
             FROM pay_information_requests WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let worker = crate::models::records::find_worker(&ctx.db, row.worker_pid).await?;
    super::ensure_org_in_scope(ctx, caller, &worker.organization_ref).await?;
    Ok((row, worker))
}

/// What an answer reads of a worker.
struct Paid {
    pid: Uuid,
    job_title: String,
    salary_minor: Option<i64>,
    salary_currency: Option<String>,
    hired_on: NaiveDate,
    terminated_on: Option<NaiveDate>,
}

/// The pay of the workers of each sex in the asker's category, or why there is none, with the floor to use.
async fn pools_for(
    ctx: &AppContext,
    staff: &[Paid],
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
    let sex_category = super::pay_transparency::sex_category();
    if !config.has_category(&sex_category) {
        return Ok((floor, Err(Reason::NoSexRecorded)));
    }
    let answers: BTreeMap<Uuid, String> = sqlx::query!(
        "SELECT worker_pid, value FROM equality_declarations WHERE category = $1",
        sex_category,
    )
    .fetch_all(db::pool(ctx))
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
    let staff: Vec<Paid> = sqlx::query_as!(
        Paid,
        "SELECT pid, job_title, salary_minor, salary_currency, hired_on, terminated_on FROM workers
          WHERE organization_ref = $1 AND deleted_at IS NULL ORDER BY id",
        worker.organization_ref,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .filter(|w| is_employed_on(today, w.hired_on, w.terminated_on))
    .collect();
    let categories = sqlx::query!(
        "SELECT name, basis, job_titles FROM pay_categories
          WHERE organization_ref = $1 AND deleted_at IS NULL ORDER BY name",
        worker.organization_ref,
    )
    .fetch_all(db::pool(&ctx))
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
    db::audit(
        &mut *db::pool(&ctx).acquire().await?,
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
    let today = Utc::now().date_naive();
    let mut tx = db::pool(&ctx).begin().await?;
    let saved = sqlx::query_as!(
        pay_information_requests::Model,
        r#"UPDATE pay_information_requests SET answered_on = $2, answered_by = $3 WHERE id = $1
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, asked_on, due_on, answered_on,
                  answered_by, deleted_at AS "deleted_at: _""#,
        row.id,
        today,
        caller.actor(),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "pay_information_request",
        row.pid,
        "pay_information_answered",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
