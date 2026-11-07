//! The **employee directory**: `GET /api/directory` — who works where,
//! searchable, with nothing sensitive in it (see [`crate::rules::directory`]).
//!
//! Lists workers *employed today* ([`metric_rules::is_employed_on`], the one
//! definition of "employed") in the organizations the caller can read.
//! A manager's name is shown only when the manager is in the same readable
//! set, so the directory never names someone outside the caller's scope.
//!
//! Anyone **on call today** in a rota the caller can read is marked with the
//! rota's name (`rotas::on_call_today`).
//!
//! A worker on **approved leave today** is shown as away — never the kind
//! of leave — with whoever covers for them ([`cover_rules::resolve`]: their
//! best-ranked backup who is employed and not away themselves). Backups
//! outside the caller's readable set are not considered, so a cover is
//! only ever someone the caller could find in the directory anyway.

use axa::prelude::*;
use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::{Page, Window};
use crate::auth::MaybeAuthUser;
use crate::db::pool;
use crate::models::memberships;
use crate::rules::cover::{self as cover_rules, Candidate};
use crate::rules::directory as rules;

/// Default page size: a directory is browsed, not exported.
const DIRECTORY_DEFAULT_LIMIT: u64 = 50;

/// Query for the directory.
#[derive(Debug, Deserialize)]
struct DirectoryParams {
    /// Free-text search over name, title, department, location, manager.
    #[serde(default)]
    q: Option<String>,
    /// Exact department (case-insensitive).
    #[serde(default)]
    department: Option<String>,
    /// Page size; absent, zero or unparseable ⇒ the default.
    #[serde(default)]
    limit: Option<String>,
    /// Rows to skip; absent ⇒ 0.
    #[serde(default)]
    offset: Option<String>,
}

/// The whole of a page: how many workers match, and the pids of the page in order.
struct Matches {
    total: u64,
    page: Vec<Uuid>,
}

/// The listing's filter, in SQL, with the same meaning as [`rules::search`]: every whitespace-separated
/// term must appear (ignoring case) in the name, title, department, location, the readable manager's
/// name or the name of a rota the worker is on call for today; and the department, if given, matches
/// ignoring case.
///
/// The SQL is one fixed statement so that it is checked at build time. The terms are an array, and a
/// worker matches when no term fails. `IS NOT TRUE` is what makes that sound: a missing location or
/// manager makes the test NULL rather than false, and a plain `NOT` would let that term through.
///
/// A term is bound as a value, never spliced, and `strpos` takes no pattern, so a `%` or `_` in it is
/// just a character. `readable` is the one set the listing and a manager's name are drawn from, and it
/// is `NOT MATERIALIZED` so that a manager is found by the primary key and not by a scan per worker.
/// One statement counts and pages, so the total cannot disagree with the rows.
async fn matches(
    ctx: &AppContext,
    refs: Option<&[String]>,
    today: NaiveDate,
    params: &DirectoryParams,
    on_call: &HashMap<Uuid, Vec<String>>,
    window: Window,
) -> Result<Matches> {
    let (limit, offset) = window.bounds();
    let department = params
        .department
        .as_deref()
        .filter(|d| !d.is_empty())
        .map(str::to_lowercase);
    let terms: Vec<String> = params
        .q
        .as_deref()
        .unwrap_or("")
        .split_whitespace()
        .map(str::to_lowercase)
        .collect();
    // One row per rota: lowered here, as the terms are, so the two are compared alike.
    let (rota_pids, rota_names): (Vec<Uuid>, Vec<String>) = on_call
        .iter()
        .flat_map(|(pid, rotas)| rotas.iter().map(|r| (*pid, r.to_lowercase())))
        .unzip();

    let found = sqlx::query!(
        r#"
        WITH readable AS NOT MATERIALIZED (
            SELECT pid, manager_pid, display_name, job_title, department, location
            FROM workers
            WHERE deleted_at IS NULL
              AND hired_on <= $1
              AND (terminated_on IS NULL OR terminated_on > $1)
              AND ($2::text[] IS NULL OR organization_ref = ANY($2))
        ),
        matching AS (
            SELECT w.pid, lower(w.display_name) COLLATE "C" AS sort_name
            FROM readable w
            LEFT JOIN readable m ON m.pid = w.manager_pid
            WHERE ($3::text IS NULL OR lower(w.department) = $3)
              AND NOT EXISTS (
                  SELECT 1
                  FROM unnest($4::text[]) AS t(term)
                  WHERE (
                      strpos(lower(w.display_name), t.term) > 0
                      OR strpos(lower(w.job_title), t.term) > 0
                      OR strpos(lower(w.department), t.term) > 0
                      OR strpos(lower(w.location), t.term) > 0
                      OR strpos(lower(m.display_name), t.term) > 0
                      OR EXISTS (
                          SELECT 1
                          FROM unnest($5::uuid[], $6::text[]) AS rota(pid, name)
                          WHERE rota.pid = w.pid AND strpos(rota.name, t.term) > 0
                      )
                  ) IS NOT TRUE
              )
        )
        SELECT
            (SELECT count(*) FROM matching) AS "total!",
            ARRAY(
                SELECT pid FROM matching ORDER BY sort_name, pid LIMIT $7 OFFSET $8
            ) AS "page!"
        "#,
        today,
        refs,
        department,
        &terms,
        &rota_pids,
        &rota_names,
        limit,
        offset,
    )
    .fetch_one(pool(ctx))
    .await?;
    Ok(Matches {
        total: found.total.unsigned_abs(),
        page: found.page,
    })
}

/// Who among `pids` is on approved leave `today`.
async fn away_among(ctx: &AppContext, pids: &[Uuid], today: NaiveDate) -> Result<HashSet<Uuid>> {
    let away = sqlx::query_scalar!(
        "SELECT worker_pid FROM leave_requests
         WHERE worker_pid = ANY($1)
           AND status = 'approved'
           AND deleted_at IS NULL
           AND start_on <= $2
           AND end_on >= $2",
        pids,
        today,
    )
    .fetch_all(pool(ctx))
    .await?;
    Ok(away.into_iter().collect())
}

/// `GET /api/directory?q=&department=&limit=&offset=` — directory entries
/// ordered by name, with `x-total-count` of all matches.
///
/// The search, the order and the page are done by the database, so the work is the size of the page
/// and not of the organization (WPM-D90); the leave, backups and managers are read for the page only.
#[debug_handler]
#[allow(clippy::too_many_lines)] // the page's query, then the page's leave, backups and managers
async fn directory(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(params): axa::extract::Query<DirectoryParams>,
) -> Result<Response> {
    let page = Page {
        limit: params.limit.clone(),
        offset: params.offset.clone(),
    };
    let window = super::resolve(&page, DIRECTORY_DEFAULT_LIMIT)?;

    let refs = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let refs = refs.as_deref();
    let today = chrono::Utc::now().date_naive();
    let on_call = super::rotas::on_call_today(&ctx, &caller).await?;

    let found = matches(&ctx, refs, today, &params, &on_call, window).await?;
    let staff = sqlx::query!(
        r#"
        SELECT w.pid, w.display_name, w.job_title, w.department, w.location,
               w.organization_ref, w.manager_pid
        FROM unnest($1::uuid[]) WITH ORDINALITY AS page(pid, position)
        JOIN workers w ON w.pid = page.pid
        ORDER BY page.position
        "#,
        &found.page,
    )
    .fetch_all(pool(&ctx))
    .await?;

    // Who is away today: approved leave covering today, for this page.
    let mut away = away_among(&ctx, &found.page, today).await?;
    let mut backups_by_worker: HashMap<Uuid, Vec<_>> = HashMap::new();
    if !away.is_empty() {
        for b in sqlx::query!(
            "SELECT worker_pid, backup_pid, priority, starts_on, ends_on
             FROM worker_backups
             WHERE worker_pid = ANY($1) AND deleted_at IS NULL",
            &away.iter().copied().collect::<Vec<_>>(),
        )
        .fetch_all(pool(&ctx))
        .await?
        {
            backups_by_worker.entry(b.worker_pid).or_default().push(b);
        }
    }

    // The names a page may show beyond its own: managers and backups, only if in the readable set.
    // The same rule as `readable` in [`matches`].
    let mut wanted: Vec<Uuid> = staff.iter().filter_map(|w| w.manager_pid).collect();
    wanted.extend(backups_by_worker.values().flatten().map(|b| b.backup_pid));
    let known: HashMap<Uuid, String> = sqlx::query!(
        r#"
        SELECT pid, display_name
        FROM workers
        WHERE pid = ANY($1)
          AND deleted_at IS NULL
          AND hired_on <= $2
          AND (terminated_on IS NULL OR terminated_on > $2)
          AND ($3::text[] IS NULL OR organization_ref = ANY($3))
        "#,
        &wanted,
        today,
        refs,
    )
    .fetch_all(pool(&ctx))
    .await?
    .into_iter()
    .map(|w| (w.pid, w.display_name))
    .collect();
    let backup_pids: Vec<Uuid> = backups_by_worker
        .values()
        .flatten()
        .map(|b| b.backup_pid)
        .filter(|pid| known.contains_key(pid))
        .collect();
    away.extend(away_among(&ctx, &backup_pids, today).await?);

    let slice: Vec<rules::Entry> = staff
        .iter()
        .map(|w| rules::Entry {
            pid: w.pid,
            display_name: w.display_name.clone(),
            job_title: w.job_title.clone(),
            department: w.department.clone(),
            location: w.location.clone(),
            organization_ref: w.organization_ref.clone(),
            manager_name: w.manager_pid.and_then(|m| known.get(&m)).cloned(),
            on_call: on_call.get(&w.pid).cloned().unwrap_or_default(),
            away_today: away.contains(&w.pid),
            covered_by: if away.contains(&w.pid) {
                // Only backups in the readable set (`known`) can cover here.
                let candidates: Vec<Candidate> = backups_by_worker
                    .get(&w.pid)
                    .into_iter()
                    .flatten()
                    .map(|b| Candidate {
                        backup: b.backup_pid,
                        priority: b.priority,
                        starts_on: b.starts_on,
                        ends_on: b.ends_on,
                        employed: known.contains_key(&b.backup_pid),
                        on_leave: away.contains(&b.backup_pid),
                    })
                    .collect();
                cover_rules::resolve(&candidates, today)
                    .and_then(|id| known.get(&id))
                    .cloned()
            } else {
                None
            },
        })
        .collect();
    Ok(window.respond(i64::try_from(found.total).unwrap_or(i64::MAX), slice))
}

/// The directory route.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/directory", get(directory))
}
