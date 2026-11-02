//! Qualifications and the qualification gap (WPM-R171, WPM-T316). A person (or HR) adds a qualification, which is `declared`; HR marks
//! it `verified` with an evidence reference, since no certificate is stored. A role profile may require qualifications, and a person's
//! list shows how each requirement of their current role stands: met only by a verified match in date. No role recorded reads
//! `role: null`, never an empty gap. The arithmetic is `rules::qualification`. Spec: `spec/staff-records-and-pay.md`.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use super::mandatory_learning::ensure_hr;
use super::{ensure_valid, writable_worker};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    qualifications, role_profiles, role_qualification_requirements, worker_framework_roles, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::qualification as rules;

fn trimmed(s: Option<&str>) -> Option<String> {
    s.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

fn held_of(q: &qualifications::Model) -> rules::Held {
    rules::Held {
        title: q.title.clone(),
        kind: q.kind.clone(),
        framework: q.framework.clone(),
        level: q.level,
        expires_on: q.expires_on,
        verified: q.verified_on.is_some(),
    }
}

fn row(q: &qualifications::Model, today: NaiveDate) -> Value {
    json!({
        "pid": q.pid, "kind": q.kind, "title": q.title, "awarding_body": q.awarding_body,
        "framework": q.framework, "level": q.level, "awarded_on": q.awarded_on, "expires_on": q.expires_on,
        "status": if q.verified_on.is_some() { "verified" } else { "declared" },
        "expired": q.expires_on.is_some_and(|e| e < today),
        "verified_on": q.verified_on, "evidence_ref": q.evidence_ref,
    })
}

fn standing_name(s: rules::Standing) -> &'static str {
    match s {
        rules::Standing::Met => "met",
        rules::Standing::DeclaredOnly => "declared_only",
        rules::Standing::Expired => "expired",
        rules::Standing::Missing => "missing",
    }
}

#[derive(Debug, Deserialize)]
struct AddPayload {
    kind: String,
    title: String,
    awarding_body: Option<String>,
    framework: Option<String>,
    level: Option<i32>,
    awarded_on: Option<NaiveDate>,
    expires_on: Option<NaiveDate>,
}

/// `POST /api/workers/{pid}/qualifications` — the person or HR adds a qualification; it is `declared`.
#[debug_handler]
async fn add(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(p): Json<AddPayload>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let title = p.title.trim().to_string();
    let body = trimmed(p.awarding_body.as_deref());
    let framework = trimmed(p.framework.as_deref());
    let mut problems = Vec::new();
    if !rules::KINDS.contains(&p.kind.as_str()) {
        problems.push(format!("kind must be one of {}", rules::KINDS.join(", ")));
    }
    if title.is_empty() || title.chars().count() > 200 {
        problems.push("title must be 1 to 200 characters".to_string());
    }
    if body.as_ref().is_some_and(|b| b.chars().count() > 200) {
        problems.push("awarding_body must be at most 200 characters".to_string());
    }
    if framework.as_ref().is_some_and(|f| f.chars().count() > 40) {
        problems.push("framework must be at most 40 characters".to_string());
    }
    if framework.is_some() != p.level.is_some() {
        problems.push("framework and level go together".to_string());
    }
    if p.level.is_some_and(|l| !(0..=20).contains(&l)) {
        problems.push("level must be 0 to 20".to_string());
    }
    if let (Some(a), Some(e)) = (p.awarded_on, p.expires_on)
        && e < a
    {
        problems.push("expires_on must not be before awarded_on".to_string());
    }
    if p.awarded_on.is_some_and(|a| a > Utc::now().date_naive()) {
        problems.push("awarded_on must not be in the future".to_string());
    }
    ensure_valid(&problems)?;
    let id = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    qualifications::ActiveModel {
        pid: ActiveValue::set(id),
        worker_pid: ActiveValue::set(worker.pid),
        kind: ActiveValue::set(p.kind),
        title: ActiveValue::set(title),
        awarding_body: ActiveValue::set(body),
        framework: ActiveValue::set(framework),
        level: ActiveValue::set(p.level),
        awarded_on: ActiveValue::set(p.awarded_on),
        expires_on: ActiveValue::set(p.expires_on),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "qualification",
        id,
        "qualification_declared",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": id }))
}

async fn find(ctx: &AppContext, pid: Uuid) -> Result<qualifications::Model> {
    qualifications::Entity::find()
        .filter(qualifications::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

#[derive(Debug, Deserialize)]
struct VerifyPayload {
    evidence_ref: String,
}

/// `POST /api/qualifications/{pid}/verify` — HR marks it verified with a reference to where the proof is held. Verifying twice keeps
/// the first verification.
#[debug_handler]
async fn verify(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(p): Json<VerifyPayload>,
) -> Result<Response> {
    let q = find(&ctx, pid).await?;
    let worker = records::find_worker(&ctx.db, q.worker_pid).await?;
    ensure_hr(&ctx, &caller, &worker.organization_ref).await?;
    let evidence = p.evidence_ref.trim().to_string();
    ensure_valid(
        &(if evidence.is_empty() || evidence.chars().count() > 200 {
            vec!["evidence_ref must be 1 to 200 characters".to_string()]
        } else {
            Vec::new()
        }),
    )?;
    if q.verified_on.is_none() {
        let txn = ctx.db.begin().await?;
        let mut m: qualifications::ActiveModel = q.into();
        m.verified_on = ActiveValue::set(Some(Utc::now().date_naive()));
        m.verified_by = ActiveValue::set(caller.actor().map(ToString::to_string));
        m.evidence_ref = ActiveValue::set(Some(evidence));
        m.update(&txn).await?;
        Audit::record(
            &txn,
            "qualification",
            pid,
            "qualification_verified",
            caller.actor(),
            None,
        )
        .await?;
        txn.commit().await?;
    }
    format::json(json!({ "pid": pid, "status": "verified" }))
}

/// `DELETE /api/qualifications/{pid}` — the person or HR removes a qualification.
#[debug_handler]
async fn remove(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let q = find(&ctx, pid).await?;
    let worker = records::find_worker(&ctx.db, q.worker_pid).await?;
    writable_worker(&ctx, &caller, &worker.pid.to_string()).await?;
    let txn = ctx.db.begin().await?;
    qualifications::Entity::delete_many()
        .filter(qualifications::Column::Pid.eq(pid))
        .exec(&txn)
        .await?;
    Audit::record(
        &txn,
        "qualification",
        pid,
        "qualification_removed",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// The worker's current role profile, if one is recorded.
async fn current_role(
    ctx: &AppContext,
    worker: &workers::Model,
) -> Result<Option<role_profiles::Model>> {
    let Some(link) = worker_framework_roles::Entity::find()
        .filter(worker_framework_roles::Column::WorkerPid.eq(worker.pid))
        .filter(worker_framework_roles::Column::EndedAt.is_null())
        .filter(worker_framework_roles::Column::RoleProfilePid.is_not_null())
        .order_by_desc(worker_framework_roles::Column::StartedAt)
        .one(&ctx.db)
        .await?
    else {
        return Ok(None);
    };
    let Some(profile_pid) = link.role_profile_pid else {
        return Ok(None);
    };
    Ok(role_profiles::Entity::find()
        .filter(role_profiles::Column::Pid.eq(profile_pid))
        .filter(role_profiles::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?)
}

/// The worker's current role and how each of its requirements stands for `held`. No role recorded is `Null`, not a clean gap.
async fn role_and_gap(
    ctx: &AppContext,
    worker: &workers::Model,
    held: &[rules::Held],
    today: NaiveDate,
) -> Result<(Value, Vec<Value>)> {
    let Some(profile) = current_role(ctx, worker).await? else {
        return Ok((Value::Null, Vec::new()));
    };
    let reqs = role_qualification_requirements::Entity::find()
        .filter(role_qualification_requirements::Column::RoleProfilePid.eq(profile.pid))
        .order_by_asc(role_qualification_requirements::Column::Title)
        .all(&ctx.db)
        .await?;
    let gap = reqs
        .iter()
        .map(|r| {
            let req = rules::Requirement {
                title: r.title.clone(),
                kind: r.kind.clone(),
                framework: r.framework.clone(),
                min_level: r.min_level,
            };
            json!({
                "title": r.title, "kind": r.kind, "framework": r.framework, "min_level": r.min_level,
                "standing": standing_name(rules::standing(&req, held, today)),
            })
        })
        .collect();
    Ok((
        json!({ "pid": profile.pid, "job_title": profile.job_title }),
        gap,
    ))
}

/// `GET /api/workers/{pid}/team-qualifications` — for a manager: each person below them, by name, with their qualifications and the gap
/// against their current role. Read as the manager `{pid}`; anyone else needs HR's access (WPM-R171: a work record, so a manager sees
/// their reports by name). Only people in the manager's own organization; no role recorded is `role: null`, never a clean gap.
#[debug_handler]
async fn team(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let manager = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    super::ensure_org_in_scope(&ctx, &caller, &manager.organization_ref).await?;
    if crate::auth::acting_for_other(&caller, &manager.person_ref)
        && !super::policies::privileged(&ctx, &caller, &manager.organization_ref).await?
    {
        return Err(super::forbidden(
            "a team's qualifications are for that team's manager",
        ));
    }
    let today = Utc::now().date_naive();
    let staff = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&manager.organization_ref))
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::TerminatedOn.is_null());
    super::ensure_scale(&ctx, staff.clone()).await?;
    let staff = staff.all(&ctx.db).await?;
    let mut reports_of: std::collections::BTreeMap<Uuid, Vec<Uuid>> =
        std::collections::BTreeMap::new();
    for w in &staff {
        if let Some(boss) = w.manager_pid {
            reports_of.entry(boss).or_default().push(w.pid);
        }
    }
    let below = crate::rules::org::downline(manager.pid, &reports_of);
    let by_pid: std::collections::HashMap<Uuid, &workers::Model> =
        staff.iter().map(|w| (w.pid, w)).collect();
    let pids: Vec<Uuid> = below.iter().map(|(p, _)| *p).collect();
    let all = qualifications::Entity::find()
        .filter(qualifications::Column::WorkerPid.is_in(pids))
        .order_by_asc(qualifications::Column::Title)
        .all(&ctx.db)
        .await?;
    let mut people: Vec<Value> = Vec::new();
    for (p, depth) in &below {
        let Some(w) = by_pid.get(p) else { continue };
        let mine: Vec<&qualifications::Model> =
            all.iter().filter(|q| q.worker_pid == w.pid).collect();
        let held: Vec<rules::Held> = mine.iter().map(|q| held_of(q)).collect();
        let (role, gap) = role_and_gap(&ctx, w, &held, today).await?;
        people.push(json!({
            "worker_pid": w.pid, "display_name": w.display_name,
            "report_kind": crate::rules::org::report_kind(*depth).map(crate::rules::org::ReportKind::as_str),
            "qualifications": mine.iter().map(|q| row(q, today)).collect::<Vec<_>>(),
            "role": role, "gap": gap,
        }));
    }
    format::json(json!({ "manager_pid": manager.pid, "as_of": today, "people": people }))
}

/// `GET /api/workers/{pid}/qualifications` — the person's qualifications and how each requirement of their current role stands:
/// the person themself and HR. No role recorded is `role: null`, not a clean gap.
#[debug_handler]
async fn of_worker(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let today = Utc::now().date_naive();
    let mine = qualifications::Entity::find()
        .filter(qualifications::Column::WorkerPid.eq(worker.pid))
        .order_by_asc(qualifications::Column::Title)
        .all(&ctx.db)
        .await?;
    let held: Vec<rules::Held> = mine.iter().map(held_of).collect();
    let (role, gap) = role_and_gap(&ctx, &worker, &held, today).await?;
    format::json(json!({
        "worker_pid": worker.pid, "as_of": today,
        "qualifications": mine.iter().map(|q| row(q, today)).collect::<Vec<_>>(),
        "role": role, "gap": gap,
    }))
}

#[derive(Debug, Deserialize)]
struct RequirementPayload {
    title: String,
    kind: Option<String>,
    framework: Option<String>,
    min_level: Option<i32>,
}

async fn profile_pid(ctx: &AppContext, pid: &str) -> Result<Uuid> {
    let profile = role_profiles::Entity::find()
        .filter(role_profiles::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(role_profiles::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    Ok(profile.pid)
}

/// `GET /api/role-profiles/{pid}/qualification-requirements` — what a role asks for. Policy, not a person's record.
#[debug_handler]
async fn list_requirements(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let profile = profile_pid(&ctx, &pid).await?;
    let rows: Vec<Value> = role_qualification_requirements::Entity::find()
        .filter(role_qualification_requirements::Column::RoleProfilePid.eq(profile))
        .order_by_asc(role_qualification_requirements::Column::Title)
        .all(&ctx.db)
        .await?
        .iter()
        .map(|r| {
            json!({ "pid": r.pid, "title": r.title, "kind": r.kind, "framework": r.framework, "min_level": r.min_level })
        })
        .collect();
    format::json(json!({ "role_profile_pid": profile, "requirements": rows }))
}

/// `POST /api/role-profiles/{pid}/qualification-requirements` — add a requirement to a role profile (as the skill requirements are).
#[debug_handler]
async fn add_requirement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(p): Json<RequirementPayload>,
) -> Result<Response> {
    let profile = profile_pid(&ctx, &pid).await?;
    let title = p.title.trim().to_string();
    let framework = trimmed(p.framework.as_deref());
    let mut problems = Vec::new();
    if title.is_empty() || title.chars().count() > 200 {
        problems.push("title must be 1 to 200 characters".to_string());
    }
    if p.kind
        .as_ref()
        .is_some_and(|k| !rules::KINDS.contains(&k.as_str()))
    {
        problems.push(format!("kind must be one of {}", rules::KINDS.join(", ")));
    }
    if framework.is_some() != p.min_level.is_some() {
        problems.push("framework and min_level go together".to_string());
    }
    if framework.as_ref().is_some_and(|f| f.chars().count() > 40) {
        problems.push("framework must be at most 40 characters".to_string());
    }
    if p.min_level.is_some_and(|l| !(0..=20).contains(&l)) {
        problems.push("min_level must be 0 to 20".to_string());
    }
    ensure_valid(&problems)?;
    let id = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    role_qualification_requirements::ActiveModel {
        pid: ActiveValue::set(id),
        role_profile_pid: ActiveValue::set(profile),
        title: ActiveValue::set(title),
        kind: ActiveValue::set(p.kind),
        framework: ActiveValue::set(framework),
        min_level: ActiveValue::set(p.min_level),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "role_qualification_requirement",
        id,
        "role_qualification_requirement_added",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": id }))
}

#[derive(Debug, Deserialize)]
struct OrgQuery {
    organization: String,
}

/// `GET /api/qualifications/unverified?organization=` — HR's queue: the qualifications people in the organization declared that nobody
/// has verified, oldest first, each with the person's name. A queue of records to check, not a ranking of people.
#[debug_handler]
async fn unverified(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Query(q): Query<OrgQuery>,
) -> Result<Response> {
    ensure_hr(&ctx, &caller, &q.organization).await?;
    let today = Utc::now().date_naive();
    let in_org = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&q.organization))
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::TerminatedOn.is_null());
    super::ensure_scale(&ctx, in_org.clone()).await?;
    let people = in_org.all(&ctx.db).await?;
    let pids: Vec<Uuid> = people.iter().map(|w| w.pid).collect();
    let waiting = qualifications::Entity::find()
        .filter(qualifications::Column::WorkerPid.is_in(pids))
        .filter(qualifications::Column::VerifiedOn.is_null())
        .order_by_asc(qualifications::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    let name_of: std::collections::HashMap<Uuid, &str> = people
        .iter()
        .map(|w| (w.pid, w.display_name.as_str()))
        .collect();
    let queue: Vec<Value> = waiting
        .iter()
        .map(|w| {
            let mut v = row(w, today);
            v["worker_pid"] = json!(w.worker_pid);
            v["display_name"] = json!(name_of.get(&w.worker_pid));
            v["declared_on"] = json!(w.created_at.date_naive());
            v
        })
        .collect();
    format::json(json!({ "organization": q.organization, "as_of": today, "queue": queue }))
}

/// `GET /api/qualifications/rollup?organization=` — HR: for each role profile that people in the organization currently hold and that asks
/// for qualifications, per requirement the counts of met, declared-only, expired and missing, and the share met. A group under the floor
/// shows nothing; a role nobody holds, and a person with no role recorded, are not in it. It names no one.
#[debug_handler]
async fn rollup(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Query(q): Query<OrgQuery>,
) -> Result<Response> {
    ensure_hr(&ctx, &caller, &q.organization).await?;
    let today = Utc::now().date_naive();
    let in_org = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&q.organization))
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::TerminatedOn.is_null());
    super::ensure_scale(&ctx, in_org.clone()).await?;
    let people = in_org.all(&ctx.db).await?;
    let pids: Vec<Uuid> = people.iter().map(|w| w.pid).collect();
    let links = worker_framework_roles::Entity::find()
        .filter(worker_framework_roles::Column::WorkerPid.is_in(pids.clone()))
        .filter(worker_framework_roles::Column::EndedAt.is_null())
        .filter(worker_framework_roles::Column::RoleProfilePid.is_not_null())
        .order_by_desc(worker_framework_roles::Column::StartedAt)
        .all(&ctx.db)
        .await?;
    // Each person's newest current role, as the person's own list reads it.
    let mut role_of: std::collections::HashMap<Uuid, Uuid> = std::collections::HashMap::new();
    for l in &links {
        if let Some(r) = l.role_profile_pid {
            role_of.entry(l.worker_pid).or_insert(r);
        }
    }
    let mut holders: std::collections::BTreeMap<Uuid, Vec<Uuid>> =
        std::collections::BTreeMap::new();
    for (w, r) in &role_of {
        holders.entry(*r).or_default().push(*w);
    }
    let role_pids: Vec<Uuid> = holders.keys().copied().collect();
    let profiles = role_profiles::Entity::find()
        .filter(role_profiles::Column::Pid.is_in(role_pids.clone()))
        .filter(role_profiles::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let reqs = role_qualification_requirements::Entity::find()
        .filter(role_qualification_requirements::Column::RoleProfilePid.is_in(role_pids))
        .order_by_asc(role_qualification_requirements::Column::Title)
        .all(&ctx.db)
        .await?;
    let all = qualifications::Entity::find()
        .filter(qualifications::Column::WorkerPid.is_in(pids))
        .all(&ctx.db)
        .await?;
    let mut held_by: std::collections::HashMap<Uuid, Vec<rules::Held>> =
        std::collections::HashMap::new();
    for qual in &all {
        held_by
            .entry(qual.worker_pid)
            .or_default()
            .push(held_of(qual));
    }
    let floor = crate::rules::equality_monitoring::DEFAULT_FLOOR;
    let mut cells: Vec<Value> = Vec::new();
    for profile in &profiles {
        let group = holders.get(&profile.pid).cloned().unwrap_or_default();
        for r in reqs.iter().filter(|r| r.role_profile_pid == profile.pid) {
            let req = rules::Requirement {
                title: r.title.clone(),
                kind: r.kind.clone(),
                framework: r.framework.clone(),
                min_level: r.min_level,
            };
            let none: Vec<rules::Held> = Vec::new();
            let standings: Vec<rules::Standing> = group
                .iter()
                .map(|w| rules::standing(&req, held_by.get(w).unwrap_or(&none), today))
                .collect();
            let head = json!({
                "role_profile_pid": profile.pid, "job_title": profile.job_title,
                "title": r.title, "kind": r.kind, "framework": r.framework, "min_level": r.min_level,
            });
            let mut cell = head;
            match rules::rollup(&standings, floor) {
                rules::Rollup::Hidden => cell["hidden"] = json!(true),
                rules::Rollup::Shown {
                    met,
                    declared_only,
                    expired,
                    missing,
                    rate,
                } => {
                    cell["hidden"] = json!(false);
                    cell["met"] = json!(met);
                    cell["declared_only"] = json!(declared_only);
                    cell["expired"] = json!(expired);
                    cell["missing"] = json!(missing);
                    cell["rate"] = json!(rate);
                }
            }
            cells.push(cell);
        }
    }
    format::json(
        json!({ "organization": q.organization, "as_of": today, "floor": floor, "requirements": cells }),
    )
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/qualifications", get(of_worker))
        .add("/workers/{pid}/qualifications", post(add))
        .add("/workers/{pid}/team-qualifications", get(team))
        .add("/qualifications/unverified", get(unverified))
        .add("/qualifications/rollup", get(rollup))
        .add("/qualifications/{pid}/verify", post(verify))
        .add("/qualifications/{pid}", delete(remove))
        .add(
            "/role-profiles/{pid}/qualification-requirements",
            get(list_requirements),
        )
        .add(
            "/role-profiles/{pid}/qualification-requirements",
            post(add_requirement),
        )
}
