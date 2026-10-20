//! The employed workforce as the plan views need it, counted by the database (part of [`super`]).
//!
//! The forecast and cost views used to read every employed worker of the planned departments into
//! memory. They need only a few columns, and the skill declarations only as counts, so the database
//! groups them and the views work from the groups (WPM-D95): the time and memory depend on the
//! number of distinct groups, not on the number of workers.

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

use sea_orm::{FromQueryResult, QuerySelect};

/// Employed workers of the organization in the departments, today: the query every function here
/// narrows (the rule [`is_employed_on`], in SQL).
fn employed_in(
    org: &str,
    departments: &BTreeSet<String>,
    today: chrono::NaiveDate,
) -> sea_orm::Select<workers::Entity> {
    workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::OrganizationRef.eq(org))
        .filter(workers::Column::Department.is_in(departments.iter().cloned()))
        .filter(workers::Column::HiredOn.lte(today))
        .filter(
            Condition::any()
                .add(workers::Column::TerminatedOn.is_null())
                .add(workers::Column::TerminatedOn.gt(today)),
        )
}

/// One group of employed workers who look the same to the projection.
#[derive(Debug, FromQueryResult)]
struct PersonGroup {
    department: String,
    employment_type: String,
    fte_percent: i32,
    engagement_ends_on: Option<chrono::NaiveDate>,
    people: i64,
}

/// The employed people of each department, as the projection sees them. Refused over the deployer's
/// ceiling (`WPM_MAX_ORG_WORKERS`, WPM-D90).
pub(super) async fn people_by_department(
    ctx: &AppContext,
    org: &str,
    departments: &BTreeSet<String>,
    today: chrono::NaiveDate,
) -> Result<BTreeMap<String, Vec<rules::Person>>> {
    let scope = employed_in(org, departments, today);
    crate::controllers::ensure_scale(ctx, scope.clone()).await?;
    let groups = scope
        .select_only()
        .column(workers::Column::Department)
        .column(workers::Column::EmploymentType)
        .column(workers::Column::FtePercent)
        .column(workers::Column::EngagementEndsOn)
        .column_as(workers::Column::Id.count(), "people")
        .group_by(workers::Column::Department)
        .group_by(workers::Column::EmploymentType)
        .group_by(workers::Column::FtePercent)
        .group_by(workers::Column::EngagementEndsOn)
        .into_model::<PersonGroup>()
        .all(&ctx.db)
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
    let rows = ctx
        .db
        .query_all_raw(sea_orm::Statement::from_sql_and_values(
            sea_orm::DbBackend::Postgres,
            "SELECT w.department, k.skill_pid, k.proficiency, count(*) AS n
             FROM worker_skills k JOIN workers w ON w.pid = k.worker_pid
             WHERE k.deleted_at IS NULL AND k.skill_pid = ANY($1)
               AND w.deleted_at IS NULL AND w.organization_ref = $2 AND w.department = ANY($3)
               AND w.hired_on <= $4 AND (w.terminated_on IS NULL OR w.terminated_on > $4)
             GROUP BY w.department, k.skill_pid, k.proficiency",
            [
                skills.iter().copied().collect::<Vec<Uuid>>().into(),
                org.into(),
                departments.iter().cloned().collect::<Vec<String>>().into(),
                today.into(),
            ],
        ))
        .await?;
    let rows: Vec<(String, Uuid, i32, i64)> = rows
        .into_iter()
        .map(|r| {
            Ok((
                r.try_get("", "department")?,
                r.try_get("", "skill_pid")?,
                r.try_get("", "proficiency")?,
                r.try_get("", "n")?,
            ))
        })
        .collect::<std::result::Result<_, sea_orm::DbErr>>()?;
    let mut out: BTreeMap<(String, Uuid), Vec<(i32, usize)>> = BTreeMap::new();
    for (department, skill, level, n) in rows {
        out.entry((department, skill))
            .or_default()
            .push((level, usize::try_from(n).unwrap_or(0)));
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
    let open: BTreeMap<Uuid, String> = conversion_plans::Entity::find()
        .filter(conversion_plans::Column::Intent.eq("convert"))
        .filter(conversion_plans::Column::Status.is_in(["proposed", "approved"]))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|c| (c.worker_pid, c.status))
        .collect();
    let pids: Vec<Uuid> = open.keys().copied().collect();
    let mut out: BTreeMap<String, Vec<(String, i32)>> = BTreeMap::new();
    for chunk in pids.chunks(5_000) {
        let rows: Vec<(Uuid, String, i32)> = employed_in(org, departments, today)
            .select_only()
            .columns([
                workers::Column::Pid,
                workers::Column::Department,
                workers::Column::FtePercent,
            ])
            .filter(workers::Column::Pid.is_in(chunk.to_vec()))
            .into_tuple()
            .all(&ctx.db)
            .await?;
        for (pid, department, fte) in rows {
            if let Some(status) = open.get(&pid) {
                out.entry(department)
                    .or_default()
                    .push((status.clone(), fte));
            }
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
    let rows: Vec<(String, i64)> = employed_in(org, departments, today)
        .select_only()
        .columns([workers::Column::Department, workers::Column::SalaryMinor])
        .filter(workers::Column::SalaryCurrency.eq(currency))
        .filter(workers::Column::SalaryMinor.is_not_null())
        .into_tuple()
        .all(&ctx.db)
        .await?;
    let mut out: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    for (department, salary) in rows {
        out.entry(department).or_default().push(salary);
    }
    Ok(out)
}
