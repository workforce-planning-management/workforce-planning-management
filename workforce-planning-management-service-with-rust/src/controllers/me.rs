//! **`/api/me`: what a worker does about themself** (WPM-R124, WPM-R129, WPM-R130, WPM-D76).
//!
//! Every route here resolves the person from the verified token alone: there is no worker id in
//! the path for anyone to change. The writes among them are the explicit allow-list in
//! [`crate::rules::self_service`]; they skip the policy's action check (which lets only HR write)
//! and nothing else, and a valid token is still required.
//!
//! - **Contact details**: home address, telephone numbers, personal e-mail. Replaced as a whole;
//!   the audit entry says they changed, never what they were; the old value is not kept.
//! - **Emergency contacts**: the existing rules (a limit, a ranking), written by the worker.
//! - **My time-off**: allowances, days remaining and past absence, read-only. No reason for sick
//!   leave, and no score of any kind.
//!
//! HR and payroll read a worker's contact details at `GET /api/workers/{pid}/contact-details`
//! (the worker and privileged callers only, by the need-to-know guard); that read is audited.

use chrono::{Datelike, Utc};
use loco_rs::controller::ErrorDetail;
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, ConnectionTrait, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;

use super::contacts::{
    ContactPayload, ContactUpdate, add_contact_for, delete_contact_for, find_contact,
    list_contacts_for, update_contact_for,
};
use super::equality_monitoring::{
    DeclarePayload, declare_for, view_for as view_equality_for,
    withdraw_for as withdraw_equality_for,
};
use super::flexible_working::{
    AppealPayload, RequestPayload, RespondPayload, appeal_for, create_for as create_flexible_for,
    list_for as list_flexible_for, respond_for, withdraw_for as withdraw_flexible_for,
};
use super::health_requirements::view_for as view_health_for;
use super::pay_information::{
    ask_for as ask_pay_information_for, list_for as list_pay_information_for,
};
use super::resignations::{LogPayload, latest, log_for, own_json, withdraw_for};
use super::workforce::{LeaveRequestPayload, create_leave_for, decide_leave};
use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{
    leave_entitlements, leave_requests, worker_contact_details, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::contact_details as contact_rules;
use crate::rules::phone as phone_rules;
use crate::rules::time_off as time_rules;

/// The caller's own worker record. A person with several records (in several organizations) gets
/// the earliest one still employed, else the earliest.
pub(crate) async fn me_worker(ctx: &AppContext, caller: &MaybeAuthUser) -> Result<workers::Model> {
    let Some(claims) = caller.claims() else {
        return Err(Error::CustomError(
            axum::http::StatusCode::UNAUTHORIZED,
            ErrorDetail::new("unauthorized", "sign in to use /api/me"),
        ));
    };
    let mut rows = workers::Entity::find()
        .filter(workers::Column::PersonRef.eq(format!("person:{}", claims.sub)))
        .filter(workers::Column::DeletedAt.is_null())
        .order_by_asc(workers::Column::Id)
        .all(&ctx.db)
        .await?;
    if let Some(i) = rows.iter().position(|w| w.terminated_on.is_none()) {
        return Ok(rows.swap_remove(i));
    }
    rows.into_iter().next().ok_or(Error::NotFound)
}

// ─── Contact details ────────────────────────────────────────────────────────

fn details_json(row: &worker_contact_details::Model) -> serde_json::Value {
    json!({
        "address_line1": row.address_line1,
        "address_line2": row.address_line2,
        "city": row.city,
        "region": row.region,
        "postcode": row.postcode,
        "address_text": row.address_text,
        "country": row.country,
        "phone_mobile": row.phone_mobile,
        "phone_mobile_digits": row.phone_mobile_digits,
        "phone_mobile_e164": row.phone_mobile_e164,
        "phone_home": row.phone_home,
        "phone_home_digits": row.phone_home_digits,
        "phone_home_e164": row.phone_home_e164,
        "personal_email": row.personal_email,
        "on_behalf": row.on_behalf,
    })
}

async fn details_of(
    ctx: &AppContext,
    worker_pid: uuid::Uuid,
) -> Result<Option<worker_contact_details::Model>> {
    Ok(worker_contact_details::Entity::find()
        .filter(worker_contact_details::Column::WorkerPid.eq(worker_pid))
        .one(&ctx.db)
        .await?)
}

/// The derived forms of a worker's two numbers (WPM-D152): ASCII digits, and E.164 read in the address's country, else
/// the country of the place the work is done.
fn phone_forms(
    country: Option<&str>,
    mobile: Option<&str>,
    home: Option<&str>,
    work_jurisdiction: Option<&str>,
) -> (phone_rules::Forms, phone_rules::Forms) {
    let country = phone_rules::country_for(country, work_jurisdiction);
    (
        phone_rules::forms(mobile, country),
        phone_rules::forms(home, country),
    )
}

/// Re-derive the E.164 forms of `worker`'s numbers after the place of work changed (an address country, where there is
/// one, still wins). Runs on the caller's connection or transaction.
///
/// # Errors
///
/// A database error.
pub(crate) async fn refresh_phone_forms<C: ConnectionTrait>(
    db: &C,
    worker: &workers::Model,
) -> Result<()> {
    let Some(row) = worker_contact_details::Entity::find()
        .filter(worker_contact_details::Column::WorkerPid.eq(worker.pid))
        .one(db)
        .await?
    else {
        return Ok(());
    };
    let (mobile, home) = phone_forms(
        row.country.as_deref(),
        row.phone_mobile.as_deref(),
        row.phone_home.as_deref(),
        worker.work_jurisdiction.as_deref(),
    );
    if mobile.e164 == row.phone_mobile_e164 && home.e164 == row.phone_home_e164 {
        return Ok(());
    }
    let mut active: worker_contact_details::ActiveModel = row.into();
    active.phone_mobile_e164 = ActiveValue::set(mobile.e164);
    active.phone_home_e164 = ActiveValue::set(home.e164);
    active.update(db).await?;
    Ok(())
}

/// `GET /api/me/contact-details` — the caller's own, or `null`.
#[debug_handler]
async fn get_my_details(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let row = details_of(&ctx, worker.pid).await?;
    format::json(json!({ "contact_details": row.as_ref().map(details_json) }))
}

/// `GET /api/me/countries` — the country codes the caller's organization has added that ISO 3166-1 lacks (WPM-D152), for the
/// contact details picker; the ISO 3166-1 codes themselves are the UI's own list, named by the browser.
#[debug_handler]
async fn my_countries(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let added = super::jurisdictions::deployer_countries(&ctx.db, &worker.organization_ref).await?;
    format::json(json!({
        "deployer_codes": added
            .into_iter()
            .map(|(code, name)| json!({ "code": code, "name": name }))
            .collect::<Vec<_>>(),
    }))
}

/// `PUT /api/me/contact-details` body. A field left out, or blank, is cleared: this replaces the
/// whole record.
#[derive(Debug, Default, Deserialize)]
struct DetailsPayload {
    #[serde(default)]
    address_line1: Option<String>,
    #[serde(default)]
    address_line2: Option<String>,
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    region: Option<String>,
    #[serde(default)]
    postcode: Option<String>,
    /// The whole address as one free-text block (WPM-D152), for a country the UI's table lacks.
    #[serde(default)]
    address_text: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    phone_mobile: Option<String>,
    #[serde(default)]
    phone_home: Option<String>,
    #[serde(default)]
    personal_email: Option<String>,
}

impl DetailsPayload {
    fn cleaned(&self) -> contact_rules::Details {
        let c = |v: &Option<String>| contact_rules::clean(v.as_deref());
        contact_rules::Details {
            address_line1: c(&self.address_line1),
            address_line2: c(&self.address_line2),
            city: c(&self.city),
            region: c(&self.region),
            postcode: c(&self.postcode),
            address_text: contact_rules::clean_block(self.address_text.as_deref()),
            country: c(&self.country).map(|v| v.to_ascii_uppercase()),
            phone_mobile: c(&self.phone_mobile),
            phone_home: c(&self.phone_home),
            personal_email: c(&self.personal_email),
        }
    }
}

/// `PUT /api/me/contact-details` — replace the caller's own details. The audit entry says they
/// changed, not what they were; the previous value is not kept.
#[debug_handler]
async fn set_my_details(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<DetailsPayload>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let details = payload.cleaned();
    contact_rules::validate(&details).map_err(|e| unprocessable(&e))?;
    if let Some(country) = &details.country {
        let added: Vec<String> =
            super::jurisdictions::deployer_countries(&ctx.db, &worker.organization_ref)
                .await?
                .into_iter()
                .map(|(code, _)| code)
                .collect();
        contact_rules::validate_country_known(country, &added).map_err(|e| unprocessable(&e))?;
    }
    let recorded_by = caller.actor().map(ToString::to_string);
    let (mobile, home) = phone_forms(
        details.country.as_deref(),
        details.phone_mobile.as_deref(),
        details.phone_home.as_deref(),
        worker.work_jurisdiction.as_deref(),
    );
    let existing = details_of(&ctx, worker.pid).await?;
    let txn = ctx.db.begin().await?;
    let saved = if let Some(row) = existing {
        let mut active: worker_contact_details::ActiveModel = row.into();
        active.address_line1 = ActiveValue::set(details.address_line1);
        active.address_line2 = ActiveValue::set(details.address_line2);
        active.city = ActiveValue::set(details.city);
        active.region = ActiveValue::set(details.region);
        active.postcode = ActiveValue::set(details.postcode);
        active.address_text = ActiveValue::set(details.address_text);
        active.country = ActiveValue::set(details.country);
        active.phone_mobile = ActiveValue::set(details.phone_mobile);
        active.phone_home = ActiveValue::set(details.phone_home);
        active.phone_mobile_digits = ActiveValue::set(mobile.digits);
        active.phone_mobile_e164 = ActiveValue::set(mobile.e164);
        active.phone_home_digits = ActiveValue::set(home.digits);
        active.phone_home_e164 = ActiveValue::set(home.e164);
        active.personal_email = ActiveValue::set(details.personal_email);
        active.recorded_by = ActiveValue::set(recorded_by);
        active.on_behalf = ActiveValue::set(false);
        active.update(&txn).await?
    } else {
        worker_contact_details::ActiveModel {
            worker_pid: ActiveValue::set(worker.pid),
            address_line1: ActiveValue::set(details.address_line1),
            address_line2: ActiveValue::set(details.address_line2),
            city: ActiveValue::set(details.city),
            region: ActiveValue::set(details.region),
            postcode: ActiveValue::set(details.postcode),
            address_text: ActiveValue::set(details.address_text),
            country: ActiveValue::set(details.country),
            phone_mobile: ActiveValue::set(details.phone_mobile),
            phone_home: ActiveValue::set(details.phone_home),
            phone_mobile_digits: ActiveValue::set(mobile.digits),
            phone_mobile_e164: ActiveValue::set(mobile.e164),
            phone_home_digits: ActiveValue::set(home.digits),
            phone_home_e164: ActiveValue::set(home.e164),
            personal_email: ActiveValue::set(details.personal_email),
            recorded_by: ActiveValue::set(recorded_by),
            on_behalf: ActiveValue::set(false),
            ..Default::default()
        }
        .insert(&txn)
        .await?
    };
    Audit::record(
        &txn,
        "worker",
        worker.pid,
        "contact_details_set",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "contact_details": details_json(&saved) }))
}

/// `DELETE /api/me/contact-details` — remove them all (none recorded is a 404).
#[debug_handler]
async fn clear_my_details(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let row = details_of(&ctx, worker.pid).await?.ok_or(Error::NotFound)?;
    let txn = ctx.db.begin().await?;
    worker_contact_details::Entity::delete_by_id(row.id)
        .exec(&txn)
        .await?;
    Audit::record(
        &txn,
        "worker",
        worker.pid,
        "contact_details_cleared",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::empty()
}

/// `GET /api/workers/{pid}/contact-details` — for HR and payroll (and the worker). Each read by
/// someone other than the worker is audited, without the values.
#[debug_handler]
async fn get_worker_details(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let row = details_of(&ctx, worker.pid).await?;
    if row.is_some() && auth::acting_for_other(&caller, &worker.person_ref) {
        Audit::record(
            &ctx.db,
            "worker",
            worker.pid,
            "contact_details_read",
            caller.actor(),
            None,
        )
        .await?;
    }
    format::json(json!({ "contact_details": row.as_ref().map(details_json) }))
}

#[derive(Debug, Deserialize)]
struct PhoneSearch {
    organization: String,
    phone: String,
}

/// `GET /api/contact-details/search?organization=&phone=` — HR finds the workers of an organization whose own mobile or
/// home number is `phone` (WPM-D152). The comparison is over the stored ASCII digits and E.164 forms, never the typed
/// text, so a number typed in Arabic-Indic digits is found by one typed in ASCII. The answer names the workers and
/// which of their numbers matched, never a number; each worker found is audited (`contact_details_found`), without it.
#[debug_handler]
async fn search_by_phone(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(q): axum::extract::Query<PhoneSearch>,
) -> Result<Response> {
    super::ensure_valid(&super::pay_transparency::organization_problems(
        &q.organization,
    ))?;
    super::ensure_org_in_scope(&ctx, &caller, &q.organization).await?;
    if !super::policies::privileged(&ctx, &caller, &q.organization).await? {
        return Err(super::forbidden(
            "searching contact details by telephone number is HR's",
        ));
    }
    if phone_rules::search_digits(&q.phone).is_none_or(|k| k.len() < phone_rules::SEARCH_MIN) {
        return Err(unprocessable(&format!(
            "phone must be a telephone number with at least {} digits after any leading zeros",
            phone_rules::SEARCH_MIN
        )));
    }
    let members: std::collections::HashMap<uuid::Uuid, workers::Model> = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(q.organization.clone()))
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| (w.pid, w))
        .collect();
    let rows = worker_contact_details::Entity::find()
        .filter(worker_contact_details::Column::WorkerPid.is_in(members.keys().copied()))
        .all(&ctx.db)
        .await?;
    let mut found = Vec::new();
    for details in rows {
        let Some(worker) = members.get(&details.worker_pid).cloned() else {
            continue;
        };
        let matched: Vec<&str> = [
            (
                "phone_mobile",
                &details.phone_mobile_digits,
                &details.phone_mobile_e164,
            ),
            (
                "phone_home",
                &details.phone_home_digits,
                &details.phone_home_e164,
            ),
        ]
        .into_iter()
        .filter(|(_, digits, e164)| {
            phone_rules::matches(digits.as_deref(), e164.as_deref(), &q.phone)
        })
        .map(|(field, _, _)| field)
        .collect();
        if !matched.is_empty() {
            found.push((worker, matched));
        }
    }
    found.sort_by(|a, b| a.0.worker_number.cmp(&b.0.worker_number));
    let txn = ctx.db.begin().await?;
    for (worker, _) in &found {
        Audit::record(
            &txn,
            "worker",
            worker.pid,
            "contact_details_found",
            caller.actor(),
            None,
        )
        .await?;
    }
    txn.commit().await?;
    format::json(json!({
        "workers": found.iter().map(|(w, matched)| json!({
            "pid": w.pid,
            "worker_number": w.worker_number,
            "display_name": w.display_name,
            "matched": matched,
        })).collect::<Vec<_>>(),
    }))
}

// ─── Emergency contacts ─────────────────────────────────────────────────────

/// `GET /api/me/emergency-contacts`.
#[debug_handler]
async fn list_my_contacts(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    list_contacts_for(&ctx, &worker).await
}

/// `POST /api/me/emergency-contacts`.
#[debug_handler]
async fn add_my_contact(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<ContactPayload>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    add_contact_for(&ctx, &caller, &worker, payload).await
}

/// `PUT /api/me/emergency-contacts/{pid}` — one of the caller's own; anyone else's is a 404.
#[debug_handler]
async fn update_my_contact(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ContactUpdate>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let row = find_contact(&ctx, &pid).await?;
    if row.worker_pid != worker.pid {
        return Err(Error::NotFound);
    }
    update_contact_for(&ctx, &caller, &worker, row, payload).await
}

/// `DELETE /api/me/emergency-contacts/{pid}`.
#[debug_handler]
async fn delete_my_contact(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let row = find_contact(&ctx, &pid).await?;
    if row.worker_pid != worker.pid {
        return Err(Error::NotFound);
    }
    delete_contact_for(&ctx, &caller, &worker, row).await
}

// ─── My time-off ────────────────────────────────────────────────────────────

/// Query for the time-off view.
#[derive(Debug, Deserialize)]
struct TimeOffQuery {
    year: Option<i32>,
}

/// `GET /api/me/time-off?year=` — allowances, days taken, booked and remaining, the year's
/// requests, and past absence by year. In calendar days. Read-only, and the caller's own.
#[debug_handler]
async fn my_time_off(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<TimeOffQuery>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let today = Utc::now().date_naive();
    let year = query.year.unwrap_or_else(|| today.year());
    let entitlements: Vec<time_rules::Entitlement> = leave_entitlements::Entity::find()
        .filter(leave_entitlements::Column::WorkerPid.eq(worker.pid))
        .filter(leave_entitlements::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|e| time_rules::Entitlement {
            kind: e.kind,
            year: e.year,
            entitled: e.entitled_days,
            used: e.used_days,
        })
        .collect();
    let rows = leave_requests::Entity::find()
        .filter(leave_requests::Column::WorkerPid.eq(worker.pid))
        .filter(leave_requests::Column::DeletedAt.is_null())
        .order_by_asc(leave_requests::Column::StartOn)
        .all(&ctx.db)
        .await?;
    let requests: Vec<time_rules::Request> = rows
        .iter()
        .map(|r| time_rules::Request {
            kind: r.kind.clone(),
            start_on: r.start_on,
            end_on: r.end_on,
            days: r.days,
            status: r.status.clone(),
        })
        .collect();
    let balances: Vec<serde_json::Value> = time_rules::balances(&entitlements, &requests, today)
        .into_iter()
        .filter(|b| b.year == year)
        .map(|b| {
            json!({
                "kind": b.kind, "year": b.year, "entitled": b.entitled, "used": b.used,
                "taken": b.taken, "booked": b.booked, "requested": b.requested,
                "remaining": b.remaining,
                "remaining_if_requested_approved": b.remaining_if_requested_approved,
            })
        })
        .collect();
    let year_requests: Vec<serde_json::Value> = rows
        .iter()
        .filter(|r| r.start_on.year() == year)
        .map(|r| {
            json!({
                "pid": r.pid, "kind": r.kind, "start_on": r.start_on, "end_on": r.end_on,
                "days": r.days, "status": r.status,
                // Sick leave never carries a reason (WPM-D73); nothing is shown for it.
                "reason": if r.kind == "sick" { None } else { r.reason.clone() },
            })
        })
        .collect();
    let history: Vec<serde_json::Value> = time_rules::history(&requests, today)
        .into_iter()
        .map(|(y, absences)| {
            json!({
                "year": y,
                "absences": absences.into_iter().map(|a| json!({
                    "kind": a.kind, "absences": a.absences, "days": a.days,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    format::json(json!({
        "unit": "calendar_days",
        "as_of": today,
        "year": year,
        "balances": balances,
        "requests": year_requests,
        "history": history,
        "derivation": "remaining = days allowed less the days the service has already subtracted for approved \
                       requests. Taken is approved and over; booked is approved and still to come or under \
                       way; requested awaits a decision. A kind with no allowance recorded has no remaining \
                       figure (unknown, not zero). A request counts in the year it starts. Past absence \
                       counts approved requests that are over. No rate, trigger or score is computed.",
    }))
}

// ─── My leave requests ──────────────────────────────────────────────────────

/// `POST /api/me/leave-requests` — request leave for the caller. The same checks as HR's route:
/// the balance, and no reason on sick leave. A person with authority still decides it.
#[debug_handler]
async fn request_my_leave(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<LeaveRequestPayload>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    create_leave_for(&ctx, &caller, &worker, payload).await
}

/// `POST /api/me/leave-requests/{pid}/cancel` — cancel one of the caller's own requests while it
/// awaits a decision, or once approved if it has not started. Anyone else's is a 404.
#[debug_handler]
async fn cancel_my_leave(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let request = leave_requests::Entity::find()
        .filter(leave_requests::Column::Pid.eq(records::parse_pid(&pid)?))
        .filter(leave_requests::Column::WorkerPid.eq(worker.pid))
        .filter(leave_requests::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    time_rules::can_worker_cancel(&request.status, request.start_on, Utc::now().date_naive())
        .map_err(|e| unprocessable(&e))?;
    decide_leave(&ctx, &caller, &pid, "cancelled").await
}

// ─── My resignation ─────────────────────────────────────────────────────────

/// `GET /api/me/resignation` — the caller's latest, with the reason they gave, or `null`.
#[debug_handler]
async fn my_resignation(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let row = latest(&ctx, worker.pid).await?;
    format::json(json!({ "resignation": row.as_ref().map(own_json) }))
}

/// `POST /api/me/resignation` — log an intent to resign (a proposed last day that allows the
/// notice, and optionally a reason from a closed list). A person accepts it; nothing ends here.
#[debug_handler]
async fn log_my_resignation(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<LogPayload>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    log_for(&ctx, &caller, &worker, payload).await
}

/// `POST /api/me/resignation/withdraw` — take it back while it awaits acceptance.
#[debug_handler]
async fn withdraw_my_resignation(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    withdraw_for(&ctx, &caller, &worker).await
}

// ─── My pay information requests ────────────────────────────────────────────

/// `GET /api/me/pay-information-requests` — the caller's own requests, newest first, each with its due date and standing.
#[debug_handler]
async fn my_pay_information_requests(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    list_pay_information_for(&ctx, &worker).await
}

/// `POST /api/me/pay-information-requests` — ask for your own pay level and the average pay, by sex, of your category of
/// workers. No body; the due date is computed.
#[debug_handler]
async fn ask_pay_information(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    ask_pay_information_for(&ctx, &caller, &worker).await
}

// ─── My policies ────────────────────────────────────────────────────────────

/// `GET /api/me/policies` — the policies the caller is asked to acknowledge, and where they stand.
#[debug_handler]
async fn my_policies(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    super::policies::list_for(&ctx, &worker).await
}

/// `POST /api/me/policy-acknowledgements` — the caller says they have read the newest version in force aimed at them.
#[debug_handler]
async fn acknowledge_my_policy(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<super::policies::AckPayload>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    super::policies::acknowledge_for(&ctx, &caller, &worker, payload).await
}

// ─── My flexible working requests ───────────────────────────────────────────

/// `GET /api/me/flexible-working` — the caller's own requests, newest first, each with its
/// decide-by date and whether it is overdue.
#[debug_handler]
async fn my_flexible_requests(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    list_flexible_for(&ctx, &worker).await
}

/// `POST /api/me/flexible-working` — ask for a different working arrangement. A person decides.
#[debug_handler]
async fn request_flexible(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<RequestPayload>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    create_flexible_for(&ctx, &caller, &worker, payload).await
}

/// `POST /api/me/flexible-working/{pid}/withdraw`.
#[debug_handler]
async fn withdraw_flexible(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    withdraw_flexible_for(&ctx, &caller, &worker, &pid).await
}

/// `POST /api/me/flexible-working/{pid}/respond` — accept or decline a counter-proposal.
#[debug_handler]
async fn respond_flexible(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<RespondPayload>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    respond_for(&ctx, &caller, &worker, &pid, payload).await
}

/// `POST /api/me/flexible-working/{pid}/appeal` — appeal a refusal, once, within the window.
#[debug_handler]
async fn appeal_flexible(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<AppealPayload>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    appeal_for(&ctx, &caller, &worker, &pid, payload).await
}

// ─── My equality monitoring declarations ────────────────────────────────────

/// `GET /api/me/equality-monitoring` — whether monitoring is on, the categories on offer, and the
/// caller's own declarations. Only the worker can read their own.
#[debug_handler]
async fn my_equality(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    view_equality_for(&ctx, &worker).await
}

/// `PUT /api/me/equality-monitoring` — declare, or change, answers for the categories given.
/// Voluntary; `prefer_not_to_say` is always an answer.
#[debug_handler]
async fn declare_my_equality(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<DeclarePayload>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    declare_for(&ctx, &caller, &worker, payload).await
}

/// `DELETE /api/me/equality-monitoring` — take back every answer.
#[debug_handler]
async fn withdraw_my_equality(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    withdraw_equality_for(&ctx, &caller, &worker, None).await
}

/// `DELETE /api/me/equality-monitoring/{category}` — take back one answer.
#[debug_handler]
async fn withdraw_my_equality_category(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(category): Path<String>,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    withdraw_equality_for(&ctx, &caller, &worker, Some(&category)).await
}

// ─── My workplace health requirements ───────────────────────────────────────

/// `GET /api/me/health-requirements` — each requirement that applies to the caller, with their own
/// standing, dates and whether a reminder is due. Only they and occupational health see the detail;
/// their manager and HR are told only whether they are cleared. Off unless a deployer enables it.
#[debug_handler]
async fn my_health_requirements(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    view_health_for(&ctx, &worker).await
}

/// The `/api/me` routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/me/contact-details", get(get_my_details))
        .add("/me/contact-details", put(set_my_details))
        .add("/me/contact-details", delete(clear_my_details))
        .add("/me/countries", get(my_countries))
        .add("/me/emergency-contacts", get(list_my_contacts))
        .add("/me/emergency-contacts", post(add_my_contact))
        .add("/me/emergency-contacts/{pid}", put(update_my_contact))
        .add("/me/emergency-contacts/{pid}", delete(delete_my_contact))
        .add("/me/time-off", get(my_time_off))
        .add("/me/leave-requests", post(request_my_leave))
        .add("/me/equality-monitoring", get(my_equality))
        .add("/me/equality-monitoring", put(declare_my_equality))
        .add("/me/equality-monitoring", delete(withdraw_my_equality))
        .add(
            "/me/equality-monitoring/{category}",
            delete(withdraw_my_equality_category),
        )
        .add("/me/health-requirements", get(my_health_requirements))
        .add(
            "/me/pay-information-requests",
            get(my_pay_information_requests),
        )
        .add("/me/pay-information-requests", post(ask_pay_information))
        .add("/me/credentials", post(super::credentials::issue))
        .add("/me/credentials", get(super::credentials::mine))
        .add(
            "/me/credentials/{pid}/revoke",
            post(super::credentials::revoke_mine),
        )
        .add("/me/policies", get(my_policies))
        .add("/me/policy-acknowledgements", post(acknowledge_my_policy))
        .add("/me/flexible-working", get(my_flexible_requests))
        .add("/me/flexible-working", post(request_flexible))
        .add(
            "/me/flexible-working/{pid}/withdraw",
            post(withdraw_flexible),
        )
        .add("/me/flexible-working/{pid}/respond", post(respond_flexible))
        .add("/me/flexible-working/{pid}/appeal", post(appeal_flexible))
        .add("/me/resignation", get(my_resignation))
        .add("/me/resignation", post(log_my_resignation))
        .add("/me/resignation/withdraw", post(withdraw_my_resignation))
        .add("/me/leave-requests/{pid}/cancel", post(cancel_my_leave))
        .add("/workers/{pid}/contact-details", get(get_worker_details))
        .add("/contact-details/search", get(search_by_phone))
}
