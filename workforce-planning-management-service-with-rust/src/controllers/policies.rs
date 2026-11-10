//! Policies and acknowledgements (WPM-R156, WPM-D105): versioned policies with an audience, held by reference (no text), and a
//! worker's dated acknowledgement of the newest version in force. A manager sees counts from the floor; HR sees the named record,
//! and that read is audited. An acknowledgement is the worker's own act: HR cannot make one for them. Spec:
//! `spec/communication-and-leadership.md`.

use std::collections::BTreeMap;

use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, forbidden, unprocessable, writable_worker};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{policies, policy_versions, workers};
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
    Ok(sqlx::query_as!(
        policy_versions::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, policy_pid, version, title,
                  document_ref, effective_on, audience_kind, audience_value, acknowledge_within_days, recorded_by
             FROM policy_versions
            WHERE policy_pid = $1
            ORDER BY version"#,
        policy,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// The organization's policies, by name.
async fn policies_of(ctx: &AppContext, organization: &str) -> Result<Vec<policies::Model>> {
    Ok(sqlx::query_as!(
        policies::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name, recorded_by
             FROM policies
            WHERE organization_ref = $1
            ORDER BY name"#,
        organization,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// The policy with this pid, wherever it is.
async fn policy_by_pid(ctx: &AppContext, pid: Uuid) -> Result<policies::Model> {
    sqlx::query_as!(
        policies::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name, recorded_by
             FROM policies WHERE pid = $1"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// The version with this pid.
async fn version_by_pid(ctx: &AppContext, pid: Uuid) -> Result<policy_versions::Model> {
    sqlx::query_as!(
        policy_versions::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, policy_pid, version, title,
                  document_ref, effective_on, audience_kind, audience_value, acknowledge_within_days, recorded_by
             FROM policy_versions WHERE pid = $1"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
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
    axa::extract::Query(query): axa::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let rows = policies_of(&ctx, &query.organization).await?;
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
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<PolicyPayload>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&payload.organization_ref))?;
    ensure_org_in_scope(&ctx, &caller, &payload.organization_ref).await?;
    let name = payload.name.trim().to_string();
    let n = name.chars().count();
    if n == 0 || n > 120 {
        return Err(unprocessable("name must be 1 to 120 characters"));
    }
    let clash = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!" FROM policies WHERE organization_ref = $1 AND name = $2"#,
        payload.organization_ref,
        name,
    )
    .fetch_one(db::pool(&ctx))
    .await?;
    if clash > 0 {
        return Err(unprocessable(
            "another policy in this organization has that name",
        ));
    }
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO policies (pid, organization_ref, name, recorded_by) VALUES ($1, $2, $3, $4)",
        pid,
        payload.organization_ref,
        name,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "policy",
        pid,
        "policy_created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

#[derive(Debug, Deserialize)]
struct RenamePayload {
    name: String,
}

/// `PATCH /api/policies/{pid}` — correct a policy's name. The name is unique within the organization; versions, their titles and every
/// acknowledgement are unchanged. Audited in the same transaction.
#[debug_handler]
async fn rename(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(payload): Json<RenamePayload>,
) -> Result<Response> {
    let policy = find_policy(&ctx, &caller, pid).await?;
    let name = payload.name.trim().to_string();
    let n = name.chars().count();
    if n == 0 || n > 120 {
        return Err(unprocessable("name must be 1 to 120 characters"));
    }
    let clash = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!" FROM policies WHERE organization_ref = $1 AND name = $2 AND pid <> $3"#,
        policy.organization_ref,
        name,
        pid,
    )
    .fetch_one(db::pool(&ctx))
    .await?;
    if clash > 0 {
        return Err(unprocessable(
            "another policy in this organization has that name",
        ));
    }
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE policies SET name = $2 WHERE id = $1",
        policy.id,
        name,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "policy",
        pid,
        "policy_renamed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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

async fn find_policy(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: Uuid,
) -> Result<policies::Model> {
    let row = policy_by_pid(ctx, pid).await?;
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
    let days = payload
        .acknowledge_within_days
        .unwrap_or(rules::DEFAULT_DAYS);
    ensure_valid(&rules::version_problems(
        &payload.title,
        &payload.document_ref,
        &payload.audience,
        payload.audience_value.as_deref(),
        days,
    ))?;
    let latest = versions_of(&ctx, pid)
        .await?
        .last()
        .map_or(0, |v| v.version);
    let version_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO policy_versions (pid, policy_pid, version, title, document_ref, effective_on, audience_kind,
                                      audience_value, acknowledge_within_days, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        version_pid,
        policy.pid,
        latest + 1,
        payload.title.trim(),
        payload.document_ref.trim(),
        payload.effective_on,
        payload.audience,
        payload
            .audience_value
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty()),
        days,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "policy_version",
        version_pid,
        "policy_version_added",
        caller.actor(),
        Some(json!({ "version": latest + 1 })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": version_pid, "version": latest + 1 }))
}

/// `GET /api/workers/{pid}/policies` — the newest version in force of each policy aimed at the worker, and where they stand on it.
/// The worker themself and HR.
#[debug_handler]
async fn of_worker(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    list_for(&ctx, &worker).await
}

/// What `worker` is asked to acknowledge and where they stand. Also served as `GET /api/me/policies`.
pub(crate) async fn list_for(ctx: &AppContext, worker: &workers::Model) -> Result<Response> {
    let today = Utc::now().date_naive();
    let acked: BTreeMap<Uuid, NaiveDate> = sqlx::query!(
        "SELECT version_pid, acknowledged_on FROM policy_acknowledgements WHERE worker_pid = $1",
        worker.pid,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
    .map(|a| (a.version_pid, a.acknowledged_on))
    .collect();
    let rows = policies_of(ctx, &worker.organization_ref).await?;
    let mut out = Vec::new();
    let mut waiting = 0_usize;
    for p in &rows {
        let versions = versions_of(ctx, p.pid).await?;
        let facts: Vec<Fact> = versions
            .iter()
            .map(|v| Fact {
                version: v.version,
                effective_on: v.effective_on,
            })
            .collect();
        let Some(i) = rules::current(&facts, today) else {
            continue;
        };
        let v = &versions[i];
        if !rules::in_audience(
            &v.audience_kind,
            v.audience_value.as_deref(),
            &worker.department,
            &worker.job_title,
        ) {
            continue;
        }
        let standing = rules::standing(
            v.effective_on,
            v.acknowledge_within_days,
            acked.get(&v.pid).copied(),
            today,
        );
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
    let v = version_by_pid(ctx, version_pid).await?;
    let policy = policy_by_pid(ctx, v.policy_pid).await?;
    if policy.organization_ref != worker.organization_ref
        || !rules::in_audience(
            &v.audience_kind,
            v.audience_value.as_deref(),
            &worker.department,
            &worker.job_title,
        )
    {
        return Err(Error::NotFound);
    }
    let versions = versions_of(ctx, policy.pid).await?;
    let facts: Vec<Fact> = versions
        .iter()
        .map(|x| Fact {
            version: x.version,
            effective_on: x.effective_on,
        })
        .collect();
    let newest = rules::current(&facts, today).map(|i| versions[i].pid);
    if newest != Some(v.pid) {
        return Err(unprocessable(
            "only the newest version in force can be acknowledged",
        ));
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
    let existing = sqlx::query_scalar!(
        "SELECT acknowledged_on FROM policy_acknowledgements WHERE version_pid = $1 AND worker_pid = $2 LIMIT 1",
        version.pid,
        worker.pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?;
    if let Some(acknowledged_on) = existing {
        return format::json(json!({ "acknowledged_on": acknowledged_on }));
    }
    let ack_pid = Uuid::new_v4();
    let mut tx = db::pool(ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO policy_acknowledgements (pid, version_pid, worker_pid, acknowledged_on) VALUES ($1, $2, $3, $4)",
        ack_pid,
        version.pid,
        worker.pid,
        today,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "policy_acknowledgement",
        ack_pid,
        "policy_acknowledged",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "acknowledged_on": today }))
}

#[derive(Debug, Deserialize)]
pub(crate) struct CountQuery {
    version_pid: Uuid,
}

/// Whether the caller may see more than their own: HR, payroll, service and administrators, or anyone when sign-in is off.
pub(crate) async fn privileged(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    organization: &str,
) -> Result<bool> {
    if !auth::require_auth() {
        return Ok(true);
    }
    let Some(claims) = caller.claims() else {
        return Ok(false);
    };
    Ok(crate::need_to_know::has_privileged_attrs(claims)
        || crate::need_to_know::privileged_by_membership(ctx, claims, Some(organization)).await?)
}

async fn audience_workers(
    ctx: &AppContext,
    version: &policy_versions::Model,
    organization: &str,
    on: NaiveDate,
) -> Result<Vec<workers::Model>> {
    Ok(sqlx::query_as!(
        workers::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, person_ref,
                  upstream_worker_ref, organization_ref, worker_number, display_name, status, employment_type,
                  fte_percent, department, job_title, location, work_jurisdiction, manager_pid, salary_minor,
                  salary_currency, hired_on, terminated_on, engagement_ends_on, deleted_at AS "deleted_at: _"
             FROM workers WHERE organization_ref = $1 AND deleted_at IS NULL
            ORDER BY id"#,
        organization,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
        .filter(|w| is_employed_on(on, w.hired_on, w.terminated_on))
        .filter(|w| {
            rules::in_audience(
                &version.audience_kind,
                version.audience_value.as_deref(),
                &w.department,
                &w.job_title,
            )
        })
        .collect())
}

/// The version and its policy's organization. `scoped` applies the caller's organization scope; a manager reading their own team's
/// counts has none to apply (a plain worker holds no membership), and the version must be of their own organization instead.
async fn version_and_org(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: Uuid,
    scoped: bool,
) -> Result<(policy_versions::Model, String)> {
    let v = version_by_pid(ctx, pid).await?;
    if scoped {
        let policy = find_policy(ctx, caller, v.policy_pid).await?;
        return Ok((v, policy.organization_ref));
    }
    let policy = policy_by_pid(ctx, v.policy_pid).await?;
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
    axa::extract::Query(query): axa::extract::Query<CountQuery>,
) -> Result<Response> {
    let manager = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let someone_else = auth::acting_for_other(&caller, &manager.person_ref);
    let (version, organization) =
        version_and_org(&ctx, &caller, query.version_pid, someone_else).await?;
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
    let acknowledged = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!" FROM policy_acknowledgements WHERE version_pid = $1 AND worker_pid = ANY($2)"#,
        version.pid,
        &team,
    )
    .fetch_one(db::pool(&ctx))
    .await?;
    let count = rules::team_count(
        team.len(),
        usize::try_from(acknowledged).unwrap_or(0),
        DEFAULT_FLOOR,
    );
    format::json(json!({ "version_pid": version.pid, "floor": DEFAULT_FLOOR, "count": count }))
}

/// `GET /api/policy-versions/{pid}/acknowledgements` — the named record: everyone in the audience, with the day they acknowledged
/// or null. HR and the like only; the read is audited.
#[debug_handler]
async fn named_record(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let (version, organization) = version_and_org(&ctx, &caller, pid, true).await?;
    if !privileged(&ctx, &caller, &organization).await? {
        return Err(forbidden("the named record is for HR"));
    }
    let people = audience_workers(&ctx, &version, &organization, Utc::now().date_naive()).await?;
    let acked: BTreeMap<Uuid, NaiveDate> = sqlx::query!(
        "SELECT worker_pid, acknowledged_on FROM policy_acknowledgements WHERE version_pid = $1",
        version.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|a| (a.worker_pid, a.acknowledged_on))
    .collect();
    db::audit(
        &mut *db::pool(&ctx).acquire().await?,
        "policy_version",
        version.pid,
        "policy_acknowledgements_read",
        caller.actor(),
        Some(json!({ "audience": people.len() })),
    )
    .await?;
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
