//! **Expense claims** (WPM-R55): a worker's request to be repaid for money they
//! spent, built from dated, categorised items; submitted; decided by someone
//! **other than the claimant** (their manager or HR); then marked reimbursed (see
//! [`crate::rules::expenses`]).
//!
//! Who can do what — never decided by the claimant, even as HR:
//! - **see** a claim: the claimant, their manager, HR in their organization;
//! - **build** it (add or remove items, submit, withdraw, cancel): the claimant, or
//!   HR on their behalf — a manager reviews, they do not write it;
//! - **decide** it (approve, reject, mark reimbursed): the manager or HR.
//!
//! With auth off every check passes (the demo default). Audit entries and
//! notifications name no amount, title or description. Items are exported and the
//! free text scrubbed on erasure; the amounts stay (financial records).

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, QuerySelect, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

use super::{ensure_valid, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{expense_claims, expense_items, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::notifications::Model as Notification;
use crate::models::{memberships, records};
use crate::rules::career;
use crate::rules::expenses::{self as rules, Standing};
use crate::validation::Problems;

/// Roles that act for HR on expense claims.
const HR_ROLES: &[&str] = &["hr_admin"];

/// What the caller may do with one claim. With auth off, everything.
struct Gate(Option<Standing>);

impl Gate {
    fn view(&self) -> bool {
        self.0.is_none_or(rules::may_view)
    }
    fn edit(&self) -> bool {
        self.0.is_none_or(rules::may_edit)
    }
    fn decide(&self) -> bool {
        self.0.is_none_or(rules::may_decide)
    }
}

/// Work out who the caller is in relation to the claimant.
async fn gate(ctx: &AppContext, caller: &MaybeAuthUser, claimant: &workers::Model) -> Result<Gate> {
    if !auth::require_auth() {
        return Ok(Gate(None));
    }
    let Some(claims) = caller.claims() else {
        return Ok(Gate(Some(Standing {
            claimant: false,
            manager: false,
            hr: false,
        })));
    };
    let is_claimant = career::is_self(&claims.sub, &claimant.person_ref);
    let manager = match claimant.manager_pid {
        Some(pid) => workers::Entity::find()
            .filter(workers::Column::Pid.eq(pid))
            .filter(workers::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?
            .is_some_and(|m| career::is_self(&claims.sub, &m.person_ref)),
        None => false,
    };
    let hr = memberships::has_role_in(
        &ctx.db,
        caller.claims(),
        &claimant.organization_ref,
        HR_ROLES,
    )
    .await?;
    Ok(Gate(Some(Standing {
        claimant: is_claimant,
        manager,
        hr,
    })))
}

fn denied() -> Error {
    record_rejection((
        axum::http::StatusCode::FORBIDDEN,
        "not permitted on this expense claim".to_string(),
    ))
}

async fn find_claim(ctx: &AppContext, pid: &str) -> Result<expense_claims::Model> {
    expense_claims::Entity::find()
        .filter(expense_claims::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(expense_claims::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

async fn claimant_of(ctx: &AppContext, claim: &expense_claims::Model) -> Result<workers::Model> {
    records::find_worker(&ctx.db, claim.worker_pid).await
}

async fn items_of(ctx: &AppContext, claim_pid: Uuid) -> Result<Vec<expense_items::Model>> {
    Ok(expense_items::Entity::find()
        .filter(expense_items::Column::ClaimPid.eq(claim_pid))
        .order_by_asc(expense_items::Column::IncurredOn)
        .order_by_asc(expense_items::Column::Id)
        .all(&ctx.db)
        .await?)
}

fn total(items: &[expense_items::Model]) -> Result<i64> {
    let amounts: Vec<i64> = items.iter().map(|i| i.amount_minor).collect();
    rules::total_minor(&amounts).map_err(|e| unprocessable(&e))
}

fn claim_row(c: &expense_claims::Model, total_minor: i64, item_count: usize) -> serde_json::Value {
    json!({
        "pid": c.pid,
        "worker_pid": c.worker_pid,
        "title": c.title,
        "description": c.description,
        "currency": c.currency,
        "status": c.status,
        "total_minor": total_minor,
        "item_count": item_count,
        "submitted_at": c.submitted_at,
        "decided_at": c.decided_at,
        "decision_note": c.decision_note,
        "reimbursed_on": c.reimbursed_on,
        "on_behalf": c.on_behalf,
    })
}

// ─── Create and list ────────────────────────────────────────────────────────

/// `POST` body.
#[derive(Debug, Deserialize)]
struct ClaimPayload {
    title: String,
    currency: String,
    #[serde(default)]
    description: Option<String>,
}

/// `POST /api/workers/{pid}/expense-claims` — start a draft claim.
#[debug_handler]
async fn create_claim(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ClaimPayload>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    if !gate(&ctx, &caller, &worker).await?.edit() {
        return Err(denied());
    }
    let mut problems = Problems::new();
    problems.require_text("title", &payload.title);
    problems.cap_text("title", &payload.title);
    problems.cap_opt("description", payload.description.as_deref());
    if let Err(e) = rules::validate_currency(&payload.currency) {
        problems.push(e);
    }
    ensure_valid(&problems.into_vec())?;
    let row = expense_claims::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        title: ActiveValue::set(payload.title.trim().to_string()),
        description: ActiveValue::set(
            payload
                .description
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty()),
        ),
        currency: ActiveValue::set(payload.currency.clone()),
        status: ActiveValue::set("draft".to_string()),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        on_behalf: ActiveValue::set(auth::acting_for_other(&caller, &worker.person_ref)),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "expense_claim",
        row.pid,
        "expense_claim_created",
        caller.actor(),
        None,
    )
    .await?;
    format::json(claim_row(&row, 0, 0))
}

/// `GET /api/workers/{pid}/expense-claims` — the worker's claims, newest first.
/// Seen by the claimant, their manager and HR.
#[debug_handler]
async fn list_claims(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    if !gate(&ctx, &caller, &worker).await?.view() {
        return Err(denied());
    }
    let claims = expense_claims::Entity::find()
        .filter(expense_claims::Column::WorkerPid.eq(worker.pid))
        .filter(expense_claims::Column::DeletedAt.is_null())
        .order_by_desc(expense_claims::Column::Id)
        .all(&ctx.db)
        .await?;
    let mut out = Vec::with_capacity(claims.len());
    for c in &claims {
        let items = items_of(&ctx, c.pid).await?;
        out.push(claim_row(c, total(&items)?, items.len()));
    }
    format::json(out)
}

/// One claim in full: its items (with duplicate flags) and what the caller may do.
#[debug_handler]
async fn get_claim(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let claim = find_claim(&ctx, &pid).await?;
    let worker = claimant_of(&ctx, &claim).await?;
    let gate = gate(&ctx, &caller, &worker).await?;
    if !gate.view() {
        // Not theirs to know exists.
        return Err(Error::NotFound);
    }
    let items = items_of(&ctx, claim.pid).await?;
    let others = other_keys(&ctx, &claim).await?;
    let keys: Vec<rules::ItemKey<'_>> = items
        .iter()
        .map(|i| (i.incurred_on, i.category.as_str(), i.amount_minor))
        .collect();
    let other_keys_ref: Vec<rules::ItemKey<'_>> = others
        .iter()
        .map(|(d, c, a)| (*d, c.as_str(), *a))
        .collect();
    let dupes = rules::possible_duplicates(&keys, &other_keys_ref);
    let rows: Vec<_> = items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            json!({
                "pid": it.pid,
                "incurred_on": it.incurred_on,
                "category": it.category,
                "amount_minor": it.amount_minor,
                "description": it.description,
                "receipt_ref": it.receipt_ref,
                "possible_duplicate": dupes.contains(&i),
            })
        })
        .collect();
    let mut view = claim_row(&claim, total(&items)?, items.len());
    view["items"] = json!(rows);
    view["worker_name"] = json!(worker.display_name);
    view["can"] = json!({
        "edit": gate.edit() && claim.status == "draft",
        "submit": gate.edit() && claim.status == "draft" && !items.is_empty(),
        "withdraw": gate.edit() && claim.status == "submitted",
        "cancel": gate.edit() && matches!(claim.status.as_str(), "draft" | "submitted"),
        "decide": gate.decide() && claim.status == "submitted",
        "reimburse": gate.decide() && claim.status == "approved",
    });
    format::json(view)
}

/// Date, category and amount of the claimant's items on their *other* live
/// claims — for the duplicate flag.
async fn other_keys(
    ctx: &AppContext,
    claim: &expense_claims::Model,
) -> Result<Vec<(NaiveDate, String, i64)>> {
    let other_claims: Vec<Uuid> = expense_claims::Entity::find()
        .filter(expense_claims::Column::WorkerPid.eq(claim.worker_pid))
        .filter(expense_claims::Column::DeletedAt.is_null())
        .filter(expense_claims::Column::Pid.ne(claim.pid))
        .filter(expense_claims::Column::Status.is_not_in(["cancelled", "rejected"]))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|c| c.pid)
        .collect();
    if other_claims.is_empty() {
        return Ok(Vec::new());
    }
    Ok(expense_items::Entity::find()
        .filter(expense_items::Column::ClaimPid.is_in(other_claims))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|i| (i.incurred_on, i.category, i.amount_minor))
        .collect())
}

// ─── Items ──────────────────────────────────────────────────────────────────

/// `POST` body.
#[derive(Debug, Deserialize)]
struct ItemPayload {
    incurred_on: NaiveDate,
    category: String,
    amount_minor: i64,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    receipt_ref: Option<String>,
}

/// `POST /api/expense-claims/{pid}/items` — add an item to a draft claim.
#[debug_handler]
async fn add_item(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ItemPayload>,
) -> Result<Response> {
    let claim = find_claim(&ctx, &pid).await?;
    let worker = claimant_of(&ctx, &claim).await?;
    if !gate(&ctx, &caller, &worker).await?.edit() {
        return Err(denied());
    }
    if claim.status != "draft" {
        return Err(unprocessable(
            "items can only be changed while the claim is a draft",
        ));
    }
    let mut problems = Problems::new();
    problems.cap_opt("description", payload.description.as_deref());
    problems.cap_opt("receipt_ref", payload.receipt_ref.as_deref());
    ensure_valid(&problems.into_vec())?;
    rules::validate_item(
        &payload.category,
        payload.amount_minor,
        payload.incurred_on,
        Utc::now().date_naive(),
    )
    .map_err(|e| unprocessable(&e))?;
    let existing = items_of(&ctx, claim.pid).await?;
    if existing.len() >= rules::MAX_ITEMS {
        return Err(unprocessable(&format!(
            "at most {} items per claim",
            rules::MAX_ITEMS
        )));
    }
    let mut amounts: Vec<i64> = existing.iter().map(|i| i.amount_minor).collect();
    amounts.push(payload.amount_minor);
    rules::total_minor(&amounts).map_err(|e| unprocessable(&e))?;
    let clean = |v: Option<String>| v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let row = expense_items::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        claim_pid: ActiveValue::set(claim.pid),
        worker_pid: ActiveValue::set(claim.worker_pid),
        incurred_on: ActiveValue::set(payload.incurred_on),
        category: ActiveValue::set(payload.category.clone()),
        amount_minor: ActiveValue::set(payload.amount_minor),
        description: ActiveValue::set(clean(payload.description)),
        receipt_ref: ActiveValue::set(clean(payload.receipt_ref)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    format::json(json!({ "pid": row.pid }))
}

/// `DELETE /api/expense-items/{pid}` — remove an item from a draft claim.
#[debug_handler]
async fn remove_item(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let item = expense_items::Entity::find()
        .filter(expense_items::Column::Pid.eq(records::parse_pid(&pid)?))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let claim = expense_claims::Entity::find()
        .filter(expense_claims::Column::Pid.eq(item.claim_pid))
        .filter(expense_claims::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let worker = claimant_of(&ctx, &claim).await?;
    if !gate(&ctx, &caller, &worker).await?.edit() {
        return Err(denied());
    }
    if claim.status != "draft" {
        return Err(unprocessable(
            "items can only be changed while the claim is a draft",
        ));
    }
    expense_items::Entity::delete_by_id(item.id)
        .exec(&ctx.db)
        .await?;
    format::empty()
}

// ─── Moves ──────────────────────────────────────────────────────────────────

/// Who is allowed to make a move.
#[derive(Clone, Copy)]
enum Who {
    Claimant,
    Decider,
}

/// Body for moves that carry a note or a date.
#[derive(Debug, Default, Deserialize)]
struct MoveBody {
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    reimbursed_on: Option<NaiveDate>,
}

/// Tell the right person about a move: the manager when a claim is submitted, the
/// claimant when it is decided. The text names no amount, title or description.
async fn notify_move<C: sea_orm::ConnectionTrait>(
    db: &C,
    to: &str,
    worker: &workers::Model,
    row: &expense_claims::Model,
) -> Result<()> {
    match to {
        "submitted" => {
            if let Some(manager) = worker.manager_pid {
                Notification::push(
                    db,
                    manager,
                    "expense_submitted",
                    &format!(
                        "{} submitted an expense claim for your decision.",
                        worker.display_name
                    ),
                    json!({ "claim_pid": row.pid, "worker_pid": worker.pid }),
                )
                .await?;
            }
        }
        "approved" | "rejected" | "reimbursed" => {
            Notification::push(
                db,
                worker.pid,
                "expense_decided",
                &format!("Your expense claim was {to}."),
                json!({ "claim_pid": row.pid, "outcome": to }),
            )
            .await?;
        }
        _ => {}
    }
    Ok(())
}

/// One status move, serialized on the locked claim row.
async fn transition(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
    to: &str,
    who: Who,
    body: MoveBody,
) -> Result<Response> {
    let pid = records::parse_pid(pid)?;
    let peek = expense_claims::Entity::find()
        .filter(expense_claims::Column::Pid.eq(pid))
        .filter(expense_claims::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let worker = claimant_of(ctx, &peek).await?;
    let gate = gate(ctx, caller, &worker).await?;
    if !gate.view() {
        return Err(Error::NotFound);
    }
    let allowed = match who {
        Who::Claimant => gate.edit(),
        Who::Decider => gate.decide(),
    };
    if !allowed {
        return Err(denied());
    }
    let mut problems = Problems::new();
    problems.cap_opt("note", body.note.as_deref());
    ensure_valid(&problems.into_vec())?;
    let note = body
        .note
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    if to == "rejected" && note.is_none() {
        return Err(unprocessable("say why the claim is rejected (note)"));
    }
    let today = Utc::now().date_naive();
    if to == "reimbursed" && body.reimbursed_on.is_some_and(|d| d > today) {
        return Err(unprocessable("reimbursed_on cannot be in the future"));
    }

    let txn = ctx.db.begin().await?;
    let claim = expense_claims::Entity::find()
        .filter(expense_claims::Column::Pid.eq(pid))
        .filter(expense_claims::Column::DeletedAt.is_null())
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or(Error::NotFound)?;
    rules::check_transition(&claim.status, to).map_err(|e| unprocessable(&e))?;
    if to == "submitted" {
        let count = expense_items::Entity::find()
            .filter(expense_items::Column::ClaimPid.eq(claim.pid))
            .all(&txn)
            .await?
            .len();
        rules::can_submit(count).map_err(|e| unprocessable(&e))?;
    }
    let mut active: expense_claims::ActiveModel = claim.into();
    active.status = ActiveValue::set(to.to_string());
    match to {
        "submitted" => active.submitted_at = ActiveValue::set(Some(Utc::now().into())),
        "approved" | "rejected" => {
            active.decided_by = ActiveValue::set(caller.actor().map(ToString::to_string));
            active.decided_at = ActiveValue::set(Some(Utc::now().into()));
            active.decision_note = ActiveValue::set(note);
        }
        "reimbursed" => {
            active.reimbursed_on = ActiveValue::set(Some(body.reimbursed_on.unwrap_or(today)));
            active.reimbursed_by = ActiveValue::set(caller.actor().map(ToString::to_string));
        }
        _ => {}
    }
    let row = active.update(&txn).await?;
    // The audit entry and notifications name no amount, title or description.
    Audit::record(
        &txn,
        "expense_claim",
        row.pid,
        &format!("expense_claim_{to}"),
        caller.actor(),
        None,
    )
    .await?;
    notify_move(&txn, to, &worker, &row).await?;
    txn.commit().await?;
    let items = items_of(ctx, row.pid).await?;
    format::json(claim_row(&row, total(&items)?, items.len()))
}

/// The optional JSON body of a move: empty means no note and no date.
fn parse_body(bytes: &[u8]) -> Result<MoveBody> {
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(MoveBody::default());
    }
    serde_json::from_slice(bytes).map_err(|e| unprocessable(&format!("invalid JSON body: {e}")))
}

macro_rules! mover {
    ($name:ident, $to:expr, $who:expr) => {
        #[debug_handler]
        async fn $name(
            State(ctx): State<AppContext>,
            caller: MaybeAuthUser,
            Path(pid): Path<String>,
            body: axum::body::Bytes,
        ) -> Result<Response> {
            transition(&ctx, &caller, &pid, $to, $who, parse_body(&body)?).await
        }
    };
}

mover!(submit, "submitted", Who::Claimant);
mover!(withdraw, "draft", Who::Claimant);
mover!(cancel, "cancelled", Who::Claimant);
mover!(approve, "approved", Who::Decider);
mover!(reject, "rejected", Who::Decider);
mover!(reimburse, "reimbursed", Who::Decider);

// ─── Awaiting a decision ────────────────────────────────────────────────────

/// Query for the decision queue.
#[derive(Debug, Deserialize)]
struct QueueParams {
    status: Option<String>,
}

/// `GET /api/expense-claims?status=submitted` — claims the caller may decide, in
/// the given status (default `submitted`): those of their direct reports, and,
/// for HR, of their organization — never their own.
#[debug_handler]
async fn queue(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(params): axum::extract::Query<QueueParams>,
) -> Result<Response> {
    let status = params.status.unwrap_or_else(|| "submitted".to_string());
    if !rules::STATUSES.contains(&status.as_str()) {
        return Err(unprocessable(&format!(
            "status must be one of: {}",
            rules::STATUSES.join(", ")
        )));
    }
    let mut query = expense_claims::Entity::find()
        .filter(expense_claims::Column::DeletedAt.is_null())
        .filter(expense_claims::Column::Status.eq(&status));
    if auth::require_auth() {
        let Some(claims) = caller.claims() else {
            return format::json(Vec::<serde_json::Value>::new());
        };
        let person_ref = format!("person:{}", claims.sub);
        let mine: Vec<Uuid> = workers::Entity::find()
            .filter(workers::Column::PersonRef.eq(&person_ref))
            .filter(workers::Column::DeletedAt.is_null())
            .all(&ctx.db)
            .await?
            .into_iter()
            .map(|w| w.pid)
            .collect();
        let mut deciding: BTreeSet<Uuid> = BTreeSet::new();
        if !mine.is_empty() {
            deciding.extend(
                workers::Entity::find()
                    .filter(workers::Column::ManagerPid.is_in(mine.clone()))
                    .filter(workers::Column::DeletedAt.is_null())
                    .all(&ctx.db)
                    .await?
                    .into_iter()
                    .map(|w| w.pid),
            );
        }
        let hr_orgs: Vec<String> = memberships::caller_memberships(&ctx.db, caller.claims())
            .await?
            .into_iter()
            .filter(|m| HR_ROLES.contains(&m.role.as_str()))
            .map(|m| m.organization_ref)
            .collect();
        if !hr_orgs.is_empty() {
            deciding.extend(
                workers::Entity::find()
                    .filter(workers::Column::OrganizationRef.is_in(hr_orgs))
                    .filter(workers::Column::DeletedAt.is_null())
                    .all(&ctx.db)
                    .await?
                    .into_iter()
                    .map(|w| w.pid),
            );
        }
        // Never your own claim, whatever your role.
        for pid in &mine {
            deciding.remove(pid);
        }
        query = query.filter(expense_claims::Column::WorkerPid.is_in(deciding));
    }
    let claims = query
        .order_by_asc(expense_claims::Column::SubmittedAt)
        .order_by_asc(expense_claims::Column::Id)
        .all(&ctx.db)
        .await?;
    let pids: Vec<Uuid> = claims.iter().map(|c| c.worker_pid).collect();
    let names: HashMap<Uuid, String> = workers::Entity::find()
        .filter(workers::Column::Pid.is_in(pids))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| (w.pid, w.display_name))
        .collect();
    let mut out = Vec::with_capacity(claims.len());
    for c in &claims {
        let items = items_of(&ctx, c.pid).await?;
        let mut row = claim_row(c, total(&items)?, items.len());
        row["worker_name"] = json!(names.get(&c.worker_pid));
        out.push(row);
    }
    format::json(out)
}

/// The expense-claim routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/expense-claims", post(create_claim))
        .add("/workers/{pid}/expense-claims", get(list_claims))
        .add("/expense-claims", get(queue))
        .add("/expense-claims/{pid}", get(get_claim))
        .add("/expense-claims/{pid}/items", post(add_item))
        .add("/expense-items/{pid}", delete(remove_item))
        .add("/expense-claims/{pid}/submit", post(submit))
        .add("/expense-claims/{pid}/withdraw", post(withdraw))
        .add("/expense-claims/{pid}/cancel", post(cancel))
        .add("/expense-claims/{pid}/approve", post(approve))
        .add("/expense-claims/{pid}/reject", post(reject))
        .add("/expense-claims/{pid}/reimburse", post(reimburse))
}
