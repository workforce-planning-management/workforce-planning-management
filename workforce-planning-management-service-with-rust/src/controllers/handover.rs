//! A **leaver's handover** (`rules::movements::HandoverKind`): on or before
//! the last day, list everything the leaver still holds and reassign or close
//! each item — with an audit trail of what went where, when, and who did it.
//!
//! What a leaver can hold:
//! - **ownerships** — people they manage (`direct_report`, `dotted_line`), a
//!   group they lead, their seat in an on-call rota, a mentorship they give;
//! - **bookings** — future shifts, on-call swaps, being named as someone's
//!   backup;
//! - **tasks** — checklist items assigned to them;
//! - **access** — their organization roles, which are **revoked** (ended on
//!   the last day), never handed to someone.
//!
//! Each reassignment is one transaction: the change, a `handover_actions` row,
//! an audit entry and a notification to the new holder.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use super::movements::find_movement;
use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    dotted_line_reports, group_members, groups, handover_actions, mentorships, movement_items,
    movements, organization_memberships, rota_members, rota_overrides, rotas, shift_assignments,
    shifts, worker_backups, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::notifications::Model as Notification;
use crate::models::records;
use crate::rules::metrics::is_employed_on;
use crate::rules::movements::{HandoverKind, validate_handover};
use crate::rules::org::would_create_cycle;

/// One thing the leaver still holds.
#[derive(Debug, Clone)]
pub(super) struct Held {
    kind: HandoverKind,
    /// The thing's own id (a row pid, or the rota's pid for a seat).
    subject: Uuid,
    label: String,
}

/// Everything `leaver` still holds, as of the movement's last day.
#[allow(clippy::too_many_lines)] // one block per kind of thing a leaver can hold
pub(super) async fn inventory(
    ctx: &AppContext,
    movement: &movements::Model,
    leaver: &workers::Model,
) -> Result<Vec<Held>> {
    let db = &ctx.db;
    let last_day = movement.effective_on;
    let mut out = Vec::new();

    // Direct reports still employed.
    for w in workers::Entity::find()
        .filter(workers::Column::ManagerPid.eq(leaver.pid))
        .filter(workers::Column::DeletedAt.is_null())
        .order_by_asc(workers::Column::Id)
        .all(db)
        .await?
    {
        if is_employed_on(last_day, w.hired_on, w.terminated_on) {
            out.push(Held {
                kind: HandoverKind::DirectReport,
                subject: w.pid,
                label: w.display_name,
            });
        }
    }
    // Dotted-line reports.
    for d in dotted_line_reports::Entity::find()
        .filter(dotted_line_reports::Column::ManagerPid.eq(leaver.pid))
        .filter(dotted_line_reports::Column::EndedAt.is_null())
        .all(db)
        .await?
    {
        let name = records::find_worker(db, d.report_pid)
            .await
            .map(|w| w.display_name)
            .unwrap_or_default();
        out.push(Held {
            kind: HandoverKind::DottedLine,
            subject: d.pid,
            label: name,
        });
    }
    // Groups they lead.
    for g in group_members::Entity::find()
        .filter(group_members::Column::WorkerPid.eq(leaver.pid))
        .filter(group_members::Column::Role.eq("lead"))
        .filter(group_members::Column::LeftAt.is_null())
        .all(db)
        .await?
    {
        let name = groups::Entity::find()
            .filter(groups::Column::Pid.eq(g.group_pid))
            .one(db)
            .await?
            .map(|g| g.name)
            .unwrap_or_default();
        out.push(Held {
            kind: HandoverKind::GroupLead,
            subject: g.pid,
            label: name,
        });
    }
    // Their seat in on-call rotas.
    for m in rota_members::Entity::find()
        .filter(rota_members::Column::WorkerPid.eq(leaver.pid))
        .all(db)
        .await?
    {
        if let Some(r) = rotas::Entity::find()
            .filter(rotas::Column::Pid.eq(m.rota_pid))
            .filter(rotas::Column::DeletedAt.is_null())
            .one(db)
            .await?
        {
            out.push(Held {
                kind: HandoverKind::RotaMembership,
                subject: r.pid,
                label: r.name,
            });
        }
    }
    // On-call swaps still ahead.
    for o in rota_overrides::Entity::find()
        .filter(rota_overrides::Column::WorkerPid.eq(leaver.pid))
        .filter(rota_overrides::Column::EndsOn.gte(last_day))
        .all(db)
        .await?
    {
        let name = rotas::Entity::find()
            .filter(rotas::Column::Pid.eq(o.rota_pid))
            .one(db)
            .await?
            .map(|r| r.name)
            .unwrap_or_default();
        out.push(Held {
            kind: HandoverKind::RotaSwap,
            subject: o.pid,
            label: format!("{name} {}–{}", o.starts_on, o.ends_on),
        });
    }
    // Named as someone's backup.
    for b in worker_backups::Entity::find()
        .filter(worker_backups::Column::BackupPid.eq(leaver.pid))
        .filter(worker_backups::Column::DeletedAt.is_null())
        .all(db)
        .await?
    {
        let name = records::find_worker(db, b.worker_pid)
            .await
            .map(|w| w.display_name)
            .unwrap_or_default();
        out.push(Held {
            kind: HandoverKind::Backup,
            subject: b.pid,
            label: name,
        });
    }
    // Mentorships they give.
    for m in mentorships::Entity::find()
        .filter(mentorships::Column::MentorPid.eq(leaver.pid))
        .filter(mentorships::Column::DeletedAt.is_null())
        .filter(mentorships::Column::Status.is_in(["proposed", "active"]))
        .all(db)
        .await?
    {
        let name = records::find_worker(db, m.mentee_pid)
            .await
            .map(|w| w.display_name)
            .unwrap_or_default();
        out.push(Held {
            kind: HandoverKind::Mentorship,
            subject: m.pid,
            label: name,
        });
    }
    // Future shifts.
    for a in shift_assignments::Entity::find()
        .filter(shift_assignments::Column::WorkerPid.eq(leaver.pid))
        .filter(shift_assignments::Column::DeletedAt.is_null())
        .all(db)
        .await?
    {
        if let Some(s) = shifts::Entity::find()
            .filter(shifts::Column::Pid.eq(a.shift_pid))
            .filter(shifts::Column::DeletedAt.is_null())
            .one(db)
            .await?
            && s.starts_at.date_naive() >= last_day
        {
            out.push(Held {
                kind: HandoverKind::Shift,
                subject: a.pid,
                label: format!("{} {}", s.department, s.starts_at.format("%Y-%m-%d %H:%M")),
            });
        }
    }
    // Checklist tasks assigned to them (not on their own record).
    for i in movement_items::Entity::find()
        .filter(movement_items::Column::AssigneePid.eq(leaver.pid))
        .filter(movement_items::Column::DoneOn.is_null())
        .filter(movement_items::Column::SkippedReason.is_null())
        .all(db)
        .await?
    {
        let theirs = movements::Entity::find()
            .filter(movements::Column::Pid.eq(i.movement_pid))
            .one(db)
            .await?
            .is_some_and(|m| m.status == "open" && m.worker_pid != leaver.pid);
        if theirs {
            out.push(Held {
                kind: HandoverKind::Task,
                subject: i.pid,
                label: i.title,
            });
        }
    }
    // Access: organization roles that run past the last day.
    for m in organization_memberships::Entity::find()
        .filter(organization_memberships::Column::PersonRef.eq(leaver.person_ref.clone()))
        .filter(organization_memberships::Column::DeletedAt.is_null())
        .all(db)
        .await?
    {
        if m.ends_on.is_none_or(|e| e > last_day) {
            out.push(Held {
                kind: HandoverKind::Access,
                subject: m.pid,
                label: format!("{} — {}", m.organization_ref, m.role),
            });
        }
    }
    Ok(out)
}

/// How many things the leaver still holds (for completion).
pub(super) async fn unassigned_count(
    ctx: &AppContext,
    movement: &movements::Model,
    leaver: &workers::Model,
) -> Result<usize> {
    Ok(inventory(ctx, movement, leaver).await?.len())
}

fn held_json(h: &Held) -> serde_json::Value {
    serde_json::json!({
        "kind": h.kind.as_str(),
        "subject_pid": h.subject,
        "label": h.label,
        "can_reassign": h.kind.can_reassign(),
        "needs_new_holder": h.kind.needs_target(),
    })
}

/// A leaver's movement, or 422 for a joiner.
async fn leaver_movement(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<(movements::Model, workers::Model)> {
    let (movement, worker) = find_movement(ctx, caller, pid).await?;
    if movement.kind != "leaver" {
        return Err(unprocessable("only a leaver has a handover"));
    }
    Ok((movement, worker))
}

/// `GET /api/movements/{pid}/handover` — everything the leaver still holds,
/// grouped by kind with counts, as of their last day.
#[debug_handler]
async fn get_handover(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (movement, leaver) = leaver_movement(&ctx, &caller, &pid).await?;
    let held = inventory(&ctx, &movement, &leaver).await?;
    let mut counts = serde_json::Map::new();
    for k in HandoverKind::ALL {
        let n = held.iter().filter(|h| h.kind == k).count();
        if n > 0 {
            counts.insert(k.as_str().to_string(), serde_json::json!(n));
        }
    }
    format::json(serde_json::json!({
        "movement_pid": movement.pid,
        "worker_pid": leaver.pid,
        "last_day": movement.effective_on,
        "remaining": held.len(),
        "counts": counts,
        "items": held.iter().map(held_json).collect::<Vec<_>>(),
    }))
}

/// A new holder: an employed worker of the leaver's organization, not the leaver.
async fn successor(
    ctx: &AppContext,
    to: Uuid,
    leaver: &workers::Model,
    on: NaiveDate,
) -> Result<workers::Model> {
    let w = records::find_worker(&ctx.db, to)
        .await
        .map_err(|_| unprocessable("the new holder is not a worker here"))?;
    if w.organization_ref != leaver.organization_ref {
        return Err(unprocessable(
            "the new holder must be in the leaver's organization",
        ));
    }
    if !is_employed_on(on, w.hired_on, w.terminated_on) {
        return Err(unprocessable("the new holder is not employed"));
    }
    Ok(w)
}

/// Apply one reassignment (or closure) inside `txn`; returns the verb recorded.
#[allow(clippy::too_many_lines)] // one arm per kind
async fn apply(
    txn: &impl ConnectionTrait,
    held: &Held,
    to: Option<&workers::Model>,
    leaver: &workers::Model,
    last_day: NaiveDate,
    manager_of: &HashMap<Uuid, Uuid>,
) -> Result<&'static str> {
    let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
    let verb = match (held.kind, to) {
        (HandoverKind::DirectReport, Some(new)) => {
            if would_create_cycle(held.subject, new.pid, manager_of) {
                return Err(unprocessable(
                    "that would put the new manager below their own report",
                ));
            }
            let w = records::find_worker(txn, held.subject).await?;
            let mut a: workers::ActiveModel = w.into();
            a.manager_pid = ActiveValue::set(Some(new.pid));
            a.update(txn).await?;
            "reassigned"
        }
        (HandoverKind::DottedLine, to) => {
            let row = dotted_line_reports::Entity::find()
                .filter(dotted_line_reports::Column::Pid.eq(held.subject))
                .one(txn)
                .await?
                .ok_or(Error::NotFound)?;
            let report = row.report_pid;
            let mut a: dotted_line_reports::ActiveModel = row.into();
            match to {
                Some(new) if new.pid != report => {
                    let already = dotted_line_reports::Entity::find()
                        .filter(dotted_line_reports::Column::ReportPid.eq(report))
                        .filter(dotted_line_reports::Column::ManagerPid.eq(new.pid))
                        .filter(dotted_line_reports::Column::EndedAt.is_null())
                        .one(txn)
                        .await?
                        .is_some();
                    if already {
                        a.ended_at = ActiveValue::set(Some(now));
                    } else {
                        a.manager_pid = ActiveValue::set(new.pid);
                    }
                    a.update(txn).await?;
                    "reassigned"
                }
                _ => {
                    a.ended_at = ActiveValue::set(Some(now));
                    a.update(txn).await?;
                    "closed"
                }
            }
        }
        (HandoverKind::GroupLead, to) => {
            let row = group_members::Entity::find()
                .filter(group_members::Column::Pid.eq(held.subject))
                .one(txn)
                .await?
                .ok_or(Error::NotFound)?;
            let group = row.group_pid;
            let mut leaving: group_members::ActiveModel = row.into();
            leaving.left_at = ActiveValue::set(Some(now));
            leaving.update(txn).await?;
            if let Some(new) = to {
                let theirs = group_members::Entity::find()
                    .filter(group_members::Column::GroupPid.eq(group))
                    .filter(group_members::Column::WorkerPid.eq(new.pid))
                    .filter(group_members::Column::LeftAt.is_null())
                    .one(txn)
                    .await?
                    .ok_or_else(|| {
                        unprocessable("the new lead must already be a member of that group")
                    })?;
                let mut a: group_members::ActiveModel = theirs.into();
                a.role = ActiveValue::set("lead".to_string());
                a.update(txn).await?;
                "reassigned"
            } else {
                "closed"
            }
        }
        (HandoverKind::RotaMembership, to) => {
            let mine = rota_members::Entity::find()
                .filter(rota_members::Column::RotaPid.eq(held.subject))
                .filter(rota_members::Column::WorkerPid.eq(leaver.pid))
                .one(txn)
                .await?
                .ok_or(Error::NotFound)?;
            let taken = match to {
                Some(new) => rota_members::Entity::find()
                    .filter(rota_members::Column::RotaPid.eq(held.subject))
                    .filter(rota_members::Column::WorkerPid.eq(new.pid))
                    .one(txn)
                    .await?
                    .is_some(),
                None => true,
            };
            if let (Some(new), false) = (to, taken) {
                let mut a: rota_members::ActiveModel = mine.into();
                a.worker_pid = ActiveValue::set(new.pid);
                a.update(txn).await?;
                "reassigned"
            } else {
                rota_members::Entity::delete_by_id(mine.id)
                    .exec(txn)
                    .await?;
                "closed"
            }
        }
        (HandoverKind::RotaSwap, to) => {
            let row = rota_overrides::Entity::find()
                .filter(rota_overrides::Column::Pid.eq(held.subject))
                .one(txn)
                .await?
                .ok_or(Error::NotFound)?;
            if let Some(new) = to {
                let mut a: rota_overrides::ActiveModel = row.into();
                a.worker_pid = ActiveValue::set(new.pid);
                a.update(txn).await?;
                "reassigned"
            } else {
                rota_overrides::Entity::delete_by_id(row.id)
                    .exec(txn)
                    .await?;
                "closed"
            }
        }
        (HandoverKind::Backup, to) => {
            let row = worker_backups::Entity::find()
                .filter(worker_backups::Column::Pid.eq(held.subject))
                .one(txn)
                .await?
                .ok_or(Error::NotFound)?;
            let owner = row.worker_pid;
            let replace = match to {
                Some(new) if new.pid != owner => worker_backups::Entity::find()
                    .filter(worker_backups::Column::WorkerPid.eq(owner))
                    .filter(worker_backups::Column::BackupPid.eq(new.pid))
                    .filter(worker_backups::Column::DeletedAt.is_null())
                    .one(txn)
                    .await?
                    .is_none(),
                _ => false,
            };
            let mut a: worker_backups::ActiveModel = row.into();
            if let (Some(new), true) = (to, replace) {
                a.backup_pid = ActiveValue::set(new.pid);
                a.update(txn).await?;
                "reassigned"
            } else {
                a.deleted_at = ActiveValue::set(Some(now));
                a.update(txn).await?;
                "closed"
            }
        }
        (HandoverKind::Mentorship, to) => {
            let row = mentorships::Entity::find()
                .filter(mentorships::Column::Pid.eq(held.subject))
                .one(txn)
                .await?
                .ok_or(Error::NotFound)?;
            let mentee = row.mentee_pid;
            let mut a: mentorships::ActiveModel = row.into();
            match to {
                Some(new) if new.pid != mentee => {
                    a.mentor_pid = ActiveValue::set(new.pid);
                    a.update(txn).await?;
                    "reassigned"
                }
                _ => {
                    a.status = ActiveValue::set("ended".to_string());
                    a.ended_on = ActiveValue::set(Some(last_day));
                    a.update(txn).await?;
                    "closed"
                }
            }
        }
        (HandoverKind::Shift, to) => {
            let row = shift_assignments::Entity::find()
                .filter(shift_assignments::Column::Pid.eq(held.subject))
                .one(txn)
                .await?
                .ok_or(Error::NotFound)?;
            let shift = row.shift_pid;
            let taken = match to {
                Some(new) => shift_assignments::Entity::find()
                    .filter(shift_assignments::Column::ShiftPid.eq(shift))
                    .filter(shift_assignments::Column::WorkerPid.eq(new.pid))
                    .filter(shift_assignments::Column::DeletedAt.is_null())
                    .one(txn)
                    .await?
                    .is_some(),
                None => true,
            };
            let mut a: shift_assignments::ActiveModel = row.into();
            if let (Some(new), false) = (to, taken) {
                a.worker_pid = ActiveValue::set(new.pid);
                a.update(txn).await?;
                "reassigned"
            } else {
                a.deleted_at = ActiveValue::set(Some(now));
                a.update(txn).await?;
                "closed"
            }
        }
        (HandoverKind::Task, to) => {
            let row = movement_items::Entity::find()
                .filter(movement_items::Column::Pid.eq(held.subject))
                .one(txn)
                .await?
                .ok_or(Error::NotFound)?;
            let mut a: movement_items::ActiveModel = row.into();
            a.assignee_pid = ActiveValue::set(to.map(|w| w.pid));
            a.update(txn).await?;
            if to.is_some() { "reassigned" } else { "closed" }
        }
        (HandoverKind::Access, _) => {
            let row = organization_memberships::Entity::find()
                .filter(organization_memberships::Column::Pid.eq(held.subject))
                .one(txn)
                .await?
                .ok_or(Error::NotFound)?;
            let end = row.starts_on.max(last_day);
            let mut a: organization_memberships::ActiveModel = row.into();
            a.ends_on = ActiveValue::set(Some(end));
            a.update(txn).await?;
            "revoked"
        }
        (HandoverKind::DirectReport, None) => {
            return Err(unprocessable("a direct report needs a new manager"));
        }
    };
    Ok(verb)
}

/// The live `manager_of` map for the cycle check.
async fn manager_map(ctx: &AppContext) -> Result<HashMap<Uuid, Uuid>> {
    Ok(workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::ManagerPid.is_not_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter_map(|w| w.manager_pid.map(|m| (w.pid, m)))
        .collect())
}

/// Do `held` → `to` in one transaction: change, audit-trail row, audit entry,
/// and a notification to the new holder.
#[allow(clippy::too_many_arguments)] // the movement, the leaver, the item, the target and the audit inputs
async fn perform(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    movement: &movements::Model,
    leaver: &workers::Model,
    held: &Held,
    to: Option<&workers::Model>,
    note: Option<&str>,
    managers: &HashMap<Uuid, Uuid>,
) -> Result<&'static str> {
    let txn = ctx.db.begin().await?;
    let verb = apply(&txn, held, to, leaver, movement.effective_on, managers).await?;
    handover_actions::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        movement_pid: ActiveValue::set(movement.pid),
        kind: ActiveValue::set(held.kind.as_str().to_string()),
        subject_pid: ActiveValue::set(held.subject),
        subject_label: ActiveValue::set(Some(held.label.clone())),
        from_worker: ActiveValue::set(leaver.pid),
        to_worker: ActiveValue::set(to.map(|w| w.pid)),
        action: ActiveValue::set(verb.to_string()),
        note: ActiveValue::set(note.map(str::to_string)),
        performed_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        performed_at: ActiveValue::set(Utc::now().into()),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "worker",
        leaver.pid,
        &format!("handover_{verb}"),
        caller.actor(),
        Some(serde_json::json!({
            "kind": held.kind.as_str(),
            "subject_pid": held.subject,
            "to_worker_pid": to.map(|w| w.pid),
        })),
    )
    .await?;
    if let Some(new) = to {
        Notification::push(
            &txn,
            new.pid,
            "handover_received",
            &format!(
                "{} has handed over to you: {}.",
                leaver.display_name, held.label
            ),
            serde_json::json!({ "movement_pid": movement.pid, "kind": held.kind.as_str() }),
        )
        .await?;
    }
    txn.commit().await?;
    Ok(verb)
}

/// `POST /api/movements/{pid}/handover` body.
#[derive(Debug, Deserialize)]
struct ReassignPayload {
    /// A kind from the inventory.
    kind: String,
    /// The thing, from the inventory.
    subject_pid: Uuid,
    /// The new holder; omitted = close / remove / revoke where that is allowed.
    #[serde(default)]
    to_worker_pid: Option<Uuid>,
    #[serde(default)]
    note: Option<String>,
}

/// `POST /api/movements/{pid}/handover` — reassign (or close) one thing the
/// leaver holds. A direct report needs a new manager; access can only be
/// revoked. Recorded in the audit trail, and the new holder is told.
#[debug_handler]
async fn reassign(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ReassignPayload>,
) -> Result<Response> {
    let (movement, leaver) = leaver_movement(&ctx, &caller, &pid).await?;
    if movement.status != "open" {
        return Err(unprocessable(&format!(
            "that record is already {}",
            movement.status
        )));
    }
    let kind = HandoverKind::parse(&payload.kind).ok_or_else(|| unprocessable("unknown kind"))?;
    validate_handover(kind, payload.to_worker_pid, leaver.pid).map_err(|e| unprocessable(&e))?;
    let held = inventory(&ctx, &movement, &leaver)
        .await?
        .into_iter()
        .find(|h| h.kind == kind && h.subject == payload.subject_pid)
        .ok_or_else(|| unprocessable("the leaver does not hold that"))?;
    let new = match payload.to_worker_pid {
        Some(to) => Some(successor(&ctx, to, &leaver, movement.effective_on).await?),
        None => None,
    };
    let managers = manager_map(&ctx).await?;
    let verb = perform(
        &ctx,
        &caller,
        &movement,
        &leaver,
        &held,
        new.as_ref(),
        payload
            .note
            .as_deref()
            .map(str::trim)
            .filter(|n| !n.is_empty()),
        &managers,
    )
    .await?;
    let remaining = inventory(&ctx, &movement, &leaver).await?.len();
    format::json(serde_json::json!({ "action": verb, "remaining": remaining }))
}

/// `POST /api/movements/{pid}/handover/all` body.
#[derive(Debug, Deserialize)]
struct AllPayload {
    /// Who takes over everything that can be handed over.
    to_worker_pid: Uuid,
}

/// `POST /api/movements/{pid}/handover/all` — hand everything that can be
/// handed over to one successor (people they manage, groups, rota seats,
/// swaps, shifts, mentorships, tasks, backup roles) and revoke their access.
/// Each item is its own audited action; whatever cannot move (for example a
/// group lead whose successor is not in the group) is reported, not skipped
/// silently.
#[debug_handler]
async fn reassign_all(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<AllPayload>,
) -> Result<Response> {
    let (movement, leaver) = leaver_movement(&ctx, &caller, &pid).await?;
    if movement.status != "open" {
        return Err(unprocessable(&format!(
            "that record is already {}",
            movement.status
        )));
    }
    validate_handover(HandoverKind::Task, Some(payload.to_worker_pid), leaver.pid)
        .map_err(|e| unprocessable(&e))?;
    let new = successor(&ctx, payload.to_worker_pid, &leaver, movement.effective_on).await?;
    let mut moved = 0usize;
    let mut revoked = 0usize;
    let mut failed: Vec<serde_json::Value> = Vec::new();
    for held in inventory(&ctx, &movement, &leaver).await? {
        let target = if held.kind.can_reassign() {
            Some(&new)
        } else {
            None
        };
        // A direct report may have become the successor's manager meanwhile: rebuild each time.
        let managers = manager_map(&ctx).await?;
        match perform(
            &ctx, &caller, &movement, &leaver, &held, target, None, &managers,
        )
        .await
        {
            Ok("revoked") => revoked += 1,
            Ok(_) => moved += 1,
            Err(e) => failed.push(serde_json::json!({
                "kind": held.kind.as_str(),
                "subject_pid": held.subject,
                "label": held.label,
                "reason": e.to_string(),
            })),
        }
    }
    let remaining = inventory(&ctx, &movement, &leaver).await?.len();
    format::json(serde_json::json!({
        "handed_over": moved,
        "access_revoked": revoked,
        "failed": failed,
        "remaining": remaining,
    }))
}

/// `GET /api/movements/{pid}/handover/actions` — the audit trail: every
/// reassignment, closure and revocation, oldest first.
#[debug_handler]
async fn actions(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (movement, _) = leaver_movement(&ctx, &caller, &pid).await?;
    let rows = handover_actions::Entity::find()
        .filter(handover_actions::Column::MovementPid.eq(movement.pid))
        .order_by_asc(handover_actions::Column::Id)
        .all(&ctx.db)
        .await?;
    let ids: Vec<Uuid> = rows.iter().filter_map(|r| r.to_worker).collect();
    let names: HashMap<Uuid, String> = workers::Entity::find()
        .filter(workers::Column::Pid.is_in(ids))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| (w.pid, w.display_name))
        .collect();
    format::json(
        rows.iter()
            .map(|r| {
                serde_json::json!({
                    "kind": r.kind,
                    "subject_pid": r.subject_pid,
                    "label": r.subject_label,
                    "action": r.action,
                    "to_worker_pid": r.to_worker,
                    "to_worker_name": r.to_worker.and_then(|t| names.get(&t)),
                    "note": r.note,
                    "performed_by": r.performed_by,
                    "performed_at": r.performed_at,
                })
            })
            .collect::<Vec<_>>(),
    )
}

/// The handover routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/movements/{pid}/handover", get(get_handover))
        .add("/movements/{pid}/handover", post(reassign))
        .add("/movements/{pid}/handover/all", post(reassign_all))
        .add("/movements/{pid}/handover/actions", get(actions))
}
