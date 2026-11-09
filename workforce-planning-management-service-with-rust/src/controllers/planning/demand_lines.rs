//! Demand lines (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;
use axa::extract::{Json, Path};

/// `PUT /api/workforce-plans/{pid}/demand-lines` body.
#[derive(Debug, Deserialize)]
pub(super) struct LinePayload {
    department: String,
    #[serde(default)]
    role_profile_pid: Option<Uuid>,
    target_on: chrono::NaiveDate,
    target_headcount: i32,
    /// FTE across the headcount, in percent of one person (`250` is two and a half people); not
    /// stated is unknown, never assumed to be the headcount.
    #[serde(default)]
    target_fte_percent: Option<i32>,
    /// `permanent` (default), `fixed_term`, `contractor` or `intern` (WPM-R82).
    #[serde(default = "default_basis")]
    basis: String,
    /// The last day a basis that ends runs.
    #[serde(default)]
    expected_ends_on: Option<chrono::NaiveDate>,
    /// The programme the demand serves (`thing:` URN).
    #[serde(default)]
    programme_ref: Option<String>,
    #[serde(default)]
    note: Option<String>,
}

pub(super) fn default_basis() -> String {
    "permanent".to_string()
}

/// `PUT /api/workforce-plans/{pid}/demand-lines` — set planned headcount
/// for a department (and optionally a role) at a date; upsert on that key.
#[debug_handler]
pub(super) async fn set_line(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<LinePayload>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    ensure_editable(&plan)?;
    let mut problems = Problems::new();
    problems.cap_text("department", &payload.department);
    problems.ref_opt(
        "programme_ref",
        entity_ref::EntityType::Thing,
        payload.programme_ref.as_deref(),
    );
    if let Some(note) = &payload.note {
        problems.cap_text("note", note);
    }
    ensure_valid(&problems.into_vec())?;
    rules::validate_line(
        &payload.department,
        payload.target_on,
        plan.horizon_start,
        plan.horizon_end,
        payload.target_headcount,
    )
    .map_err(|e| unprocessable(&e))?;
    rules::validate_line_basis(
        &payload.basis,
        payload.target_on,
        payload.expected_ends_on,
        payload.target_headcount,
        payload.target_fte_percent,
    )
    .map_err(|e| unprocessable(&e))?;
    let db = crate::db::pool(&ctx);
    if let Some(role) = payload.role_profile_pid {
        sqlx::query_scalar!(
            "SELECT id FROM role_profiles WHERE pid = $1 AND deleted_at IS NULL",
            role
        )
        .fetch_optional(db)
        .await?
        .ok_or(Error::NotFound)?;
    }
    let mut tx = db.begin().await?;
    // One upsert on the line's key, the unique index over the plan, department, role, day, basis and programme (no role
    // and no programme count as one value each), where it was a read and then an insert or an update.
    let line = sqlx::query_scalar!(
        "INSERT INTO plan_demand_lines (pid, plan_pid, department, role_profile_pid, target_on, target_headcount,
                                        target_fte_percent, basis, expected_ends_on, programme_ref, note)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         ON CONFLICT (plan_pid, department, COALESCE(role_profile_pid, '00000000-0000-0000-0000-000000000000'::uuid),
                      target_on, basis, COALESCE(programme_ref, ''::character varying))
         DO UPDATE SET target_headcount = EXCLUDED.target_headcount, target_fte_percent = EXCLUDED.target_fte_percent,
                       expected_ends_on = EXCLUDED.expected_ends_on, note = EXCLUDED.note
         RETURNING pid",
        Uuid::new_v4(),
        plan.pid,
        payload.department.trim(),
        payload.role_profile_pid,
        payload.target_on,
        payload.target_headcount,
        payload.target_fte_percent,
        payload.basis,
        payload.expected_ends_on,
        payload.programme_ref,
        payload.note,
    )
    .fetch_one(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "workforce_plan",
        plan.pid,
        "demand_line_set",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: line.to_string(),
    })
}

/// `DELETE /api/workforce-plans/{pid}/demand-lines/{line_pid}`.
#[debug_handler]
pub(super) async fn remove_line(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, line_pid)): Path<(String, String)>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    ensure_editable(&plan)?;
    let line_pid = records::parse_pid(&line_pid)?;
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "DELETE FROM demand_line_objectives WHERE line_pid = $1 AND EXISTS
             (SELECT 1 FROM plan_demand_lines WHERE pid = $1 AND plan_pid = $2)",
        line_pid,
        plan.pid,
    )
    .execute(&mut *tx)
    .await?;
    let removed = sqlx::query!(
        "DELETE FROM plan_demand_lines WHERE pid = $1 AND plan_pid = $2",
        line_pid,
        plan.pid,
    )
    .execute(&mut *tx)
    .await?;
    if removed.rows_affected() == 0 {
        return Err(Error::NotFound);
    }
    crate::db::audit(
        &mut tx,
        "workforce_plan",
        plan.pid,
        "demand_line_removed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}
