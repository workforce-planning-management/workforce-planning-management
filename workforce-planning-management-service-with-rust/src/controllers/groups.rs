//! **Groups** — communities of practice, communities of interest, and other
//! self-organised sets of people (`rules::groups`). A worker can belong to
//! **many** groups at once; each membership has a start and, when they leave,
//! a stop, so past membership is kept.
//!
//! A group is a shared label, not a reporting line: membership never changes
//! anyone's manager, access, or pay. Joining or leaving is a write to the
//! *worker's* record, so a person manages their own, and HR can do it on
//! their behalf (recorded as such).

use axa::extract::{Json, Path};
use axa::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{group_members, groups, workers};
use crate::models::{memberships, records};
use crate::rules::groups as rules;

fn group_json(g: &groups::Model, members: usize) -> serde_json::Value {
    serde_json::json!({
        "pid": g.pid,
        "organization_ref": g.organization_ref,
        "scope": g.scope,
        "name": g.name,
        "kind": g.kind,
        "description": g.description,
        "members": members,
    })
}

/// The organizations the caller may read (`None` = unrestricted).
async fn scope(ctx: &AppContext, caller: &MaybeAuthUser) -> Result<Option<Vec<String>>> {
    memberships::scope_organization_refs(ctx, caller.claims()).await
}

fn in_scope(scope: Option<&Vec<String>>, organization_ref: &str) -> bool {
    scope.is_none_or(|refs| refs.iter().any(|r| r == organization_ref))
}

/// Whether the caller may read this group: any organization it covers is one
/// they can read (a confederation group is readable from any member org).
fn can_read(allowed: Option<&Vec<String>>, g: &groups::Model, edges: &[(String, String)]) -> bool {
    rules::visible_to(
        allowed.map(Vec::as_slice),
        &g.scope,
        &g.organization_ref,
        edges,
    )
}

async fn edges(ctx: &AppContext) -> Result<Vec<(String, String)>> {
    crate::models::confederations::live_edges(ctx).await
}

/// A live group in one of the caller's organizations; one elsewhere is `404`.
async fn find_group(ctx: &AppContext, caller: &MaybeAuthUser, pid: &str) -> Result<groups::Model> {
    let group = sqlx::query_as!(
        groups::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, scope, name,
                  kind, description, deleted_at AS "deleted_at: _"
             FROM groups WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    if !can_read(
        scope(ctx, caller).await?.as_ref(),
        &group,
        &edges(ctx).await?,
    ) {
        return Err(Error::NotFound);
    }
    Ok(group)
}

/// Current member counts by group, counted by the database.
async fn member_counts(ctx: &AppContext) -> Result<BTreeMap<Uuid, usize>> {
    let rows = sqlx::query!(
        r#"SELECT group_pid, COUNT(id) AS "members!" FROM group_members WHERE left_at IS NULL GROUP BY group_pid"#,
    )
    .fetch_all(db::pool(ctx))
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| (r.group_pid, usize::try_from(r.members).unwrap_or(0)))
        .collect())
}

/// Every live group, by name.
async fn live_groups(ctx: &AppContext) -> Result<Vec<groups::Model>> {
    Ok(sqlx::query_as!(
        groups::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, scope, name,
                  kind, description, deleted_at AS "deleted_at: _"
             FROM groups WHERE deleted_at IS NULL ORDER BY name, id"#,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// A group's memberships, current only unless `include_past`.
async fn memberships_of_group(
    ctx: &AppContext,
    group_pid: Uuid,
    include_past: bool,
) -> Result<Vec<group_members::Model>> {
    Ok(sqlx::query_as!(
        group_members::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, group_pid, worker_pid, role,
                  joined_at AS "joined_at: _", left_at AS "left_at: _", recorded_by, on_behalf
             FROM group_members WHERE group_pid = $1 AND ($2 OR left_at IS NULL)
            ORDER BY joined_at, id"#,
        group_pid,
        include_past,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// A worker's current membership of a group, if any.
async fn current_membership(
    ctx: &AppContext,
    group_pid: Uuid,
    worker_pid: Uuid,
) -> Result<Option<group_members::Model>> {
    Ok(sqlx::query_as!(
        group_members::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, group_pid, worker_pid, role,
                  joined_at AS "joined_at: _", left_at AS "left_at: _", recorded_by, on_behalf
             FROM group_members WHERE group_pid = $1 AND worker_pid = $2 AND left_at IS NULL
            LIMIT 1"#,
        group_pid,
        worker_pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?)
}

/// `POST /api/groups` and `PUT /api/groups/{pid}` body.
#[derive(Debug, Deserialize)]
struct GroupPayload {
    /// `organization` (default) or `confederation`: a community spanning the
    /// organization and every organization beneath it. Create only.
    #[serde(default)]
    scope: Option<String>,
    /// The organization the group belongs to (required on create; a group
    /// cannot move, so it is ignored on update).
    #[serde(default)]
    organization_ref: Option<String>,
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

async fn ensure_name_free(
    ctx: &AppContext,
    organization_ref: &str,
    name: &str,
    except: Option<Uuid>,
) -> Result<()> {
    let wanted = name.trim().to_lowercase();
    let taken = sqlx::query!(
        "SELECT pid, name FROM groups WHERE deleted_at IS NULL AND organization_ref = $1",
        organization_ref,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
    .any(|g| g.name.trim().to_lowercase() == wanted && Some(g.pid) != except);
    if taken {
        return Err(unprocessable(
            "a group with that name already exists in this organization",
        ));
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
    let organization_ref = payload
        .organization_ref
        .clone()
        .filter(|o| !o.trim().is_empty())
        .ok_or_else(|| unprocessable("organization_ref is required"))?;
    let group_scope = payload
        .scope
        .clone()
        .unwrap_or_else(|| "organization".to_string());
    rules::validate_scope(&group_scope).map_err(|e| unprocessable(&e))?;
    if !in_scope(scope(&ctx, &caller).await?.as_ref(), &organization_ref) {
        return Err(unprocessable(
            "you cannot create groups in that organization",
        ));
    }
    if group_scope == "confederation"
        && crate::rules::org_access::descendants_of(&edges(&ctx).await?, &organization_ref)
            .is_empty()
    {
        return Err(unprocessable(
            "a confederation group needs an organization with member organizations beneath it",
        ));
    }
    ensure_name_free(&ctx, &organization_ref, &payload.name, None).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    let pid = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO groups (pid, organization_ref, scope, name, kind, description) VALUES ($1, $2, $3, $4, $5, $6)",
        pid,
        organization_ref,
        group_scope,
        payload.name.trim(),
        payload.kind,
        payload.description,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "group", pid, "created", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(PidRef { pid })
}

/// Query for the group list.
#[derive(Debug, Deserialize)]
struct ListQuery {
    /// Only groups open to this organization: its own, and confederation
    /// groups that cover it.
    organization_ref: Option<String>,
}

/// `GET /api/groups?organization_ref=` — the groups in the caller's
/// organizations (or one of them) with their current member counts.
#[debug_handler]
async fn list_groups(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(query): axa::extract::Query<ListQuery>,
) -> Result<Response> {
    let allowed = scope(&ctx, &caller).await?;
    let graph = edges(&ctx).await?;
    let rows: Vec<groups::Model> = live_groups(&ctx)
        .await?
        .into_iter()
        .filter(|g| can_read(allowed.as_ref(), g, &graph))
        .filter(|g| {
            query
                .organization_ref
                .as_ref()
                .is_none_or(|o| rules::is_open_to(&g.scope, &g.organization_ref, o, &graph))
        })
        .collect();
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
    let row = find_group(&ctx, &caller, &pid).await?;
    rules::validate_group(&payload.name, &payload.kind, payload.description.as_deref())
        .map_err(|e| unprocessable(&e))?;
    ensure_name_free(&ctx, &row.organization_ref, &payload.name, Some(row.pid)).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE groups SET name = $2, kind = $3, description = $4, updated_at = now() WHERE id = $1",
        row.id,
        payload.name.trim(),
        payload.kind,
        payload.description,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "group", row.pid, "updated", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(PidRef { pid: row.pid })
}

/// `DELETE /api/groups/{pid}` — retire a group (soft-delete); its membership
/// history stays.
#[debug_handler]
async fn delete_group(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = find_group(&ctx, &caller, &pid).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE groups SET deleted_at = now(), updated_at = now() WHERE id = $1",
        row.id
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "group", row.pid, "retired", caller.actor(), None).await?;
    tx.commit().await?;
    format::empty_json()
}

/// What a member listing shows of a worker.
struct Shown {
    display_name: String,
    job_title: String,
    department: String,
}

fn member_json(m: &group_members::Model, w: Option<&Shown>) -> serde_json::Value {
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
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axa::extract::Query(query): axa::extract::Query<MembersQuery>,
) -> Result<Response> {
    let group = find_group(&ctx, &caller, &pid).await?;
    let mut rows = memberships_of_group(&ctx, group.pid, query.include_past).await?;
    rows.sort_by_key(|m| (m.left_at.is_some(), m.role != "lead"));
    let pids: Vec<Uuid> = rows.iter().map(|m| m.worker_pid).collect();
    let staff: BTreeMap<Uuid, Shown> = sqlx::query!(
        "SELECT pid, display_name, job_title, department FROM workers WHERE deleted_at IS NULL AND pid = ANY($1)",
        &pids,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|w| {
        (
            w.pid,
            Shown {
                display_name: w.display_name,
                job_title: w.job_title,
                department: w.department,
            },
        )
    })
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
    let worker = records::worker_by_pid(ctx, pid).await?;
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
    let group = find_group(&ctx, &caller, &pid).await?;
    let role = payload.role.unwrap_or_else(|| "member".to_string());
    rules::validate_role(&role).map_err(|e| unprocessable(&e))?;
    let worker = writable_worker(&ctx, &caller, payload.worker_pid).await?;
    if !rules::is_open_to(
        &group.scope,
        &group.organization_ref,
        &worker.organization_ref,
        &edges(&ctx).await?,
    ) {
        return Err(unprocessable(
            "a worker can only join groups open to their organization",
        ));
    }
    let current = current_membership(&ctx, group.pid, worker.pid).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    let row = if let Some(existing) = current {
        sqlx::query_as!(
            group_members::Model,
            r#"UPDATE group_members SET role = $2, updated_at = now() WHERE id = $1
            RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, group_pid, worker_pid, role,
                      joined_at AS "joined_at: _", left_at AS "left_at: _", recorded_by, on_behalf"#,
            existing.id,
            role,
        )
        .fetch_one(&mut *tx)
        .await?
    } else {
        sqlx::query_as!(
            group_members::Model,
            r#"INSERT INTO group_members (pid, group_pid, worker_pid, role, joined_at, recorded_by, on_behalf)
               VALUES ($1, $2, $3, $4, now(), $5, $6)
            RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, group_pid, worker_pid, role,
                      joined_at AS "joined_at: _", left_at AS "left_at: _", recorded_by, on_behalf"#,
            Uuid::new_v4(),
            group.pid,
            worker.pid,
            role,
            caller.actor(),
            auth::acting_for_other(&caller, &worker.person_ref),
        )
        .fetch_one(&mut *tx)
        .await?
    };
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "group_joined",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    let shown = Shown {
        display_name: worker.display_name,
        job_title: worker.job_title,
        department: worker.department,
    };
    format::json(member_json(&row, Some(&shown)))
}

/// `DELETE /api/groups/{pid}/members/{worker_pid}` — leave a group. The
/// membership is closed, not deleted: it stays as past membership.
#[debug_handler]
async fn leave_group(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, worker_pid)): Path<(String, String)>,
) -> Result<Response> {
    let group = find_group(&ctx, &caller, &pid).await?;
    let worker = writable_worker(&ctx, &caller, records::parse_pid(&worker_pid)?).await?;
    let current = current_membership(&ctx, group.pid, worker.pid)
        .await?
        .ok_or(Error::NotFound)?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE group_members SET left_at = now(), updated_at = now() WHERE id = $1",
        current.id
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "group_left",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}

/// `GET /api/groups/{pid}/skills` — what the group knows, **in aggregate**:
/// per skill, how many current members declare it and how their levels are
/// spread. Declared, not inferred; never names a member. A skill declared by
/// fewer than three members (or any skill, in a group of fewer than three) is
/// withheld so a distribution cannot point at one person.
#[debug_handler]
async fn group_skills(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let group = find_group(&ctx, &caller, &pid).await?;
    let members: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT worker_pid FROM group_members WHERE group_pid = $1 AND left_at IS NULL",
        group.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let declared = sqlx::query!(
        "SELECT skill_pid, proficiency FROM worker_skills WHERE worker_pid = ANY($1) AND deleted_at IS NULL",
        &members,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let mut by_skill: BTreeMap<Uuid, Vec<i32>> = BTreeMap::new();
    for d in &declared {
        by_skill.entry(d.skill_pid).or_default().push(d.proficiency);
    }
    // Every skill's name, retired ones too: a member may still declare one.
    let names: BTreeMap<Uuid, String> = sqlx::query!("SELECT pid, name FROM skills")
        .fetch_all(db::pool(&ctx))
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
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axa::extract::Query(query): axa::extract::Query<MembersQuery>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let memberships = sqlx::query_as!(
        group_members::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, group_pid, worker_pid, role,
                  joined_at AS "joined_at: _", left_at AS "left_at: _", recorded_by, on_behalf
             FROM group_members WHERE worker_pid = $1 AND ($2 OR left_at IS NULL)
            ORDER BY joined_at DESC, id DESC"#,
        worker.pid,
        query.include_past,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let allowed = scope(&ctx, &caller).await?;
    let graph = edges(&ctx).await?;
    let by_pid: BTreeMap<Uuid, groups::Model> = live_groups(&ctx)
        .await?
        .into_iter()
        .filter(|g| can_read(allowed.as_ref(), g, &graph))
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
