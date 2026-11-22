//! **Reporting lines**: who a worker reports to (their *upline*), who reports to
//! them (their *downline*), and the distinction between a **direct report**
//! (straight to the manager) and an **indirect report** (through someone
//! else). Derived from `manager_pid` over the workers **employed today**;
//! pure rules in [`crate::rules::org`].
//!
//! Scoped to the caller's organizations like the org chart: a worker outside
//! the caller's scope is `404`, and people outside it never appear.

use crate::sql_time::{SqlDate, SqlTimestamp};
use axa::extract::{Json, Path};
use axa::prelude::*;
use jiff::Timestamp;
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{dotted_line_reports, workers};
use crate::models::{memberships, records};
use crate::rules::metrics::is_employed_on;
use crate::rules::org::{self, ReportKind};

/// A person on a reporting line, as these views show them.
struct Person {
    pid: Uuid,
    display_name: String,
    job_title: String,
    department: String,
    manager_pid: Option<Uuid>,
}

impl From<&workers::Model> for Person {
    fn from(w: &workers::Model) -> Self {
        Self {
            pid: w.pid,
            display_name: w.display_name.clone(),
            job_title: w.job_title.clone(),
            department: w.department.clone(),
            manager_pid: w.manager_pid,
        }
    }
}

/// Those of `pids` who are employed today and visible, by pid. Reads only them (WPM-R142): the staff of an
/// organization of 200,000 are never loaded to answer for a handful.
async fn staff_among(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pids: &[Uuid],
) -> Result<BTreeMap<Uuid, Person>> {
    let today = crate::dates::today();
    let scope = memberships::scope_organization_refs(ctx, caller.claims()).await?;
    Ok(sqlx::query_as!(
        Person,
        "SELECT pid, display_name, job_title, department, manager_pid FROM workers
          WHERE pid = ANY($1) AND deleted_at IS NULL AND hired_on <= $2 AND (terminated_on IS NULL OR terminated_on > $2)
            AND ($3::text[] IS NULL OR organization_ref = ANY($3))",
        pids,
        SqlDate(today),
        scope.as_deref(),
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
    .map(|w| (w.pid, w))
    .collect())
}

/// Everyone below `manager` who is employed today and visible, in one recursive statement, where it was one read per
/// level; `UNION` keeps each person once, so a cycle revisits no one.
async fn staff_below(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    manager: Uuid,
) -> Result<BTreeMap<Uuid, Person>> {
    let today = crate::dates::today();
    let scope = memberships::scope_organization_refs(ctx, caller.claims()).await?;
    Ok(sqlx::query_as!(
        Person,
        r#"WITH RECURSIVE staff AS (
               SELECT pid, display_name, job_title, department, manager_pid FROM workers
                WHERE deleted_at IS NULL AND hired_on <= $2 AND (terminated_on IS NULL OR terminated_on > $2)
                  AND ($3::text[] IS NULL OR organization_ref = ANY($3))),
           down AS (
               SELECT s.* FROM staff s WHERE s.manager_pid = $1
               UNION
               SELECT s.* FROM staff s JOIN down d ON s.manager_pid = d.pid)
           SELECT pid AS "pid!", display_name AS "display_name!", job_title AS "job_title!", department AS "department!",
                  manager_pid
             FROM down WHERE pid <> $1"#,
        manager,
        SqlDate(today),
        scope.as_deref(),
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
    .map(|w| (w.pid, w))
    .collect())
}

/// The worker, required to be visible to the caller (a worker who is not, or no longer, employed is
/// still addressable; one outside the caller's organizations is not).
async fn visible_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<workers::Model> {
    let worker = records::worker_by_pid(ctx, records::parse_pid(pid)?).await?;
    let scope = memberships::scope_organization_refs(ctx, caller.claims()).await?;
    if scope.is_some_and(|refs| !refs.contains(&worker.organization_ref)) {
        return Err(Error::NotFound);
    }
    Ok(worker)
}

fn person_json(w: &Person) -> serde_json::Value {
    serde_json::json!({
        "pid": w.pid,
        "display_name": w.display_name,
        "job_title": w.job_title,
        "department": w.department,
    })
}

/// `GET /api/workers/{pid}/upline` — the management chain above a worker,
/// nearest first: `level` 1 is their direct manager, 2 their manager's
/// manager, and so on up to the top.
#[debug_handler]
async fn upline(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = visible_worker(&ctx, &caller, &pid).await?;
    // One manager at a time, up from the worker, while each is employed and visible; a cycle stops.
    let today = crate::dates::today();
    let mut chain: Vec<serde_json::Value> = Vec::new();
    let mut seen = std::collections::BTreeSet::from([worker.pid]);
    let mut level = 0_usize;
    let mut manager = is_employed_on(
        today,
        worker.hired_on.get(),
        worker.terminated_on.map(SqlDate::get),
    )
    .then_some(worker.manager_pid)
    .flatten();
    while let Some(pid) = manager.filter(|m| seen.insert(*m)) {
        let Some(boss) = staff_among(&ctx, &caller, &[pid]).await?.remove(&pid) else {
            break;
        };
        // One step per manager visited, each a distinct worker held in memory.
        super::tally::bump(&mut level);
        chain.push(with_fields(
            person_json(&boss),
            serde_json::json!({
                "level": level,
                "direct_manager": chain.is_empty(),
            }),
        ));
        manager = boss.manager_pid;
    }
    format::json(serde_json::json!({
        "worker": person_json(&Person::from(&worker)),
        "upline": chain,
    }))
}

/// Query for the reports listing.
#[derive(Debug, Deserialize)]
struct KindQuery {
    /// `direct`, `indirect`, or absent for both.
    kind: Option<String>,
    /// How many people to list, nearest first (default 5,000, at most 20,000); the totals are of everyone.
    limit: Option<usize>,
}

/// Query for the downline.
#[derive(Debug, Deserialize)]
struct LimitQuery {
    /// How many people to list, nearest first (default 5,000, at most 20,000); the summary is of everyone.
    limit: Option<usize>,
}

const LIST_DEFAULT_LIMIT: usize = 5_000;
const LIST_MAX_LIMIT: usize = 20_000;

/// The page size asked for (default 5,000, at most 20,000, as the query documents); one outside 1 to 20,000 is refused, not clamped.
fn list_limit(asked: Option<usize>) -> Result<usize> {
    let limit = asked.unwrap_or(LIST_DEFAULT_LIMIT);
    if (1..=LIST_MAX_LIMIT).contains(&limit) {
        Ok(limit)
    } else {
        Err(unprocessable(&format!(
            "limit must be 1 to {LIST_MAX_LIMIT}"
        )))
    }
}

/// The downline of a manager: `(worker, depth)` for everyone below.
fn below(manager: Uuid, staff: &BTreeMap<Uuid, Person>) -> Vec<(&Person, usize)> {
    let mut reports_of: BTreeMap<Uuid, Vec<Uuid>> = BTreeMap::new();
    for w in staff.values() {
        if let Some(boss) = w.manager_pid {
            reports_of.entry(boss).or_default().push(w.pid);
        }
    }
    org::downline(manager, &reports_of)
        .into_iter()
        .filter_map(|(pid, depth)| staff.get(&pid).map(|w| (w, depth)))
        .collect()
}

fn report_json(w: &Person, depth: usize) -> serde_json::Value {
    with_fields(
        person_json(w),
        serde_json::json!({
            "depth": depth,
            "report_kind": org::report_kind(depth).map(ReportKind::as_str),
        }),
    )
}

/// `base` (an object, or null for an empty one) with the members of the object `extra` added over it.
/// A base that is neither is returned unchanged.
pub(crate) fn with_fields(
    mut base: serde_json::Value,
    extra: serde_json::Value,
) -> serde_json::Value {
    if base.is_null() {
        base = serde_json::json!({});
    }
    if let (serde_json::Value::Object(target), serde_json::Value::Object(more)) = (&mut base, extra)
    {
        target.extend(more);
    }
    base
}

/// `GET /api/workers/{pid}/downline` — everyone below a manager: their
/// **direct reports** (depth 1) and **indirect reports** (reports of
/// reports, any depth), each marked with `report_kind` and `depth`, plus the
/// counts.
#[debug_handler]
async fn downline(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axa::extract::Query(query): axa::extract::Query<LimitQuery>,
) -> Result<Response> {
    let limit = list_limit(query.limit)?;
    let manager = visible_worker(&ctx, &caller, &pid).await?;
    let staff = staff_below(&ctx, &caller, manager.pid).await?;
    let team = below(manager.pid, &staff);
    let direct = team.iter().filter(|(_, d)| *d == 1).count();
    format::json(serde_json::json!({
        "manager": person_json(&Person::from(&manager)),
        "summary": { "direct": direct, "indirect": team.iter().filter(|(_, d)| *d != 1).count(), "total": team.len() },
        "downline": team.iter().take(limit).map(|(w, d)| report_json(w, *d)).collect::<Vec<_>>(),
        "truncated": team.len() > limit,
    }))
}

/// `GET /api/workers/{pid}/reports?kind=direct|indirect` — just the direct or
/// just the indirect reports (both when `kind` is absent).
#[debug_handler]
async fn reports(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axa::extract::Query(query): axa::extract::Query<KindQuery>,
) -> Result<Response> {
    let wanted = match query.kind.as_deref() {
        None => None,
        Some("direct") => Some(ReportKind::Direct),
        Some("indirect") => Some(ReportKind::Indirect),
        Some(_) => return Err(unprocessable("kind must be direct or indirect")),
    };
    let manager = visible_worker(&ctx, &caller, &pid).await?;
    let staff = staff_below(&ctx, &caller, manager.pid).await?;
    let limit = list_limit(query.limit)?;
    let matching: Vec<(&Person, usize)> = below(manager.pid, &staff)
        .into_iter()
        .filter(|(_, d)| wanted.is_none_or(|k| org::report_kind(*d) == Some(k)))
        .collect();
    let team: Vec<serde_json::Value> = matching
        .iter()
        .take(limit)
        .map(|(w, d)| report_json(w, *d))
        .collect();
    format::json(serde_json::json!({
        "manager": person_json(&Person::from(&manager)),
        "kind": query.kind,
        "reports": team,
        "total": matching.len(),
        "truncated": matching.len() > limit,
    }))
}

fn dotted_json(d: &dotted_line_reports::Model, person: Option<&Person>) -> serde_json::Value {
    with_fields(
        person.map_or_else(|| serde_json::json!({}), person_json),
        serde_json::json!({
            "note": d.note,
            "started_at": d.started_at,
            "ended_at": d.ended_at,
            "current": d.ended_at.is_none(),
            "on_behalf": d.on_behalf,
        }),
    )
}

/// `GET /api/workers/{pid}/dotted-line?include_past=` — a worker's
/// **dotted-line managers** (who they report to on a secondary line) and
/// **dotted-line reports** (who reports to them that way). A dotted line is
/// separate from the solid line: it changes no org chart and, by itself,
/// grants no access to anyone's private or manager-visible aspirations.
#[debug_handler]
async fn dotted_line(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axa::extract::Query(query): axa::extract::Query<PastQuery>,
) -> Result<Response> {
    let worker = visible_worker(&ctx, &caller, &pid).await?;
    let rows = sqlx::query_as!(
        dotted_line_reports::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, report_pid, manager_pid, note,
                  started_at AS "started_at: _", ended_at AS "ended_at: _", recorded_by, on_behalf
             FROM dotted_line_reports
            WHERE (report_pid = $1 OR manager_pid = $1) AND ($2 OR ended_at IS NULL)
            ORDER BY started_at DESC"#,
        worker.pid,
        query.include_past,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let others: Vec<Uuid> = rows
        .iter()
        .map(|d| {
            if d.report_pid == worker.pid {
                d.manager_pid
            } else {
                d.report_pid
            }
        })
        .collect();
    let staff = staff_among(&ctx, &caller, &others).await?;
    let managers: Vec<serde_json::Value> = rows
        .iter()
        .filter(|d| d.report_pid == worker.pid)
        .filter_map(|d| staff.get(&d.manager_pid).map(|m| dotted_json(d, Some(m))))
        .collect();
    let reports: Vec<serde_json::Value> = rows
        .iter()
        .filter(|d| d.manager_pid == worker.pid)
        .filter_map(|d| staff.get(&d.report_pid).map(|r| dotted_json(d, Some(r))))
        .collect();
    format::json(serde_json::json!({
        "worker": person_json(&Person::from(&worker)),
        "dotted_line_managers": managers,
        "dotted_line_reports": reports,
    }))
}

/// Query for past entries.
#[derive(Debug, Deserialize)]
struct PastQuery {
    #[serde(default)]
    include_past: bool,
}

/// `POST /api/workers/{pid}/dotted-line-managers` body.
#[derive(Debug, Deserialize)]
struct DottedPayload {
    /// The dotted-line manager — anyone employed, not only the solid-line chain.
    manager_pid: Uuid,
    #[serde(default)]
    note: Option<String>,
}

/// `POST /api/workers/{pid}/dotted-line-managers` — give a worker a
/// dotted-line manager (they may have several). Choosing one they already have
/// only updates the note. A write to the *report's* record: they manage their
/// own, HR on their behalf.
#[debug_handler]
async fn add_dotted_manager(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<DottedPayload>,
) -> Result<Response> {
    let report = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&report),
    )
    .map_err(record_rejection)?;
    org::validate_dotted_line(&report.pid, &payload.manager_pid).map_err(|e| unprocessable(&e))?;
    if payload
        .note
        .as_ref()
        .is_some_and(|n| n.chars().count() > 500)
    {
        return Err(unprocessable("note is longer than 500 characters"));
    }
    let staff = staff_among(&ctx, &caller, &[payload.manager_pid]).await?;
    if !staff.contains_key(&payload.manager_pid) {
        return Err(unprocessable(
            "the dotted-line manager must be an employed worker you can see",
        ));
    }
    let mut tx = db::pool(&ctx).begin().await?;
    // One current line per report and manager: one upsert on that partial index. Choosing one already there changes
    // only its note.
    let row = sqlx::query_as!(
        dotted_line_reports::Model,
        r#"INSERT INTO dotted_line_reports (pid, report_pid, manager_pid, note, started_at, recorded_by, on_behalf)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           ON CONFLICT (report_pid, manager_pid) WHERE ended_at IS NULL DO UPDATE SET note = EXCLUDED.note
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, report_pid, manager_pid, note,
                  started_at AS "started_at: _", ended_at AS "ended_at: _", recorded_by, on_behalf"#,
        Uuid::new_v4(),
        report.pid,
        payload.manager_pid,
        payload.note,
        SqlTimestamp(Timestamp::now()),
        caller.actor(),
        auth::acting_for_other(&caller, &report.person_ref),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        report.pid,
        "dotted_line_added",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(dotted_json(&row, staff.get(&row.manager_pid)))
}

/// `DELETE /api/workers/{pid}/dotted-line-managers/{manager_pid}` — end a
/// dotted-line relationship; it is closed, not deleted.
#[debug_handler]
async fn end_dotted_manager(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, manager_pid)): Path<(String, String)>,
) -> Result<Response> {
    let report = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&report),
    )
    .map_err(record_rejection)?;
    let mut tx = db::pool(&ctx).begin().await?;
    let ended = sqlx::query!(
        "UPDATE dotted_line_reports SET ended_at = $3 WHERE report_pid = $1 AND manager_pid = $2 AND ended_at IS NULL",
        report.pid,
        records::parse_pid(&manager_pid)?,
        SqlTimestamp(Timestamp::now()),
    )
    .execute(&mut *tx)
    .await?;
    if ended.rows_affected() == 0 {
        return Err(Error::NotFound);
    }
    db::audit(
        &mut tx,
        "worker",
        report.pid,
        "dotted_line_ended",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}

/// The reporting-line routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/upline", get(upline))
        .add("/workers/{pid}/downline", get(downline))
        .add("/workers/{pid}/reports", get(reports))
        .add("/workers/{pid}/dotted-line", get(dotted_line))
        .add(
            "/workers/{pid}/dotted-line-managers",
            post(add_dotted_manager),
        )
        .add(
            "/workers/{pid}/dotted-line-managers/{manager_pid}",
            delete(end_dotted_manager),
        )
}

#[cfg(test)]
mod tests {
    use super::{LIST_DEFAULT_LIMIT, LIST_MAX_LIMIT, list_limit};

    #[test]
    fn a_page_size_outside_one_to_the_maximum_is_refused_not_clamped() {
        assert_eq!(list_limit(None).ok(), Some(LIST_DEFAULT_LIMIT));
        assert_eq!(list_limit(Some(1)).ok(), Some(1));
        assert_eq!(list_limit(Some(LIST_MAX_LIMIT)).ok(), Some(LIST_MAX_LIMIT));
        assert!(list_limit(Some(0)).is_err());
        assert!(list_limit(Some(LIST_MAX_LIMIT + 1)).is_err());
        assert!(list_limit(Some(usize::MAX)).is_err());
    }
}
