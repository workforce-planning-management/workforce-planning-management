//! **Groups** — communities of practice, communities of interest, and other
//! self-organised sets of people (`rules::groups`). A worker can belong to
//! **many** groups at once; each membership has a start and, when they leave,
//! a stop, so past membership is kept.
//!
//! A group is a shared label, not a reporting line: membership never changes
//! anyone's manager, access, or pay. Joining or leaving is a write to the
//! *worker's* record, so a person manages their own, and HR can do it on
//! their behalf (recorded as such).

use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{group_members, groups, skills, worker_skills, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::groups as rules;

fn group_json(g: &groups::Model, members: usize) -> serde_json::Value {
    serde_json::json!({
        "pid": g.pid,
        "name": g.name,
        "kind": g.kind,
        "description": g.description,
        "members": members,
    })
}

async fn find_group(ctx: &AppContext, pid: &str) -> Result<groups::Model> {
    groups::Entity::find()
        .filter(groups::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(groups::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

/// Current member counts by group.
async fn member_counts(ctx: &AppContext) -> Result<BTreeMap<Uuid, usize>> {
    let mut counts = BTreeMap::new();
    for m in group_members::Entity::find()
        .filter(group_members::Column::LeftAt.is_null())
        .all(&ctx.db)
        .await?
    {
        *counts.entry(m.group_pid).or_insert(0) += 1;
    }
    Ok(counts)
}

/// `POST /api/groups` and `PUT /api/groups/{pid}` body.
#[derive(Debug, Deserialize)]
struct GroupPayload {
    name: String,
    /// `practice` (community of practice), `interest` (community of
    /// interest), or `other`.
    kind: String,
    #[serde(default)]
    description: Option<String>,
}

/// A `{pid}` response.
#[derive(serde::Serialize)]
struct PidRef {
    pid: Uuid,
}

async fn ensure_name_free(ctx: &AppContext, name: &str, except: Option<Uuid>) -> Result<()> {
    let wanted = name.trim().to_lowercase();
    let taken = groups::Entity::find()
        .filter(groups::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .any(|g| g.name.trim().to_lowercase() == wanted && Some(g.pid) != except);
    if taken {
        return Err(unprocessable("a group with that name already exists"));
    }
    Ok(())
}

/// `POST /api/groups` — start a group.
#[debug_handler]
async fn create_group(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<GroupPayload>,
) -> Result<Response> {
    rules::validate_group(&payload.name, &payload.kind, payload.description.as_deref())
        .map_err(|e| unprocessable(&e))?;
    ensure_name_free(&ctx, &payload.name, None).await?;
    let row = groups::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        name: ActiveValue::set(payload.name.trim().to_string()),
        kind: ActiveValue::set(payload.kind),
        description: ActiveValue::set(payload.description),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(&ctx.db, "group", row.pid, "created", caller.actor(), None).await?;
    format::json(PidRef { pid: row.pid })
}

/// `GET /api/groups` — every group with its current member count.
#[debug_handler]
async fn list_groups(State(ctx): State<AppContext>) -> Result<Response> {
    let rows = groups::Entity::find()
        .filter(groups::Column::DeletedAt.is_null())
        .order_by_asc(groups::Column::Name)
        .all(&ctx.db)
        .await?;
    let counts = member_counts(&ctx).await?;
    format::json(
        rows.iter()
            .map(|g| group_json(g, counts.get(&g.pid).copied().unwrap_or(0)))
            .collect::<Vec<_>>(),
    )
}

/// `PUT /api/groups/{pid}` — rename or re-describe a group.
#[debug_handler]
async fn update_group(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<GroupPayload>,
) -> Result<Response> {
    let row = find_group(&ctx, &pid).await?;
    rules::validate_group(&payload.name, &payload.kind, payload.description.as_deref())
        .map_err(|e| unprocessable(&e))?;
    ensure_name_free(&ctx, &payload.name, Some(row.pid)).await?;
    let mut active: groups::ActiveModel = row.into();
    active.name = ActiveValue::set(payload.name.trim().to_string());
    active.kind = ActiveValue::set(payload.kind);
    active.description = ActiveValue::set(payload.description);
    let updated = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "group",
        updated.pid,
        "updated",
        caller.actor(),
        None,
    )
    .await?;
    format::json(PidRef { pid: updated.pid })
}

/// `DELETE /api/groups/{pid}` — retire a group (soft-delete); its membership
/// history stays.
#[debug_handler]
async fn delete_group(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = find_group(&ctx, &pid).await?;
    let group_pid = row.pid;
    let mut active: groups::ActiveModel = row.into();
    active.deleted_at = ActiveValue::set(Some(Utc::now().into()));
    active.update(&ctx.db).await?;
    Audit::record(&ctx.db, "group", group_pid, "retired", caller.actor(), None).await?;
    format::empty_json()
}

fn member_json(m: &group_members::Model, w: Option<&workers::Model>) -> serde_json::Value {
    serde_json::json!({
        "worker_pid": m.worker_pid,
        "display_name": w.map(|w| &w.display_name),
        "job_title": w.map(|w| &w.job_title),
        "department": w.map(|w| &w.department),
        "role": m.role,
        "joined_at": m.joined_at,
        "left_at": m.left_at,
        "current": m.left_at.is_none(),
        "recorded_by": m.recorded_by,
        "on_behalf": m.on_behalf,
    })
}

/// Query for a member listing.
#[derive(Debug, Deserialize)]
struct MembersQuery {
    /// Include people who have left (default: current members only).
    #[serde(default)]
    include_past: bool,
}

/// `GET /api/groups/{pid}/members?include_past=` — the group's members (leads
/// first); past members too when asked.
#[debug_handler]
async fn list_members(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
    axum::extract::Query(query): axum::extract::Query<MembersQuery>,
) -> Result<Response> {
    let group = find_group(&ctx, &pid).await?;
    let mut select =
        group_members::Entity::find().filter(group_members::Column::GroupPid.eq(group.pid));
    if !query.include_past {
        select = select.filter(group_members::Column::LeftAt.is_null());
    }
    let mut rows = select
        .order_by_asc(group_members::Column::JoinedAt)
        .all(&ctx.db)
        .await?;
    rows.sort_by_key(|m| (m.left_at.is_some(), m.role != "lead"));
    let staff: BTreeMap<Uuid, workers::Model> = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| (w.pid, w))
        .collect();
    format::json(serde_json::json!({
        "group": group_json(&group, rows.iter().filter(|m| m.left_at.is_none()).count()),
        "members": rows.iter().map(|m| member_json(m, staff.get(&m.worker_pid))).collect::<Vec<_>>(),
    }))
}

/// `POST /api/groups/{pid}/members` body.
#[derive(Debug, Deserialize)]
struct JoinPayload {
    worker_pid: Uuid,
    #[serde(default)]
    role: Option<String>,
}

/// A worker, authorized for a write to their record.
async fn writable_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: Uuid,
) -> Result<workers::Model> {
    let worker = records::find_worker(&ctx.db, pid).await?;
    auth::authorize_record(
        caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    Ok(worker)
}

/// `POST /api/groups/{pid}/members` — a worker joins a group (or changes their
/// part in it). A worker may be in many groups; joining one they are already
/// in only updates their role.
#[debug_handler]
async fn join_group(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<JoinPayload>,
) -> Result<Response> {
    let group = find_group(&ctx, &pid).await?;
    let role = payload.role.unwrap_or_else(|| "member".to_string());
    rules::validate_role(&role).map_err(|e| unprocessable(&e))?;
    let worker = writable_worker(&ctx, &caller, payload.worker_pid).await?;
    let current = group_members::Entity::find()
        .filter(group_members::Column::GroupPid.eq(group.pid))
        .filter(group_members::Column::WorkerPid.eq(worker.pid))
        .filter(group_members::Column::LeftAt.is_null())
        .one(&ctx.db)
        .await?;
    let row = if let Some(existing) = current {
        let mut active: group_members::ActiveModel = existing.into();
        active.role = ActiveValue::set(role);
        active.update(&ctx.db).await?
    } else {
        group_members::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            group_pid: ActiveValue::set(group.pid),
            worker_pid: ActiveValue::set(worker.pid),
            role: ActiveValue::set(role),
            joined_at: ActiveValue::set(Utc::now().into()),
            left_at: ActiveValue::set(None),
            recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
            on_behalf: ActiveValue::set(auth::acting_for_other(&caller, &worker.person_ref)),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?
    };
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "group_joined",
        caller.actor(),
        None,
    )
    .await?;
    format::json(member_json(&row, Some(&worker)))
}

/// `DELETE /api/groups/{pid}/members/{worker_pid}` — leave a group. The
/// membership is closed, not deleted: it stays as past membership.
#[debug_handler]
async fn leave_group(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, worker_pid)): Path<(String, String)>,
) -> Result<Response> {
    let group = find_group(&ctx, &pid).await?;
    let worker = writable_worker(&ctx, &caller, records::parse_pid(&worker_pid)?).await?;
    let current = group_members::Entity::find()
        .filter(group_members::Column::GroupPid.eq(group.pid))
        .filter(group_members::Column::WorkerPid.eq(worker.pid))
        .filter(group_members::Column::LeftAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let mut active: group_members::ActiveModel = current.into();
    active.left_at = ActiveValue::set(Some(Utc::now().into()));
    active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "group_left",
        caller.actor(),
        None,
    )
    .await?;
    format::empty_json()
}

/// `GET /api/groups/{pid}/skills` — what the group knows, **in aggregate**:
/// per skill, how many current members declare it and how their levels are
/// spread. Declared, not inferred; never names a member. A skill declared by
/// fewer than three members (or any skill, in a group of fewer than three) is
/// withheld so a distribution cannot point at one person.
#[debug_handler]
async fn group_skills(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let group = find_group(&ctx, &pid).await?;
    let members: Vec<Uuid> = group_members::Entity::find()
        .filter(group_members::Column::GroupPid.eq(group.pid))
        .filter(group_members::Column::LeftAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|m| m.worker_pid)
        .collect();
    let declared = worker_skills::Entity::find()
        .filter(worker_skills::Column::WorkerPid.is_in(members.clone()))
        .filter(worker_skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let mut by_skill: BTreeMap<Uuid, Vec<i32>> = BTreeMap::new();
    for d in &declared {
        by_skill.entry(d.skill_pid).or_default().push(d.proficiency);
    }
    let names: BTreeMap<Uuid, String> = skills::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|s| (s.pid, s.name))
        .collect();
    let mut shown = Vec::new();
    let mut withheld = 0usize;
    for (skill, levels) in &by_skill {
        match rules::rollup(members.len(), levels) {
            Some(r) => shown.push(serde_json::json!({
                "skill_pid": skill,
                "skill": names.get(skill),
                "declared": r.declared,
                "coverage": r.coverage(),
                "levels": { "1": r.levels[0], "2": r.levels[1], "3": r.levels[2], "4": r.levels[3], "5": r.levels[4] },
            })),
            None => withheld += 1,
        }
    }
    shown.sort_by(|a, b| {
        b["declared"]
            .as_u64()
            .cmp(&a["declared"].as_u64())
            .then_with(|| {
                a["skill"]
                    .as_str()
                    .map(str::to_lowercase)
                    .cmp(&b["skill"].as_str().map(str::to_lowercase))
            })
    });
    format::json(serde_json::json!({
        "group": group_json(&group, members.len()),
        "floor": rules::MIN_ROLLUP,
        "derivation": "declared skills of current members, in aggregate; a skill declared by fewer \
                       than the floor is withheld, and no member is named",
        "skills": shown,
        "withheld_below_floor": withheld,
    }))
}

/// `GET /api/workers/{pid}/groups?include_past=` — every group a worker is in
/// (several at once is normal); past memberships too when asked.
#[debug_handler]
async fn worker_groups(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
    axum::extract::Query(query): axum::extract::Query<MembersQuery>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let mut select =
        group_members::Entity::find().filter(group_members::Column::WorkerPid.eq(worker.pid));
    if !query.include_past {
        select = select.filter(group_members::Column::LeftAt.is_null());
    }
    let memberships = select
        .order_by_desc(group_members::Column::JoinedAt)
        .all(&ctx.db)
        .await?;
    let by_pid: BTreeMap<Uuid, groups::Model> = groups::Entity::find()
        .filter(groups::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|g| (g.pid, g))
        .collect();
    let out: Vec<serde_json::Value> = memberships
        .iter()
        .filter_map(|m| {
            by_pid.get(&m.group_pid).map(|g| {
                serde_json::json!({
                    "group_pid": g.pid,
                    "name": g.name,
                    "kind": g.kind,
                    "role": m.role,
                    "joined_at": m.joined_at,
                    "left_at": m.left_at,
                    "current": m.left_at.is_none(),
                    "on_behalf": m.on_behalf,
                })
            })
        })
        .collect();
    format::json(serde_json::json!({ "worker_pid": worker.pid, "groups": out }))
}

/// The group routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/groups", get(list_groups))
        .add("/groups", post(create_group))
        .add("/groups/{pid}", put(update_group))
        .add("/groups/{pid}", delete(delete_group))
        .add("/groups/{pid}/skills", get(group_skills))
        .add("/groups/{pid}/members", get(list_members))
        .add("/groups/{pid}/members", post(join_group))
        .add("/groups/{pid}/members/{worker_pid}", delete(leave_group))
        .add("/workers/{pid}/groups", get(worker_groups))
}
