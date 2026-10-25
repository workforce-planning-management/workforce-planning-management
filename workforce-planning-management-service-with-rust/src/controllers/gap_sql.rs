//! The workforce skill-gap views, counted by the database (WPM-R142).
//!
//! [`crate::rules::skill_gap::merge`] says what one person's gaps are: their role's requirements and
//! their own targets, merged per skill (highest level, strongest importance), graded against what they
//! declared. These queries say the same thing for everyone employed in the caller's scope at once, without
//! reading a row per worker into memory; a test grades the same people both ways, with and without the skills crosswalk. Aspirations are private
//! and are never part of a workforce view, so they are not here.

use std::fmt::Write;

use loco_rs::prelude::*;
use sea_orm::{DbBackend, Statement, Value};
use uuid::Uuid;

use crate::rules::skill_gap::{Row, Status};
use crate::tasks::import_framework::PCF_SLUG;

/// The people employed on `today` in `scope` (all organizations when `None`), optionally in one
/// department (ignoring case), with each of their merged needs graded against their declarations.
/// Returns the SQL (a `WITH … graded AS (…)` prefix) and its bound values; `$1` is `today`.
fn graded(
    scope: Option<&[String]>,
    department: Option<&str>,
    today: chrono::NaiveDate,
) -> (String, Vec<Value>) {
    let mut values: Vec<Value> = vec![today.into(), PCF_SLUG.into()];
    let mut staff_filter = String::new();
    if let Some(refs) = scope {
        if refs.is_empty() {
            staff_filter.push_str(" AND FALSE");
        } else {
            let marks: Vec<String> = refs
                .iter()
                .map(|r| {
                    values.push(r.as_str().into());
                    format!("${}", values.len())
                })
                .collect();
            let _ = write!(
                staff_filter,
                " AND organization_ref IN ({})",
                marks.join(", ")
            );
        }
    }
    if let Some(department) = department {
        values.push(department.into());
        let _ = write!(
            staff_filter,
            " AND lower(department) = lower(${})",
            values.len()
        );
    }
    let sql = format!(
        "WITH staff AS (
             SELECT pid, department FROM workers
              WHERE deleted_at IS NULL AND hired_on <= $1
                AND (terminated_on IS NULL OR terminated_on > $1){staff_filter}),
         needs AS (
             SELECT r.worker_pid, q.skill_pid, q.min_proficiency AS required,
                    CASE q.importance WHEN 'critical' THEN 3 WHEN 'important' THEN 2 ELSE 1 END AS weight
               FROM worker_framework_roles r
               JOIN role_skill_requirements q ON q.role_profile_pid = r.role_profile_pid
              WHERE r.framework_slug = $2 AND r.ended_at IS NULL
             UNION ALL
             SELECT worker_pid, skill_pid, target, 2 FROM worker_skills
              WHERE deleted_at IS NULL AND target IS NOT NULL),
         merged AS (
             SELECT n.worker_pid, n.skill_pid, s.department, MAX(n.required) AS required, MAX(n.weight) AS weight
               FROM needs n JOIN staff s ON s.pid = n.worker_pid
              GROUP BY n.worker_pid, n.skill_pid, s.department),
         -- The skills crosswalk (`rules::crosswalk::apply`): a need the person did not declare is graded through the best
         -- exact or close mapping from a skill they did declare, the most confident first, then the highest declared level.
         -- Driven from the declarations and joined to the needs once, which measured about a quarter of the cost of looking a
         -- mapping up for every need. A direct declaration wins below; a broader or narrower mapping is never counted.
         via AS (
             SELECT DISTINCT ON (mg.worker_pid, mg.skill_pid) mg.worker_pid, mg.skill_pid, ds.proficiency AS level
               FROM worker_skills ds
               JOIN skill_mappings m ON m.from_skill_pid = ds.skill_pid AND m.confidence IN ('exact', 'close')
               JOIN merged mg ON mg.worker_pid = ds.worker_pid AND mg.skill_pid = m.to_skill_pid
              WHERE ds.deleted_at IS NULL
              ORDER BY mg.worker_pid, mg.skill_pid, (m.confidence = 'exact') DESC, ds.proficiency DESC, m.from_skill_pid),
         graded AS (
             SELECT mg.worker_pid, mg.skill_pid, mg.department, mg.required, mg.weight,
                    COALESCE(d.proficiency, v.level) AS declared
               FROM merged mg
               LEFT JOIN worker_skills d ON d.worker_pid = mg.worker_pid AND d.skill_pid = mg.skill_pid
                                        AND d.deleted_at IS NULL
               LEFT JOIN via v ON v.worker_pid = mg.worker_pid AND v.skill_pid = mg.skill_pid)"
    );
    (sql, values)
}

/// The importance a weight stands for, in the words the rules use.
const fn importance_of(weight: i32) -> &'static str {
    match weight {
        3 => "critical",
        2 => "important",
        _ => "useful",
    }
}

/// How many people are employed on `today` in `scope` (and `department`).
///
/// # Errors
///
/// Propagates a database error.
pub(crate) async fn staff_count(
    ctx: &AppContext,
    scope: Option<&[String]>,
    department: Option<&str>,
    today: chrono::NaiveDate,
) -> Result<usize> {
    let (sql, values) = graded(scope, department, today);
    let row = ctx
        .db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            format!("{sql} SELECT (SELECT COUNT(*) FROM staff)::bigint AS total"),
            values,
        ))
        .await?;
    let total: i64 = row.map_or(Ok(0), |r| r.try_get("", "total"))?;
    Ok(usize::try_from(total).unwrap_or(0))
}

/// Every graded need of the staff as workforce-roll-up rows with the number of people who share
/// each, and how many people have at least one need.
///
/// # Errors
///
/// Propagates a database error.
pub(crate) async fn gap_rows(
    ctx: &AppContext,
    scope: Option<&[String]>,
    department: Option<&str>,
    today: chrono::NaiveDate,
) -> Result<(Vec<(Row, usize)>, usize)> {
    let (sql, values) = graded(scope, department, today);
    let rows = ctx
        .db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            format!(
                "{sql} SELECT skill_pid, department, weight,
                        CASE WHEN declared IS NULL THEN 'undeclared'
                             WHEN declared >= required THEN 'met' ELSE 'below' END AS status,
                        CASE WHEN declared < required THEN required - declared END AS shortfall,
                        COUNT(*)::bigint AS people
                   FROM graded GROUP BY skill_pid, department, weight, status, shortfall"
            ),
            values.clone(),
        ))
        .await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let status = match row.try_get::<String>("", "status")?.as_str() {
            "met" => Status::Met,
            "below" => Status::Below,
            _ => Status::Undeclared,
        };
        let people: i64 = row.try_get("", "people")?;
        out.push((
            Row {
                skill: row.try_get("", "skill_pid")?,
                department: row.try_get("", "department")?,
                status,
                shortfall: row.try_get("", "shortfall")?,
                importance: importance_of(row.try_get("", "weight")?).to_string(),
            },
            usize::try_from(people).unwrap_or(0),
        ));
    }
    let with_needs = ctx
        .db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            // Who has a need does not depend on the crosswalk, so this leaves the mapping work (`via`) unevaluated.
            format!("{sql} SELECT COUNT(DISTINCT worker_pid)::bigint AS total FROM merged"),
            values,
        ))
        .await?;
    let with_needs: i64 = with_needs.map_or(Ok(0), |r| r.try_get("", "total"))?;
    Ok((out, usize::try_from(with_needs).unwrap_or(0)))
}

/// The staff who are below at least one need, in id order: the only people whose recommended
/// training hours can be above zero.
///
/// # Errors
///
/// Propagates a database error.
pub(crate) async fn workers_below_a_need(
    ctx: &AppContext,
    scope: Option<&[String]>,
    department: Option<&str>,
    today: chrono::NaiveDate,
) -> Result<Vec<Uuid>> {
    let (sql, values) = graded(scope, department, today);
    let mut pids = Vec::new();
    for row in ctx
        .db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            format!(
                "{sql} SELECT g.worker_pid FROM graded g
                   WHERE g.declared < g.required GROUP BY g.worker_pid
                   ORDER BY (SELECT id FROM workers w WHERE w.pid = g.worker_pid)"
            ),
            values,
        ))
        .await?
    {
        pids.push(row.try_get("", "worker_pid")?);
    }
    Ok(pids)
}
