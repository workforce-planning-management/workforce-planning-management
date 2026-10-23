//! Demand lines (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

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
    if let Some(role) = payload.role_profile_pid {
        role_profiles::Entity::find()
            .filter(role_profiles::Column::Pid.eq(role))
            .filter(role_profiles::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?
            .ok_or(Error::NotFound)?;
    }
    let department = payload.department.trim().to_string();
    let mut existing = plan_demand_lines::Entity::find()
        .filter(plan_demand_lines::Column::PlanPid.eq(plan.pid))
        .filter(plan_demand_lines::Column::Department.eq(&department))
        .filter(plan_demand_lines::Column::TargetOn.eq(payload.target_on))
        .filter(plan_demand_lines::Column::Basis.eq(&payload.basis));
    existing = match payload.programme_ref.as_deref() {
        Some(programme) => existing.filter(plan_demand_lines::Column::ProgrammeRef.eq(programme)),
        None => existing.filter(plan_demand_lines::Column::ProgrammeRef.is_null()),
    };
    existing = match payload.role_profile_pid {
        Some(role) => existing.filter(plan_demand_lines::Column::RoleProfilePid.eq(role)),
        None => existing.filter(plan_demand_lines::Column::RoleProfilePid.is_null()),
    };
    let existing = existing.one(&ctx.db).await?;
    let txn = ctx.db.begin().await?;
    let row = match existing {
        Some(row) => {
            let mut active: plan_demand_lines::ActiveModel = row.into();
            active.target_headcount = ActiveValue::set(payload.target_headcount);
            active.target_fte_percent = ActiveValue::set(payload.target_fte_percent);
            active.expected_ends_on = ActiveValue::set(payload.expected_ends_on);
            active.note = ActiveValue::set(payload.note.clone());
            active.update(&txn).await?
        }
        None => {
            plan_demand_lines::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                plan_pid: ActiveValue::set(plan.pid),
                department: ActiveValue::set(department),
                role_profile_pid: ActiveValue::set(payload.role_profile_pid),
                target_on: ActiveValue::set(payload.target_on),
                target_headcount: ActiveValue::set(payload.target_headcount),
                target_fte_percent: ActiveValue::set(payload.target_fte_percent),
                basis: ActiveValue::set(payload.basis.clone()),
                expected_ends_on: ActiveValue::set(payload.expected_ends_on),
                programme_ref: ActiveValue::set(payload.programme_ref.clone()),
                note: ActiveValue::set(payload.note.clone()),
                ..Default::default()
            }
            .insert(&txn)
            .await?
        }
    };
    Audit::record(
        &txn,
        "workforce_plan",
        plan.pid,
        "demand_line_set",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
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
    let row = plan_demand_lines::Entity::find()
        .filter(plan_demand_lines::Column::PlanPid.eq(plan.pid))
        .filter(plan_demand_lines::Column::Pid.eq(line_pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let txn = ctx.db.begin().await?;
    demand_line_objectives::Entity::delete_many()
        .filter(demand_line_objectives::Column::LinePid.eq(line_pid))
        .exec(&txn)
        .await?;
    row.delete(&txn).await?;
    Audit::record(
        &txn,
        "workforce_plan",
        plan.pid,
        "demand_line_removed",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::empty_json()
}
