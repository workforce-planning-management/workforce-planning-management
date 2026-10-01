//! Multi-organization membership: a signed-in person can belong to
//! several organizations in one sign-in — no switcher, unified
//! visibility — with a role per organization. One membership table
//! covers both a literal employment relationship (`worker_pid` set)
//! and a staff/admin-only privilege grant (`worker_pid` null); see
//! `migration/src/m20260928_000018_organization_memberships.rs`.
//!
//! Also **org confederation** (WPM-Rxx): a parent organization that
//! transitively contains member organizations (`organization_ref` is
//! never re-used across the two concepts — a membership names a
//! *person's* access, a confederation edge names an *organization's*
//! place in a tree; see
//! `migration/src/m20260929_000019_organization_confederations.rs`).
//! A membership in a parent reads as membership in every descendant
//! too ([`crate::models::memberships::caller_scope_refs`]).
//!
//! Authorization for grant/revoke is a **second, WPM-local pass** on
//! top of the blanket ABAC guard every mutation already goes through
//! (`enforce` + the shipped policy's `access=write`/`hr=true` rules) —
//! the same two-pass shape [`crate::auth::authorize_record`] uses for
//! record-level attributes, just DB-backed instead of policy-JSON,
//! since "does this caller hold a qualifying role in this specific
//! organization" depends on rows the ABAC engine has no way to see
//! (the token carries no per-organization claims — see
//! [`crate::models::memberships`]'s own doc comment). [`may_manage_membership`]
//! and [`may_manage_confederation`] are both a no-op (`true`) while
//! `WPM_REQUIRE_AUTH` is off, matching every other authorization pass
//! in this crate.

use loco_rs::prelude::*;
use axum::http::StatusCode;
use sea_orm::{ActiveValue, PaginatorTrait, QueryOrder, TransactionTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ensure_valid, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{organization_confederations, organization_memberships};
use crate::models::audit_logs::Model as Audit;
use crate::models::{confederations, memberships, records};
use crate::rules::{org_access, tokens};
use crate::streaming;
use crate::validation::Problems;

/// Whether `caller` may grant or revoke a `target_role` membership in
/// `organization_ref`: either the coarse operator bypass this crate's
/// other admin-only actions already use (`svc=true` or `access=admin`
/// in the token's `attrs`), or an org-scoped membership of their own
/// whose role satisfies [`org_access::can_manage_membership`].
///
/// # Errors
///
/// Any query error from resolving the caller's own memberships.
async fn may_manage_membership(
    db: &sea_orm::DatabaseConnection,
    caller: &MaybeAuthUser,
    organization_ref: &str,
    target_role: &str,
) -> Result<bool> {
    if !auth::require_auth() {
        return Ok(true);
    }
    let claims = caller.claims();
    let is_operator = claims.is_some_and(|c| {
        c.attrs
            .get("svc")
            .is_some_and(|v| v.iter().any(|s| s == "true"))
            || c
                .attrs
                .get("access")
                .is_some_and(|v| v.iter().any(|s| s == "admin"))
    });
    if is_operator {
        return Ok(true);
    }
    let mine = memberships::caller_memberships(db, claims).await?;
    Ok(mine
        .iter()
        .any(|m| m.organization_ref == organization_ref
            && org_access::can_manage_membership(&m.role, target_role)))
}

/// A `403` for [`may_manage_membership`] returning `false`.
fn not_privileged() -> Error {
    record_rejection((
        StatusCode::FORBIDDEN,
        "not privileged to manage this role in this organization".to_string(),
    ))
}

/// Whether `caller` may declare or revoke a confederation edge naming
/// `parent_organization_ref` as the parent: either the coarse operator
/// bypass (see [`may_manage_membership`]), or a **direct** `org_admin`
/// membership in that exact organization. Deliberately not extended
/// through an ancestor confederation (unlike read-scoping, which does
/// expand transitively) — declaring where an organization sits in the
/// tree is a structural change with a bigger blast radius than adding
/// one member, so it stays scoped to admins of the organization named
/// as parent, not admins further up the tree.
///
/// # Errors
///
/// Any query error from resolving the caller's own memberships.
async fn may_manage_confederation(
    db: &sea_orm::DatabaseConnection,
    caller: &MaybeAuthUser,
    parent_organization_ref: &str,
) -> Result<bool> {
    if !auth::require_auth() {
        return Ok(true);
    }
    let claims = caller.claims();
    let is_operator = claims.is_some_and(|c| {
        c.attrs
            .get("svc")
            .is_some_and(|v| v.iter().any(|s| s == "true"))
            || c
                .attrs
                .get("access")
                .is_some_and(|v| v.iter().any(|s| s == "admin"))
    });
    if is_operator {
        return Ok(true);
    }
    let mine = memberships::caller_memberships(db, claims).await?;
    Ok(mine
        .iter()
        .any(|m| m.organization_ref == parent_organization_ref && m.role == "org_admin"))
}

/// A `403` for [`may_manage_confederation`] returning `false`.
fn not_privileged_for_confederation() -> Error {
    record_rejection((
        StatusCode::FORBIDDEN,
        "not privileged to manage this organization's confederation edges".to_string(),
    ))
}

/// One membership row, as returned to a caller.
#[derive(Debug, Serialize)]
struct MembershipView {
    pid: String,
    person_ref: String,
    organization_ref: String,
    worker_pid: Option<Uuid>,
    /// Convenience flag: `true` when this membership is a literal
    /// employment relationship (`worker_pid` is set), so a consumer
    /// doesn't have to infer it from nullability itself.
    employed: bool,
    role: String,
    starts_on: chrono::NaiveDate,
    ends_on: Option<chrono::NaiveDate>,
}

impl From<organization_memberships::Model> for MembershipView {
    fn from(row: organization_memberships::Model) -> Self {
        Self {
            pid: row.pid.to_string(),
            person_ref: row.person_ref,
            organization_ref: row.organization_ref,
            worker_pid: row.worker_pid,
            employed: row.worker_pid.is_some(),
            role: row.role,
            starts_on: row.starts_on,
            ends_on: row.ends_on,
        }
    }
}

/// The live (not soft-deleted) memberships for one `person_ref`,
/// oldest first — shared by the token-derived "mine" read and the
/// `?person_ref=` admin listing below, so the two never drift.
async fn memberships_for(
    db: &sea_orm::DatabaseConnection,
    person_ref: &str,
) -> Result<Vec<MembershipView>> {
    let rows = organization_memberships::Entity::find()
        .filter(organization_memberships::Column::PersonRef.eq(person_ref))
        .filter(organization_memberships::Column::DeletedAt.is_null())
        .order_by_asc(organization_memberships::Column::Id)
        .all(db)
        .await?;
    Ok(rows.into_iter().map(MembershipView::from).collect())
}

/// `GET /api/me/organizations` — the caller's own memberships across
/// every organization they belong to (no switcher: the whole set, at
/// once). A signed-out caller sees an empty list rather than `401`,
/// matching this crate's "absent claims ⇒ neutral, always boots"
/// convention elsewhere (e.g. [`MaybeAuthUser::actor`]).
#[debug_handler]
async fn list_my_organizations(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
) -> Result<Response> {
    let Some(claims) = caller.claims() else {
        return format::json(Vec::<MembershipView>::new());
    };
    let person_ref = format!("person:{}", claims.sub);
    format::json(memberships_for(&ctx.db, &person_ref).await?)
}

/// `GET /api/me/organizations/scope` — every organization the caller
/// can currently read, **expanded through confederation**: each of
/// their memberships' own `organization_ref` plus every transitive
/// descendant ([`memberships::caller_scope_refs`]). A flat list of
/// URNs, not membership rows — distinct from [`list_my_organizations`]
/// (literal membership grants only): a caller with one membership in a
/// parent confederation has one row there, but several entries here.
/// The front-end uses this, not `/me/organizations`, to decide which
/// per-organization sections to render on pages like `/org-chart` and
/// `/benchmarks`. Empty (never `401`) when signed out, matching this
/// crate's "absent claims ⇒ neutral" convention.
#[debug_handler]
async fn my_organization_scope(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
) -> Result<Response> {
    format::json(memberships::caller_scope_refs(&ctx.db, caller.claims()).await?)
}

/// `GET /api/organization-memberships?person_ref=` — the admin/HR
/// listing for an explicit person, mirroring `list_workers`'
/// query-param-filter shape. Distinct from `/me/organizations`
/// (token-derived) so a caller who can already see other people's
/// records (HR, an org admin) can look theirs up without needing to
/// impersonate a token — and so this surface is exercisable by the
/// existing DB-gated request-test harness, which never mints a bearer
/// token (verified: no test in this crate sets an `Authorization`
/// header).
#[derive(Debug, Deserialize)]
struct ListQuery {
    #[serde(default)]
    person_ref: Option<String>,
}

#[debug_handler]
async fn list_memberships(
    State(ctx): State<AppContext>,
    Query(query): Query<ListQuery>,
) -> Result<Response> {
    let person_ref = match query.person_ref {
        Some(v) if !v.trim().is_empty() => v,
        _ => return Err(unprocessable("person_ref is required")),
    };
    format::json(memberships_for(&ctx.db, &person_ref).await?)
}

/// `POST /api/organization-memberships` body.
#[derive(Debug, Deserialize)]
struct GrantPayload {
    person_ref: String,
    organization_ref: String,
    #[serde(default)]
    worker_pid: Option<Uuid>,
    role: String,
    starts_on: chrono::NaiveDate,
    #[serde(default)]
    ends_on: Option<chrono::NaiveDate>,
}

/// A `{pid}` reference response.
#[derive(Debug, Serialize)]
struct PidRef {
    pid: String,
}

/// `POST /api/organization-memberships` — grant a person a role in an
/// organization. Rejects a duplicate live `(person_ref,
/// organization_ref, role)` grant (the partial unique index's own
/// invariant, checked ahead of the insert for an honest `422` rather
/// than a raw constraint-violation `500`). When `worker_pid` is given,
/// the referenced live worker's own `person_ref`/`organization_ref`
/// must match the payload's — the only referential check available,
/// since this schema has no DB foreign keys anywhere (app-level
/// integrity throughout, e.g. `benefit_enrollments.worker_pid`).
#[debug_handler]
async fn grant(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<GrantPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_ref(
        "person_ref",
        entity_ref::EntityType::Person,
        &payload.person_ref,
    );
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        &payload.organization_ref,
    );
    problems.require_token("role", tokens::ORGANIZATION_ROLES, &payload.role);
    if payload.ends_on.is_some_and(|end| end < payload.starts_on) {
        problems.push("ends_on is before starts_on");
    }
    ensure_valid(&problems.into_vec())?;

    if !may_manage_membership(&ctx.db, &caller, &payload.organization_ref, &payload.role).await? {
        return Err(not_privileged());
    }

    if let Some(worker_pid) = payload.worker_pid {
        let worker = records::find_worker(&ctx.db, worker_pid).await?;
        let matches = worker.person_ref == payload.person_ref
            && worker.organization_ref == payload.organization_ref;
        if !matches {
            return Err(unprocessable(
                "worker_pid does not match person_ref/organization_ref",
            ));
        }
    }

    let existing = organization_memberships::Entity::find()
        .filter(organization_memberships::Column::PersonRef.eq(&payload.person_ref))
        .filter(organization_memberships::Column::OrganizationRef.eq(&payload.organization_ref))
        .filter(organization_memberships::Column::Role.eq(&payload.role))
        .filter(organization_memberships::Column::DeletedAt.is_null())
        .count(&ctx.db)
        .await?;
    if existing > 0 {
        return Err(unprocessable(
            "this person already holds this role in this organization",
        ));
    }

    let txn = ctx.db.begin().await?;
    let row = organization_memberships::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        person_ref: ActiveValue::set(payload.person_ref),
        organization_ref: ActiveValue::set(payload.organization_ref),
        worker_pid: ActiveValue::set(payload.worker_pid),
        role: ActiveValue::set(payload.role),
        starts_on: ActiveValue::set(payload.starts_on),
        ends_on: ActiveValue::set(payload.ends_on),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "organization_membership",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    streaming::emit_on(
        &txn,
        "organization_membership",
        "granted",
        &row.pid.to_string(),
        &row.role,
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}

/// `DELETE /api/organization-memberships/{pid}` — revoke (soft
/// delete), mirroring `unenroll`'s shape exactly.
#[debug_handler]
async fn revoke(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = records::find_organization_membership(&ctx.db, records::parse_pid(&pid)?).await?;
    if !may_manage_membership(&ctx.db, &caller, &row.organization_ref, &row.role).await? {
        return Err(not_privileged());
    }
    let txn = ctx.db.begin().await?;
    let pid = row.pid;
    let mut active: organization_memberships::ActiveModel = row.into();
    active.deleted_at = ActiveValue::set(Some(chrono::Utc::now().into()));
    active.update(&txn).await?;
    Audit::record(
        &txn,
        "organization_membership",
        pid,
        "deleted",
        caller.actor(),
        None,
    )
    .await?;
    streaming::emit_on(
        &txn,
        "organization_membership",
        "revoked",
        &pid.to_string(),
        "",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::empty_json()
}

/// One confederation edge, as returned to a caller.
#[derive(Debug, Serialize)]
struct ConfederationView {
    pid: String,
    parent_organization_ref: String,
    child_organization_ref: String,
    starts_on: chrono::NaiveDate,
    ends_on: Option<chrono::NaiveDate>,
}

impl From<organization_confederations::Model> for ConfederationView {
    fn from(row: organization_confederations::Model) -> Self {
        Self {
            pid: row.pid.to_string(),
            parent_organization_ref: row.parent_organization_ref,
            child_organization_ref: row.child_organization_ref,
            starts_on: row.starts_on,
            ends_on: row.ends_on,
        }
    }
}

/// `GET /api/organization-confederations?parent_organization_ref=` or
/// `?child_organization_ref=` — the **direct** edges naming one side
/// (not the transitive descendant/ancestor set; that's
/// [`memberships::caller_scope_refs`]'s job for a caller, or
/// [`org_access::descendants_of`] for arbitrary code). At least one
/// filter is required.
#[derive(Debug, Deserialize)]
struct ConfederationListQuery {
    #[serde(default)]
    parent_organization_ref: Option<String>,
    #[serde(default)]
    child_organization_ref: Option<String>,
}

#[debug_handler]
async fn list_confederations(
    State(ctx): State<AppContext>,
    Query(query): Query<ConfederationListQuery>,
) -> Result<Response> {
    let mut find = organization_confederations::Entity::find()
        .filter(organization_confederations::Column::DeletedAt.is_null());
    match (
        query.parent_organization_ref.filter(|v| !v.trim().is_empty()),
        query.child_organization_ref.filter(|v| !v.trim().is_empty()),
    ) {
        (None, None) => {
            return Err(unprocessable(
                "parent_organization_ref or child_organization_ref is required",
            ));
        }
        (parent, child) => {
            if let Some(parent) = parent {
                find = find.filter(organization_confederations::Column::ParentOrganizationRef.eq(parent));
            }
            if let Some(child) = child {
                find = find.filter(organization_confederations::Column::ChildOrganizationRef.eq(child));
            }
        }
    }
    let rows = find
        .order_by_asc(organization_confederations::Column::Id)
        .all(&ctx.db)
        .await?;
    format::json(rows.into_iter().map(ConfederationView::from).collect::<Vec<_>>())
}

/// `POST /api/organization-confederations` body.
#[derive(Debug, Deserialize)]
struct GrantConfederationPayload {
    parent_organization_ref: String,
    child_organization_ref: String,
    starts_on: chrono::NaiveDate,
    #[serde(default)]
    ends_on: Option<chrono::NaiveDate>,
}

/// `POST /api/organization-confederations` — declare that
/// `parent_organization_ref` transitively contains
/// `child_organization_ref`. Rejects a self-loop, a duplicate live
/// edge, and any edge that would close a cycle
/// ([`org_access::would_create_cycle`], checked against every other
/// live edge — a confederation graph is a DAG, never a ring).
#[debug_handler]
async fn grant_confederation(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<GrantConfederationPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_ref(
        "parent_organization_ref",
        entity_ref::EntityType::Organization,
        &payload.parent_organization_ref,
    );
    problems.require_ref(
        "child_organization_ref",
        entity_ref::EntityType::Organization,
        &payload.child_organization_ref,
    );
    if payload.ends_on.is_some_and(|end| end < payload.starts_on) {
        problems.push("ends_on is before starts_on");
    }
    ensure_valid(&problems.into_vec())?;

    if !may_manage_confederation(&ctx.db, &caller, &payload.parent_organization_ref).await? {
        return Err(not_privileged_for_confederation());
    }

    let edges = confederations::live_edges(&ctx.db).await?;
    if org_access::would_create_cycle(
        &edges,
        &payload.parent_organization_ref,
        &payload.child_organization_ref,
    ) {
        return Err(unprocessable(
            "this edge would create a cycle in the confederation graph",
        ));
    }

    let existing = organization_confederations::Entity::find()
        .filter(
            organization_confederations::Column::ParentOrganizationRef
                .eq(&payload.parent_organization_ref),
        )
        .filter(
            organization_confederations::Column::ChildOrganizationRef
                .eq(&payload.child_organization_ref),
        )
        .filter(organization_confederations::Column::DeletedAt.is_null())
        .count(&ctx.db)
        .await?;
    if existing > 0 {
        return Err(unprocessable(
            "this parent already confederates this child organization",
        ));
    }

    let txn = ctx.db.begin().await?;
    let row = organization_confederations::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        parent_organization_ref: ActiveValue::set(payload.parent_organization_ref),
        child_organization_ref: ActiveValue::set(payload.child_organization_ref),
        starts_on: ActiveValue::set(payload.starts_on),
        ends_on: ActiveValue::set(payload.ends_on),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "organization_confederation",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    streaming::emit_on(
        &txn,
        "organization_confederation",
        "granted",
        &row.pid.to_string(),
        &row.child_organization_ref,
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}

/// `DELETE /api/organization-confederations/{pid}` — revoke (soft
/// delete) one confederation edge, mirroring [`revoke`]'s shape.
#[debug_handler]
async fn revoke_confederation(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = records::find_organization_confederation(&ctx.db, records::parse_pid(&pid)?).await?;
    if !may_manage_confederation(&ctx.db, &caller, &row.parent_organization_ref).await? {
        return Err(not_privileged_for_confederation());
    }
    let txn = ctx.db.begin().await?;
    let pid = row.pid;
    let mut active: organization_confederations::ActiveModel = row.into();
    active.deleted_at = ActiveValue::set(Some(chrono::Utc::now().into()));
    active.update(&txn).await?;
    Audit::record(
        &txn,
        "organization_confederation",
        pid,
        "deleted",
        caller.actor(),
        None,
    )
    .await?;
    streaming::emit_on(
        &txn,
        "organization_confederation",
        "revoked",
        &pid.to_string(),
        "",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::empty_json()
}

/// The organization-membership and org-confederation routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/me/organizations", get(list_my_organizations))
        .add("/me/organizations/scope", get(my_organization_scope))
        .add("/organization-memberships", get(list_memberships))
        .add("/organization-memberships", post(grant))
        .add("/organization-memberships/{pid}", delete(revoke))
        .add("/organization-confederations", get(list_confederations))
        .add("/organization-confederations", post(grant_confederation))
        .add(
            "/organization-confederations/{pid}",
            delete(revoke_confederation),
        )
}
