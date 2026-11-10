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

use super::ensure_org_in_scope;
use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::{Page, forbidden, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{announcements, workers};
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
    memberships::has_role_in(ctx, caller.claims(), org, rules::EDITOR_ROLES).await
}

async fn require_editor(ctx: &AppContext, caller: &MaybeAuthUser, org: &str) -> Result<()> {
    if can_edit(ctx, caller, org).await? {
        Ok(())
    } else {
        Err(forbidden(
            "only an editor of that organization may change its announcements",
        ))
    }
}

/// An announcement, if it exists and its organization is readable by the caller.
async fn find(ctx: &AppContext, caller: &MaybeAuthUser, pid: &str) -> Result<announcements::Model> {
    let row = sqlx::query_as!(
        announcements::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, title, body, pinned,
                  publish_on, expires_on, department, links, author, deleted_at AS "deleted_at: _"
             FROM announcements WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &row.organization_ref).await?;
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
        sqlx::query_scalar!(
            "SELECT department FROM workers WHERE person_ref = $1 AND deleted_at IS NULL ORDER BY id",
            person_ref,
        )
        .fetch_all(db::pool(ctx))
        .await?,
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
}

/// `GET /api/announcements?organization=&include=all&limit=&cursor=` — the
/// feed for the caller's organizations: live posts, pinned first then newest,
/// a page at a time with a `Link` to the next (WPM-T478). `include=all` adds scheduled and expired posts, for
/// the organizations the caller edits.
#[debug_handler]
async fn feed(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(params): axa::extract::Query<FeedParams>,
    page: Page,
) -> Result<Response> {
    let window = super::resolve(&page, FEED_DEFAULT_LIMIT)?;
    let scope = memberships::scope_organization_refs(&ctx, caller.claims()).await?;
    let today = Utc::now().date_naive();
    let mut rows = sqlx::query_as!(
        announcements::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, title, body, pinned,
                  publish_on, expires_on, department, links, author, deleted_at AS "deleted_at: _"
             FROM announcements
            WHERE deleted_at IS NULL AND ($1::text IS NULL OR organization_ref = $1)
              AND ($2::text[] IS NULL OR organization_ref = ANY($2))
            ORDER BY id DESC"#,
        params.organization,
        scope.as_deref(),
    )
    .fetch_all(db::pool(&ctx))
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
    let (page_rows, more) = window.split(
        kept.iter()
            .skip(usize::try_from(window.offset()).unwrap_or(usize::MAX))
            .take(
                usize::try_from(window.limit())
                    .unwrap_or(usize::MAX)
                    .saturating_add(1),
            )
            .collect::<Vec<&announcements::Model>>(),
    );
    // Editors see how many have read each post — a count, never who.
    let ids: Vec<Uuid> = page_rows.iter().map(|a| a.pid).collect();
    let counts: HashMap<Uuid, usize> = sqlx::query!(
        r#"SELECT announcement_pid, COUNT(*) AS "reads!" FROM announcement_reads
            WHERE announcement_pid = ANY($1) GROUP BY announcement_pid"#,
        &ids,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|r| (r.announcement_pid, usize::try_from(r.reads).unwrap_or(0)))
    .collect();
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
    Ok(super::with_next(
        axum::Json(slice).into_response(),
        &window,
        more,
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
    ensure_org_in_scope(&ctx, &caller, &payload.organization_ref).await?;
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
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO announcements (pid, organization_ref, title, body, pinned, publish_on, expires_on, department, links, author)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        pid,
        payload.organization_ref,
        payload.title.trim(),
        payload.body.trim(),
        payload.pinned.unwrap_or(false),
        publish_on,
        payload.expires_on,
        payload
            .department
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty()),
        links_json(&payload.links),
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "announcement", pid, "posted", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": pid }))
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
    let mut tx = db::pool(&ctx).begin().await?;
    let updated = sqlx::query_as!(
        announcements::Model,
        r#"UPDATE announcements SET department = $2, links = $3, title = $4, body = $5, pinned = $6, publish_on = $7,
                  expires_on = $8
            WHERE id = $1
            RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, title, body, pinned,
                  publish_on, expires_on, department, links, author, deleted_at AS "deleted_at: _""#,
        row.id,
        department,
        links,
        title.trim(),
        body.trim(),
        pinned,
        publish_on,
        expires_on,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "announcement",
        updated.pid,
        "edited",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE announcements SET deleted_at = $2 WHERE id = $1",
        row.id,
        Utc::now()
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "announcement",
        row.pid,
        "retired",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    // The first read is kept: one insert on the unique pair, which a second read leaves alone.
    sqlx::query!(
        "INSERT INTO announcement_reads (announcement_pid, worker_pid, read_at) VALUES ($1, $2, $3)
         ON CONFLICT (announcement_pid, worker_pid) DO NOTHING",
        row.pid,
        worker.pid,
        Utc::now(),
    )
    .execute(db::pool(&ctx))
    .await?;
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
    let read: HashSet<Uuid> = sqlx::query_scalar!(
        "SELECT announcement_pid FROM announcement_reads WHERE worker_pid = $1",
        worker.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
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
