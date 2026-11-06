//! The consultation register (WPM-R105, WPM-D68): one record per proposal that may trigger a collective duty. It holds the body consulted, the
//! proposal as counts and dates, topics, a document **reference**, when consultation began and ended and how, never an attendee, a
//! representative or what was said. The timeline reads the pure rules in `rules::consultation` (a suggestion with its evidence, the deployer's
//! to confirm) and says whether consultation began in time. HR writes and reads; audited without text. Spec: `spec/worker-unions.md`.

use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::bargaining::find_body;
use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db::{self, pool};
use crate::models::_entities::consultations;
use crate::rules::agreement as agreement_rules;
use crate::rules::consultation as consult;
use crate::rules::consultation_register as rules;

fn json_of(c: &consultations::Model) -> serde_json::Value {
    json!({
        "pid": c.pid, "organization_ref": c.organization_ref, "body_pid": c.body_pid, "plan_pid": c.plan_pid,
        "establishment": c.establishment, "profile": c.profile, "establishment_size": c.establishment_size,
        "dismissals": c.dismissals, "earliest_termination_on": c.earliest_termination_on,
        "topics": c.topics.split(',').collect::<Vec<_>>(), "document_ref": c.document_ref,
        "started_on": c.started_on, "ended_on": c.ended_on, "outcome": c.outcome, "agreement_pid": c.agreement_pid,
    })
}

async fn find(ctx: &AppContext, caller: &MaybeAuthUser, pid: Uuid) -> Result<consultations::Model> {
    let row = sqlx::query_as!(
        consultations::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, body_pid, plan_pid,
                  establishment, profile, establishment_size, dismissals, earliest_termination_on, topics, document_ref, started_on,
                  ended_on, outcome, agreement_pid, recorded_by
           FROM consultations WHERE pid = $1"#,
        pid,
    )
    .fetch_optional(pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &row.organization_ref).await?;
    Ok(row)
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListQuery {
    organization: String,
}

/// `GET /api/consultations?organization=` — the register, newest termination date first. HR.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let rows = sqlx::query_as!(
        consultations::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, body_pid, plan_pid,
                  establishment, profile, establishment_size, dismissals, earliest_termination_on, topics, document_ref, started_on,
                  ended_on, outcome, agreement_pid, recorded_by
           FROM consultations WHERE organization_ref = $1 ORDER BY earliest_termination_on DESC"#,
        query.organization,
    )
    .fetch_all(pool(&ctx))
    .await?;
    format::json(json!({ "consultations": rows.iter().map(json_of).collect::<Vec<_>>() }))
}

#[derive(Debug, Deserialize)]
struct OpenPayload {
    body_pid: Uuid,
    plan_pid: Option<Uuid>,
    establishment: String,
    profile: String,
    establishment_size: i32,
    dismissals: i32,
    earliest_termination_on: NaiveDate,
    topics: Vec<String>,
    document_ref: Option<String>,
}

/// `POST /api/consultations` — open a record for a proposal: the body, the establishment, the profile of thresholds in use, counts and a date,
/// topics and a document reference. Nothing about who is affected or who attends.
#[debug_handler]
async fn open(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<OpenPayload>,
) -> Result<Response> {
    let body = find_body(&ctx, &caller, payload.body_pid).await?;
    ensure_valid(&rules::open_problems(
        &payload.establishment,
        payload.establishment_size,
        payload.dismissals,
        payload.document_ref.as_deref(),
        &payload.topics,
    ))?;
    if consult::profile(&payload.profile).is_none() {
        return Err(unprocessable(&format!(
            "profile must be one of: {}",
            consult::PROFILES.join(", ")
        )));
    }
    if let Some(plan) = payload.plan_pid {
        let found = sqlx::query_scalar!(
            r#"SELECT EXISTS (
                   SELECT 1 FROM workforce_plans WHERE pid = $1 AND organization_ref = $2
               ) AS "found!""#,
            plan,
            body.organization_ref,
        )
        .fetch_one(pool(&ctx))
        .await?;
        if !found {
            return Err(unprocessable(
                "plan_pid is not a workforce plan of this organization",
            ));
        }
    }
    let topics = rules::clean_topics(&payload.topics).map_err(|p| unprocessable(&p))?;
    let pid = Uuid::new_v4();
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO consultations
             (pid, organization_ref, body_pid, plan_pid, establishment, profile, establishment_size, dismissals,
              earliest_termination_on, topics, document_ref, outcome, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'open', $12)",
        pid,
        body.organization_ref,
        body.pid,
        payload.plan_pid,
        payload.establishment.trim(),
        payload.profile,
        payload.establishment_size,
        payload.dismissals,
        payload.earliest_termination_on,
        topics,
        payload.document_ref.map(|r| r.trim().to_string()),
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "consultation",
        pid,
        "consultation_opened",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

#[derive(Debug, Deserialize)]
struct BeginPayload {
    started_on: NaiveDate,
    document_ref: Option<String>,
}

/// `POST /api/consultations/{pid}/begin` — record the day consultation began (once, not in the future), and optionally the reference to the documents shared.
#[debug_handler]
async fn begin(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(payload): Json<BeginPayload>,
) -> Result<Response> {
    let row = find(&ctx, &caller, pid).await?;
    let mut problems = rules::begin_problems(
        payload.started_on,
        Utc::now().date_naive(),
        row.started_on.is_some(),
    );
    if row.outcome != "open" {
        problems.push("a closed consultation cannot begin".to_string());
    }
    problems.extend(rules::open_problems(
        &row.establishment,
        row.establishment_size,
        row.dismissals,
        payload.document_ref.as_deref(),
        &[String::from("other")],
    ));
    ensure_valid(&problems)?;
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE consultations SET started_on = $2, document_ref = COALESCE($3, document_ref) WHERE pid = $1",
        pid,
        payload.started_on,
        payload.document_ref.map(|r| r.trim().to_string()),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "consultation",
        pid,
        "consultation_begun",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

#[derive(Debug, Deserialize)]
struct ClosePayload {
    outcome: String,
    ended_on: NaiveDate,
    agreement_pid: Option<Uuid>,
}

/// `POST /api/consultations/{pid}/close` — end it: agreed (optionally linking the agreement, which must be one of this body's units'),
/// `no_agreement` or `withdrawn`, on a day not in the future and not before it began.
#[debug_handler]
async fn close(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(payload): Json<ClosePayload>,
) -> Result<Response> {
    let row = find(&ctx, &caller, pid).await?;
    let mut problems = rules::close_problems(
        &payload.outcome,
        payload.ended_on,
        row.started_on,
        Utc::now().date_naive(),
        payload.agreement_pid.is_some(),
    );
    if row.outcome != "open" {
        problems.push("this consultation is already closed".to_string());
    }
    ensure_valid(&problems)?;
    if let Some(agreement) = payload.agreement_pid {
        let belongs = sqlx::query_scalar!(
            r#"SELECT EXISTS (
                   SELECT 1 FROM collective_agreements
                   WHERE pid = $1 AND unit_pid IN (SELECT pid FROM bargaining_units WHERE body_pid = $2)
               ) AS "belongs!""#,
            agreement,
            row.body_pid,
        )
        .fetch_one(pool(&ctx))
        .await?;
        if !belongs {
            return Err(unprocessable(
                "agreement_pid is not an agreement of a unit of the body consulted",
            ));
        }
    }
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE consultations SET outcome = $2, ended_on = $3, agreement_pid = $4 WHERE pid = $1",
        pid,
        payload.outcome,
        payload.ended_on,
        payload.agreement_pid,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "consultation",
        pid,
        "consultation_closed",
        caller.actor(),
        Some(json!({ "outcome": payload.outcome })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// `GET /api/consultations/{pid}/timeline` — the record with the pure rules read over it: whether a duty may apply and the evidence, the latest
/// day consultation should begin and the authority be notified, a longer period an agreement sets (the longest in force today for the
/// body's units), the checklist, and whether consultation began in time. A suggestion, not a verdict; the thresholds are the deployer's to confirm.
#[debug_handler]
async fn timeline(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let row = find(&ctx, &caller, pid).await?;
    let today = Utc::now().date_naive();
    let profile = consult::profile(&row.profile)
        .ok_or_else(|| Error::string("the record names a profile this service does not know"))?;
    let others: Vec<(NaiveDate, u32)> = sqlx::query!(
        "SELECT earliest_termination_on, dismissals FROM consultations
         WHERE organization_ref = $1 AND establishment = $2 AND pid <> $3 AND outcome <> 'withdrawn'",
        row.organization_ref,
        row.establishment,
        row.pid,
    )
    .fetch_all(pool(&ctx))
    .await?
    .into_iter()
    .map(|o| {
        (
            o.earliest_termination_on,
            u32::try_from(o.dismissals).unwrap_or(0),
        )
    })
    .collect();
    let agreements = sqlx::query!(
        "SELECT unit_pid, title, version, effective_from, effective_to, consultation_days FROM collective_agreements
         WHERE unit_pid IN (SELECT pid FROM bargaining_units WHERE body_pid = $1)",
        row.body_pid,
    )
    .fetch_all(pool(&ctx))
    .await?;
    let agreed_period = agreements
        .iter()
        .filter(|a| {
            let same: Vec<agreement_rules::Version> = agreements
                .iter()
                .filter(|o| o.unit_pid == a.unit_pid && o.title == a.title)
                .map(|o| agreement_rules::Version {
                    version: o.version,
                    from: o.effective_from,
                    to: o.effective_to,
                })
                .collect();
            agreement_rules::in_force(&same, today) == Some(a.version)
        })
        .filter_map(|a| a.consultation_days)
        .max()
        .map(i64::from);
    let proposal = consult::Proposal {
        establishment_size: u32::try_from(row.establishment_size).unwrap_or(0),
        dismissals: u32::try_from(row.dismissals).unwrap_or(0),
        earliest_termination: row.earliest_termination_on,
        others,
        agreed_period_days: agreed_period,
    };
    let assessment = consult::assess(&profile, &proposal, today);
    let timeliness = rules::timeliness(&assessment, row.started_on, today);
    format::json(json!({
        "consultation": json_of(&row),
        "assessment": {
            "profile": assessment.profile, "source": assessment.source, "may_apply": assessment.may_apply,
            "evidence": { "counted": assessment.evidence.counted, "needed": assessment.evidence.needed, "window_days": assessment.evidence.window_days },
            "consultation_days": assessment.consultation_days,
            "latest_consultation_start": assessment.latest_consultation_start,
            "latest_authority_notice": assessment.latest_authority_notice,
            "days_remaining": assessment.days_remaining, "already_late": assessment.already_late,
            "checklist": assessment.checklist,
            "agreed_period_days": agreed_period,
            "deployer_to_confirm": assessment.deployer_to_confirm,
        },
        "timeliness": timeliness,
        "derivation": "a suggestion with its evidence, never a verdict: whether a collective duty may apply under the named profile (the deployer's to confirm), the latest days to begin consultation and notify the authority, and whether consultation began in time. A longer period set by a collective agreement in force today is used if there is one.",
    }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/consultations", get(list))
        .add("/consultations", post(open))
        .add("/consultations/{pid}/begin", post(begin))
        .add("/consultations/{pid}/close", post(close))
        .add("/consultations/{pid}/timeline", get(timeline))
}
