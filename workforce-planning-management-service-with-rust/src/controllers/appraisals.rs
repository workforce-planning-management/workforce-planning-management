//! 360° appraisals (WPM-R29) — multi-rater feedback around a subject
//! worker: nominations by group (`self | manager | peer | report`),
//! once-per-rater responses while collecting, and a group-floored
//! report once shared. Rater anonymity is **procedural** (WPM-D21):
//! the store links a response to its nomination so once-per-rater and
//! completion tracking hold, but no endpoint ever serves rater-level
//! content — the detail view shows *who* responded, the report shows
//! group aggregates only, with `peer`/`report` cells withheld below a
//! floor of 3 (count included).

use std::collections::BTreeMap;

use super::row_json;
use axa::extract::{Json, Path};
use axa::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use super::{ensure_valid, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::appraisals;
use crate::models::records;
use crate::rules::appraisal as rules;
use crate::rules::notify;
use crate::validation::Problems;

/// A `{pid}` reference response.
#[derive(serde::Serialize)]
struct PidRef {
    pid: String,
}

/// `POST /api/workers/{pid}/appraisals` body.
#[derive(Debug, Deserialize)]
struct AppraisalPayload {
    competencies: Vec<String>,
}

/// `POST /api/workers/{pid}/appraisals` — open a draft 360° for the
/// subject; the subject's `self` nomination is created automatically.
#[debug_handler]
async fn create_appraisal(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<AppraisalPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    if payload.competencies.is_empty() {
        problems.push("at least one competency is required");
    }
    if payload.competencies.len() > rules::MAX_COMPETENCIES {
        problems.push(format!("at most {} competencies", rules::MAX_COMPETENCIES));
    }
    problems.cap_list("competencies", &payload.competencies);
    let mut seen = std::collections::HashSet::new();
    if payload
        .competencies
        .iter()
        .any(|c| !seen.insert(c.trim().to_lowercase()))
    {
        problems.push("competencies must be unique");
    }
    ensure_valid(&problems.into_vec())?;
    let subject = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let appraisal_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO appraisals (pid, worker_pid, competencies, status, shared_on, deleted_at)
         VALUES ($1, $2, $3, 'draft', NULL, NULL)",
        appraisal_pid,
        subject.pid,
        serde_json::json!(payload.competencies),
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO appraisal_nominations (pid, appraisal_pid, rater_pid, rater_group) VALUES ($1, $2, $3, 'self')",
        Uuid::new_v4(),
        appraisal_pid,
        subject.pid,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "appraisal",
        appraisal_pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: appraisal_pid.to_string(),
    })
}

/// `GET /api/workers/{pid}/appraisals` — the subject's appraisals
/// with nomination/response counts (never content).
#[debug_handler]
async fn list_appraisals(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let subject = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &auth::worker_resource_attrs(&subject),
    )
    .map_err(record_rejection)?;
    let rows = sqlx::query!(
        r#"SELECT a.pid, a.status, a.competencies, a.shared_on,
                  (SELECT COUNT(*) FROM appraisal_nominations n WHERE n.appraisal_pid = a.pid) AS "nominated!",
                  (SELECT COUNT(*) FROM appraisal_responses r WHERE r.appraisal_pid = a.pid) AS "responded!"
             FROM appraisals a
            WHERE a.worker_pid = $1 AND a.deleted_at IS NULL
            ORDER BY a.id DESC"#,
        subject.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let view: Vec<_> = rows
        .iter()
        .map(|appraisal| {
            serde_json::json!({
                "pid": appraisal.pid,
                "status": appraisal.status,
                "competencies": appraisal.competencies,
                "shared_on": appraisal.shared_on,
                "nominated": appraisal.nominated,
                "responded": appraisal.responded,
            })
        })
        .collect();
    format::json(view)
}

/// `POST /api/appraisals/{pid}/nominations` body.
#[derive(Debug, Deserialize)]
struct NominationPayload {
    rater_pid: Uuid,
    group: String,
}

/// `POST /api/appraisals/{pid}/nominations` — invite a rater (draft
/// only; `self` is automatic and cannot be nominated; one nomination
/// per rater; at most [`rules::MAX_RATERS`]).
#[debug_handler]
async fn nominate(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<NominationPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token("group", rules::GROUPS, &payload.group);
    if payload.group == "self" {
        problems.push("the self nomination is automatic");
    }
    ensure_valid(&problems.into_vec())?;
    let appraisal = find_appraisal(&ctx, &pid).await?;
    if appraisal.status != "draft" {
        return Err(unprocessable("nominations are frozen once collecting"));
    }
    let rater = records::worker_by_pid(&ctx, payload.rater_pid).await?;
    if rater.pid == appraisal.worker_pid {
        return Err(unprocessable("the subject rates only as `self`"));
    }
    let existing = sqlx::query_scalar!(
        "SELECT rater_pid FROM appraisal_nominations WHERE appraisal_pid = $1",
        appraisal.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    if existing.contains(&rater.pid) {
        return Err(unprocessable("this rater is already nominated"));
    }
    if existing.len() >= rules::MAX_RATERS {
        return Err(unprocessable("at most 12 raters"));
    }
    let nomination_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO appraisal_nominations (pid, appraisal_pid, rater_pid, rater_group) VALUES ($1, $2, $3, $4)",
        nomination_pid,
        appraisal.pid,
        rater.pid,
        payload.group,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "appraisal_nomination",
        nomination_pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: nomination_pid.to_string(),
    })
}

/// `POST /api/appraisals/{pid}/status` body.
#[derive(Debug, Deserialize)]
struct StatusPayload {
    to: String,
}

/// `POST /api/appraisals/{pid}/status` — the lifecycle move (pure
/// machine); `collecting` requires ≥ 3 non-self nominations; sharing
/// stamps `shared_on`.
#[debug_handler]
async fn appraisal_status(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<StatusPayload>,
) -> Result<Response> {
    let appraisal = find_appraisal(&ctx, &pid).await?;
    rules::transition(&appraisal.status, &payload.to).map_err(|reason| unprocessable(&reason))?;
    if payload.to == "collecting" {
        let non_self = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM appraisal_nominations WHERE appraisal_pid = $1 AND rater_group <> 'self'"#,
            appraisal.pid,
        )
        .fetch_one(db::pool(&ctx))
        .await?;
        let non_self = usize::try_from(non_self).unwrap_or(0);
        if non_self < rules::MIN_NON_SELF_RATERS {
            return Err(unprocessable(&format!(
                "collecting requires at least {} non-self raters, have {non_self}",
                rules::MIN_NON_SELF_RATERS
            )));
        }
    }
    let from = appraisal.status.clone();
    let row_pid = appraisal.pid;
    let shared_on = (payload.to == "shared").then(|| chrono::Utc::now().date_naive());
    let mut tx = db::pool(&ctx).begin().await?;
    let updated = sqlx::query_as!(
        appraisals::Model,
        r#"UPDATE appraisals SET status = $2, shared_on = COALESCE($3, shared_on), updated_at = now()
            WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, competencies,
                  status, shared_on, deleted_at AS "deleted_at: _""#,
        appraisal.id,
        payload.to,
        shared_on,
    )
    .fetch_one(&mut *tx)
    .await?;
    // Fan out the in-app notifications (WPM-R31): collecting tells
    // every rater; shared tells the subject. Reference-only bodies —
    // never scores or comments (WPM-D23).
    let rater_pids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT rater_pid FROM appraisal_nominations WHERE appraisal_pid = $1",
        row_pid,
    )
    .fetch_all(&mut *tx)
    .await?;
    let subject_name = sqlx::query_scalar!(
        "SELECT display_name FROM workers WHERE pid = $1 LIMIT 1",
        updated.worker_pid,
    )
    .fetch_optional(&mut *tx)
    .await?
    .unwrap_or_default();
    for (recipient, kind) in
        notify::appraisal_recipients(&payload.to, updated.worker_pid, &rater_pids)
    {
        let body = if kind == "appraisal_shared" {
            "Your 360\u{b0} report is ready".to_string()
        } else {
            format!("360\u{b0} feedback requested for {subject_name}")
        };
        db::notify(
            &mut tx,
            recipient,
            kind,
            &body,
            serde_json::json!({ "appraisal_pid": row_pid }),
        )
        .await?;
    }
    db::audit(
        &mut tx,
        "appraisal",
        row_pid,
        "status_changed",
        caller.actor(),
        Some(serde_json::json!({ "from": from, "to": payload.to })),
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&updated)?)
}

/// `GET /api/appraisals/{pid}` — the appraisal + its nominations with
/// display names and a responded flag. **Who** responded, never what
/// (WPM-D21).
#[debug_handler]
async fn get_appraisal(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let appraisal = find_appraisal(&ctx, &pid).await?;
    // A rater's name is the first worker row with that pid; responded is whether a response of this appraisal names the
    // nomination.
    let nominations = sqlx::query!(
        r#"SELECT n.pid, n.rater_pid, n.rater_group,
                  (SELECT w.display_name FROM workers w WHERE w.pid = n.rater_pid ORDER BY w.id LIMIT 1) AS display_name,
                  EXISTS (SELECT 1 FROM appraisal_responses r
                           WHERE r.appraisal_pid = n.appraisal_pid AND r.nomination_pid = n.pid) AS "responded!"
             FROM appraisal_nominations n
            WHERE n.appraisal_pid = $1
            ORDER BY n.id"#,
        appraisal.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let nomination_view: Vec<_> = nominations
        .iter()
        .map(|nomination| {
            serde_json::json!({
                "pid": nomination.pid,
                "rater_pid": nomination.rater_pid,
                "display_name": nomination.display_name,
                "group": nomination.rater_group,
                "responded": nomination.responded,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "pid": appraisal.pid,
        "worker_pid": appraisal.worker_pid,
        "status": appraisal.status,
        "competencies": appraisal.competencies,
        "shared_on": appraisal.shared_on,
        "nominations": nomination_view,
    }))
}

/// `POST /api/appraisals/{pid}/responses` body.
#[derive(Debug, Deserialize)]
struct ResponsePayload {
    rater_pid: Uuid,
    scores: BTreeMap<String, i32>,
    #[serde(default)]
    comment: Option<String>,
}

/// `POST /api/appraisals/{pid}/responses` — one rater's response:
/// collecting only, nominated raters only, once per rater, every
/// declared competency scored 1–5. `$sub` ownership applies to the
/// rater. The audit row carries the actor (procedural anonymity —
/// accountability without content disclosure: no endpoint serves
/// rater-level answers).
#[debug_handler]
async fn respond(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ResponsePayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.cap_opt("comment", payload.comment.as_deref());
    ensure_valid(&problems.into_vec())?;
    let appraisal = find_appraisal(&ctx, &pid).await?;
    if appraisal.status != "collecting" {
        return Err(unprocessable(
            "responses are accepted only while collecting",
        ));
    }
    let declared: Vec<String> =
        serde_json::from_value(appraisal.competencies.clone()).unwrap_or_default();
    rules::check_scores(&declared, &payload.scores).map_err(|reason| unprocessable(&reason))?;
    let rater = records::worker_by_pid(&ctx, payload.rater_pid).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&rater),
    )
    .map_err(record_rejection)?;
    let nomination = sqlx::query!(
        "SELECT pid, rater_group FROM appraisal_nominations WHERE appraisal_pid = $1 AND rater_pid = $2 LIMIT 1",
        appraisal.pid,
        rater.pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or_else(|| unprocessable("this worker is not a nominated rater"))?;
    let already = sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM appraisal_responses WHERE nomination_pid = $1 LIMIT 1"#,
        nomination.pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?;
    if already.is_some() {
        return Err(unprocessable("this rater has already responded"));
    }
    let response_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO appraisal_responses (pid, appraisal_pid, nomination_pid, rater_group, scores, comment)
         VALUES ($1, $2, $3, $4, $5, $6)",
        response_pid,
        appraisal.pid,
        nomination.pid,
        nomination.rater_group,
        serde_json::json!(payload.scores),
        payload.comment,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "appraisal_response",
        response_pid,
        "submitted",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "submitted": true }))
}

/// `GET /api/appraisals/{pid}/report` — the group-floored report,
/// readable once `shared` (`422` before). Per group × competency
/// count + mean, and group-pooled comments sorted alphabetically (so
/// ordering reveals no submission sequence); `peer`/`report` cells
/// below the floor of 3 are withheld — count included (WPM-D21).
/// Reads are audited (review-content sensitivity, WPM-R10 posture).
#[debug_handler]
async fn report(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let appraisal = find_appraisal(&ctx, &pid).await?;
    if appraisal.status != "shared" {
        return Err(unprocessable("the report is readable once shared"));
    }
    let subject = records::worker_by_pid(&ctx, appraisal.worker_pid).await?;
    let obligations = auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &auth::worker_resource_attrs(&subject),
    )
    .map_err(record_rejection)?;
    // Comments are review-content tier (spec auth.md): a masked read
    // keeps the numeric aggregates but the words are withheld.
    let comments_masked = obligations.iter().any(|o| o == "mask");
    let declared: Vec<String> =
        serde_json::from_value(appraisal.competencies.clone()).unwrap_or_default();
    let responses = sqlx::query!(
        "SELECT rater_group, scores, comment FROM appraisal_responses WHERE appraisal_pid = $1",
        appraisal.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let mut groups = Vec::new();
    for group in rules::GROUPS {
        let group_responses: Vec<_> = responses
            .iter()
            .filter(|r| r.rater_group == *group)
            .collect();
        if group_responses.is_empty() {
            continue;
        }
        if !rules::group_discloses(group, group_responses.len()) {
            groups.push(serde_json::json!({ "group": group, "withheld": true }));
            continue;
        }
        let mut competencies = serde_json::Map::new();
        for competency in &declared {
            let scores: Vec<i32> = group_responses
                .iter()
                .filter_map(|r| r.scores.get(competency).and_then(serde_json::Value::as_i64))
                .filter_map(|s| i32::try_from(s).ok())
                .collect();
            if let Some((count, mean)) = rules::competency_mean(&scores) {
                competencies.insert(
                    competency.clone(),
                    serde_json::json!({ "count": count, "mean": mean }),
                );
            }
        }
        let mut comments: Vec<&str> = group_responses
            .iter()
            .filter_map(|r| r.comment.as_deref())
            .filter(|c| !c.trim().is_empty())
            .collect();
        comments.sort_unstable();
        groups.push(serde_json::json!({
            "group": group,
            "withheld": false,
            "responses": group_responses.len(),
            "competencies": competencies,
            "comments": if comments_masked { serde_json::Value::Null } else {
                serde_json::json!(comments)
            },
            "comments_withheld": comments_masked,
        }));
    }
    db::audit(
        &mut *db::pool(&ctx).acquire().await?,
        "appraisal",
        appraisal.pid,
        "report_read",
        caller.actor(),
        None,
    )
    .await?;
    format::json(serde_json::json!({
        "appraisal": {
            "pid": appraisal.pid,
            "worker_pid": appraisal.worker_pid,
            "competencies": declared,
            "shared_on": appraisal.shared_on,
        },
        "groups": groups,
        "derivation": "group aggregates only — no rater-level content exists on any \
                       endpoint; peer/report cells with fewer than 3 responses are \
                       withheld (count included); manager/self disclose at 1 by \
                       convention; comments are pooled per group and sorted \
                       alphabetically; development-facing, not a payroll input",
    }))
}

/// `GET /api/workers/{pid}/appraisal-requests` — the rater's own
/// pending requests: `collecting` appraisals where they are nominated
/// and have not yet responded, with the subject, group, and declared
/// competencies. `$sub`-owned; discloses only what the rater already
/// knows (that they were invited).
#[debug_handler]
async fn rater_requests(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let rater = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &auth::worker_resource_attrs(&rater),
    )
    .map_err(record_rejection)?;
    // Per nomination, oldest first: its live collecting appraisal (a pid is unique), unless the nomination has a response.
    let rows = sqlx::query!(
        r#"SELECT a.pid, a.worker_pid, a.competencies, n.rater_group,
                  (SELECT w.display_name FROM workers w WHERE w.pid = a.worker_pid ORDER BY w.id LIMIT 1) AS subject
             FROM appraisal_nominations n
             JOIN appraisals a ON a.pid = n.appraisal_pid AND a.deleted_at IS NULL AND a.status = 'collecting'
            WHERE n.rater_pid = $1
              AND NOT EXISTS (SELECT 1 FROM appraisal_responses r WHERE r.nomination_pid = n.pid)
            ORDER BY n.id"#,
        rater.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let requests: Vec<_> = rows
        .iter()
        .map(|row| {
            serde_json::json!({
                "appraisal_pid": row.pid,
                "subject_pid": row.worker_pid,
                "subject": row.subject,
                "group": row.rater_group,
                "competencies": row.competencies,
            })
        })
        .collect();
    format::json(requests)
}

/// Find one live appraisal by pid, or 404.
async fn find_appraisal(ctx: &AppContext, pid: &str) -> Result<appraisals::Model> {
    sqlx::query_as!(
        appraisals::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, competencies, status,
                  shared_on, deleted_at AS "deleted_at: _"
             FROM appraisals WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// The appraisal routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/appraisals", post(create_appraisal))
        .add("/workers/{pid}/appraisals", get(list_appraisals))
        .add("/workers/{pid}/appraisal-requests", get(rater_requests))
        .add("/appraisals/{pid}", get(get_appraisal))
        .add("/appraisals/{pid}/nominations", post(nominate))
        .add("/appraisals/{pid}/status", post(appraisal_status))
        .add("/appraisals/{pid}/responses", post(respond))
        .add("/appraisals/{pid}/report", get(report))
}
