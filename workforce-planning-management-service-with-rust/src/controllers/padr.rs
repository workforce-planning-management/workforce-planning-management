//! **The personal development review** ([`crate::rules::padr`], WPM-R172, WPM-D176).
//!
//! A PADR is a review of kind `padr`, and it answers only here: none of the performance lifecycle applies to it. A manager in the
//! person's chain opens one and becomes its reviewer. The reviewer and the person write it, and each signs. The first signature
//! freezes the version's content and takes a snapshot of its objectives, which are the person's goals in its period (WPM-R225,
//! WPM-D165); both signatures complete it; a correction is a new version with the previous kept, and never moves the due date.
//!
//! Every write is under `/api/me`, resolved from the verified token, because neither party is HR. The person, the reviewer and
//! managers up the chain read it, audited with the pid and no content. HR reads the roll-up's counts and nothing else. Whether a
//! wellbeing conversation was offered is a yes or no, and nothing here can hold what was said.

use std::collections::HashMap;

use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use sea_orm::{ActiveValue, ConnectionTrait, QueryOrder, QuerySelect, SqlErr, TransactionTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::individual_goals::{
    audit_reads, check_plan_item, goal_snapshot, is_reviewer, objectives_of, reports_of, subject,
    views,
};
use super::{ensure_org_in_scope, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{assignments, padr_versions, reviews, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::notifications::Model as Notification;
use crate::models::records;
use crate::need_to_know::is_line_manager;
use crate::rules::career::is_self;
use crate::rules::individual_goals::{self as goals, RollupRow};
use crate::rules::padr::{self as rules, SignOffs, Signer, Standing};

/// How the caller stands to a PADR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    /// The person being reviewed.
    Reviewee,
    /// The person doing the review.
    Reviewer,
    /// A manager in the chain, who reads and does not write.
    Manager,
}

impl Role {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Reviewee => "reviewee",
            Self::Reviewer => "reviewer",
            Self::Manager => "manager",
        }
    }
}

/// The caller's role, or a 404: the record is not there for anyone else, and that includes HR (WPM-D176).
async fn role_of(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    review: &reviews::Model,
    worker: &workers::Model,
) -> Result<Role> {
    let sub = subject(caller)?;
    if is_self(sub, &worker.person_ref) {
        return Ok(Role::Reviewee);
    }
    if is_reviewer(ctx, caller, review).await? {
        return Ok(Role::Reviewer);
    }
    if is_line_manager(ctx, worker, sub).await? {
        return Ok(Role::Manager);
    }
    Err(Error::NotFound)
}

/// The two who write it; a manager further up reads.
fn writer(role: Role) -> Result<Signer> {
    match role {
        Role::Reviewee => Ok(Signer::Reviewee),
        Role::Reviewer => Ok(Signer::Reviewer),
        Role::Manager => Err(unprocessable(
            "only the reviewer and the person being reviewed can change a PADR",
        )),
    }
}

/// A live PADR and its worker.
async fn padr_and_worker(ctx: &AppContext, pid: &str) -> Result<(reviews::Model, workers::Model)> {
    let review = reviews::Entity::find()
        .filter(reviews::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(reviews::Column::Kind.eq("padr"))
        .filter(reviews::Column::DeletedAt.is_null())
        .one(&crate::db::orm(ctx))
        .await?
        .ok_or(Error::NotFound)?;
    let worker = records::worker_by_pid(ctx, review.worker_pid).await?;
    Ok((review, worker))
}

/// A PADR's versions, oldest first.
async fn versions_of<C: ConnectionTrait>(
    db: &C,
    review: Uuid,
) -> Result<Vec<padr_versions::Model>> {
    Ok(padr_versions::Entity::find()
        .filter(padr_versions::Column::ReviewPid.eq(review))
        .order_by_asc(padr_versions::Column::Version)
        .all(db)
        .await?)
}

/// The latest version, held for the transaction, so that two people signing at once cannot each miss the other's signature.
async fn latest_locked<C: ConnectionTrait>(db: &C, review: Uuid) -> Result<padr_versions::Model> {
    padr_versions::Entity::find()
        .filter(padr_versions::Column::ReviewPid.eq(review))
        .order_by_desc(padr_versions::Column::Version)
        .lock_exclusive()
        .one(db)
        .await?
        .ok_or(Error::NotFound)
}

const fn sign_offs(version: &padr_versions::Model) -> SignOffs {
    SignOffs {
        reviewer: version.reviewer_signed_on,
        reviewee: version.reviewee_signed_on,
    }
}

/// The worker a PADR's reviewer reference names. Every PADR's reviewer is set here to a worker's pid.
fn reviewer_pid(review: &reviews::Model) -> Option<Uuid> {
    review
        .reviewer_ref
        .rsplit(':')
        .next()
        .and_then(|id| Uuid::parse_str(id).ok())
}

/// The day each of these people's last PADR was completed.
async fn last_completed(ctx: &AppContext, people: &[Uuid]) -> Result<HashMap<Uuid, NaiveDate>> {
    let live = super::find_in(
        ctx,
        reviews::Entity::find()
            .filter(reviews::Column::Kind.eq("padr"))
            .filter(reviews::Column::DeletedAt.is_null()),
        reviews::Column::WorkerPid,
        people,
    )
    .await?;
    let pids: Vec<Uuid> = live.iter().map(|r| r.pid).collect();
    let mut by_review: HashMap<Uuid, Vec<padr_versions::Model>> = HashMap::new();
    for row in super::find_in(
        ctx,
        padr_versions::Entity::find().order_by_asc(padr_versions::Column::Version),
        padr_versions::Column::ReviewPid,
        &pids,
    )
    .await?
    {
        by_review.entry(row.review_pid).or_default().push(row);
    }
    let mut last: HashMap<Uuid, NaiveDate> = HashMap::new();
    for review in &live {
        let offs: Vec<SignOffs> = by_review
            .get(&review.pid)
            .map(|rows| rows.iter().map(sign_offs).collect())
            .unwrap_or_default();
        if let Some(day) = rules::completed_on(&offs) {
            let entry = last.entry(review.worker_pid).or_insert(day);
            *entry = (*entry).max(day);
        }
    }
    Ok(last)
}

/// The start of each of these people's current primary assignment.
async fn primary_starts(
    ctx: &AppContext,
    people: &[Uuid],
    today: NaiveDate,
) -> Result<HashMap<Uuid, NaiveDate>> {
    let rows = super::find_in(
        ctx,
        assignments::Entity::find().filter(assignments::Column::IsPrimary.eq(true)),
        assignments::Column::WorkerPid,
        people,
    )
    .await?;
    Ok(rows
        .into_iter()
        .filter(|row| row.ends_on.is_none_or(|end| end >= today))
        .map(|row| (row.worker_pid, row.starts_on))
        .collect())
}

/// Where each of these people stands against the cycle today.
async fn standings(ctx: &AppContext, people: &[Uuid]) -> Result<HashMap<Uuid, Standing>> {
    let today = Utc::now().date_naive();
    let last = last_completed(ctx, people).await?;
    let starts = primary_starts(ctx, people, today).await?;
    let months =
        rules::first_after_months(std::env::var("WPM_PADR_FIRST_AFTER_MONTHS").ok().as_deref());
    Ok(people
        .iter()
        .map(|pid| {
            (
                *pid,
                rules::standing(
                    last.get(pid).copied(),
                    starts.get(pid).copied(),
                    months,
                    today,
                ),
            )
        })
        .collect())
}

fn standing_json(standing: Option<Standing>) -> serde_json::Value {
    let standing = standing.unwrap_or(Standing::Unknown);
    serde_json::json!({ "standing": standing.word(), "due_on": standing.due_on() })
}

/// One line for each of these PADRs: where it has got to, and no content.
async fn summaries(
    ctx: &AppContext,
    people: &[Uuid],
) -> Result<HashMap<Uuid, Vec<serde_json::Value>>> {
    let mut rows = reviews::Entity::find()
        .filter(reviews::Column::Kind.eq("padr"))
        .filter(reviews::Column::DeletedAt.is_null())
        .filter(reviews::Column::WorkerPid.is_in(people.iter().copied()))
        .order_by_desc(reviews::Column::Id)
        .all(&crate::db::orm(ctx))
        .await?;
    let mut lines: HashMap<Uuid, Vec<serde_json::Value>> = HashMap::new();
    for review in rows.drain(..) {
        let versions = versions_of(&crate::db::orm(ctx), review.pid).await?;
        let offs: Vec<SignOffs> = versions.iter().map(sign_offs).collect();
        let latest = versions.last();
        lines
            .entry(review.worker_pid)
            .or_default()
            .push(serde_json::json!({
                "pid": review.pid,
                "status": review.status,
                "completed_on": rules::completed_on(&offs),
                "version": latest.map(|v| v.version),
                "period_start": latest.map(|v| v.period_start),
                "period_end": latest.map(|v| v.period_end),
            }));
    }
    Ok(lines)
}

/// `GET /api/me/padrs` — your PADRs and when the next is due, and the same for everyone below you, by name. No content, so nothing
/// here is a read of a review.
#[debug_handler]
pub(crate) async fn mine_and_team(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
) -> Result<Response> {
    let sub = subject(&caller)?.to_string();
    let me = super::me::me_worker(&ctx, &caller).await?;
    let reports = reports_of(&ctx, &sub).await?;
    let people: Vec<Uuid> = reports
        .iter()
        .map(|w| w.pid)
        .chain(std::iter::once(me.pid))
        .collect();
    let standing = standings(&ctx, &people).await?;
    let mut lines = summaries(&ctx, &people).await?;
    let team: Vec<serde_json::Value> = reports
        .iter()
        .map(|worker| {
            serde_json::json!({
                "worker_pid": worker.pid,
                "display_name": worker.display_name,
                "due": standing_json(standing.get(&worker.pid).copied()),
                "padrs": lines.remove(&worker.pid).unwrap_or_default(),
            })
        })
        .collect();
    format::json(serde_json::json!({
        "worker_pid": me.pid,
        "due": standing_json(standing.get(&me.pid).copied()),
        "padrs": lines.remove(&me.pid).unwrap_or_default(),
        "team": team,
    }))
}

/// `POST /api/me/reports/{pid}/padrs` body.
#[derive(Debug, Deserialize)]
pub(crate) struct OpenPayload {
    period_start: NaiveDate,
    period_end: NaiveDate,
}

/// `POST /api/me/reports/{pid}/padrs` — open a PADR with someone in your reporting line. You become its reviewer.
#[debug_handler]
pub(crate) async fn open(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<OpenPayload>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let sub = subject(&caller)?;
    if is_self(sub, &worker.person_ref) || !is_line_manager(&ctx, &worker, sub).await? {
        return Err(Error::NotFound);
    }
    if worker.terminated_on.is_some() {
        return Err(unprocessable("this person has left"));
    }
    rules::valid_period(payload.period_start, payload.period_end).map_err(|e| unprocessable(&e))?;
    let me = super::me::me_worker(&ctx, &caller).await?;
    let txn = crate::db::orm(&ctx).begin().await?;
    let review = reviews::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        cycle_pid: ActiveValue::set(None),
        kind: ActiveValue::set("padr".to_string()),
        worker_pid: ActiveValue::set(worker.pid),
        reviewer_ref: ActiveValue::set(format!("worker:{}", me.pid)),
        status: ActiveValue::set("draft".to_string()),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(|e| match e.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(_)) => {
            unprocessable("this person already has a PADR open")
        }
        _ => Error::from(e),
    })?;
    padr_versions::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        review_pid: ActiveValue::set(review.pid),
        worker_pid: ActiveValue::set(worker.pid),
        version: ActiveValue::set(1),
        period_start: ActiveValue::set(payload.period_start),
        period_end: ActiveValue::set(payload.period_end),
        development_needs: ActiveValue::set(serde_json::json!([])),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "padr",
        review.pid,
        "padr_opened",
        caller.actor(),
        None,
    )
    .await?;
    Notification::push(
        &txn,
        worker.pid,
        "padr_opened",
        "A personal development review has been opened with you.",
        serde_json::json!({ "padr_pid": review.pid }),
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": review.pid }))
}

/// `GET /api/me/padrs/{pid}` — one PADR with every version: the person, the reviewer and managers in the chain. An unsigned
/// version shows the live goals of its period and a signed one what it froze. Audited with the pid and no content.
#[debug_handler]
pub(crate) async fn detail(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (review, worker) = padr_and_worker(&ctx, &pid).await?;
    let role = role_of(&ctx, &caller, &review, &worker).await?;
    let versions = versions_of(&crate::db::orm(&ctx), review.pid).await?;
    let latest = versions.last().ok_or(Error::NotFound)?;
    let live = if sign_offs(latest).frozen() {
        Vec::new()
    } else {
        objectives_of(
            &crate::db::orm(&ctx),
            worker.pid,
            (latest.period_start, latest.period_end),
        )
        .await?
    };
    Audit::record(
        &crate::db::orm(&ctx),
        "padr",
        review.pid,
        "padr_read",
        caller.actor(),
        None,
    )
    .await?;
    audit_reads(&ctx, &caller, &live).await?;
    let offs: Vec<SignOffs> = versions.iter().map(sign_offs).collect();
    let rows: Vec<serde_json::Value> = versions
        .iter()
        .map(|version| {
            let state = sign_offs(version);
            serde_json::json!({
                "version": version,
                "frozen": state.frozen(),
                "complete": state.complete(),
            })
        })
        .collect();
    format::json(serde_json::json!({
        "pid": review.pid,
        "worker_pid": worker.pid,
        "display_name": worker.display_name,
        "role": role.as_str(),
        "status": review.status,
        "completed_on": rules::completed_on(&offs),
        "versions": rows,
        "objectives": views(&ctx, &live).await?,
    }))
}

/// One development need, which may link to a step on the person's plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Need {
    need: String,
    #[serde(default)]
    plan_item_pid: Option<Uuid>,
}

/// `PUT /api/me/padrs/{pid}` body.
#[derive(Debug, Deserialize)]
pub(crate) struct EditPayload {
    period_start: NaiveDate,
    period_end: NaiveDate,
    #[serde(default)]
    values_summary: Option<String>,
    #[serde(default)]
    wellbeing_offered: Option<bool>,
    #[serde(default)]
    development_needs: Vec<Need>,
}

/// `PUT /api/me/padrs/{pid}` — change the latest version until someone signs it.
#[debug_handler]
pub(crate) async fn edit(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<EditPayload>,
) -> Result<Response> {
    let (review, worker) = padr_and_worker(&ctx, &pid).await?;
    writer(role_of(&ctx, &caller, &review, &worker).await?)?;
    let reject = |e: String| unprocessable(&e);
    rules::valid_period(payload.period_start, payload.period_end).map_err(reject)?;
    let summary = rules::clean_text(payload.values_summary.as_deref()).map_err(reject)?;
    rules::valid_needs(payload.development_needs.iter().map(|n| n.need.as_str()))
        .map_err(reject)?;
    for need in &payload.development_needs {
        if let Some(item) = need.plan_item_pid {
            check_plan_item(&ctx, &worker, item).await?;
        }
    }
    let needs: Vec<Need> = payload
        .development_needs
        .iter()
        .map(|n| Need {
            need: n.need.trim().to_string(),
            plan_item_pid: n.plan_item_pid,
        })
        .collect();
    let txn = crate::db::orm(&ctx).begin().await?;
    let latest = latest_locked(&txn, review.pid).await?;
    rules::check_editable(sign_offs(&latest)).map_err(reject)?;
    let mut active: padr_versions::ActiveModel = latest.into();
    active.period_start = ActiveValue::set(payload.period_start);
    active.period_end = ActiveValue::set(payload.period_end);
    active.values_summary = ActiveValue::set(summary);
    active.wellbeing_offered = ActiveValue::set(payload.wellbeing_offered);
    active.development_needs = ActiveValue::set(serde_json::json!(needs));
    active.update(&txn).await?;
    Audit::record(
        &txn,
        "padr",
        review.pid,
        "padr_edited",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": review.pid }))
}

/// `POST /api/me/padrs/{pid}/sign` — sign the latest version, dated today. The first signature takes the objectives' snapshot, and
/// both complete the version. The other person is told their signature is waiting (WPM-R232).
#[debug_handler]
pub(crate) async fn sign(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (review, worker) = padr_and_worker(&ctx, &pid).await?;
    let who = writer(role_of(&ctx, &caller, &review, &worker).await?)?;
    let today = Utc::now().date_naive();
    let txn = crate::db::orm(&ctx).begin().await?;
    let latest = latest_locked(&txn, review.pid).await?;
    rules::ready_to_sign(latest.wellbeing_offered).map_err(unprocessable)?;
    let before = sign_offs(&latest);
    let after = rules::sign(before, who, today, today).map_err(|e| unprocessable(&e))?;
    let period = (latest.period_start, latest.period_end);
    let version = latest.version;
    let mut active: padr_versions::ActiveModel = latest.into();
    if !before.frozen() {
        let mut snapshot = Vec::new();
        for goal in objectives_of(&txn, worker.pid, period).await? {
            snapshot.push(goal_snapshot(&txn, &goal).await?);
        }
        active.objectives = ActiveValue::set(Some(serde_json::json!(snapshot)));
    }
    active.reviewer_signed_on = ActiveValue::set(after.reviewer);
    active.reviewee_signed_on = ActiveValue::set(after.reviewee);
    active.update(&txn).await?;
    Audit::record(
        &txn,
        "padr",
        review.pid,
        "padr_signed",
        caller.actor(),
        Some(serde_json::json!({
            "version": version,
            "by": if who == Signer::Reviewer { "reviewer" } else { "reviewee" },
        })),
    )
    .await?;
    if !after.complete() {
        let other = match who {
            Signer::Reviewer => Some(worker.pid),
            Signer::Reviewee => reviewer_pid(&review),
        };
        if let Some(other) = other {
            Notification::push(
                &txn,
                other,
                "padr_to_sign",
                "A personal development review is waiting for your signature.",
                serde_json::json!({ "padr_pid": review.pid }),
            )
            .await?;
        }
    }
    let complete = after.complete();
    if complete && review.status == "draft" {
        let mut row: reviews::ActiveModel = review.clone().into();
        row.status = ActiveValue::set("shared".to_string());
        row.update(&txn).await?;
    }
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": review.pid, "complete": complete }))
}

/// `POST /api/me/padrs/{pid}/comment` body.
#[derive(Debug, Deserialize)]
pub(crate) struct CommentPayload {
    #[serde(default)]
    comment: Option<String>,
}

/// `POST /api/me/padrs/{pid}/comment` — the person's comment on the latest version. It is not content, so it stays open after
/// signing, and nobody else can write it.
#[debug_handler]
pub(crate) async fn comment(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<CommentPayload>,
) -> Result<Response> {
    let (review, worker) = padr_and_worker(&ctx, &pid).await?;
    if role_of(&ctx, &caller, &review, &worker).await? != Role::Reviewee {
        return Err(unprocessable(
            "only the person being reviewed can comment here",
        ));
    }
    let text = rules::clean_text(payload.comment.as_deref()).map_err(|e| unprocessable(&e))?;
    let txn = crate::db::orm(&ctx).begin().await?;
    let mut active: padr_versions::ActiveModel = latest_locked(&txn, review.pid).await?.into();
    active.reviewee_comment = ActiveValue::set(text);
    active.update(&txn).await?;
    Audit::record(
        &txn,
        "padr",
        review.pid,
        "padr_commented",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": review.pid }))
}

/// `POST /api/me/padrs/{pid}/versions` — a correction: a new unsigned version copied from the signed one, which is kept.
#[debug_handler]
pub(crate) async fn correct(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (review, worker) = padr_and_worker(&ctx, &pid).await?;
    writer(role_of(&ctx, &caller, &review, &worker).await?)?;
    let txn = crate::db::orm(&ctx).begin().await?;
    let latest = latest_locked(&txn, review.pid).await?;
    rules::may_correct(sign_offs(&latest)).map_err(unprocessable)?;
    let next = padr_versions::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        review_pid: ActiveValue::set(review.pid),
        worker_pid: ActiveValue::set(latest.worker_pid),
        version: ActiveValue::set(latest.version + 1),
        period_start: ActiveValue::set(latest.period_start),
        period_end: ActiveValue::set(latest.period_end),
        values_summary: ActiveValue::set(latest.values_summary),
        wellbeing_offered: ActiveValue::set(latest.wellbeing_offered),
        development_needs: ActiveValue::set(latest.development_needs),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "padr",
        review.pid,
        "padr_versioned",
        caller.actor(),
        Some(serde_json::json!({ "version": next.version })),
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": review.pid, "version": next.version }))
}

/// Query for the roll-up.
#[derive(Debug, Deserialize)]
struct RollupQuery {
    organization: String,
}

/// `GET /api/padrs/rollup?organization=` — people whose PADR is current, overdue or unknown, by department, with the floor. Counts
/// only.
#[debug_handler]
async fn rollup(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(query): axa::extract::Query<RollupQuery>,
) -> Result<Response> {
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let staff = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&query.organization))
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::TerminatedOn.is_null())
        .all(&crate::db::orm(&ctx))
        .await?;
    let people: Vec<Uuid> = staff.iter().map(|w| w.pid).collect();
    let standing = standings(&ctx, &people).await?;
    let rows: Vec<RollupRow> = staff
        .into_iter()
        .map(|worker| RollupRow {
            status: standing
                .get(&worker.pid)
                .copied()
                .unwrap_or(Standing::Unknown)
                .word()
                .to_string(),
            department: worker.department,
            worker: worker.pid,
        })
        .collect();
    format::json(serde_json::json!({
        "organization": query.organization,
        "departments": goals::rollup(&rows),
        "floor": goals::ROLLUP_FLOOR,
        "derivation": "People in each department by where they stand against the twelve-month cycle. Unknown means no PADR is \
                       on record and no due date can be worked out, and is never counted as current. A department with too \
                       few people is withheld, and so is the next smallest when exactly one is. Nobody is named.",
    }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/padrs/rollup", get(rollup))
}
