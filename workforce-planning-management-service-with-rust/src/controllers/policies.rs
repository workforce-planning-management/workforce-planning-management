//! Policies and acknowledgements (WPM-R156, WPM-D105): versioned policies with an audience, held by reference (no text), and a
//! worker's dated acknowledgement of the newest version in force. A manager sees counts from the floor; HR sees the named record,
//! and that read is audited. An acknowledgement is the worker's own act: HR cannot make one for them. Spec:
//! `spec/communication-and-leadership.md`.

use std::collections::BTreeMap;

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, forbidden, unprocessable, writable_worker};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{policies, policy_acknowledgements, policy_versions, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::equality_monitoring::DEFAULT_FLOOR;
use crate::rules::metrics::is_employed_on;
use crate::rules::policy::{self as rules, Fact};

fn version_json(v: &policy_versions::Model) -> serde_json::Value {
    json!({
        "pid": v.pid, "version": v.version, "title": v.title, "document_ref": v.document_ref,
        "effective_on": v.effective_on, "audience": v.audience_kind, "audience_value": v.audience_value,
        "acknowledge_within_days": v.acknowledge_within_days,
    })
}

async fn versions_of(ctx: &AppContext, policy: Uuid) -> Result<Vec<policy_versions::Model>> {
    Ok(policy_versions::Entity::find()
        .filter(policy_versions::Column::PolicyPid.eq(policy))
        .order_by_asc(policy_versions::Column::Version)
        .all(&ctx.db)
        .await?)
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListQuery {
    organization: String,
}

/// `GET /api/policies?organization=` — the policies and their versions. Any signed-in caller of the organization.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let rows = policies::Entity::find()
        .filter(policies::Column::OrganizationRef.eq(&query.organization))
        .order_by_asc(policies::Column::Name)
        .all(&ctx.db)
        .await?;
    let mut out = Vec::new();
    for p in &rows {
        let versions = versions_of(&ctx, p.pid).await?;
        out.push(json!({ "pid": p.pid, "name": p.name, "versions": versions.iter().map(version_json).collect::<Vec<_>>() }));
    }
    format::json(json!({ "policies": out }))
}

#[derive(Debug, Deserialize)]
struct PolicyPayload {
    organization_ref: String,
    name: String,
}

/// `POST /api/policies` — name a policy. A name is unique within the organization.
#[debug_handler]
async fn create(State(ctx): State<AppContext>, caller: MaybeAuthUser, Json(payload): Json<PolicyPayload>) -> Result<Response> {
    ensure_valid(&organization_problems(&payload.organization_ref))?;
    ensure_org_in_scope(&ctx, &caller, &payload.organization_ref).await?;
    let name = payload.name.trim().to_string();
    let n = name.chars().count();
    if n == 0 || n > 120 {
        return Err(unprocessable("name must be 1 to 120 characters"));
    }
    let clash = policies::Entity::find()
        .filter(policies::Column::OrganizationRef.eq(&payload.organization_ref))
        .filter(policies::Column::Name.eq(&name))
        .count(&ctx.db)
        .await?;
    if clash > 0 {
        return Err(unprocessable("another policy in this organization has that name"));
    }
    let pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    policies::ActiveModel {
        pid: ActiveValue::set(pid),
        organization_ref: ActiveValue::set(payload.organization_ref),
        name: ActiveValue::set(name),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(&txn, "policy", pid, "policy_created", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

#[derive(Debug, Deserialize)]
struct RenamePayload {
    name: String,
}

/// `PATCH /api/policies/{pid}` — correct a policy's name. The name is unique within the organization; versions, their titles and every
/// acknowledgement are unchanged. Audited in the same transaction.
#[debug_handler]
async fn rename(State(ctx): State<AppContext>, caller: MaybeAuthUser, Path(pid): Path<Uuid>, Json(payload): Json<RenamePayload>) -> Result<Response> {
    let policy = find_policy(&ctx, &caller, pid).await?;
    let name = payload.name.trim().to_string();
    let n = name.chars().count();
    if n == 0 || n > 120 {
        return Err(unprocessable("name must be 1 to 120 characters"));
    }
    let clash = policies::Entity::find()
        .filter(policies::Column::OrganizationRef.eq(&policy.organization_ref))
        .filter(policies::Column::Name.eq(&name))
        .filter(policies::Column::Pid.ne(pid))
        .count(&ctx.db)
        .await?;
    if clash > 0 {
        return Err(unprocessable("another policy in this organization has that name"));
    }
    let txn = ctx.db.begin().await?;
    let mut active: policies::ActiveModel = policy.into();
    active.name = ActiveValue::set(name.clone());
    active.update(&txn).await?;
    Audit::record(&txn, "policy", pid, "policy_renamed", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid, "name": name }))
}

#[derive(Debug, Deserialize)]
struct VersionPayload {
    title: String,
    document_ref: String,
    effective_on: NaiveDate,
    audience: String,
    audience_value: Option<String>,
    acknowledge_within_days: Option<i32>,
}

async fn find_policy(ctx: &AppContext, caller: &MaybeAuthUser, pid: Uuid) -> Result<policies::Model> {
    let row = policies::Entity::find()
        .filter(policies::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &row.organization_ref).await?;
    Ok(row)
}

/// `POST /api/policies/{pid}/versions` — add the next version. A version is never edited: a change is a new version, which every
/// worker in its audience must acknowledge again.
#[debug_handler]
async fn add_version(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(payload): Json<VersionPayload>,
) -> Result<Response> {
    let policy = find_policy(&ctx, &caller, pid).await?;
    let days = payload.acknowledge_within_days.unwrap_or(rules::DEFAULT_DAYS);
    ensure_valid(&rules::version_problems(
        &payload.title,
        &payload.document_ref,
        &payload.audience,
        payload.audience_value.as_deref(),
        days,
    ))?;
    let latest = versions_of(&ctx, pid).await?.last().map_or(0, |v| v.version);
    let version_pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    policy_versions::ActiveModel {
        pid: ActiveValue::set(version_pid),
        policy_pid: ActiveValue::set(policy.pid),
        version: ActiveValue::set(latest + 1),
        title: ActiveValue::set(payload.title.trim().to_string()),
        document_ref: ActiveValue::set(payload.document_ref.trim().to_string()),
        effective_on: ActiveValue::set(payload.effective_on),
        audience_kind: ActiveValue::set(payload.audience.clone()),
        audience_value: ActiveValue::set(payload.audience_value.as_ref().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())),
        acknowledge_within_days: ActiveValue::set(days),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(&txn, "policy_version", version_pid, "policy_version_added", caller.actor(), Some(json!({ "version": latest + 1 }))).await?;
    txn.commit().await?;
    format::json(json!({ "pid": version_pid, "version": latest + 1 }))
}

/// `GET /api/workers/{pid}/policies` — the newest version in force of each policy aimed at the worker, and where they stand on it.
/// The worker themself and HR.
#[debug_handler]
async fn of_worker(State(ctx): State<AppContext>, caller: MaybeAuthUser, Path(pid): Path<String>) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    list_for(&ctx, &worker).await
}

/// What `worker` is asked to acknowledge and where they stand. Also served as `GET /api/me/policies`.
pub(crate) async fn list_for(ctx: &AppContext, worker: &workers::Model) -> Result<Response> {
    let today = Utc::now().date_naive();
    let acked: BTreeMap<Uuid, NaiveDate> = policy_acknowledgements::Entity::find()
        .filter(policy_acknowledgements::Column::WorkerPid.eq(worker.pid))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|a| (a.version_pid, a.acknowledged_on))
        .collect();
    let rows = policies::Entity::find()
        .filter(policies::Column::OrganizationRef.eq(&worker.organization_ref))
        .order_by_asc(policies::Column::Name)
        .all(&ctx.db)
        .await?;
    let mut out = Vec::new();
    let mut waiting = 0_usize;
    for p in &rows {
        let versions = versions_of(ctx, p.pid).await?;
        let facts: Vec<Fact> = versions.iter().map(|v| Fact { version: v.version, effective_on: v.effective_on }).collect();
        let Some(i) = rules::current(&facts, today) else { continue };
        let v = &versions[i];
        if !rules::in_audience(&v.audience_kind, v.audience_value.as_deref(), &worker.department, &worker.job_title) {
            continue;
        }
        let standing = rules::standing(v.effective_on, v.acknowledge_within_days, acked.get(&v.pid).copied(), today);
        if !matches!(standing, rules::Standing::Done { .. }) {
            waiting += 1;
        }
        out.push(json!({ "policy": p.name, "version": version_json(v), "standing": standing }));
    }
    format::json(json!({ "worker_pid": worker.pid, "waiting": waiting, "policies": out }))
}

#[derive(Debug, Deserialize)]
pub(crate) struct AckPayload {
    pub(crate) version_pid: Uuid,
}

/// The version, its policy, and a check that the worker's organization is the policy's and the version is in force and aimed at them.
async fn applicable(
    ctx: &AppContext,
    worker: &workers::Model,
    version_pid: Uuid,
    today: NaiveDate,
) -> Result<policy_versions::Model> {
    let v = policy_versions::Entity::find()
        .filter(policy_versions::Column::Pid.eq(version_pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let policy = policies::Entity::find()
        .filter(policies::Column::Pid.eq(v.policy_pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    if policy.organization_ref != worker.organization_ref
        || !rules::in_audience(&v.audience_kind, v.audience_value.as_deref(), &worker.department, &worker.job_title)
    {
        return Err(Error::NotFound);
    }
    let versions = versions_of(ctx, policy.pid).await?;
    let facts: Vec<Fact> = versions.iter().map(|x| Fact { version: x.version, effective_on: x.effective_on }).collect();
    let newest = rules::current(&facts, today).map(|i| versions[i].pid);
    if newest != Some(v.pid) {
        return Err(unprocessable("only the newest version in force can be acknowledged"));
    }
    Ok(v)
}

/// The worker says they have read the newest version in force aimed at them (`POST /api/me/policy-acknowledgements`). **Their own
/// act**: the route is under `/api/me`, which resolves the worker from the caller's token, so no one else, HR included, can make
/// it for them. Idempotent: the first day is kept.
pub(crate) async fn acknowledge_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
    payload: AckPayload,
) -> Result<Response> {
    let today = Utc::now().date_naive();
    let version = applicable(ctx, worker, payload.version_pid, today).await?;
    let existing = policy_acknowledgements::Entity::find()
        .filter(policy_acknowledgements::Column::VersionPid.eq(version.pid))
        .filter(policy_acknowledgements::Column::WorkerPid.eq(worker.pid))
        .one(&ctx.db)
        .await?;
    if let Some(row) = existing {
        return format::json(json!({ "acknowledged_on": row.acknowledged_on }));
    }
    let ack_pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    policy_acknowledgements::ActiveModel {
        pid: ActiveValue::set(ack_pid),
        version_pid: ActiveValue::set(version.pid),
        worker_pid: ActiveValue::set(worker.pid),
        acknowledged_on: ActiveValue::set(today),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(&txn, "policy_acknowledgement", ack_pid, "policy_acknowledged", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(json!({ "acknowledged_on": today }))
}

#[derive(Debug, Deserialize)]
pub(crate) struct CountQuery {
    version_pid: Uuid,
}

/// Whether the caller may see more than their own: HR, payroll, service and administrators, or anyone when sign-in is off.
async fn privileged(ctx: &AppContext, caller: &MaybeAuthUser, organization: &str) -> Result<bool> {
    if !auth::require_auth() {
        return Ok(true);
    }
    let Some(claims) = caller.claims() else { return Ok(false) };
    Ok(crate::need_to_know::has_privileged_attrs(claims)
        || crate::need_to_know::privileged_by_membership(ctx, claims, Some(organization)).await?)
}

async fn audience_workers(ctx: &AppContext, version: &policy_versions::Model, organization: &str, on: NaiveDate) -> Result<Vec<workers::Model>> {
    Ok(workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(organization))
        .filter(workers::Column::DeletedAt.is_null())
        .order_by_asc(workers::Column::Id)
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| is_employed_on(on, w.hired_on, w.terminated_on))
        .filter(|w| rules::in_audience(&version.audience_kind, version.audience_value.as_deref(), &w.department, &w.job_title))
        .collect())
}

/// The version and its policy's organization. `scoped` applies the caller's organization scope; a manager reading their own team's
/// counts has none to apply (a plain worker holds no membership), and the version must be of their own organization instead.
async fn version_and_org(ctx: &AppContext, caller: &MaybeAuthUser, pid: Uuid, scoped: bool) -> Result<(policy_versions::Model, String)> {
    let v = policy_versions::Entity::find()
        .filter(policy_versions::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    if scoped {
        let policy = find_policy(ctx, caller, v.policy_pid).await?;
        return Ok((v, policy.organization_ref));
    }
    let policy = policies::Entity::find()
        .filter(policies::Column::Pid.eq(v.policy_pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    Ok((v, policy.organization_ref))
}

/// `GET /api/workers/{pid}/team-policy-counts?version_pid=` — for a manager: how many of their direct reports in the version's
/// audience have acknowledged and how many have not, from the floor; under it, nothing. Read as the manager `{pid}`; anyone else
/// needs HR's access.
#[debug_handler]
async fn team_counts(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axum::extract::Query(query): axum::extract::Query<CountQuery>,
) -> Result<Response> {
    let manager = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let someone_else = auth::acting_for_other(&caller, &manager.person_ref);
    let (version, organization) = version_and_org(&ctx, &caller, query.version_pid, someone_else).await?;
    if organization != manager.organization_ref {
        return Err(Error::NotFound);
    }
    if someone_else && !privileged(&ctx, &caller, &organization).await? {
        return Err(forbidden("a manager's team counts are for that manager"));
    }
    let team: Vec<Uuid> = audience_workers(&ctx, &version, &organization, Utc::now().date_naive())
        .await?
        .into_iter()
        .filter(|w| w.manager_pid == Some(manager.pid))
        .map(|w| w.pid)
        .collect();
    let acknowledged = policy_acknowledgements::Entity::find()
        .filter(policy_acknowledgements::Column::VersionPid.eq(version.pid))
        .filter(policy_acknowledgements::Column::WorkerPid.is_in(team.clone()))
        .count(&ctx.db)
        .await?;
    let count = rules::team_count(team.len(), usize::try_from(acknowledged).unwrap_or(0), DEFAULT_FLOOR);
    format::json(json!({ "version_pid": version.pid, "floor": DEFAULT_FLOOR, "count": count }))
}

/// `GET /api/policy-versions/{pid}/acknowledgements` — the named record: everyone in the audience, with the day they acknowledged
/// or null. HR and the like only; the read is audited.
#[debug_handler]
async fn named_record(State(ctx): State<AppContext>, caller: MaybeAuthUser, Path(pid): Path<Uuid>) -> Result<Response> {
    let (version, organization) = version_and_org(&ctx, &caller, pid, true).await?;
    if !privileged(&ctx, &caller, &organization).await? {
        return Err(forbidden("the named record is for HR"));
    }
    let people = audience_workers(&ctx, &version, &organization, Utc::now().date_naive()).await?;
    let acked: BTreeMap<Uuid, NaiveDate> = policy_acknowledgements::Entity::find()
        .filter(policy_acknowledgements::Column::VersionPid.eq(version.pid))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|a| (a.worker_pid, a.acknowledged_on))
        .collect();
    Audit::record(&ctx.db, "policy_version", version.pid, "policy_acknowledgements_read", caller.actor(), Some(json!({ "audience": people.len() }))).await?;
    let rows: Vec<_> = people
        .iter()
        .map(|w| json!({ "worker_pid": w.pid, "worker_number": w.worker_number, "acknowledged_on": acked.get(&w.pid) }))
        .collect();
    format::json(json!({ "version": version_json(&version), "people": rows }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/policies", get(list))
        .add("/policies", post(create))
        .add("/policies/{pid}", patch(rename))
        .add("/policies/{pid}/versions", post(add_version))
        .add("/workers/{pid}/policies", get(of_worker))
        .add("/workers/{pid}/team-policy-counts", get(team_counts))
        .add("/policy-versions/{pid}/acknowledgements", get(named_record))
}
