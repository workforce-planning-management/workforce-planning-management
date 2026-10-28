//! Assignments (WPM-R161, WPM-D112): the jobs a worker holds. HR writes; the worker reads their own. A change of title, department,
//! location, categories or FTE **ends one assignment and starts another** on the effective day, so the record answers "what did they
//! do on that day". The worker row stays the record of the primary assignment and is written through when the primary changes.
//! Spec: `spec/staff-records-and-pay.md`.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{
    ensure_org_in_scope, ensure_scale, ensure_valid, forbidden, row_json, unprocessable,
    writable_worker,
};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{assignments, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::assignment as rules;

fn as_rule(a: &assignments::Model) -> rules::Assignment {
    rules::Assignment {
        fte_percent: a.fte_percent,
        starts_on: a.starts_on,
        ends_on: a.ends_on,
        primary: a.is_primary,
    }
}

async fn of(ctx: &AppContext, worker: Uuid) -> Result<Vec<assignments::Model>> {
    Ok(assignments::Entity::find()
        .filter(assignments::Column::WorkerPid.eq(worker))
        .order_by_desc(assignments::Column::StartsOn)
        .all(&ctx.db)
        .await?)
}

/// The worker `pid` for a write: in the caller's scope and HR's to change, not the worker's own.
async fn hr_worker(ctx: &AppContext, caller: &MaybeAuthUser, pid: Uuid) -> Result<workers::Model> {
    let worker = records::find_worker(&ctx.db, pid).await?;
    ensure_org_in_scope(ctx, caller, &worker.organization_ref).await?;
    if !super::policies::privileged(ctx, caller, &worker.organization_ref).await? {
        return Err(forbidden("assignments are changed by HR"));
    }
    Ok(worker)
}

/// `GET /api/workers/{pid}/assignments` — what the worker holds and has held, newest first. The worker themself and HR.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let today = Utc::now().date_naive();
    let rows = of(&ctx, worker.pid).await?;
    let all: Vec<rules::Assignment> = rows.iter().map(as_rule).collect();
    let mut out = Vec::new();
    for row in &rows {
        let mut v = row_json(row)?;
        if let Some(o) = v.as_object_mut() {
            o.remove("recorded_by");
            for key in [
                "site",
                "area_of_work",
                "job_role",
                "staff_group",
                "occupation_code",
                "cost_centre",
            ] {
                let shown = rules::category_or_unknown(row_str(row, key));
                o.insert(key.into(), json!(shown));
            }
            o.insert("current".into(), json!(as_rule(row).current_on(today)));
        }
        out.push(v);
    }
    format::json(json!({
        "worker_pid": worker.pid,
        "fte_total": rules::fte_total(&all, today),
        "assignments": out,
    }))
}

/// A `422` when the vocabulary gives the job role a different staff group from the one recorded (WPM-R162).
async fn ensure_role_agrees(
    ctx: &AppContext,
    organization: &str,
    job_role: Option<&str>,
    staff_group: Option<&str>,
) -> Result<()> {
    let Some(role) = job_role else { return Ok(()) };
    let of_role = super::category_vocabulary::staff_group_of_role(ctx, organization, role).await?;
    if rules::agreement(of_role.as_deref(), staff_group) == rules::Agreement::Mismatch {
        return Err(unprocessable(
            "the job role belongs to a different staff group in this organization's vocabulary",
        ));
    }
    Ok(())
}

fn row_str<'a>(row: &'a assignments::Model, key: &str) -> Option<&'a str> {
    match key {
        "site" => row.site.as_deref(),
        "area_of_work" => row.area_of_work.as_deref(),
        "job_role" => row.job_role.as_deref(),
        "staff_group" => row.staff_group.as_deref(),
        "cost_centre" => row.cost_centre.as_deref(),
        _ => row.occupation_code.as_deref(),
    }
}

fn clean(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

#[derive(Debug, Deserialize)]
struct AddPayload {
    job_title: String,
    department: String,
    location: Option<String>,
    site: Option<String>,
    area_of_work: Option<String>,
    job_role: Option<String>,
    staff_group: Option<String>,
    occupation_code: Option<String>,
    cost_centre: Option<String>,
    fte_percent: i32,
    starts_on: Option<NaiveDate>,
    ends_on: Option<NaiveDate>,
}

/// `POST /api/workers/{pid}/assignments` — add a further (not primary) assignment. The FTE of the assignments in force may not go
/// over 100 on any day; the `422` names the total. HR.
#[debug_handler]
async fn add(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(p): Json<AddPayload>,
) -> Result<Response> {
    let worker = hr_worker(&ctx, &caller, pid).await?;
    let today = Utc::now().date_naive();
    let new = rules::Assignment {
        fte_percent: p.fte_percent,
        starts_on: p.starts_on.unwrap_or(today),
        ends_on: p.ends_on,
        primary: false,
    };
    let mut problems = rules::text_problems(
        &p.job_title,
        &p.department,
        p.location.as_deref(),
        &[
            ("site", p.site.as_deref()),
            ("area_of_work", p.area_of_work.as_deref()),
            ("job_role", p.job_role.as_deref()),
            ("staff_group", p.staff_group.as_deref()),
            ("occupation_code", p.occupation_code.as_deref()),
        ],
    );
    problems.extend(rules::assignment_problems(&new));
    if new.starts_on < worker.hired_on {
        problems.push("starts_on must not be before the worker's first day".to_string());
    }
    ensure_valid(&problems)?;
    if let Some(code) = clean(p.cost_centre.clone()) {
        super::cost_centres::ensure_open(&ctx, &worker.organization_ref, &code, new.starts_on)
            .await?;
    }
    ensure_role_agrees(
        &ctx,
        &worker.organization_ref,
        clean(p.job_role.clone()).as_deref(),
        clean(p.staff_group.clone()).as_deref(),
    )
    .await?;
    let existing: Vec<rules::Assignment> =
        of(&ctx, worker.pid).await?.iter().map(as_rule).collect();
    ensure_valid(&rules::fte_problems(&existing, &new))?;
    let pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    assignments::ActiveModel {
        pid: ActiveValue::set(pid),
        worker_pid: ActiveValue::set(worker.pid),
        job_title: ActiveValue::set(p.job_title.trim().to_string()),
        department: ActiveValue::set(p.department.trim().to_string()),
        location: ActiveValue::set(clean(p.location)),
        site: ActiveValue::set(clean(p.site)),
        area_of_work: ActiveValue::set(clean(p.area_of_work)),
        job_role: ActiveValue::set(clean(p.job_role)),
        staff_group: ActiveValue::set(clean(p.staff_group)),
        occupation_code: ActiveValue::set(clean(p.occupation_code)),
        cost_centre: ActiveValue::set(clean(p.cost_centre)),
        fte_percent: ActiveValue::set(new.fte_percent),
        is_primary: ActiveValue::set(false),
        starts_on: ActiveValue::set(new.starts_on),
        ends_on: ActiveValue::set(new.ends_on),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "assignment",
        pid,
        "assignment_added",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

#[derive(Debug, Deserialize)]
struct ChangePayload {
    effective_on: Option<NaiveDate>,
    job_title: Option<String>,
    department: Option<String>,
    location: Option<String>,
    site: Option<String>,
    area_of_work: Option<String>,
    job_role: Option<String>,
    staff_group: Option<String>,
    occupation_code: Option<String>,
    cost_centre: Option<String>,
    fte_percent: Option<i32>,
}

/// `POST /api/assignments/{pid}/change` — end this assignment the day before `effective_on` and start another from it, with the
/// fields given changed and the rest kept. `effective_on` is today or earlier. A primary change writes through to the worker row. HR.
#[debug_handler]
async fn change(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(p): Json<ChangePayload>,
) -> Result<Response> {
    let old = assignments::Entity::find()
        .filter(assignments::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let worker = hr_worker(&ctx, &caller, old.worker_pid).await?;
    let today = Utc::now().date_naive();
    let effective_on = p.effective_on.unwrap_or(today);
    if effective_on > today {
        return Err(unprocessable("effective_on must not be in the future"));
    }
    let end = rules::end_for_change(&as_rule(&old), effective_on).map_err(|m| unprocessable(&m))?;
    let f = Fields::merged(&old, &p);
    let new = rules::Assignment {
        fte_percent: p.fte_percent.unwrap_or(old.fte_percent),
        starts_on: effective_on,
        ends_on: old.ends_on,
        primary: old.is_primary,
    };
    let mut problems = f.problems();
    problems.extend(rules::assignment_problems(&new));
    ensure_valid(&problems)?;
    if let Some(code) = &f.cost_centre {
        super::cost_centres::ensure_open(&ctx, &worker.organization_ref, code, effective_on)
            .await?;
    }
    ensure_role_agrees(
        &ctx,
        &worker.organization_ref,
        f.job_role.as_deref(),
        f.staff_group.as_deref(),
    )
    .await?;
    let others = with_ended(&of(&ctx, worker.pid).await?, old.pid, end);
    ensure_valid(&rules::fte_problems(&others, &new))?;

    let new_pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    let mut ended: assignments::ActiveModel = old.clone().into();
    ended.ends_on = ActiveValue::set(Some(end));
    ended.update(&txn).await?;
    assignments::ActiveModel {
        pid: ActiveValue::set(new_pid),
        worker_pid: ActiveValue::set(worker.pid),
        job_title: ActiveValue::set(f.job_title.trim().to_string()),
        department: ActiveValue::set(f.department.trim().to_string()),
        location: ActiveValue::set(f.location.clone()),
        site: ActiveValue::set(f.site.clone()),
        area_of_work: ActiveValue::set(f.area_of_work.clone()),
        job_role: ActiveValue::set(f.job_role.clone()),
        staff_group: ActiveValue::set(f.staff_group.clone()),
        occupation_code: ActiveValue::set(f.occupation_code.clone()),
        cost_centre: ActiveValue::set(f.cost_centre.clone()),
        fte_percent: ActiveValue::set(new.fte_percent),
        is_primary: ActiveValue::set(old.is_primary),
        starts_on: ActiveValue::set(effective_on),
        ends_on: ActiveValue::set(old.ends_on),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    if old.is_primary {
        write_through(
            &txn,
            worker,
            &f.job_title,
            &f.department,
            f.location.clone(),
            new.fte_percent,
        )
        .await?;
    }
    Audit::record(
        &txn,
        "assignment",
        new_pid,
        "assignment_changed",
        caller.actor(),
        Some(json!({ "replaces": pid })),
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": new_pid, "replaces": pid, "previous_ends_on": end }))
}

/// The words of an assignment after a change: what was given, over what was there.
struct Fields {
    job_title: String,
    department: String,
    location: Option<String>,
    site: Option<String>,
    area_of_work: Option<String>,
    job_role: Option<String>,
    staff_group: Option<String>,
    occupation_code: Option<String>,
    cost_centre: Option<String>,
}

impl Fields {
    fn merged(old: &assignments::Model, p: &ChangePayload) -> Self {
        let pick =
            |new: &Option<String>, was: &Option<String>| clean(new.clone()).or_else(|| was.clone());
        Self {
            job_title: p.job_title.clone().unwrap_or_else(|| old.job_title.clone()),
            department: p
                .department
                .clone()
                .unwrap_or_else(|| old.department.clone()),
            location: pick(&p.location, &old.location),
            site: pick(&p.site, &old.site),
            area_of_work: pick(&p.area_of_work, &old.area_of_work),
            job_role: pick(&p.job_role, &old.job_role),
            staff_group: pick(&p.staff_group, &old.staff_group),
            occupation_code: pick(&p.occupation_code, &old.occupation_code),
            cost_centre: pick(&p.cost_centre, &old.cost_centre),
        }
    }

    fn problems(&self) -> Vec<String> {
        rules::text_problems(
            &self.job_title,
            &self.department,
            self.location.as_deref(),
            &[
                ("site", self.site.as_deref()),
                ("area_of_work", self.area_of_work.as_deref()),
                ("job_role", self.job_role.as_deref()),
                ("staff_group", self.staff_group.as_deref()),
                ("occupation_code", self.occupation_code.as_deref()),
            ],
        )
    }
}

/// The worker's assignments as the rules see them, with the one `pid` ended on `end`.
fn with_ended(rows: &[assignments::Model], pid: Uuid, end: NaiveDate) -> Vec<rules::Assignment> {
    rows.iter()
        .map(|a| {
            let mut r = as_rule(a);
            if a.pid == pid {
                r.ends_on = Some(end);
            }
            r
        })
        .collect()
}

/// The worker row is the record of the primary assignment: write the new values through to it.
async fn write_through(
    txn: &sea_orm::DatabaseTransaction,
    worker: workers::Model,
    job_title: &str,
    department: &str,
    location: Option<String>,
    fte_percent: i32,
) -> Result<()> {
    let mut w: workers::ActiveModel = worker.into();
    w.job_title = ActiveValue::set(job_title.trim().to_string());
    w.department = ActiveValue::set(department.trim().to_string());
    w.location = ActiveValue::set(location);
    w.fte_percent = ActiveValue::set(fte_percent);
    w.update(txn).await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct BreakdownQuery {
    organization: String,
    by: String,
}

/// `GET /api/assignments/breakdown?organization=&by=staff_group|job_role|area_of_work` — headcount and FTE of the assignments in
/// force today, by category, from the floor; a cell under it shows nothing, and no total is given. A category not recorded is its own
/// `unknown` cell. No name, no worker, no pay.
#[debug_handler]
async fn breakdown(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(q): axum::extract::Query<BreakdownQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&q.organization))?;
    ensure_org_in_scope(&ctx, &caller, &q.organization).await?;
    if !["staff_group", "job_role", "area_of_work"].contains(&q.by.as_str()) {
        return Err(unprocessable(
            "by must be one of: staff_group, job_role, area_of_work",
        ));
    }
    let today = Utc::now().date_naive();
    let in_org = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&q.organization))
        .filter(workers::Column::DeletedAt.is_null());
    ensure_scale(&ctx, in_org.clone()).await?;
    let pids: Vec<Uuid> = in_org
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| w.pid)
        .collect();
    let rows: Vec<(Uuid, Option<String>, i32)> = assignments::Entity::find()
        .filter(assignments::Column::WorkerPid.is_in(pids))
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|a| as_rule(a).current_on(today))
        .map(|a| {
            let category = match q.by.as_str() {
                "staff_group" => a.staff_group,
                "job_role" => a.job_role,
                _ => a.area_of_work,
            };
            (a.worker_pid, category, a.fte_percent)
        })
        .collect();
    let floor = crate::rules::equality_monitoring::DEFAULT_FLOOR;
    let cells: Vec<_> = rules::breakdown(&rows, floor)
        .into_iter()
        .map(|c| json!({ "category": c.category, "headcount": c.headcount, "fte_percent": c.fte_percent }))
        .collect();
    format::json(
        json!({ "organization": q.organization, "by": q.by, "floor": floor, "cells": cells }),
    )
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/assignments/breakdown", get(breakdown))
        .add("/workers/{pid}/assignments", get(list))
        .add("/workers/{pid}/assignments", post(add))
        .add("/assignments/{pid}/change", post(change))
}
