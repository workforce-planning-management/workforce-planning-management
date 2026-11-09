//! The workforce skill-gap views, counted by the database (WPM-R142).
//!
//! [`crate::rules::skill_gap::merge`] says what one person's gaps are: their role's requirements and
//! their own targets, merged per skill (highest level, strongest importance), graded against what they
//! declared. These queries say the same thing for everyone employed in the caller's scope at once, without
//! reading a row per worker into memory; a test grades the same people both ways, with and without the skills crosswalk. Aspirations are private
//! and are never part of a workforce view, so they are not here.

use axa::prelude::*;
use uuid::Uuid;

use crate::rules::skill_gap::{Row, Status};
use crate::tasks::import_framework::PCF_SLUG;

/// What one reading of the staff gives: how many are employed, and, as asked for, their graded needs counted per skill,
/// department, importance, status and shortfall with how many people have at least one need, and who is below at least
/// one need.
struct Graded {
    considered: usize,
    rows: Vec<(Row, usize)>,
    with_needs: usize,
    below: Vec<Uuid>,
}

/// The importance a weight stands for, in the words the rules use.
const fn importance_of(weight: i32) -> &'static str {
    match weight {
        3 => "critical",
        2 => "important",
        _ => "useful",
    }
}

fn count(n: i64) -> usize {
    usize::try_from(n).unwrap_or(0)
}

/// The people employed on `today` in `scope` (all organizations when `None`), optionally in one department (ignoring
/// case), with each of their merged needs graded against their declarations, in one fixed statement (WPM-D184).
/// `rows` asks for the roll-up rows and how many have a need; `below` for the people below a need, in id order. What
/// is not asked for is not worked out.
async fn graded(
    ctx: &AppContext,
    scope: Option<&[String]>,
    department: Option<&str>,
    today: chrono::NaiveDate,
    rows: bool,
    below: bool,
) -> Result<Graded> {
    let found = sqlx::query!(
        r#"WITH staff AS (
             SELECT pid, department FROM workers
              WHERE deleted_at IS NULL AND hired_on <= $1
                AND (terminated_on IS NULL OR terminated_on > $1)
                AND ($3::text[] IS NULL OR organization_ref = ANY($3))
                AND ($4::text IS NULL OR lower(department) = lower($4))),
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
               LEFT JOIN via v ON v.worker_pid = mg.worker_pid AND v.skill_pid = mg.skill_pid),
         -- A shortfall is there only when below, so 0 stands for none and the status says which.
         counted AS (
             SELECT skill_pid, department, weight,
                    CASE WHEN declared IS NULL THEN 'undeclared'
                         WHEN declared >= required THEN 'met' ELSE 'below' END AS status,
                    COALESCE(CASE WHEN declared < required THEN required - declared END, 0) AS shortfall,
                    COUNT(*)::bigint AS people
               FROM graded WHERE $5
              GROUP BY skill_pid, department, weight, status, shortfall)
         SELECT (SELECT COUNT(*) FROM staff)::bigint AS "considered!",
                -- Who has a need does not depend on the crosswalk, so this leaves the mapping work (`via`) unevaluated.
                CASE WHEN $5 THEN (SELECT COUNT(DISTINCT worker_pid) FROM merged) ELSE 0 END::bigint AS "with_needs!",
                ARRAY(SELECT skill_pid FROM counted ORDER BY skill_pid, department, weight, status, shortfall) AS "skills!: Vec<Uuid>",
                ARRAY(SELECT department::text FROM counted ORDER BY skill_pid, department, weight, status, shortfall) AS "departments!: Vec<String>",
                ARRAY(SELECT weight FROM counted ORDER BY skill_pid, department, weight, status, shortfall) AS "weights!: Vec<i32>",
                ARRAY(SELECT status FROM counted ORDER BY skill_pid, department, weight, status, shortfall) AS "statuses!: Vec<String>",
                ARRAY(SELECT shortfall FROM counted ORDER BY skill_pid, department, weight, status, shortfall) AS "shortfalls!: Vec<i32>",
                ARRAY(SELECT people FROM counted ORDER BY skill_pid, department, weight, status, shortfall) AS "people!: Vec<i64>",
                ARRAY(SELECT g.worker_pid FROM graded g
                       WHERE $6 AND g.declared < g.required GROUP BY g.worker_pid
                       ORDER BY (SELECT id FROM workers w WHERE w.pid = g.worker_pid)) AS "below!: Vec<Uuid>""#,
        today,
        PCF_SLUG,
        scope,
        department,
        rows,
        below,
    )
    .fetch_one(crate::db::pool(ctx))
    .await?;
    let mut out = Vec::with_capacity(found.skills.len());
    for (i, skill) in found.skills.into_iter().enumerate() {
        let status = match found.statuses[i].as_str() {
            "met" => Status::Met,
            "below" => Status::Below,
            _ => Status::Undeclared,
        };
        out.push((
            Row {
                skill,
                department: found.departments[i].clone(),
                status,
                shortfall: (status == Status::Below).then_some(found.shortfalls[i]),
                importance: importance_of(found.weights[i]).to_string(),
            },
            count(found.people[i]),
        ));
    }
    Ok(Graded {
        considered: count(found.considered),
        rows: out,
        with_needs: count(found.with_needs),
        below: found.below,
    })
}

/// How many people are employed on `today` in `scope` (and `department`); every graded need of them as
/// workforce-roll-up rows with the number of people who share each; and how many people have at least one need.
///
/// # Errors
///
/// Propagates a database error.
pub(crate) async fn gap_rows(
    ctx: &AppContext,
    scope: Option<&[String]>,
    department: Option<&str>,
    today: chrono::NaiveDate,
) -> Result<(usize, Vec<(Row, usize)>, usize)> {
    let g = graded(ctx, scope, department, today, true, false).await?;
    Ok((g.considered, g.rows, g.with_needs))
}

/// How many people are employed on `today` in `scope` (and `department`), and those below at least one need, in id
/// order: the only people whose recommended training hours can be above zero.
///
/// # Errors
///
/// Propagates a database error.
pub(crate) async fn workers_below_a_need(
    ctx: &AppContext,
    scope: Option<&[String]>,
    department: Option<&str>,
    today: chrono::NaiveDate,
) -> Result<(usize, Vec<Uuid>)> {
    let g = graded(ctx, scope, department, today, false, true).await?;
    Ok((g.considered, g.below))
}
