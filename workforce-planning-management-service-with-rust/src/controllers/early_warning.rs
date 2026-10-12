//! **Early warning** (WPM-R68, WPM-T115): when a skill pool becomes over-committed in the coming
//! months, or an oversight measure is in breach in the last complete month, the organization's HR
//! administrators get an in-app notification, and the service records the date the condition was
//! first seen true and the date a person acknowledged it. The calendar days between the two are the
//! **time to report**, which is itself an oversight measure (`time_to_report`).
//!
//! The date "first true" is the date of the first sweep that saw it: a sweep run less often than
//! daily dates a condition late, and the answer says so. A warning names a pool or a measure, never a
//! person, and the notification carries no figure (WPM-D23).

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::Deserialize;
use serde_json::json;

use super::{capacity, oversight, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{early_warnings as warnings, organization_memberships};
use crate::models::audit_logs::Model as Audit;
use crate::models::memberships;
use crate::models::notifications::Model as Notification;
use crate::rules::capacity::CellStatus;
use crate::rules::early_warning::{self as rules, Key};
use crate::rules::oversight::{self as oversight_rules, Reason};

/// What a sweep did.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Report {
    /// Warnings newly opened.
    pub opened: usize,
    /// Warnings whose condition has stopped.
    pub cleared: usize,
    /// Notifications created.
    pub told: usize,
}

/// The conditions true for the organization on `today`, with the name of what each is about.
async fn true_now(ctx: &AppContext, org: &str, today: NaiveDate) -> Result<BTreeMap<Key, String>> {
    let mut found = BTreeMap::new();
    let cap = capacity::load_org_capacity(ctx, org, today, rules::HORIZON_MONTHS).await?;
    for pool in &cap.pools {
        let over = cap
            .views
            .get(&pool.pid.to_string())
            .is_some_and(|v| v.cells.iter().any(|c| c.status == CellStatus::Over));
        if over {
            found.insert(
                Key {
                    kind: "pool_over_committed".into(),
                    subject: pool.pid,
                },
                pool.name.clone(),
            );
        }
    }
    let latest = oversight::last_complete_month(today);
    for m in oversight::current_measures(ctx, org, today).await? {
        let rs = oversight::measure_readings(ctx, org, &m, latest).await?;
        let verdict = oversight_rules::evaluate_measure(&oversight::spec_of(&m), &rs, latest);
        if verdict.reason == Reason::Breach(latest) {
            found.insert(
                Key {
                    kind: "measure_breach".into(),
                    subject: m.pid,
                },
                m.name.clone(),
            );
        }
    }
    Ok(found)
}

/// The workers who hold an `hr_admin` membership in the organization.
async fn hr_admins(ctx: &AppContext, org: &str) -> Result<Vec<Uuid>> {
    let mut out: Vec<Uuid> = Vec::new();
    for m in organization_memberships::Entity::find()
        .filter(organization_memberships::Column::OrganizationRef.eq(org))
        .filter(organization_memberships::Column::Role.eq("hr_admin"))
        .filter(organization_memberships::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
    {
        if let Some(pid) = m.worker_pid
            && !out.contains(&pid)
        {
            out.push(pid);
        }
    }
    Ok(out)
}

/// Compare what is true on `today` with what is recorded as open, open and clear warnings, and tell
/// the HR administrators about each new one.
///
/// # Errors
///
/// A database error.
pub async fn sweep(ctx: &AppContext, org: &str, today: NaiveDate) -> Result<Report> {
    let now = true_now(ctx, org, today).await?;
    let open_rows = warnings::Entity::find()
        .filter(warnings::Column::OrganizationRef.eq(org))
        .filter(warnings::Column::ClearedOn.is_null())
        .all(&ctx.db)
        .await?;
    let open: Vec<Key> = open_rows
        .iter()
        .map(|w| Key {
            kind: w.kind.clone(),
            subject: w.subject_pid,
        })
        .collect();
    let keys: BTreeSet<Key> = now.keys().cloned().collect();
    let plan = rules::reconcile(&open, &keys);
    let mut report = Report::default();
    let recipients = if plan.opened.is_empty() {
        Vec::new()
    } else {
        hr_admins(ctx, org).await?
    };
    for key in &plan.opened {
        let name = now.get(key).cloned().unwrap_or_default();
        let row = warnings::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            organization_ref: ActiveValue::set(org.to_string()),
            kind: ActiveValue::set(key.kind.clone()),
            subject_pid: ActiveValue::set(key.subject),
            subject_name: ActiveValue::set(name.clone()),
            first_true_on: ActiveValue::set(today),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?;
        let what = if key.kind == "pool_over_committed" {
            format!("The skill pool \"{name}\" is over-committed in the coming months.")
        } else {
            format!("The measure \"{name}\" is in breach for the last complete month.")
        };
        for to in &recipients {
            Notification::push(
                &ctx.db,
                *to,
                "early_warning",
                &what,
                json!({ "warning_pid": row.pid, "kind": key.kind, "subject_pid": key.subject }),
            )
            .await?;
            report.told += 1;
        }
        report.opened += 1;
    }
    for key in &plan.cleared {
        if let Some(row) = open_rows
            .iter()
            .find(|w| w.kind == key.kind && w.subject_pid == key.subject)
        {
            let mut a: warnings::ActiveModel = row.clone().into();
            a.cleared_on = ActiveValue::set(Some(today));
            a.update(&ctx.db).await?;
            report.cleared += 1;
        }
    }
    Ok(report)
}

async fn ensure_org_in_scope(ctx: &AppContext, caller: &MaybeAuthUser, org: &str) -> Result<()> {
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.iter().any(|r| r == org)
    {
        return Err(Error::NotFound);
    }
    Ok(())
}

/// Query naming the organization.
#[derive(Debug, Deserialize)]
pub struct OrgQuery {
    organization: Option<String>,
}

async fn org_of(ctx: &AppContext, caller: &MaybeAuthUser, q: OrgQuery) -> Result<String> {
    let org = q
        .organization
        .ok_or_else(|| unprocessable("organization is required"))?;
    ensure_org_in_scope(ctx, caller, &org).await?;
    Ok(org)
}

/// `POST /api/early-warnings/sweep?organization=` — look now; open, clear and notify.
#[debug_handler]
async fn run_sweep(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(q): axum::extract::Query<OrgQuery>,
) -> Result<Response> {
    let org = org_of(&ctx, &caller, q).await?;
    let today = chrono::Utc::now().date_naive();
    let report = sweep(&ctx, &org, today).await?;
    format::json(json!({
        "organization_ref": org, "as_of": today,
        "opened": report.opened, "cleared": report.cleared, "told": report.told,
    }))
}

fn warning_json(w: &warnings::Model, today: NaiveDate) -> serde_json::Value {
    let until = w.cleared_on.unwrap_or(today);
    json!({
        "pid": w.pid, "kind": w.kind, "subject_pid": w.subject_pid, "subject_name": w.subject_name,
        "first_true_on": w.first_true_on, "cleared_on": w.cleared_on, "acknowledged_on": w.acknowledged_on,
        "days_open": (until - w.first_true_on).num_days(),
        "days_to_acknowledge": rules::days_to_acknowledge(w.first_true_on, w.acknowledged_on),
    })
}

/// `GET /api/early-warnings?organization=` — the warnings, newest first, and the time to report.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(q): axum::extract::Query<OrgQuery>,
) -> Result<Response> {
    let org = org_of(&ctx, &caller, q).await?;
    let today = chrono::Utc::now().date_naive();
    let rows = warnings::Entity::find()
        .filter(warnings::Column::OrganizationRef.eq(&org))
        .order_by_desc(warnings::Column::FirstTrueOn)
        .order_by_desc(warnings::Column::Id)
        .all(&ctx.db)
        .await?;
    let pairs: Vec<(NaiveDate, Option<NaiveDate>)> = rows
        .iter()
        .map(|w| (w.first_true_on, w.acknowledged_on))
        .collect();
    format::json(json!({
        "organization_ref": org,
        "warnings": rows.iter().map(|w| warning_json(w, today)).collect::<Vec<_>>(),
        "unacknowledged": rows.iter().filter(|w| w.acknowledged_on.is_none()).count(),
        "time_to_report_days": rules::time_to_report_days(&pairs),
        "basis": "The date a condition was first true is the date of the first sweep that saw it, so a sweep \
                  run less often than daily dates it late. Time to report is the longest calendar days from \
                  that date to a person acknowledging it, across all the warnings listed; it is null while \
                  any is unacknowledged, because a slow one must not read as a fast one.",
    }))
}

/// `POST /api/early-warnings/{pid}/acknowledge` — a person says they have seen it. Once.
#[debug_handler]
async fn acknowledge(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let pid = crate::models::records::parse_pid(&pid)?;
    let row = warnings::Entity::find()
        .filter(warnings::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    ensure_org_in_scope(&ctx, &caller, &row.organization_ref).await?;
    if row.acknowledged_on.is_some() {
        return Err(unprocessable("this warning is already acknowledged"));
    }
    let today = chrono::Utc::now().date_naive();
    let mut a: warnings::ActiveModel = row.into();
    a.acknowledged_on = ActiveValue::set(Some(today));
    let row = a.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "early_warning",
        row.pid,
        "acknowledged",
        caller.actor(),
        None,
    )
    .await?;
    format::json(warning_json(&row, today))
}

/// The early-warning routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/early-warnings", get(list))
        .add("/early-warnings/sweep", post(run_sweep))
        .add("/early-warnings/{pid}/acknowledge", post(acknowledge))
}
