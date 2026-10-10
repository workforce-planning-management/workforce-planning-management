//! The **announcement feed**: company and organization news.
//!
//! Everyone who can read an organization reads its live announcements
//! (published, not expired), pinned first then newest. Posting, editing and
//! retiring — and seeing scheduled or expired posts — belongs to the
//! organization's editors ([`rules::EDITOR_ROLES`]) when auth is on; a post
//! is plain text (see [`crate::rules::announcements`]).
//!
//! Three extras: a post may be aimed at **one department** (only that
//! department and the organization's editors see it — enforced when auth is
//! on, from the caller's own worker records); it may carry up to three
//! **https links**; and **read receipts** — each person can mark a post read
//! and see which they have read, while editors see only *how many* have read
//! it, never who.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::{Page, record_rejection, unprocessable, with_page_headers};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{announcement_reads, announcements, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::rules::announcements as rules;

/// Default page size.
const FEED_DEFAULT_LIMIT: u64 = 20;

/// Whether the caller may edit `org`'s announcements: always when auth is
/// off; otherwise only with an editor role in that organization.
async fn can_edit(ctx: &AppContext, caller: &MaybeAuthUser, org: &str) -> Result<bool> {
    if !auth::require_auth() {
        return Ok(true);
    }
    memberships::has_role_in(&ctx.db, caller.claims(), org, rules::EDITOR_ROLES).await
}

async fn require_editor(ctx: &AppContext, caller: &MaybeAuthUser, org: &str) -> Result<()> {
    if can_edit(ctx, caller, org).await? {
        Ok(())
    } else {
        Err(record_rejection((
            axum::http::StatusCode::FORBIDDEN,
            "only an editor of that organization may change its announcements".to_string(),
        )))
    }
}

/// An announcement, if it exists and its organization is readable by the caller.
async fn find(ctx: &AppContext, caller: &MaybeAuthUser, pid: &str) -> Result<announcements::Model> {
    let row = announcements::Entity::find()
        .filter(announcements::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(announcements::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.iter().any(|r| r == &row.organization_ref)
    {
        return Err(Error::NotFound);
    }
    Ok(row)
}

/// The departments of the caller's own worker records — `None` when auth is
/// off (nobody is filtered, as elsewhere). The caller's `sub` is the bare id
/// in their `person:<id>` reference.
async fn reader_departments(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
) -> Result<Option<Vec<String>>> {
    if !auth::require_auth() {
        return Ok(None);
    }
    let Some(claims) = caller.claims() else {
        return Ok(Some(Vec::new()));
    };
    let person_ref = format!("person:{}", claims.sub.trim().to_ascii_lowercase());
    Ok(Some(
        workers::Entity::find()
            .filter(workers::Column::PersonRef.eq(person_ref))
            .filter(workers::Column::DeletedAt.is_null())
            .all(&ctx.db)
            .await?
            .into_iter()
            .map(|w| w.department)
            .collect(),
    ))
}

/// A link as submitted.
#[derive(Debug, Deserialize)]
struct LinkPayload {
    label: String,
    url: String,
}

fn links_json(links: &[LinkPayload]) -> serde_json::Value {
    serde_json::Value::Array(
        links
            .iter()
            .map(|l| serde_json::json!({ "label": l.label.trim(), "url": l.url.trim() }))
            .collect(),
    )
}

fn validate_links_payload(links: &[LinkPayload]) -> Result<()> {
    let pairs: Vec<(String, String)> = links
        .iter()
        .map(|l| (l.label.trim().to_string(), l.url.trim().to_string()))
        .collect();
    rules::validate_links(&pairs).map_err(|e| unprocessable(&e))
}

fn row_json(a: &announcements::Model, today: NaiveDate) -> serde_json::Value {
    let status = match rules::status(a.publish_on, a.expires_on, today) {
        rules::Status::Scheduled => "scheduled",
        rules::Status::Live => "live",
        rules::Status::Expired => "expired",
    };
    serde_json::json!({
        "pid": a.pid,
        "organization_ref": a.organization_ref,
        "title": a.title,
        "body": a.body,
        "pinned": a.pinned,
        "publish_on": a.publish_on,
        "expires_on": a.expires_on,
        "department": a.department,
        "links": a.links,
        "status": status,
        "author": a.author,
    })
}

/// Query for the feed.
#[derive(Debug, Deserialize)]
struct FeedParams {
    /// Restrict to one organization.
    #[serde(default)]
    organization: Option<String>,
    /// Only organization-wide posts and those for this department.
    #[serde(default)]
    department: Option<String>,
    /// `all`: also scheduled and expired posts — editors only.
    #[serde(default)]
    include: Option<String>,
    #[serde(default)]
    limit: Option<u64>,
    #[serde(default)]
    offset: Option<u64>,
}

/// `GET /api/announcements?organization=&include=all&limit=&offset=` — the
/// feed for the caller's organizations: live posts, pinned first then newest,
/// with `x-total-count`. `include=all` adds scheduled and expired posts, for
/// the organizations the caller edits.
#[debug_handler]
async fn feed(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(params): axum::extract::Query<FeedParams>,
) -> Result<Response> {
    let page = Page {
        limit: params.limit,
        offset: params.offset,
    };
    page.check_offset()?;
    let (limit, offset) = page.resolve(FEED_DEFAULT_LIMIT);
    let mut query =
        announcements::Entity::find().filter(announcements::Column::DeletedAt.is_null());
    if let Some(org) = &params.organization {
        query = query.filter(announcements::Column::OrganizationRef.eq(org));
    }
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await? {
        query = query.filter(announcements::Column::OrganizationRef.is_in(refs));
    }
    let today = Utc::now().date_naive();
    let mut rows = query
        .order_by_desc(announcements::Column::Id)
        .all(&ctx.db)
        .await?;
    let all = params.include.as_deref() == Some("all");
    let readers = reader_departments(&ctx, &caller).await?;
    let mut allowed: HashMap<String, bool> = HashMap::new();
    let mut kept = Vec::with_capacity(rows.len());
    for row in rows.drain(..) {
        if !allowed.contains_key(&row.organization_ref) {
            let ok = can_edit(&ctx, &caller, &row.organization_ref).await?;
            allowed.insert(row.organization_ref.clone(), ok);
        }
        let editor = allowed[&row.organization_ref];
        // A department post is for that department (and editors): from the
        // caller's own worker records when auth is on, else only by the
        // explicit `?department=` filter.
        if let Some(wanted) = &params.department
            && !rules::audience_includes(
                row.department.as_deref(),
                std::slice::from_ref(wanted),
                false,
            )
        {
            continue;
        }
        if let Some(mine) = &readers
            && !rules::audience_includes(row.department.as_deref(), mine, editor)
        {
            continue;
        }
        let live = rules::status(row.publish_on, row.expires_on, today) == rules::Status::Live;
        if live || (all && editor) {
            kept.push(row);
        }
    }
    kept.sort_by(|a, b| {
        rules::feed_order(
            (a.pinned, a.publish_on, a.id),
            (b.pinned, b.publish_on, b.id),
        )
    });
    let total = kept.len() as u64;
    let page_rows: Vec<&announcements::Model> = kept
        .iter()
        .skip(usize::try_from(offset).unwrap_or(usize::MAX))
        .take(usize::try_from(limit).unwrap_or(usize::MAX))
        .collect();
    // Editors see how many have read each post — a count, never who.
    let ids: Vec<Uuid> = page_rows.iter().map(|a| a.pid).collect();
    let mut counts: HashMap<Uuid, usize> = HashMap::new();
    for r in announcement_reads::Entity::find()
        .filter(announcement_reads::Column::AnnouncementPid.is_in(ids))
        .all(&ctx.db)
        .await?
    {
        *counts.entry(r.announcement_pid).or_default() += 1;
    }
    let slice: Vec<serde_json::Value> = page_rows
        .iter()
        .map(|a| {
            let mut json = row_json(a, today);
            if allowed.get(&a.organization_ref).copied().unwrap_or(false) {
                json["read_count"] = serde_json::json!(counts.get(&a.pid).copied().unwrap_or(0));
            }
            json
        })
        .collect();
    Ok(with_page_headers(
        format::json(slice)?,
        total,
        limit,
        offset,
    ))
}

/// `GET /api/announcements/{pid}` — one announcement (live ones to anyone
/// who can read the organization; others to its editors).
#[debug_handler]
async fn get_one(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = find(&ctx, &caller, &pid).await?;
    let today = Utc::now().date_naive();
    if rules::status(row.publish_on, row.expires_on, today) != rules::Status::Live
        && !can_edit(&ctx, &caller, &row.organization_ref).await?
    {
        return Err(Error::NotFound);
    }
    format::json(row_json(&row, today))
}

/// `POST /api/announcements` body.
#[derive(Debug, Deserialize)]
struct AnnouncementPayload {
    organization_ref: String,
    title: String,
    body: String,
    #[serde(default)]
    pinned: Option<bool>,
    /// Default today.
    #[serde(default)]
    publish_on: Option<NaiveDate>,
    #[serde(default)]
    expires_on: Option<NaiveDate>,
    /// Aim the post at one department; absent = everyone.
    #[serde(default)]
    department: Option<String>,
    /// Up to three https links.
    #[serde(default)]
    links: Vec<LinkPayload>,
}

/// `POST /api/announcements` — post an announcement.
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<AnnouncementPayload>,
) -> Result<Response> {
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.iter().any(|r| r == &payload.organization_ref)
    {
        return Err(Error::NotFound);
    }
    require_editor(&ctx, &caller, &payload.organization_ref).await?;
    let today = Utc::now().date_naive();
    let publish_on = payload.publish_on.unwrap_or(today);
    rules::validate(
        &payload.title,
        &payload.body,
        publish_on,
        payload.expires_on,
    )
    .map_err(|e| unprocessable(&e))?;
    validate_links_payload(&payload.links)?;
    let row = announcements::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        organization_ref: ActiveValue::set(payload.organization_ref),
        title: ActiveValue::set(payload.title.trim().to_string()),
        body: ActiveValue::set(payload.body.trim().to_string()),
        pinned: ActiveValue::set(payload.pinned.unwrap_or(false)),
        publish_on: ActiveValue::set(publish_on),
        expires_on: ActiveValue::set(payload.expires_on),
        department: ActiveValue::set(
            payload
                .department
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty()),
        ),
        links: ActiveValue::set(links_json(&payload.links)),
        author: ActiveValue::set(caller.actor().map(ToString::to_string)),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "announcement",
        row.pid,
        "posted",
        caller.actor(),
        None,
    )
    .await?;
    format::json(serde_json::json!({ "pid": row.pid }))
}

/// `PUT /api/announcements/{pid}` body — any of these may change.
#[derive(Debug, Deserialize)]
struct AnnouncementUpdate {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    pinned: Option<bool>,
    #[serde(default)]
    publish_on: Option<NaiveDate>,
    /// Set the expiry; `clear_expiry` removes it.
    #[serde(default)]
    expires_on: Option<NaiveDate>,
    #[serde(default)]
    clear_expiry: bool,
    /// Aim at a department; `clear_department` makes it organization-wide.
    #[serde(default)]
    department: Option<String>,
    #[serde(default)]
    clear_department: bool,
    /// Replaces the links when present.
    #[serde(default)]
    links: Option<Vec<LinkPayload>>,
}

/// `PUT /api/announcements/{pid}` — edit a post.
#[debug_handler]
async fn update(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<AnnouncementUpdate>,
) -> Result<Response> {
    let row = find(&ctx, &caller, &pid).await?;
    require_editor(&ctx, &caller, &row.organization_ref).await?;
    let title = payload.title.unwrap_or_else(|| row.title.clone());
    let body = payload.body.unwrap_or_else(|| row.body.clone());
    let publish_on = payload.publish_on.unwrap_or(row.publish_on);
    let expires_on = if payload.clear_expiry {
        None
    } else {
        payload.expires_on.or(row.expires_on)
    };
    rules::validate(&title, &body, publish_on, expires_on).map_err(|e| unprocessable(&e))?;
    let pinned = payload.pinned.unwrap_or(row.pinned);
    let department = if payload.clear_department {
        None
    } else {
        payload
            .department
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty())
            .or_else(|| row.department.clone())
    };
    let links = match &payload.links {
        Some(new) => {
            validate_links_payload(new)?;
            links_json(new)
        }
        None => row.links.clone(),
    };
    let mut active: announcements::ActiveModel = row.into();
    active.department = ActiveValue::set(department);
    active.links = ActiveValue::set(links);
    active.title = ActiveValue::set(title.trim().to_string());
    active.body = ActiveValue::set(body.trim().to_string());
    active.pinned = ActiveValue::set(pinned);
    active.publish_on = ActiveValue::set(publish_on);
    active.expires_on = ActiveValue::set(expires_on);
    let updated = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "announcement",
        updated.pid,
        "edited",
        caller.actor(),
        None,
    )
    .await?;
    format::json(row_json(&updated, Utc::now().date_naive()))
}

/// `DELETE /api/announcements/{pid}` — retire a post (soft-delete).
#[debug_handler]
async fn retire(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = find(&ctx, &caller, &pid).await?;
    require_editor(&ctx, &caller, &row.organization_ref).await?;
    let pid = row.pid;
    let mut active: announcements::ActiveModel = row.into();
    active.deleted_at = ActiveValue::set(Some(Utc::now().into()));
    active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "announcement",
        pid,
        "retired",
        caller.actor(),
        None,
    )
    .await?;
    format::empty_json()
}

/// `POST /api/announcements/{pid}/read` body.
#[derive(Debug, Deserialize)]
struct ReadPayload {
    /// The reader's worker record: themself, or HR on their behalf.
    worker_pid: Uuid,
}

/// A worker, authorized for a write to their record: themself, or HR.
async fn own_worker(ctx: &AppContext, caller: &MaybeAuthUser, pid: Uuid) -> Result<workers::Model> {
    let worker = records::find_worker(&ctx.db, pid).await?;
    auth::authorize_record(
        caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    Ok(worker)
}

/// `POST /api/announcements/{pid}/read` — mark a post read. Idempotent: the
/// first read is kept. A person reads only what they can see, so a post
/// aimed at another department (or outside their organizations) is a 404.
#[debug_handler]
async fn mark_read(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ReadPayload>,
) -> Result<Response> {
    let row = find(&ctx, &caller, &pid).await?;
    let worker = own_worker(&ctx, &caller, payload.worker_pid).await?;
    if worker.organization_ref != row.organization_ref
        || !rules::audience_includes(
            row.department.as_deref(),
            std::slice::from_ref(&worker.department),
            false,
        )
    {
        return Err(Error::NotFound);
    }
    let already = announcement_reads::Entity::find()
        .filter(announcement_reads::Column::AnnouncementPid.eq(row.pid))
        .filter(announcement_reads::Column::WorkerPid.eq(worker.pid))
        .one(&ctx.db)
        .await?
        .is_some();
    if !already {
        announcement_reads::ActiveModel {
            announcement_pid: ActiveValue::set(row.pid),
            worker_pid: ActiveValue::set(worker.pid),
            read_at: ActiveValue::set(Utc::now().into()),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?;
    }
    format::json(serde_json::json!({ "read": true }))
}

/// `GET /api/workers/{pid}/announcement-reads` — the announcements this
/// person has read (ids only). Theirs alone: the person and HR.
#[debug_handler]
async fn my_reads(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = own_worker(&ctx, &caller, records::parse_pid(&pid)?).await?;
    let read: HashSet<Uuid> = announcement_reads::Entity::find()
        .filter(announcement_reads::Column::WorkerPid.eq(worker.pid))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|r| r.announcement_pid)
        .collect();
    format::json(read.into_iter().collect::<Vec<_>>())
}

/// The announcement routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/announcements", post(create))
        .add("/announcements", get(feed))
        .add("/announcements/{pid}", get(get_one))
        .add("/announcements/{pid}", put(update))
        .add("/announcements/{pid}", delete(retire))
        .add("/announcements/{pid}/read", post(mark_read))
        .add("/workers/{pid}/announcement-reads", get(my_reads))
}
