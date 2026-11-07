//! **Monthly oversight packs** (WPM-R69, WPM-T116): the oversight position, criteria, measures,
//! capacity view, key-person counts and governance hours frozen as of a date. The internal board, the
//! oversight body and partner organizations read the same stored payload, so no audience is shown a
//! different set of figures.
//!
//! The payload holds aggregates only: pools by name, months, counts and verdicts. It carries no
//! worker, no worker number, no role and no free text a person typed other than the published reason
//! for a position, so nothing needs removing for a partner. A pack is written once (a trigger refuses
//! any update or delete); a mistaken one is withdrawn by a separate row with a reason, and the mark is
//! shown beside the unchanged payload. A caller who holds only the `partner` role may read packs and
//! nothing else (see [`crate::rules::access::partner_may`]). Freezing and withdrawing are for an
//! administrator of the organization.

use super::ensure_org_in_scope;
use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::Datelike;
use serde::Deserialize;
use serde_json::json;

use super::{capacity, ensure_valid, forbidden, governance, key_person, oversight, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db::{self, pool};
use crate::models::_entities::{
    oversight_pack_withdrawals as withdrawals, oversight_packs as packs,
};
use crate::models::{memberships, records};
use crate::need_to_know::has_privileged_attrs;
use crate::validation::Problems;

/// Who may freeze or withdraw a pack: an HR or organization administrator of the organization, or a
/// caller whose token carries HR, payroll, service or administrator rights. Reading is open to anyone
/// in scope, which is the point of a pack; changing the record of what was shown is not.
pub(super) async fn ensure_may_freeze(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    org: &str,
) -> Result<()> {
    ensure_org_administrator(ctx, caller, org, "freezing or withdrawing a pack").await
}

/// An administrator of the organization (HR or organization), or a caller whose token carries HR,
/// payroll, service or administrator rights; anyone when sign-in is not enforced. `what` completes the
/// refusal: "<what> is for an administrator of the organization".
pub(crate) async fn ensure_org_administrator(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    org: &str,
    what: &str,
) -> Result<()> {
    if !crate::auth::require_auth() {
        return Ok(());
    }
    if let Some(claims) = caller.claims()
        && (has_privileged_attrs(claims)
            || memberships::has_role_in(&ctx.db, Some(claims), org, &["hr_admin", "org_admin"])
                .await?)
    {
        return Ok(());
    }
    Err(forbidden(&format!(
        "{what} is for an administrator of the organization"
    )))
}

/// `POST /api/oversight/packs` body.
#[derive(Debug, Deserialize)]
struct PackPayload {
    organization_ref: String,
}

/// The deciding body is free text entered by a person, and may be a name; the pack gives the level,
/// the date and the published reason, and leaves the body to the status route.
fn strip_deciding_body(status: &mut serde_json::Value) {
    if let Some(current) = status.get_mut("current").and_then(|c| c.as_object_mut()) {
        current.remove("deciding_body");
    }
    if let Some(list) = status.get_mut("positions").and_then(|p| p.as_array_mut()) {
        for position in list {
            if let Some(o) = position.as_object_mut() {
                o.remove("deciding_body");
            }
        }
    }
}

/// Freeze the pack for `org` as of `today`.
///
/// # Errors
///
/// A database error.
pub async fn freeze(ctx: &AppContext, org: &str, today: chrono::NaiveDate) -> Result<packs::Model> {
    let (month, payload) = pack_payload(ctx, org, today).await?;
    insert_pack(&mut *pool(ctx).acquire().await?, org, today, month, payload).await
}

/// The month and frozen payload for `org` as of `today`, read before any transaction opens.
async fn pack_payload(
    ctx: &AppContext,
    org: &str,
    today: chrono::NaiveDate,
) -> Result<(chrono::NaiveDate, serde_json::Value)> {
    let this_month = today.with_day(1).unwrap_or(today);
    let month = this_month
        .checked_sub_months(chrono::Months::new(1))
        .unwrap_or(this_month);
    let mut status = oversight::status_json(ctx, org, today).await?;
    strip_deciding_body(&mut status);
    let payload = json!({
        "organization_ref": org,
        "as_of": today,
        "month": month,
        "oversight": status,
        "capacity": capacity::pack_view(ctx, org, this_month).await?,
        "key_person": key_person::pack_json(ctx, org).await?,
        "governance": governance::pack_json(ctx, org).await?,
        "basis": "Frozen as of the date shown and never changed. The same payload is read by every \
                  audience. It holds aggregates only: no worker, no worker number, no role and no list of \
                  people. An unknown month is shown as unknown, never as a zero.",
    });
    Ok((month, payload))
}

/// Store a frozen pack on `connection` (a transaction, when the caller audits it in the same one).
async fn insert_pack(
    connection: &mut sqlx::PgConnection,
    org: &str,
    today: chrono::NaiveDate,
    month: chrono::NaiveDate,
    payload: serde_json::Value,
) -> Result<packs::Model> {
    Ok(sqlx::query_as!(
        packs::Model,
        r#"INSERT INTO oversight_packs (pid, organization_ref, as_of, month, payload)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, as_of, month, payload"#,
        Uuid::new_v4(),
        org,
        today,
        month,
        payload,
    )
    .fetch_one(connection)
    .await?)
}

async fn withdrawal_of(ctx: &AppContext, pack: Uuid) -> Result<Option<withdrawals::Model>> {
    Ok(sqlx::query_as!(
        withdrawals::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, pack_pid, withdrawn_on, reason
           FROM oversight_pack_withdrawals WHERE pack_pid = $1"#,
        pack,
    )
    .fetch_optional(pool(ctx))
    .await?)
}

fn withdrawn_json(w: Option<&withdrawals::Model>) -> serde_json::Value {
    w.map_or(
        serde_json::Value::Null,
        |w| json!({ "on": w.withdrawn_on, "reason": w.reason }),
    )
}

fn pack_json(p: &packs::Model, w: Option<&withdrawals::Model>) -> serde_json::Value {
    json!({
        "pid": p.pid, "organization_ref": p.organization_ref, "as_of": p.as_of, "month": p.month,
        "withdrawn": withdrawn_json(w), "pack": p.payload,
    })
}

/// `POST /api/oversight/packs` — freeze a pack now.
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<PackPayload>,
) -> Result<Response> {
    ensure_org_in_scope(&ctx, &caller, &p.organization_ref).await?;
    ensure_may_freeze(&ctx, &caller, &p.organization_ref).await?;
    let today = chrono::Utc::now().date_naive();
    let (month, payload) = pack_payload(&ctx, &p.organization_ref, today).await?;
    let mut tx = pool(&ctx).begin().await?;
    let row = insert_pack(&mut tx, &p.organization_ref, today, month, payload).await?;
    db::audit(
        &mut tx,
        "oversight_pack",
        row.pid,
        "frozen",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(pack_json(&row, None))
}

/// Query naming the organization.
#[derive(Debug, Deserialize)]
pub struct OrgQuery {
    organization: Option<String>,
}

/// `GET /api/oversight/packs?organization=` — the packs, newest first, without their payloads.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(q): axa::extract::Query<OrgQuery>,
    page: super::Page,
) -> Result<Response> {
    let window = super::window(&page)?;
    let org = q
        .organization
        .ok_or_else(|| unprocessable("organization is required"))?;
    ensure_org_in_scope(&ctx, &caller, &org).await?;
    let (limit, offset) = window.bounds();
    let rows = sqlx::query!(
        "SELECT pid, as_of, month FROM oversight_packs
         WHERE organization_ref = $1
         ORDER BY id DESC
         LIMIT $2 OFFSET $3",
        org,
        limit,
        offset,
    )
    .fetch_all(pool(&ctx))
    .await?;
    // The body is an object, so the page is cut here and the next page's `Link` set by hand.
    let (rows, more) = window.split(rows);
    let mut out = Vec::new();
    for p in &rows {
        let w = withdrawal_of(&ctx, p.pid).await?;
        out.push(json!({
            "pid": p.pid, "as_of": p.as_of, "month": p.month, "withdrawn": withdrawn_json(w.as_ref()),
        }));
    }
    Ok(super::with_next(
        axum::Json(json!({ "organization_ref": org, "packs": out })).into_response(),
        &window,
        more,
    ))
}

async fn pack_in_scope(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<packs::Model> {
    let pid = records::parse_pid(pid)?;
    let row = sqlx::query_as!(
        packs::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, as_of, month, payload
           FROM oversight_packs WHERE pid = $1"#,
        pid,
    )
    .fetch_optional(pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &row.organization_ref).await?;
    Ok(row)
}

/// `GET /api/oversight/packs/{pid}` — one pack, as frozen, with the withdrawal mark if it has one.
#[debug_handler]
async fn get_one(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = pack_in_scope(&ctx, &caller, &pid).await?;
    let w = withdrawal_of(&ctx, row.pid).await?;
    format::json(pack_json(&row, w.as_ref()))
}

/// `POST /api/oversight/packs/{pid}/withdraw` body.
#[derive(Debug, Deserialize)]
struct WithdrawPayload {
    reason: String,
}

/// `POST /api/oversight/packs/{pid}/withdraw` — mark a pack as withdrawn, with the reason. The pack
/// is not changed or removed: it stays on the record, and every reader sees the mark beside it.
#[debug_handler]
async fn withdraw(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(p): Json<WithdrawPayload>,
) -> Result<Response> {
    let row = pack_in_scope(&ctx, &caller, &pid).await?;
    ensure_may_freeze(&ctx, &caller, &row.organization_ref).await?;
    let mut problems = Problems::new();
    problems.require_text("reason", &p.reason);
    ensure_valid(&problems.into_vec())?;
    if withdrawal_of(&ctx, row.pid).await?.is_some() {
        return Err(unprocessable("this pack is already withdrawn"));
    }
    let mut tx = pool(&ctx).begin().await?;
    let w = sqlx::query_as!(
        withdrawals::Model,
        r#"INSERT INTO oversight_pack_withdrawals (pid, pack_pid, withdrawn_on, reason)
           VALUES ($1, $2, $3, $4)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, pack_pid, withdrawn_on, reason"#,
        Uuid::new_v4(),
        row.pid,
        chrono::Utc::now().date_naive(),
        p.reason,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "oversight_pack",
        row.pid,
        "withdrawn",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(pack_json(&row, Some(&w)))
}

/// The pack routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/oversight/packs", get(list))
        .add("/oversight/packs", post(create))
        .add("/oversight/packs/{pid}", get(get_one))
        .add("/oversight/packs/{pid}/withdraw", post(withdraw))
}
