//! Derived views: the skills matrix, training analytics and the mentorship overview.

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

/// `GET /api/learning/skills-matrix` — per-department skill coverage:
/// for each (department, skill) the count of workers at each
/// proficiency and the average, plus per-skill gap counts
/// (proficiency below a declared target).
#[debug_handler]
#[allow(clippy::too_many_lines)] // one pass over the declared skills
pub(super) async fn skills_matrix(State(ctx): State<AppContext>) -> Result<Response> {
    let skill_rows = skills::Entity::find()
        .filter(skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let skill_name: std::collections::BTreeMap<Uuid, &str> = skill_rows
        .iter()
        .map(|s| (s.pid, s.name.as_str()))
        .collect();
    // (department, skill) → (count, sum, gap_count), counted by the database over workers who still exist.
    let mut cells: std::collections::BTreeMap<(String, Uuid), (usize, i64, usize)> =
        std::collections::BTreeMap::new();
    for row in ctx
        .db
        .query_all_raw(sea_orm::Statement::from_string(
            sea_orm::DbBackend::Postgres,
            "SELECT w.department AS department, ws.skill_pid AS skill_pid,
                    COUNT(*)::bigint AS total, SUM(ws.proficiency)::bigint AS sum,
                    (COUNT(*) FILTER (WHERE ws.target IS NOT NULL AND ws.proficiency < ws.target))::bigint AS below
               FROM worker_skills ws
               JOIN workers w ON w.pid = ws.worker_pid AND w.deleted_at IS NULL
              WHERE ws.deleted_at IS NULL
              GROUP BY w.department, ws.skill_pid",
        ))
        .await?
    {
        let count: i64 = row.try_get("", "total")?;
        let cell = cells
            .entry((row.try_get("", "department")?, row.try_get("", "skill_pid")?))
            .or_default();
        cell.0 = usize::try_from(count).unwrap_or(0);
        cell.1 = row.try_get("", "sum")?;
        cell.2 = usize::try_from(row.try_get::<i64>("", "below")?).unwrap_or(0);
    }
    // The named gaps: declared below a declared target.
    let mut gaps: Vec<serde_json::Value> = Vec::new();
    for row in ctx
        .db
        .query_all_raw(sea_orm::Statement::from_string(
            sea_orm::DbBackend::Postgres,
            "SELECT ws.worker_pid AS worker_pid, w.department AS department, ws.skill_pid AS skill_pid,
                    ws.proficiency AS proficiency, ws.target AS target
               FROM worker_skills ws
               JOIN workers w ON w.pid = ws.worker_pid AND w.deleted_at IS NULL
              WHERE ws.deleted_at IS NULL AND ws.target IS NOT NULL AND ws.proficiency < ws.target
              ORDER BY ws.id",
        ))
        .await?
    {
        let skill_pid: Uuid = row.try_get("", "skill_pid")?;
        gaps.push(serde_json::json!({
            "worker_pid": row.try_get::<Uuid>("", "worker_pid")?,
            "department": row.try_get::<String>("", "department")?,
            "skill": skill_name.get(&skill_pid),
            "proficiency": row.try_get::<i32>("", "proficiency")?,
            "target": row.try_get::<Option<i32>>("", "target")?,
        }));
    }
    let matrix: Vec<serde_json::Value> = cells
        .iter()
        .map(|((department, skill_pid), (count, sum, gap))| {
            #[allow(clippy::cast_precision_loss)] // display average
            let average = *sum as f64 / *count as f64;
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
        "as_of": chrono::Utc::now(),
        "note": "coverage over declared proficiencies only; gaps = declared below a declared target",
        "matrix": matrix,
        "gaps": gaps,
    }))
}

/// Query for the training analytics window.
#[derive(Debug, Deserialize)]
pub(super) struct AnalyticsQuery {
    /// Cert-expiry horizon in days (default 90, cap 3650).
    expiring_within_days: Option<i64>,
}

/// `GET /api/learning/training-analytics?expiring_within_days=` —
/// per-department training rollups: enrolments by status, completion
/// ratio (completed / non-failed, numerator+denominator), and
/// certificates expiring within the horizon.
#[debug_handler]
pub(super) async fn training_analytics(
    axum::extract::Query(query): axum::extract::Query<AnalyticsQuery>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let horizon_days = query.expiring_within_days.unwrap_or(90).clamp(0, 3650);
    let today = chrono::Utc::now().date_naive();
    let horizon = today + chrono::Duration::days(horizon_days);
    // Counted by the database: per department and status, over enrolments of workers who still exist.
    let mut per_dept: std::collections::BTreeMap<
        String,
        (std::collections::BTreeMap<String, usize>, usize),
    > = std::collections::BTreeMap::new();
    for row in ctx
        .db
        .query_all_raw(sea_orm::Statement::from_sql_and_values(
            sea_orm::DbBackend::Postgres,
            "SELECT w.department AS department, e.status AS status, COUNT(*)::bigint AS total,
                    (COUNT(*) FILTER (WHERE e.certificate_expires_on >= $1
                                        AND e.certificate_expires_on <= $2))::bigint AS expiring
               FROM training_enrollments e
               JOIN workers w ON w.pid = e.worker_pid AND w.deleted_at IS NULL
              WHERE e.deleted_at IS NULL
              GROUP BY w.department, e.status",
            [today.into(), horizon.into()],
        ))
        .await?
    {
        let department: String = row.try_get("", "department")?;
        let status: String = row.try_get("", "status")?;
        let total: i64 = row.try_get("", "total")?;
        let expiring: i64 = row.try_get("", "expiring")?;
        let entry = per_dept.entry(department).or_default();
        *entry.0.entry(status).or_default() += usize::try_from(total).unwrap_or(0);
        entry.1 += usize::try_from(expiring).unwrap_or(0);
    }
    let departments: Vec<serde_json::Value> = per_dept
        .iter()
        .map(|(department, (by_status, expiring))| {
            let completed = by_status.get("completed").copied().unwrap_or(0);
            let denominator: usize = by_status
                .iter()
                .filter(|(status, _)| status.as_str() != "failed")
                .map(|(_, count)| *count)
                .sum();
            let value = crate::rules::display::ratio(completed, denominator)
                .map_or(serde_json::Value::Null, |v| serde_json::json!(v));
            serde_json::json!({
                "department": department,
                "by_status": by_status,
                "completion_rate": {
                    "numerator": completed, "denominator": denominator, "value": value,
                },
                "certs_expiring": expiring,
            })
        })
        .collect();
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
#[allow(clippy::too_many_lines)] // loads, groups, then one JSON body, linearly
pub(super) async fn mentorship_overview(
    axum::extract::Query(query): axum::extract::Query<MentorshipOverviewQuery>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let stale_days = query.days.unwrap_or(30).clamp(1, 365);
    let today = chrono::Utc::now().date_naive();
    let stale_before = today - chrono::Duration::days(stale_days);
    let unmatched_limit = query.unmatched_limit.unwrap_or(200).clamp(1, 1000);
    let mentorship_rows = mentorships::Entity::find()
        .filter(mentorships::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let active: Vec<&mentorships::Model> = mentorship_rows
        .iter()
        .filter(|m| m.status == "active")
        .collect();
    // Names only for the people in an active pairing.
    let paired: Vec<Uuid> = active
        .iter()
        .flat_map(|m| [m.mentor_pid, m.mentee_pid])
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let worker_rows = super::super::find_in(
        &ctx,
        workers::Entity::find().filter(workers::Column::DeletedAt.is_null()),
        workers::Column::Pid,
        &paired,
    )
    .await?;
    let name_of: std::collections::BTreeMap<Uuid, &str> = worker_rows
        .iter()
        .map(|e| (e.pid, e.display_name.as_str()))
        .collect();
    let mut last_session: std::collections::BTreeMap<Uuid, chrono::NaiveDate> =
        std::collections::BTreeMap::new();
    for row in ctx
        .db
        .query_all_raw(sea_orm::Statement::from_string(
            sea_orm::DbBackend::Postgres,
            "SELECT mentorship_pid, MAX(held_on) AS last FROM mentorship_sessions GROUP BY mentorship_pid",
        ))
        .await?
    {
        last_session.insert(row.try_get("", "mentorship_pid")?, row.try_get("", "last")?);
    }
    let mut mentor_load: std::collections::BTreeMap<Uuid, usize> =
        std::collections::BTreeMap::new();
    let mut stale = Vec::new();
    for mentorship in &active {
        *mentor_load.entry(mentorship.mentor_pid).or_default() += 1;
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
    let unmatched_where = "FROM workers
         WHERE deleted_at IS NULL AND status = 'active'
           AND pid NOT IN (SELECT mentor_pid FROM mentorships WHERE deleted_at IS NULL AND status = 'active')
           AND pid NOT IN (SELECT mentee_pid FROM mentorships WHERE deleted_at IS NULL AND status = 'active')";
    let unmatched_total: i64 = ctx
        .db
        .query_one_raw(sea_orm::Statement::from_string(
            sea_orm::DbBackend::Postgres,
            format!("SELECT COUNT(*)::bigint AS total {unmatched_where}"),
        ))
        .await?
        .map_or(Ok(0), |row| row.try_get("", "total"))?;
    let mut unmatched: Vec<serde_json::Value> = Vec::new();
    for row in ctx
        .db
        .query_all_raw(sea_orm::Statement::from_string(
            sea_orm::DbBackend::Postgres,
            format!(
                "SELECT pid, display_name, department {unmatched_where} ORDER BY id DESC LIMIT {unmatched_limit}"
            ),
        ))
        .await?
    {
        unmatched.push(serde_json::json!({
            "pid": row.try_get::<Uuid>("", "pid")?,
            "display_name": row.try_get::<String>("", "display_name")?,
            "department": row.try_get::<String>("", "department")?,
        }));
    }
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
    /// How many unmatched workers to name, newest first (default 200, at most 1,000); the total is always given.
    unmatched_limit: Option<i64>,
}
