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

use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

use super::{ensure_valid, forbidden, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{expense_claims, expense_items, workers};
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
        Some(pid) => sqlx::query_scalar!(
            "SELECT person_ref FROM workers WHERE pid = $1 AND deleted_at IS NULL",
            pid
        )
        .fetch_optional(db::pool(ctx))
        .await?
        .is_some_and(|person_ref| career::is_self(&claims.sub, &person_ref)),
        None => false,
    };
    let hr = memberships::has_role_in(ctx, caller.claims(), &claimant.organization_ref, HR_ROLES)
        .await?;
    Ok(Gate(Some(Standing {
        claimant: is_claimant,
        manager,
        hr,
    })))
}

fn denied() -> Error {
    forbidden("not permitted on this expense claim")
}

async fn find_claim(ctx: &AppContext, pid: &str) -> Result<expense_claims::Model> {
    live_claim(ctx, records::parse_pid(pid)?)
        .await?
        .ok_or(Error::NotFound)
}

/// The live claim with this pid, if any.
async fn live_claim(ctx: &AppContext, pid: Uuid) -> Result<Option<expense_claims::Model>> {
    Ok(sqlx::query_as!(
        expense_claims::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, title, description,
                  currency, status, submitted_at AS "submitted_at: _", decided_by, decided_at AS "decided_at: _",
                  decision_note, reimbursed_on, reimbursed_by, recorded_by, on_behalf, deleted_at AS "deleted_at: _"
             FROM expense_claims WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?)
}

async fn claimant_of(ctx: &AppContext, claim: &expense_claims::Model) -> Result<workers::Model> {
    records::worker_by_pid(ctx, claim.worker_pid).await
}

async fn items_of(ctx: &AppContext, claim_pid: Uuid) -> Result<Vec<expense_items::Model>> {
    Ok(sqlx::query_as!(
        expense_items::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, claim_pid, worker_pid, incurred_on,
                  category, amount_minor, description, receipt_ref
             FROM expense_items WHERE claim_pid = $1 ORDER BY incurred_on, id"#,
        claim_pid,
    )
    .fetch_all(db::pool(ctx))
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
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
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
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        expense_claims::Model,
        r#"INSERT INTO expense_claims (pid, worker_pid, title, description, currency, status, recorded_by, on_behalf)
           VALUES ($1, $2, $3, $4, $5, 'draft', $6, $7)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, title, description,
                     currency, status, submitted_at AS "submitted_at: _", decided_by, decided_at AS "decided_at: _",
                     decision_note, reimbursed_on, reimbursed_by, recorded_by, on_behalf, deleted_at AS "deleted_at: _""#,
        Uuid::new_v4(),
        worker.pid,
        payload.title.trim(),
        payload
            .description
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty()),
        payload.currency,
        caller.actor(),
        auth::acting_for_other(&caller, &worker.person_ref),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "expense_claim",
        row.pid,
        "expense_claim_created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    if !gate(&ctx, &caller, &worker).await?.view() {
        return Err(denied());
    }
    let claims = sqlx::query_as!(
        expense_claims::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, title, description,
                  currency, status, submitted_at AS "submitted_at: _", decided_by, decided_at AS "decided_at: _",
                  decision_note, reimbursed_on, reimbursed_by, recorded_by, on_behalf, deleted_at AS "deleted_at: _"
             FROM expense_claims WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY id DESC"#,
        worker.pid,
    )
    .fetch_all(db::pool(&ctx))
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
    Ok(sqlx::query!(
        "SELECT i.incurred_on, i.category, i.amount_minor
           FROM expense_items i
          WHERE i.claim_pid IN (SELECT c.pid FROM expense_claims c
                                 WHERE c.worker_pid = $1 AND c.deleted_at IS NULL AND c.pid <> $2
                                   AND c.status NOT IN ('cancelled', 'rejected'))
          ORDER BY i.id",
        claim.worker_pid,
        claim.pid,
    )
    .fetch_all(db::pool(ctx))
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
    let pid = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO expense_items (pid, claim_pid, worker_pid, incurred_on, category, amount_minor, description, receipt_ref)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        pid,
        claim.pid,
        claim.worker_pid,
        payload.incurred_on,
        payload.category,
        payload.amount_minor,
        clean(payload.description),
        clean(payload.receipt_ref),
    )
    .execute(db::pool(&ctx))
    .await?;
    format::json(json!({ "pid": pid }))
}

/// `DELETE /api/expense-items/{pid}` — remove an item from a draft claim.
#[debug_handler]
async fn remove_item(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let item = sqlx::query!(
        "SELECT id, claim_pid FROM expense_items WHERE pid = $1",
        records::parse_pid(&pid)?
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let claim = live_claim(&ctx, item.claim_pid)
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
    sqlx::query!("DELETE FROM expense_items WHERE id = $1", item.id)
        .execute(db::pool(&ctx))
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
async fn notify_move(
    tx: &mut sqlx::PgConnection,
    to: &str,
    worker: &workers::Model,
    row: &expense_claims::Model,
) -> Result<()> {
    match to {
        "submitted" => {
            if let Some(manager) = worker.manager_pid {
                db::notify(
                    tx,
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
            db::notify(
                tx,
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
#[allow(clippy::too_many_lines)] // the audit is written out once per move
async fn transition(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
    to: &str,
    who: Who,
    body: MoveBody,
) -> Result<Response> {
    let pid = records::parse_pid(pid)?;
    let peek = live_claim(ctx, pid).await?.ok_or(Error::NotFound)?;
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

    let mut tx = db::pool(ctx).begin().await?;
    let claim = sqlx::query!(
        "SELECT id, pid, status FROM expense_claims WHERE pid = $1 AND deleted_at IS NULL FOR UPDATE",
        pid,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;
    rules::check_transition(&claim.status, to).map_err(|e| unprocessable(&e))?;
    if to == "submitted" {
        let count = sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM expense_items WHERE claim_pid = $1"#,
            claim.pid
        )
        .fetch_one(&mut *tx)
        .await?;
        rules::can_submit(usize::try_from(count).unwrap_or(usize::MAX))
            .map_err(|e| unprocessable(&e))?;
    }
    let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
    let decided = matches!(to, "approved" | "rejected");
    let reimbursed = to == "reimbursed";
    let row = sqlx::query_as!(
        expense_claims::Model,
        r#"UPDATE expense_claims
              SET status = $2,
                  submitted_at = CASE WHEN $3 THEN $4::timestamptz ELSE submitted_at END,
                  decided_by = CASE WHEN $5 THEN $6::text ELSE decided_by END,
                  decided_at = CASE WHEN $5 THEN $4::timestamptz ELSE decided_at END,
                  decision_note = CASE WHEN $5 THEN $7::text ELSE decision_note END,
                  reimbursed_on = CASE WHEN $8 THEN $9::date ELSE reimbursed_on END,
                  reimbursed_by = CASE WHEN $8 THEN $6::text ELSE reimbursed_by END
            WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, title, description,
                  currency, status, submitted_at AS "submitted_at: _", decided_by, decided_at AS "decided_at: _",
                  decision_note, reimbursed_on, reimbursed_by, recorded_by, on_behalf, deleted_at AS "deleted_at: _""#,
        claim.id,
        to,
        to == "submitted",
        now,
        decided,
        caller.actor(),
        note,
        reimbursed,
        body.reimbursed_on.unwrap_or(today),
    )
    .fetch_one(&mut *tx)
    .await?;
    // The audit entry and notifications name no amount, title or description. One literal action per move.
    match to {
        "submitted" => {
            db::audit(
                &mut tx,
                "expense_claim",
                row.pid,
                "expense_claim_submitted",
                caller.actor(),
                None,
            )
            .await?;
        }
        "draft" => {
            db::audit(
                &mut tx,
                "expense_claim",
                row.pid,
                "expense_claim_draft",
                caller.actor(),
                None,
            )
            .await?;
        }
        "cancelled" => {
            db::audit(
                &mut tx,
                "expense_claim",
                row.pid,
                "expense_claim_cancelled",
                caller.actor(),
                None,
            )
            .await?;
        }
        "approved" => {
            db::audit(
                &mut tx,
                "expense_claim",
                row.pid,
                "expense_claim_approved",
                caller.actor(),
                None,
            )
            .await?;
        }
        "rejected" => {
            db::audit(
                &mut tx,
                "expense_claim",
                row.pid,
                "expense_claim_rejected",
                caller.actor(),
                None,
            )
            .await?;
        }
        "reimbursed" => {
            db::audit(
                &mut tx,
                "expense_claim",
                row.pid,
                "expense_claim_reimbursed",
                caller.actor(),
                None,
            )
            .await?;
        }
        other => return Err(super::internal(&format!("no audit action for {other}"))),
    }
    notify_move(&mut tx, to, &worker, &row).await?;
    tx.commit().await?;
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
    axa::extract::Query(params): axa::extract::Query<QueueParams>,
    page: super::Page,
) -> Result<Response> {
    let window = super::window(&page)?;
    let status = params.status.unwrap_or_else(|| "submitted".to_string());
    if !rules::STATUSES.contains(&status.as_str()) {
        return Err(unprocessable(&format!(
            "status must be one of: {}",
            rules::STATUSES.join(", ")
        )));
    }
    // With sign-in off, everyone's claims; with it on, only the ones the caller may decide.
    let mut only: Option<Vec<Uuid>> = None;
    if auth::require_auth() {
        let Some(claims) = caller.claims() else {
            return format::json(Vec::<serde_json::Value>::new());
        };
        let person_ref = format!("person:{}", claims.sub);
        let mine: Vec<Uuid> = sqlx::query_scalar!(
            "SELECT pid FROM workers WHERE person_ref = $1 AND deleted_at IS NULL",
            person_ref,
        )
        .fetch_all(db::pool(&ctx))
        .await?;
        let mut deciding: BTreeSet<Uuid> = BTreeSet::new();
        if !mine.is_empty() {
            deciding.extend(
                sqlx::query_scalar!(
                    "SELECT pid FROM workers WHERE manager_pid = ANY($1) AND deleted_at IS NULL",
                    &mine,
                )
                .fetch_all(db::pool(&ctx))
                .await?,
            );
        }
        let hr_orgs: Vec<String> = memberships::caller_memberships(&ctx, caller.claims())
            .await?
            .into_iter()
            .filter(|m| HR_ROLES.contains(&m.role.as_str()))
            .map(|m| m.organization_ref)
            .collect();
        if !hr_orgs.is_empty() {
            deciding.extend(
                sqlx::query_scalar!(
                    "SELECT pid FROM workers WHERE organization_ref = ANY($1) AND deleted_at IS NULL",
                    &hr_orgs,
                )
                .fetch_all(db::pool(&ctx))
                .await?,
            );
        }
        // Never your own claim, whatever your role.
        for pid in &mine {
            deciding.remove(pid);
        }
        only = Some(deciding.into_iter().collect());
    }
    let claims = sqlx::query_as!(
        expense_claims::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, title, description,
                  currency, status, submitted_at AS "submitted_at: _", decided_by, decided_at AS "decided_at: _",
                  decision_note, reimbursed_on, reimbursed_by, recorded_by, on_behalf, deleted_at AS "deleted_at: _"
             FROM expense_claims
            WHERE deleted_at IS NULL AND status = $1 AND ($2::uuid[] IS NULL OR worker_pid = ANY($2))
            ORDER BY submitted_at, id"#,
        status,
        only.as_deref(),
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let pids: Vec<Uuid> = claims.iter().map(|c| c.worker_pid).collect();
    let names: HashMap<Uuid, String> = sqlx::query!(
        "SELECT pid, display_name FROM workers WHERE pid = ANY($1)",
        &pids
    )
    .fetch_all(db::pool(&ctx))
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
    Ok(super::paged_vec(out, &window))
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
