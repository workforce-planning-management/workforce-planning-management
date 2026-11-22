//! Derived views: the skills matrix, training analytics and the mentorship overview.

// A section of one controller: it shares the parent module's imports.
use super::{AppContext, Deserialize, Response, Result, State, Uuid, debug_handler, format};
use crate::db::pool;
use crate::sql_time::SqlDate;
use jiff::{Timestamp, civil::Date};

/// `GET /api/learning/skills-matrix` — per-department skill coverage:
/// for each (department, skill) the count of workers at each
/// proficiency and the average, plus per-skill gap counts
/// (proficiency below a declared target).
#[debug_handler]
pub(super) async fn skills_matrix(State(ctx): State<AppContext>) -> Result<Response> {
    let skill_rows = sqlx::query!("SELECT pid, name FROM skills WHERE deleted_at IS NULL")
        .fetch_all(pool(&ctx))
        .await?;
    let skill_name: std::collections::BTreeMap<Uuid, &str> = skill_rows
        .iter()
        .map(|s| (s.pid, s.name.as_str()))
        .collect();
    // (department, skill) → (count, sum, gap_count), counted by the database over workers who still exist.
    let mut cells: std::collections::BTreeMap<(String, Uuid), (usize, i64, usize)> =
        std::collections::BTreeMap::new();
    for row in sqlx::query!(
        r#"SELECT w.department, ws.skill_pid,
                  COUNT(*) AS "total!",
                  SUM(ws.proficiency)::bigint AS "sum!",
                  COUNT(*) FILTER (WHERE ws.target IS NOT NULL AND ws.proficiency < ws.target) AS "below!"
           FROM worker_skills ws
           JOIN workers w ON w.pid = ws.worker_pid AND w.deleted_at IS NULL
           WHERE ws.deleted_at IS NULL
           GROUP BY w.department, ws.skill_pid"#,
    )
    .fetch_all(pool(&ctx))
    .await?
    {
        let cell = cells.entry((row.department, row.skill_pid)).or_default();
        cell.0 = usize::try_from(row.total).unwrap_or(0);
        cell.1 = row.sum;
        cell.2 = usize::try_from(row.below).unwrap_or(0);
    }
    // The named gaps: declared below a declared target.
    let gaps: Vec<serde_json::Value> = sqlx::query!(
        "SELECT ws.worker_pid, w.department, ws.skill_pid, ws.proficiency, ws.target
         FROM worker_skills ws
         JOIN workers w ON w.pid = ws.worker_pid AND w.deleted_at IS NULL
         WHERE ws.deleted_at IS NULL AND ws.target IS NOT NULL AND ws.proficiency < ws.target
         ORDER BY ws.id",
    )
    .fetch_all(pool(&ctx))
    .await?
    .into_iter()
    .map(|row| {
        serde_json::json!({
            "worker_pid": row.worker_pid,
            "department": row.department,
            "skill": skill_name.get(&row.skill_pid),
            "proficiency": row.proficiency,
            "target": row.target,
        })
    })
    .collect();
    let matrix: Vec<serde_json::Value> = cells
        .iter()
        .map(|((department, skill_pid), (count, sum, gap))| {
            // An average of no workers, or of a figure past 2^53, is unknown (null), never a rounded number.
            let average = crate::rules::display::average(*sum, *count);
            serde_json::json!({
                "department": department,
                "skill": skill_name.get(skill_pid),
                "workers": count,
                "average_proficiency": average,
                "below_target": gap,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "as_of": Timestamp::now(),
        "note": "coverage over declared proficiencies only; gaps = declared below a declared target",
        "matrix": matrix,
        "gaps": gaps,
    }))
}

/// `value` when it lies in `lo..=hi`, else a 422 naming the parameter: an out-of-range window is refused, never clamped
/// (RCA-INT-6).
fn within(value: i64, lo: i64, hi: i64, name: &str) -> Result<i64> {
    if (lo..=hi).contains(&value) {
        Ok(value)
    } else {
        Err(crate::controllers::unprocessable(&format!(
            "{name} must be between {lo} and {hi}"
        )))
    }
}

/// Query for the training analytics window.
#[derive(Debug, Deserialize)]
pub(super) struct AnalyticsQuery {
    /// Cert-expiry horizon in days (default 90, 0 to 3650).
    expiring_within_days: Option<i64>,
}

/// `GET /api/learning/training-analytics?expiring_within_days=` —
/// per-department training rollups: enrolments by status, completion
/// ratio (completed / non-failed, numerator+denominator), and
/// certificates expiring within the horizon.
#[debug_handler]
pub(super) async fn training_analytics(
    axa::extract::Query(query): axa::extract::Query<AnalyticsQuery>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let horizon_days = within(
        query.expiring_within_days.unwrap_or(90),
        0,
        3650,
        "expiring_within_days",
    )?;
    let today = crate::dates::today();
    let horizon = crate::controllers::tally::date_plus_days(today, horizon_days)?;
    // Counted by the database: per department and status, over enrolments of workers who still exist.
    let mut per_dept: std::collections::BTreeMap<
        String,
        (std::collections::BTreeMap<String, usize>, usize),
    > = std::collections::BTreeMap::new();
    for row in sqlx::query!(
        r#"SELECT w.department, e.status,
                  COUNT(*) AS "total!",
                  COUNT(*) FILTER (WHERE e.certificate_expires_on >= $1
                                     AND e.certificate_expires_on <= $2) AS "expiring!"
           FROM training_enrollments e
           JOIN workers w ON w.pid = e.worker_pid AND w.deleted_at IS NULL
           WHERE e.deleted_at IS NULL
           GROUP BY w.department, e.status"#,
        SqlDate(today),
        SqlDate(horizon),
    )
    .fetch_all(pool(&ctx))
    .await?
    {
        let entry = per_dept.entry(row.department).or_default();
        let total = usize::try_from(row.total)
            .map_err(|_| crate::controllers::internal("a negative enrolment count"))?;
        let expiring = usize::try_from(row.expiring)
            .map_err(|_| crate::controllers::internal("a negative expiring count"))?;
        crate::controllers::tally::add(entry.0.entry(row.status).or_default(), total)?;
        crate::controllers::tally::add(&mut entry.1, expiring)?;
    }
    let departments: Vec<serde_json::Value> = per_dept
        .iter()
        .map(
            |(department, (by_status, expiring))| -> Result<serde_json::Value> {
                let completed = by_status.get("completed").copied().unwrap_or(0);
                let denominator = by_status
                    .iter()
                    .filter(|(status, _)| status.as_str() != "failed")
                    .try_fold(0_usize, |total, (_, count)| total.checked_add(*count))
                    .ok_or_else(|| crate::controllers::internal("an enrolment count overflowed"))?;
                let value = crate::rules::display::ratio(completed, denominator)
                    .map_or(serde_json::Value::Null, |v| serde_json::json!(v));
                Ok(serde_json::json!({
                    "department": department,
                    "by_status": by_status,
                    "completion_rate": {
                        "numerator": completed, "denominator": denominator, "value": value,
                    },
                    "certs_expiring": expiring,
                }))
            },
        )
        .collect::<Result<_>>()?;
    format::json(serde_json::json!({
        "as_of": today,
        "horizon": horizon,
        "note": "completion rate = completed / (all non-failed enrolments)",
        "departments": departments,
    }))
}

/// `GET /api/learning/mentorship-overview?days=` — active pairings,
/// mentor load (active mentees per mentor), unmatched workers (no
/// active mentorship as mentor or mentee), and stale actives (no
/// session in the window).
#[debug_handler]
pub(super) async fn mentorship_overview(
    axa::extract::Query(query): axa::extract::Query<MentorshipOverviewQuery>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let stale_days = within(query.days.unwrap_or(30), 1, 365, "days")?;
    let today = crate::dates::today();
    let stale_before = crate::controllers::tally::date_minus_days(today, stale_days)?;
    let unmatched_limit = within(
        query.unmatched_limit.unwrap_or(200),
        1,
        1000,
        "unmatched_limit",
    )?;
    let active = sqlx::query!(
        "SELECT pid, mentor_pid, mentee_pid FROM mentorships
         WHERE deleted_at IS NULL AND status = 'active'",
    )
    .fetch_all(pool(&ctx))
    .await?;
    // Names only for the people in an active pairing.
    let paired: Vec<Uuid> = active
        .iter()
        .flat_map(|m| [m.mentor_pid, m.mentee_pid])
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let worker_rows = sqlx::query!(
        "SELECT pid, display_name FROM workers WHERE deleted_at IS NULL AND pid = ANY($1)",
        &paired,
    )
    .fetch_all(pool(&ctx))
    .await?;
    let name_of: std::collections::BTreeMap<Uuid, &str> = worker_rows
        .iter()
        .map(|e| (e.pid, e.display_name.as_str()))
        .collect();
    let last_session: std::collections::BTreeMap<Uuid, Date> = sqlx::query!(
        r#"SELECT mentorship_pid, MAX(held_on) AS "last!"
           FROM mentorship_sessions GROUP BY mentorship_pid"#,
    )
    .fetch_all(pool(&ctx))
    .await?
    .into_iter()
    .map(|row| (row.mentorship_pid, row.last.get()))
    .collect();
    let mut mentor_load: std::collections::BTreeMap<Uuid, usize> =
        std::collections::BTreeMap::new();
    let mut stale = Vec::new();
    for mentorship in &active {
        crate::controllers::tally::bump(mentor_load.entry(mentorship.mentor_pid).or_default());
        let last = last_session.get(&mentorship.pid).copied();
        if last.is_none_or(|d| d < stale_before) {
            stale.push(serde_json::json!({
                "pid": mentorship.pid,
                "mentor": name_of.get(&mentorship.mentor_pid),
                "mentee": name_of.get(&mentorship.mentee_pid),
                "last_session": last,
            }));
        }
    }
    let load_view: Vec<serde_json::Value> = mentor_load
        .iter()
        .map(|(mentor, count)| {
            serde_json::json!({ "mentor_pid": mentor, "mentor": name_of.get(mentor), "active_mentees": count })
        })
        .collect();
    // Active workers in no active pairing, as mentor or mentee: the count, and the newest few.
    let unmatched_total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "total!" FROM workers
           WHERE deleted_at IS NULL AND status = 'active'
             AND pid NOT IN (SELECT mentor_pid FROM mentorships WHERE deleted_at IS NULL AND status = 'active')
             AND pid NOT IN (SELECT mentee_pid FROM mentorships WHERE deleted_at IS NULL AND status = 'active')"#,
    )
    .fetch_one(pool(&ctx))
    .await?;
    let unmatched: Vec<serde_json::Value> = sqlx::query!(
        "SELECT pid, display_name, department FROM workers
         WHERE deleted_at IS NULL AND status = 'active'
           AND pid NOT IN (SELECT mentor_pid FROM mentorships WHERE deleted_at IS NULL AND status = 'active')
           AND pid NOT IN (SELECT mentee_pid FROM mentorships WHERE deleted_at IS NULL AND status = 'active')
         ORDER BY id DESC LIMIT $1",
        unmatched_limit,
    )
    .fetch_all(pool(&ctx))
    .await?
    .into_iter()
    .map(|row| {
        serde_json::json!({
            "pid": row.pid, "display_name": row.display_name, "department": row.department,
        })
    })
    .collect();
    format::json(serde_json::json!({
        "as_of": today,
        "active_pairings": active.len(),
        "mentor_load": load_view,
        "unmatched_total": unmatched_total,
        "unmatched_workers": unmatched,
        "stale_days": stale_days,
        "stale_mentorships": stale,
    }))
}

/// Query for the mentorship overview.
#[derive(Debug, Deserialize)]
pub(super) struct MentorshipOverviewQuery {
    days: Option<i64>,
    /// How many unmatched workers to name, newest first (default 200, 1 to 1,000); the total is always given.
    unmatched_limit: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::within;

    #[test]
    fn a_window_inside_its_range_is_kept_and_outside_is_refused() {
        assert_eq!(within(0, 0, 3650, "d").ok(), Some(0));
        assert_eq!(within(3650, 0, 3650, "d").ok(), Some(3650));
        assert!(within(-1, 0, 3650, "d").is_err());
        assert!(within(3651, 0, 3650, "d").is_err());
    }
}
