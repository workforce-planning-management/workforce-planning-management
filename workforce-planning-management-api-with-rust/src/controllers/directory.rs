//! The **employee directory**: `GET /api/directory` — who works where,
//! searchable, with nothing sensitive in it (see [`crate::rules::directory`]).
//!
//! Lists workers *employed today* ([`metric_rules::is_employed_on`], the one
//! definition of "employed") in the organizations the caller can read.
//! A manager's name is shown only when the manager is in the same readable
//! set, so the directory never names someone outside the caller's scope.
//!
//! A worker on **approved leave today** is shown as away — never the kind
//! of leave — with whoever covers for them ([`cover_rules::resolve`]: their
//! best-ranked backup who is employed and not away themselves). Backups
//! outside the caller's readable set are not considered, so a cover is
//! only ever someone the caller could find in the directory anyway.

use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::{Page, with_page_headers};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{leave_requests, worker_backups, workers};
use crate::models::memberships;
use crate::rules::cover::{self as cover_rules, Candidate};
use crate::rules::directory as rules;
use crate::rules::metrics as metric_rules;

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

/// `GET /api/directory?q=&department=&limit=&offset=` — directory entries
/// ordered by name, with `x-total-count` of all matches.
#[debug_handler]
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

    let mut query = workers::Entity::find().filter(workers::Column::DeletedAt.is_null());
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await? {
        query = query.filter(workers::Column::OrganizationRef.is_in(refs));
    }
    let today = chrono::Utc::now().date_naive();
    let staff: Vec<workers::Model> = query
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| metric_rules::is_employed_on(today, w.hired_on, w.terminated_on))
        .collect();

    // Who is away today: approved leave covering today, for this staff.
    let staff_pids: Vec<Uuid> = staff.iter().map(|w| w.pid).collect();
    let away: HashSet<Uuid> = leave_requests::Entity::find()
        .filter(leave_requests::Column::WorkerPid.is_in(staff_pids.clone()))
        .filter(leave_requests::Column::Status.eq("approved"))
        .filter(leave_requests::Column::DeletedAt.is_null())
        .filter(leave_requests::Column::StartOn.lte(today))
        .filter(leave_requests::Column::EndOn.gte(today))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|l| l.worker_pid)
        .collect();
    let mut backups_by_worker: HashMap<Uuid, Vec<worker_backups::Model>> = HashMap::new();
    if !away.is_empty() {
        for b in worker_backups::Entity::find()
            .filter(worker_backups::Column::WorkerPid.is_in(away.iter().copied().collect::<Vec<_>>()))
            .filter(worker_backups::Column::DeletedAt.is_null())
            .all(&ctx.db)
            .await?
        {
            backups_by_worker.entry(b.worker_pid).or_default().push(b);
        }
    }

    let names: HashMap<Uuid, &str> = staff
        .iter()
        .map(|w| (w.pid, w.display_name.as_str()))
        .collect();
    let entries: Vec<rules::Entry> = staff
        .iter()
        .map(|w| rules::Entry {
            pid: w.pid,
            display_name: w.display_name.clone(),
            job_title: w.job_title.clone(),
            department: w.department.clone(),
            location: w.location.clone(),
            organization_ref: w.organization_ref.clone(),
            manager_name: w
                .manager_pid
                .and_then(|m| names.get(&m))
                .map(|n| (*n).to_string()),
            away_today: away.contains(&w.pid),
            covered_by: if away.contains(&w.pid) {
                // Only backups in the readable set (`names`) can cover here.
                let candidates: Vec<Candidate> = backups_by_worker
                    .get(&w.pid)
                    .into_iter()
                    .flatten()
                    .map(|b| Candidate {
                        backup: b.backup_pid,
                        priority: b.priority,
                        starts_on: b.starts_on,
                        ends_on: b.ends_on,
                        employed: names.contains_key(&b.backup_pid),
                        on_leave: away.contains(&b.backup_pid),
                    })
                    .collect();
                cover_rules::resolve(&candidates, today)
                    .and_then(|id| names.get(&id))
                    .map(|n| (*n).to_string())
            } else {
                None
            },
        })
        .collect();

    let found = rules::search(
        entries,
        params.q.as_deref().unwrap_or(""),
        params.department.as_deref().filter(|d| !d.is_empty()),
    );
    let total = found.len() as u64;
    let start = usize::try_from(offset).unwrap_or(usize::MAX);
    let size = usize::try_from(limit).unwrap_or(usize::MAX);
    let slice: Vec<rules::Entry> = found.into_iter().skip(start).take(size).collect();
    Ok(with_page_headers(format::json(slice)?, total, limit, offset))
}

/// The directory route.
pub fn routes() -> Routes {
    Routes::new().prefix("/api").add("/directory", get(directory))
}
