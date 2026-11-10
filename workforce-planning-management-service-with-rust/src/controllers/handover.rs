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

use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use super::movements::find_movement;
use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::models::_entities::{movements, workers};
use crate::models::records;
use crate::rules::metrics::is_employed_on;
use crate::rules::movements::{Disposition, HandoverKind, disposition, validate_handover};
use crate::rules::org::would_create_cycle;

/// One thing the leaver still holds.
#[derive(Debug, Clone)]
pub(super) struct Held {
    kind: HandoverKind,
    /// The thing's own id (a row pid, or the rota's pid for a seat).
    subject: Uuid,
    label: String,
}

/// Everything `leaver` still holds, as of the movement's last day. A name comes from the live record it points at, or is
/// empty when that record is gone.
#[allow(clippy::too_many_lines)] // one block per kind of thing a leaver can hold
pub(super) async fn inventory(
    ctx: &AppContext,
    movement: &movements::Model,
    leaver: &workers::Model,
) -> Result<Vec<Held>> {
    let pool = db::pool(ctx);
    let last_day = movement.effective_on;
    let mut out = Vec::new();

    // Direct reports still employed.
    for w in sqlx::query!(
        "SELECT pid, display_name, hired_on, terminated_on FROM workers
          WHERE manager_pid = $1 AND deleted_at IS NULL ORDER BY id",
        leaver.pid,
    )
    .fetch_all(pool)
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
    for d in sqlx::query!(
        r#"SELECT d.pid, COALESCE(w.display_name, '') AS "name!"
             FROM dotted_line_reports d
             LEFT JOIN workers w ON w.pid = d.report_pid AND w.deleted_at IS NULL
            WHERE d.manager_pid = $1 AND d.ended_at IS NULL
            ORDER BY d.id"#,
        leaver.pid,
    )
    .fetch_all(pool)
    .await?
    {
        out.push(Held {
            kind: HandoverKind::DottedLine,
            subject: d.pid,
            label: d.name,
        });
    }
    // Groups they lead.
    for g in sqlx::query!(
        r#"SELECT m.pid, COALESCE(g.name, '') AS "name!"
             FROM group_members m
             LEFT JOIN groups g ON g.pid = m.group_pid
            WHERE m.worker_pid = $1 AND m.role = 'lead' AND m.left_at IS NULL
            ORDER BY m.id"#,
        leaver.pid,
    )
    .fetch_all(pool)
    .await?
    {
        out.push(Held {
            kind: HandoverKind::GroupLead,
            subject: g.pid,
            label: g.name,
        });
    }
    // Their seat in on-call rotas.
    for r in sqlx::query!(
        "SELECT r.pid, r.name
           FROM rota_members m
           JOIN rotas r ON r.pid = m.rota_pid AND r.deleted_at IS NULL
          WHERE m.worker_pid = $1
          ORDER BY m.id",
        leaver.pid,
    )
    .fetch_all(pool)
    .await?
    {
        out.push(Held {
            kind: HandoverKind::RotaMembership,
            subject: r.pid,
            label: r.name,
        });
    }
    // On-call swaps still ahead.
    for o in sqlx::query!(
        r#"SELECT o.pid, o.starts_on, o.ends_on, COALESCE(r.name, '') AS "name!"
             FROM rota_overrides o
             LEFT JOIN rotas r ON r.pid = o.rota_pid
            WHERE o.worker_pid = $1 AND o.ends_on >= $2
            ORDER BY o.id"#,
        leaver.pid,
        last_day,
    )
    .fetch_all(pool)
    .await?
    {
        out.push(Held {
            kind: HandoverKind::RotaSwap,
            subject: o.pid,
            label: format!("{} {}–{}", o.name, o.starts_on, o.ends_on),
        });
    }
    // Named as someone's backup.
    for b in sqlx::query!(
        r#"SELECT b.pid, COALESCE(w.display_name, '') AS "name!"
             FROM worker_backups b
             LEFT JOIN workers w ON w.pid = b.worker_pid AND w.deleted_at IS NULL
            WHERE b.backup_pid = $1 AND b.deleted_at IS NULL
            ORDER BY b.id"#,
        leaver.pid,
    )
    .fetch_all(pool)
    .await?
    {
        out.push(Held {
            kind: HandoverKind::Backup,
            subject: b.pid,
            label: b.name,
        });
    }
    // Mentorships they give.
    for m in sqlx::query!(
        r#"SELECT m.pid, COALESCE(w.display_name, '') AS "name!"
             FROM mentorships m
             LEFT JOIN workers w ON w.pid = m.mentee_pid AND w.deleted_at IS NULL
            WHERE m.mentor_pid = $1 AND m.deleted_at IS NULL AND m.status IN ('proposed', 'active')
            ORDER BY m.id"#,
        leaver.pid,
    )
    .fetch_all(pool)
    .await?
    {
        out.push(Held {
            kind: HandoverKind::Mentorship,
            subject: m.pid,
            label: m.name,
        });
    }
    // Future shifts.
    for a in sqlx::query!(
        r#"SELECT a.pid, s.department, s.starts_at AS "starts_at: DateTime<FixedOffset>"
             FROM shift_assignments a
             JOIN shifts s ON s.pid = a.shift_pid AND s.deleted_at IS NULL
            WHERE a.worker_pid = $1 AND a.deleted_at IS NULL
            ORDER BY a.id"#,
        leaver.pid,
    )
    .fetch_all(pool)
    .await?
    {
        if a.starts_at.date_naive() >= last_day {
            out.push(Held {
                kind: HandoverKind::Shift,
                subject: a.pid,
                label: format!("{} {}", a.department, a.starts_at.format("%Y-%m-%d %H:%M")),
            });
        }
    }
    // Checklist tasks assigned to them (not on their own record).
    for i in sqlx::query!(
        "SELECT i.pid, i.title
           FROM movement_items i
           JOIN movements m ON m.pid = i.movement_pid
          WHERE i.assignee_pid = $1 AND i.done_on IS NULL AND i.skipped_reason IS NULL
            AND m.status = 'open' AND m.worker_pid <> $1
          ORDER BY i.id",
        leaver.pid,
    )
    .fetch_all(pool)
    .await?
    {
        out.push(Held {
            kind: HandoverKind::Task,
            subject: i.pid,
            label: i.title,
        });
    }
    // Access: organization roles that run past the last day.
    for m in sqlx::query!(
        "SELECT pid, organization_ref, role, ends_on FROM organization_memberships
          WHERE person_ref = $1 AND deleted_at IS NULL ORDER BY id",
        leaver.person_ref,
    )
    .fetch_all(pool)
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
    let w = records::worker_by_pid(ctx, to)
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

/// Apply one reassignment (or closure) on `tx`; returns the verb recorded.
async fn apply(
    tx: &mut sqlx::PgConnection,
    held: &Held,
    to: Option<&workers::Model>,
    leaver: &workers::Model,
    last_day: NaiveDate,
    manager_of: &HashMap<Uuid, Uuid>,
) -> Result<&'static str> {
    let now: DateTime<FixedOffset> = Utc::now().into();
    let new = to.map(|w| w.pid);
    match held.kind {
        HandoverKind::DirectReport => {
            let new = to.ok_or_else(|| unprocessable("a direct report needs a new manager"))?;
            if would_create_cycle(held.subject, new.pid, manager_of) {
                return Err(unprocessable(
                    "that would put the new manager below their own report",
                ));
            }
            // The live report, read on the transaction.
            let id = sqlx::query_scalar!(
                "SELECT id FROM workers WHERE pid = $1 AND deleted_at IS NULL",
                held.subject,
            )
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(Error::NotFound)?;
            sqlx::query!(
                "UPDATE workers SET manager_pid = $2 WHERE id = $1",
                id,
                new.pid
            )
            .execute(&mut *tx)
            .await?;
            Ok("reassigned")
        }
        HandoverKind::DottedLine => hand_over_dotted_line(tx, held.subject, new, now).await,
        HandoverKind::GroupLead => hand_over_group_lead(tx, held.subject, new, now).await,
        HandoverKind::RotaMembership => {
            hand_over_rota_membership(tx, held.subject, leaver.pid, new).await
        }
        HandoverKind::RotaSwap => hand_over_rota_swap(tx, held.subject, new).await,
        HandoverKind::Backup => hand_over_backup(tx, held.subject, new, now).await,
        HandoverKind::Mentorship => hand_over_mentorship(tx, held.subject, new, last_day).await,
        HandoverKind::Shift => hand_over_shift(tx, held.subject, new, now).await,
        HandoverKind::Task => {
            sqlx::query_scalar!(
                "UPDATE movement_items SET assignee_pid = $2 WHERE pid = $1 RETURNING id",
                held.subject,
                new,
            )
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(Error::NotFound)?;
            Ok(if new.is_some() {
                "reassigned"
            } else {
                "closed"
            })
        }
        HandoverKind::Access => {
            sqlx::query_scalar!(
                "UPDATE organization_memberships SET ends_on = GREATEST(starts_on, $2) WHERE pid = $1 RETURNING id",
                held.subject,
                last_day,
            )
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(Error::NotFound)?;
            Ok("revoked")
        }
    }
}

/// The word a [`Disposition`] is reported as.
const fn verb_of(d: Disposition) -> &'static str {
    match d {
        Disposition::Reassign(_) => "reassigned",
        Disposition::Close => "closed",
    }
}

/// A dotted-line report: the line moves to the new manager, or ends when it would not move
/// (nobody to take it, or the new manager is the report); if the new manager already has a live
/// line to that report, this one ends.
async fn hand_over_dotted_line(
    tx: &mut sqlx::PgConnection,
    subject: Uuid,
    new: Option<Uuid>,
    now: DateTime<FixedOffset>,
) -> Result<&'static str> {
    let row = sqlx::query!(
        "SELECT id, report_pid FROM dotted_line_reports WHERE pid = $1",
        subject
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;
    match disposition(new, new == Some(row.report_pid)) {
        Disposition::Reassign(new) => {
            let already = sqlx::query_scalar!(
                r#"SELECT 1 AS "one!" FROM dotted_line_reports
                    WHERE report_pid = $1 AND manager_pid = $2 AND ended_at IS NULL LIMIT 1"#,
                row.report_pid,
                new,
            )
            .fetch_optional(&mut *tx)
            .await?
            .is_some();
            if already {
                sqlx::query!(
                    "UPDATE dotted_line_reports SET ended_at = $2 WHERE id = $1",
                    row.id,
                    now
                )
                .execute(&mut *tx)
                .await?;
            } else {
                sqlx::query!(
                    "UPDATE dotted_line_reports SET manager_pid = $2 WHERE id = $1",
                    row.id,
                    new
                )
                .execute(&mut *tx)
                .await?;
            }
            Ok("reassigned")
        }
        Disposition::Close => {
            sqlx::query!(
                "UPDATE dotted_line_reports SET ended_at = $2 WHERE id = $1",
                row.id,
                now
            )
            .execute(&mut *tx)
            .await?;
            Ok("closed")
        }
    }
}

/// A group lead: the leaver leaves the group; the new lead, who must already be a member, takes
/// the lead role.
async fn hand_over_group_lead(
    tx: &mut sqlx::PgConnection,
    subject: Uuid,
    new: Option<Uuid>,
    now: DateTime<FixedOffset>,
) -> Result<&'static str> {
    let group = sqlx::query_scalar!(
        "UPDATE group_members SET left_at = $2 WHERE pid = $1 RETURNING group_pid",
        subject,
        now,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;
    let Some(new) = new else {
        return Ok("closed");
    };
    let theirs = sqlx::query_scalar!(
        "SELECT id FROM group_members WHERE group_pid = $1 AND worker_pid = $2 AND left_at IS NULL LIMIT 1",
        group,
        new,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| unprocessable("the new lead must already be a member of that group"))?;
    sqlx::query!(
        "UPDATE group_members SET role = 'lead' WHERE id = $1",
        theirs
    )
    .execute(&mut *tx)
    .await?;
    Ok("reassigned")
}

/// A seat on a rota: it passes to the new worker unless they already have one.
async fn hand_over_rota_membership(
    tx: &mut sqlx::PgConnection,
    rota: Uuid,
    leaver: Uuid,
    new: Option<Uuid>,
) -> Result<&'static str> {
    let mine = sqlx::query_scalar!(
        "SELECT id FROM rota_members WHERE rota_pid = $1 AND worker_pid = $2 LIMIT 1",
        rota,
        leaver,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;
    let taken = match new {
        Some(new) => sqlx::query_scalar!(
            r#"SELECT 1 AS "one!" FROM rota_members WHERE rota_pid = $1 AND worker_pid = $2 LIMIT 1"#,
            rota,
            new,
        )
        .fetch_optional(&mut *tx)
        .await?
        .is_some(),
        None => false,
    };
    let d = disposition(new, taken);
    match d {
        Disposition::Reassign(new) => {
            sqlx::query!(
                "UPDATE rota_members SET worker_pid = $2 WHERE id = $1",
                mine,
                new
            )
            .execute(&mut *tx)
            .await?;
        }
        Disposition::Close => {
            sqlx::query!("DELETE FROM rota_members WHERE id = $1", mine)
                .execute(&mut *tx)
                .await?;
        }
    }
    Ok(verb_of(d))
}

/// A rota swap: it passes to the new worker, or is removed.
async fn hand_over_rota_swap(
    tx: &mut sqlx::PgConnection,
    subject: Uuid,
    new: Option<Uuid>,
) -> Result<&'static str> {
    let id = sqlx::query_scalar!("SELECT id FROM rota_overrides WHERE pid = $1", subject)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(Error::NotFound)?;
    let d = disposition(new, false);
    match d {
        Disposition::Reassign(new) => {
            sqlx::query!(
                "UPDATE rota_overrides SET worker_pid = $2 WHERE id = $1",
                id,
                new
            )
            .execute(&mut *tx)
            .await?;
        }
        Disposition::Close => {
            sqlx::query!("DELETE FROM rota_overrides WHERE id = $1", id)
                .execute(&mut *tx)
                .await?;
        }
    }
    Ok(verb_of(d))
}

/// A backup arrangement: the new worker backs up the owner instead, unless they are the owner or
/// already back them up, in which case it is removed.
async fn hand_over_backup(
    tx: &mut sqlx::PgConnection,
    subject: Uuid,
    new: Option<Uuid>,
    now: DateTime<FixedOffset>,
) -> Result<&'static str> {
    let row = sqlx::query!(
        "SELECT id, worker_pid FROM worker_backups WHERE pid = $1",
        subject
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;
    let owner = row.worker_pid;
    let blocked = match new {
        Some(new) if new == owner => true,
        Some(new) => sqlx::query_scalar!(
            r#"SELECT 1 AS "one!" FROM worker_backups
                WHERE worker_pid = $1 AND backup_pid = $2 AND deleted_at IS NULL LIMIT 1"#,
            owner,
            new,
        )
        .fetch_optional(&mut *tx)
        .await?
        .is_some(),
        None => false,
    };
    let d = disposition(new, blocked);
    match d {
        Disposition::Reassign(new) => {
            sqlx::query!(
                "UPDATE worker_backups SET backup_pid = $2 WHERE id = $1",
                row.id,
                new
            )
            .execute(&mut *tx)
            .await?;
        }
        Disposition::Close => {
            sqlx::query!(
                "UPDATE worker_backups SET deleted_at = $2 WHERE id = $1",
                row.id,
                now
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    Ok(verb_of(d))
}

/// A mentorship: the new worker mentors instead, unless they are the mentee, in which case it ends.
async fn hand_over_mentorship(
    tx: &mut sqlx::PgConnection,
    subject: Uuid,
    new: Option<Uuid>,
    last_day: NaiveDate,
) -> Result<&'static str> {
    let row = sqlx::query!(
        "SELECT id, mentee_pid FROM mentorships WHERE pid = $1",
        subject
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;
    let d = disposition(new, new == Some(row.mentee_pid));
    match d {
        Disposition::Reassign(new) => {
            sqlx::query!(
                "UPDATE mentorships SET mentor_pid = $2 WHERE id = $1",
                row.id,
                new
            )
            .execute(&mut *tx)
            .await?;
        }
        Disposition::Close => {
            sqlx::query!(
                "UPDATE mentorships SET status = 'ended', ended_on = $2 WHERE id = $1",
                row.id,
                last_day,
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    Ok(verb_of(d))
}

/// A shift assignment: it passes to the new worker unless they are already on that shift.
async fn hand_over_shift(
    tx: &mut sqlx::PgConnection,
    subject: Uuid,
    new: Option<Uuid>,
    now: DateTime<FixedOffset>,
) -> Result<&'static str> {
    let row = sqlx::query!(
        "SELECT id, shift_pid FROM shift_assignments WHERE pid = $1",
        subject
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;
    let taken = match new {
        Some(new) => sqlx::query_scalar!(
            r#"SELECT 1 AS "one!" FROM shift_assignments
                WHERE shift_pid = $1 AND worker_pid = $2 AND deleted_at IS NULL LIMIT 1"#,
            row.shift_pid,
            new,
        )
        .fetch_optional(&mut *tx)
        .await?
        .is_some(),
        None => false,
    };
    let d = disposition(new, taken);
    match d {
        Disposition::Reassign(new) => {
            sqlx::query!(
                "UPDATE shift_assignments SET worker_pid = $2 WHERE id = $1",
                row.id,
                new
            )
            .execute(&mut *tx)
            .await?;
        }
        Disposition::Close => {
            sqlx::query!(
                "UPDATE shift_assignments SET deleted_at = $2 WHERE id = $1",
                row.id,
                now
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    Ok(verb_of(d))
}

/// The live `manager_of` map for the cycle check.
async fn manager_map(ctx: &AppContext) -> Result<HashMap<Uuid, Uuid>> {
    Ok(sqlx::query!(
        r#"SELECT pid, manager_pid AS "manager_pid!" FROM workers
            WHERE deleted_at IS NULL AND manager_pid IS NOT NULL"#
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
    .map(|w| (w.pid, w.manager_pid))
    .collect())
}

/// Do `held` → `to` in one transaction: change, audit-trail row, audit entry,
/// and a notification to the new holder.
#[allow(clippy::too_many_arguments)] // the movement, the leaver, the item, the target and the audit inputs
#[allow(clippy::too_many_lines)] // the audit is written out once per verb
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
    let mut tx = db::pool(ctx).begin().await?;
    let verb = apply(&mut tx, held, to, leaver, movement.effective_on, managers).await?;
    sqlx::query!(
        "INSERT INTO handover_actions (pid, movement_pid, kind, subject_pid, subject_label, from_worker, to_worker, action,
                                       note, performed_by, performed_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        Uuid::new_v4(),
        movement.pid,
        held.kind.as_str(),
        held.subject,
        held.label,
        leaver.pid,
        to.map(|w| w.pid),
        verb,
        note,
        caller.actor(),
        DateTime::<FixedOffset>::from(Utc::now()),
    )
    .execute(&mut *tx)
    .await?;
    let snapshot = Some(serde_json::json!({
        "kind": held.kind.as_str(),
        "subject_pid": held.subject,
        "to_worker_pid": to.map(|w| w.pid),
    }));
    // One literal action per verb, so the audit guard can read each.
    match verb {
        "revoked" => {
            db::audit(
                &mut tx,
                "worker",
                leaver.pid,
                "handover_revoked",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
        "closed" => {
            db::audit(
                &mut tx,
                "worker",
                leaver.pid,
                "handover_closed",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
        _ => {
            db::audit(
                &mut tx,
                "worker",
                leaver.pid,
                "handover_reassigned",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
    }
    if let Some(new) = to {
        db::notify(
            &mut tx,
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
    tx.commit().await?;
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
    let rows = sqlx::query!(
        r#"SELECT kind, subject_pid, subject_label, action, to_worker, note, performed_by,
                  performed_at AS "performed_at: DateTime<FixedOffset>"
             FROM handover_actions WHERE movement_pid = $1 ORDER BY id"#,
        movement.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let ids: Vec<Uuid> = rows.iter().filter_map(|r| r.to_worker).collect();
    let names: HashMap<Uuid, String> = sqlx::query!(
        "SELECT pid, display_name FROM workers WHERE pid = ANY($1)",
        &ids
    )
    .fetch_all(db::pool(&ctx))
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
