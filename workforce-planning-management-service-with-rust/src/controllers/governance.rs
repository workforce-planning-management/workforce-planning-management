//! **Governance load** (WPM-R64, WPM-D45): a register of the recurring commitments oversight asks of
//! a **role**, and the senior hours a month they absorb. Pure rules in [`crate::rules::governance`].
//!
//! A commitment names a role and how many people in it take part, never a person. It is
//! organization-scoped like the capacity records; every write is audited. The total is an estimate
//! someone entered, not a measurement, and the answer says so.

use super::ensure_org_in_scope;
use axa::extract::{Json, Path};
use axa::prelude::*;
use serde::Deserialize;
use serde_json::json;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db::{self, pool};
use crate::models::_entities::governance_commitments as table;
use crate::models::records;
use crate::rules::display;
use crate::rules::governance as rules;
use crate::validation::Problems;

fn view(row: &table::Model) -> serde_json::Value {
    let c = commitment(row);
    json!({
        "pid": row.pid,
        "organization_ref": row.organization_ref,
        "name": row.name,
        "role": row.role,
        "senior": row.senior,
        "kind": row.kind,
        "frequency": row.frequency,
        "minutes": row.minutes,
        "holders": row.holders,
        "hours_per_month": display::from_tenths(rules::tenth_hours_per_month(rules::minutes_per_year(&c))),
    })
}

fn commitment(row: &table::Model) -> rules::Commitment {
    rules::Commitment {
        role: row.role.clone(),
        senior: row.senior,
        kind: row.kind.clone(),
        frequency: row.frequency.clone(),
        minutes: row.minutes,
        holders: row.holders,
    }
}

/// Body for create and change.
#[derive(Debug, Deserialize)]
struct Payload {
    /// Required on create; ignored on change (a commitment does not move organization).
    organization_ref: Option<String>,
    name: String,
    role: String,
    #[serde(default)]
    senior: bool,
    kind: String,
    frequency: String,
    minutes: i32,
    #[serde(default = "one")]
    holders: i32,
}

fn one() -> i32 {
    1
}

fn check_payload(payload: &Payload) -> Result<()> {
    let mut problems = Problems::new();
    problems.require_text("name", &payload.name);
    problems.require_text("role", &payload.role);
    ensure_valid(&problems.into_vec())?;
    rules::validate(
        &payload.kind,
        &payload.frequency,
        payload.minutes,
        payload.holders,
    )
    .map_err(|e| unprocessable(&e))
}

async fn find(ctx: &AppContext, caller: &MaybeAuthUser, pid: &str) -> Result<table::Model> {
    let row = sqlx::query_as!(
        table::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name, role, senior,
                  kind, frequency, minutes, holders, recorded_by
           FROM governance_commitments WHERE pid = $1"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &row.organization_ref).await?;
    Ok(row)
}

/// `POST /api/governance-commitments`.
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<Payload>,
) -> Result<Response> {
    let org = payload
        .organization_ref
        .clone()
        .ok_or_else(|| unprocessable("organization_ref is required"))?;
    let mut problems = Problems::new();
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        &org,
    );
    ensure_valid(&problems.into_vec())?;
    check_payload(&payload)?;
    ensure_org_in_scope(&ctx, &caller, &org).await?;
    let mut tx = pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        table::Model,
        r#"INSERT INTO governance_commitments
               (pid, organization_ref, name, role, senior, kind, frequency, minutes, holders, recorded_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name, role, senior,
                  kind, frequency, minutes, holders, recorded_by"#,
        Uuid::new_v4(),
        org,
        payload.name,
        payload.role,
        payload.senior,
        payload.kind,
        payload.frequency,
        payload.minutes,
        payload.holders,
        caller.actor(),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "governance_commitment",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(view(&row))
}

/// `PUT /api/governance-commitments/{pid}`.
#[debug_handler]
async fn change(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<Payload>,
) -> Result<Response> {
    let row = find(&ctx, &caller, &pid).await?;
    check_payload(&payload)?;
    let mut tx = pool(&ctx).begin().await?;
    let saved = sqlx::query_as!(
        table::Model,
        r#"UPDATE governance_commitments
           SET name = $2, role = $3, senior = $4, kind = $5, frequency = $6, minutes = $7, holders = $8, recorded_by = $9
           WHERE id = $1
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name, role, senior,
                  kind, frequency, minutes, holders, recorded_by"#,
        row.id,
        payload.name,
        payload.role,
        payload.senior,
        payload.kind,
        payload.frequency,
        payload.minutes,
        payload.holders,
        caller.actor(),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "governance_commitment",
        saved.pid,
        "changed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(view(&saved))
}

/// `DELETE /api/governance-commitments/{pid}` — the row is removed; it holds no personal data.
#[debug_handler]
async fn remove(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = find(&ctx, &caller, &pid).await?;
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!("DELETE FROM governance_commitments WHERE id = $1", row.id)
        .execute(&mut *tx)
        .await?;
    db::audit(
        &mut tx,
        "governance_commitment",
        row.pid,
        "removed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "removed": row.pid }))
}

/// Query for the list and the load.
#[derive(Debug, Deserialize)]
pub struct OrgQuery {
    organization: Option<String>,
}

async fn rows_of(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    query: &OrgQuery,
) -> Result<(String, Vec<table::Model>)> {
    let org = query
        .organization
        .clone()
        .ok_or_else(|| unprocessable("organization is required"))?;
    ensure_org_in_scope(ctx, caller, &org).await?;
    let rows = sqlx::query_as!(
        table::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name, role, senior,
                  kind, frequency, minutes, holders, recorded_by
           FROM governance_commitments WHERE organization_ref = $1 ORDER BY role, name"#,
        org,
    )
    .fetch_all(pool(ctx))
    .await?;
    Ok((org, rows))
}

/// `GET /api/governance-commitments?organization=`.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(query): axa::extract::Query<OrgQuery>,
) -> Result<Response> {
    let (_, rows) = rows_of(&ctx, &caller, &query).await?;
    format::json(rows.iter().map(view).collect::<Vec<_>>())
}

/// `GET /api/governance-commitments/load?organization=` — hours a month per role, and the senior
/// total.
#[debug_handler]
async fn load(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(query): axa::extract::Query<OrgQuery>,
) -> Result<Response> {
    let (org, rows) = rows_of(&ctx, &caller, &query).await?;
    let commitments: Vec<_> = rows.iter().map(commitment).collect();
    let total = rules::load(&commitments);
    format::json(json!({
        "organization_ref": org,
        "commitments": rows.len(),
        "senior_hours_per_month": display::from_tenths(total.senior_tenth_hours_per_month),
        "all_hours_per_month": display::from_tenths(total.all_tenth_hours_per_month),
        "roles": total.roles.iter().map(|r| json!({
            "role": r.role,
            "senior": r.senior,
            "commitments": r.commitments,
            "hours_per_month": display::from_tenths(r.tenth_hours_per_month),
        })).collect::<Vec<_>>(),
        "basis": "an estimate from the entered commitments: occurrences a year × minutes × people in \
                  the role, per month. A role, never a person; not a timesheet.",
    }))
}

/// The governance load of one organization for the monthly pack: the totals only. No role and no
/// commitment appears, since a free-text role could carry a name.
pub(super) async fn pack_json(ctx: &AppContext, org: &str) -> Result<serde_json::Value> {
    let rows = sqlx::query_as!(
        table::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name, role, senior,
                  kind, frequency, minutes, holders, recorded_by
           FROM governance_commitments WHERE organization_ref = $1"#,
        org,
    )
    .fetch_all(pool(ctx))
    .await?;
    let commitments: Vec<_> = rows.iter().map(commitment).collect();
    let total = rules::load(&commitments);
    Ok(json!({
        "commitments": rows.len(),
        "senior_hours_per_month": display::from_tenths(total.senior_tenth_hours_per_month),
        "all_hours_per_month": display::from_tenths(total.all_tenth_hours_per_month),
        "basis": "An estimate from the commitments entered as at the freeze date; not a timesheet.",
    }))
}

/// The governance routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/governance-commitments", get(list).post(create))
        .add("/governance-commitments/load", get(load))
        .add("/governance-commitments/{pid}", put(change).delete(remove))
}
