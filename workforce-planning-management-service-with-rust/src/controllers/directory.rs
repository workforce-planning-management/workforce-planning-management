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

use chrono::NaiveDate;
use loco_rs::prelude::*;
use sea_orm::sea_query::{Alias, Expr, ExprTrait, Func, Query, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    Select,
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::{Page, with_page_headers};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{leave_requests, worker_backups, workers};
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
    limit: Option<u64>,
    /// Rows to skip; absent ⇒ 0.
    #[serde(default)]
    offset: Option<u64>,
}

/// Workers employed `today` in the readable organizations: the one set every part of the directory
/// (the listing, a manager's name, a cover) is drawn from.
fn employed_scope(refs: Option<&[String]>, today: NaiveDate) -> Select<workers::Entity> {
    let mut query = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::HiredOn.lte(today))
        .filter(
            Condition::any()
                .add(workers::Column::TerminatedOn.is_null())
                .add(workers::Column::TerminatedOn.gt(today)),
        );
    if let Some(refs) = refs {
        query = query.filter(workers::Column::OrganizationRef.is_in(refs.iter().cloned()));
    }
    query
}

/// `strpos(lower(column), term) > 0`: the term appears in the column, ignoring case. A term is
/// bound as a value, never spliced, so a `%` or `_` in it is just a character.
fn contains(column: impl Into<SimpleExpr>, term: &str) -> SimpleExpr {
    Expr::expr(Func::cust(Alias::new("strpos")).args([Func::lower(column).into(), Expr::val(term)]))
        .gt(0)
}

/// The manager is in the readable set (same rule as [`employed_scope`]) and their name has the term.
fn manager_contains(refs: Option<&[String]>, today: NaiveDate, term: &str) -> SimpleExpr {
    let m = Alias::new("m");
    let col = |c: workers::Column| Expr::col((m.clone(), c));
    let mut sub = Query::select();
    sub.expr(Expr::val(1))
        .from_as(workers::Entity, m.clone())
        .and_where(col(workers::Column::Pid).equals((workers::Entity, workers::Column::ManagerPid)))
        .and_where(col(workers::Column::DeletedAt).is_null())
        .and_where(col(workers::Column::HiredOn).lte(today))
        .cond_where(
            Condition::any()
                .add(col(workers::Column::TerminatedOn).is_null())
                .add(col(workers::Column::TerminatedOn).gt(today)),
        )
        .and_where(contains(col(workers::Column::DisplayName), term));
    if let Some(refs) = refs {
        sub.and_where(col(workers::Column::OrganizationRef).is_in(refs.iter().cloned()));
    }
    Expr::exists(sub)
}

/// The listing's filter, in SQL, with the same meaning as [`rules::search`]: every whitespace-separated
/// term must appear (ignoring case) in the name, title, department, location, the readable manager's
/// name or the name of a rota the worker is on call for today; and the department, if given, matches
/// ignoring case.
fn filtered(
    refs: Option<&[String]>,
    today: NaiveDate,
    params: &DirectoryParams,
    on_call: &HashMap<Uuid, Vec<String>>,
) -> Select<workers::Entity> {
    let mut query = employed_scope(refs, today);
    if let Some(department) = params.department.as_deref().filter(|d| !d.is_empty()) {
        query = query.filter(
            Func::lower(Expr::col((workers::Entity, workers::Column::Department)))
                .eq(department.to_lowercase()),
        );
    }
    for term in params.q.as_deref().unwrap_or("").split_whitespace() {
        let term = term.to_lowercase();
        let on_call_match: Vec<Uuid> = on_call
            .iter()
            .filter(|(_, rotas)| rotas.iter().any(|r| r.to_lowercase().contains(&term)))
            .map(|(pid, _)| *pid)
            .collect();
        let mut any = Condition::any()
            .add(contains(
                Expr::col((workers::Entity, workers::Column::DisplayName)),
                &term,
            ))
            .add(contains(
                Expr::col((workers::Entity, workers::Column::JobTitle)),
                &term,
            ))
            .add(contains(
                Expr::col((workers::Entity, workers::Column::Department)),
                &term,
            ))
            .add(contains(
                Expr::col((workers::Entity, workers::Column::Location)),
                &term,
            ))
            .add(manager_contains(refs, today, &term));
        // an empty `IN ()` is not valid SQL
        if !on_call_match.is_empty() {
            any = any.add(workers::Column::Pid.is_in(on_call_match));
        }
        query = query.filter(any);
    }
    query
}

/// Who among `pids` is on approved leave `today`.
async fn away_among(ctx: &AppContext, pids: Vec<Uuid>, today: NaiveDate) -> Result<HashSet<Uuid>> {
    Ok(leave_requests::Entity::find()
        .filter(leave_requests::Column::WorkerPid.is_in(pids))
        .filter(leave_requests::Column::Status.eq("approved"))
        .filter(leave_requests::Column::DeletedAt.is_null())
        .filter(leave_requests::Column::StartOn.lte(today))
        .filter(leave_requests::Column::EndOn.gte(today))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|l| l.worker_pid)
        .collect())
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
    axum::extract::Query(params): axum::extract::Query<DirectoryParams>,
) -> Result<Response> {
    let page = Page {
        limit: params.limit,
        offset: params.offset,
    };
    page.check_offset()?;
    let (limit, offset) = page.resolve(DIRECTORY_DEFAULT_LIMIT);

    let refs = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let refs = refs.as_deref();
    let today = chrono::Utc::now().date_naive();
    let on_call = super::rotas::on_call_today(&ctx, &caller).await?;

    let matching = filtered(refs, today, &params, &on_call);
    let total = matching.clone().count(&ctx.db).await?;
    let staff: Vec<workers::Model> = matching
        .order_by_asc(Expr::cust("lower(workers.display_name) COLLATE \"C\""))
        .order_by_asc(workers::Column::Pid)
        .limit(limit)
        .offset(offset)
        .all(&ctx.db)
        .await?;

    // Who is away today: approved leave covering today, for this page.
    let mut away = away_among(&ctx, staff.iter().map(|w| w.pid).collect(), today).await?;
    let mut backups_by_worker: HashMap<Uuid, Vec<worker_backups::Model>> = HashMap::new();
    if !away.is_empty() {
        for b in worker_backups::Entity::find()
            .filter(
                worker_backups::Column::WorkerPid.is_in(away.iter().copied().collect::<Vec<_>>()),
            )
            .filter(worker_backups::Column::DeletedAt.is_null())
            .all(&ctx.db)
            .await?
        {
            backups_by_worker.entry(b.worker_pid).or_default().push(b);
        }
    }

    // The names a page may show beyond its own: managers and backups, only if in the readable set.
    let mut wanted: Vec<Uuid> = staff.iter().filter_map(|w| w.manager_pid).collect();
    wanted.extend(backups_by_worker.values().flatten().map(|b| b.backup_pid));
    let known: HashMap<Uuid, String> = employed_scope(refs, today)
        .filter(workers::Column::Pid.is_in(wanted.clone()))
        .all(&ctx.db)
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
    away.extend(away_among(&ctx, backup_pids, today).await?);

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
    Ok(with_page_headers(
        format::json(slice)?,
        total,
        limit,
        offset,
    ))
}

/// The directory route.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/directory", get(directory))
}
