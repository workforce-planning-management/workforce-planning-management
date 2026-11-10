//! The employed workforce as the plan views need it, counted by the database (part of [`super`]).
//!
//! The forecast and cost views used to read every employed worker of the planned departments into
//! memory. They need only a few columns, and the skill declarations only as counts, so the database
//! groups them and the views work from the groups (WPM-D95): the time and memory depend on the
//! number of distinct groups, not on the number of workers.

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

use crate::db::pool;

/// One group of employed workers who look the same to the projection.
#[derive(Debug)]
struct PersonGroup {
    department: String,
    employment_type: String,
    fte_percent: i32,
    engagement_ends_on: Option<chrono::NaiveDate>,
    people: i64,
}

/// The employed people of each department, as the projection sees them. Refused over the deployer's
/// ceiling (`WPM_MAX_ORG_WORKERS`, WPM-D90). "Employed" is the rule [`is_employed_on`], in SQL.
pub(super) async fn people_by_department(
    ctx: &AppContext,
    org: &str,
    departments: &BTreeSet<String>,
    today: chrono::NaiveDate,
) -> Result<BTreeMap<String, Vec<rules::Person>>> {
    let wanted: Vec<String> = departments.iter().cloned().collect();
    let count = sqlx::query_scalar!(
        r#"SELECT count(*) AS "count!" FROM workers
           WHERE deleted_at IS NULL AND organization_ref = $1 AND department = ANY($2)
             AND hired_on <= $3 AND (terminated_on IS NULL OR terminated_on > $3)"#,
        org,
        &wanted,
        today,
    )
    .fetch_one(pool(ctx))
    .await?;
    crate::controllers::ensure_within_ceiling(count.unsigned_abs())?;
    let groups = sqlx::query_as!(
        PersonGroup,
        r#"SELECT department, employment_type, fte_percent, engagement_ends_on, count(*) AS "people!"
           FROM workers
           WHERE deleted_at IS NULL AND organization_ref = $1 AND department = ANY($2)
             AND hired_on <= $3 AND (terminated_on IS NULL OR terminated_on > $3)
           GROUP BY department, employment_type, fte_percent, engagement_ends_on"#,
        org,
        &wanted,
        today,
    )
    .fetch_all(pool(ctx))
    .await?;
    let mut out: BTreeMap<String, Vec<rules::Person>> = BTreeMap::new();
    for g in groups {
        let person = rules::Person {
            basis: crate::rules::engagement::Basis::parse(&g.employment_type)
                .unwrap_or(crate::rules::engagement::Basis::Permanent),
            fte_percent: g.fte_percent,
            ends_on: g.engagement_ends_on,
        };
        out.entry(g.department)
            .or_default()
            .extend(std::iter::repeat_n(
                person,
                usize::try_from(g.people).unwrap_or(0),
            ));
    }
    Ok(out)
}

/// Workers per department, per skill, per declared level: what the competency gap counts from.
pub(super) async fn skill_levels_by_department(
    ctx: &AppContext,
    org: &str,
    departments: &BTreeSet<String>,
    skills: &BTreeSet<Uuid>,
    today: chrono::NaiveDate,
) -> Result<BTreeMap<(String, Uuid), Vec<(i32, usize)>>> {
    if skills.is_empty() {
        return Ok(BTreeMap::new());
    }
    let rows = sqlx::query!(
        r#"SELECT w.department, k.skill_pid, k.proficiency, count(*) AS "n!"
           FROM worker_skills k JOIN workers w ON w.pid = k.worker_pid
           WHERE k.deleted_at IS NULL AND k.skill_pid = ANY($1)
             AND w.deleted_at IS NULL AND w.organization_ref = $2 AND w.department = ANY($3)
             AND w.hired_on <= $4 AND (w.terminated_on IS NULL OR w.terminated_on > $4)
           GROUP BY w.department, k.skill_pid, k.proficiency"#,
        &skills.iter().copied().collect::<Vec<Uuid>>(),
        org,
        &departments.iter().cloned().collect::<Vec<String>>(),
        today,
    )
    .fetch_all(pool(ctx))
    .await?;
    let mut out: BTreeMap<(String, Uuid), Vec<(i32, usize)>> = BTreeMap::new();
    for r in rows {
        out.entry((r.department, r.skill_pid))
            .or_default()
            .push((r.proficiency, usize::try_from(r.n).unwrap_or(0)));
    }
    Ok(out)
}

/// The status and FTE of each employed worker in a department who is the subject of an open plan to
/// convert: a plan names a worker, so only these few are read, against the department they are in today.
pub(super) async fn converting_by_department(
    ctx: &AppContext,
    org: &str,
    departments: &BTreeSet<String>,
    today: chrono::NaiveDate,
) -> Result<BTreeMap<String, Vec<(String, i32)>>> {
    let open: BTreeMap<Uuid, String> = sqlx::query!(
        "SELECT worker_pid, status FROM conversion_plans
         WHERE intent = 'convert' AND status IN ('proposed', 'approved')"
    )
    .fetch_all(pool(ctx))
    .await?
    .into_iter()
    .map(|c| (c.worker_pid, c.status))
    .collect();
    let rows = sqlx::query!(
        "SELECT pid, department, fte_percent FROM workers
         WHERE deleted_at IS NULL AND organization_ref = $1 AND department = ANY($2)
           AND hired_on <= $3 AND (terminated_on IS NULL OR terminated_on > $3) AND pid = ANY($4)",
        org,
        &departments.iter().cloned().collect::<Vec<String>>(),
        today,
        &open.keys().copied().collect::<Vec<Uuid>>(),
    )
    .fetch_all(pool(ctx))
    .await?;
    let mut out: BTreeMap<String, Vec<(String, i32)>> = BTreeMap::new();
    for r in rows {
        if let Some(status) = open.get(&r.pid) {
            out.entry(r.department)
                .or_default()
                .push((status.clone(), r.fte_percent));
        }
    }
    Ok(out)
}

/// The salaries of the employed in each department, in one currency: a cost view's department average.
pub(super) async fn salaries_by_department(
    ctx: &AppContext,
    org: &str,
    departments: &BTreeSet<String>,
    currency: &str,
    today: chrono::NaiveDate,
) -> Result<BTreeMap<String, Vec<i64>>> {
    let rows = sqlx::query!(
        r#"SELECT department, salary_minor AS "salary_minor!" FROM workers
           WHERE deleted_at IS NULL AND organization_ref = $1 AND department = ANY($2)
             AND hired_on <= $3 AND (terminated_on IS NULL OR terminated_on > $3)
             AND salary_currency = $4 AND salary_minor IS NOT NULL"#,
        org,
        &departments.iter().cloned().collect::<Vec<String>>(),
        today,
        currency,
    )
    .fetch_all(pool(ctx))
    .await?;
    let mut out: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    for r in rows {
        out.entry(r.department).or_default().push(r.salary_minor);
    }
    Ok(out)
}
