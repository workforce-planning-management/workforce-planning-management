//! Objectives (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;
use axa::extract::{Json, Path};

/// `POST /api/workforce-plans/{pid}/objectives` body.
#[derive(Debug, Deserialize)]
pub(super) struct ObjectivePayload {
    title: String,
    #[serde(default)]
    owner: Option<String>,
}

/// `POST /api/workforce-plans/{pid}/objectives` — add a strategic
/// objective the plan serves.
#[debug_handler]
pub(super) async fn create_objective(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ObjectivePayload>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    ensure_editable(&plan)?;
    let mut problems = Problems::new();
    problems.require_text("title", &payload.title);
    problems.cap_text("title", &payload.title);
    if let Some(owner) = &payload.owner {
        problems.cap_text("owner", owner);
    }
    ensure_valid(&problems.into_vec())?;
    let objective = Uuid::new_v4();
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO plan_objectives (pid, plan_pid, title, owner) VALUES ($1, $2, $3, $4)",
        objective,
        plan.pid,
        payload.title.trim(),
        payload.owner,
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "workforce_plan",
        plan.pid,
        "objective_added",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: objective.to_string(),
    })
}

/// `PUT /api/workforce-plans/{pid}/demand-lines/{line_pid}/objectives` body.
#[derive(Debug, Deserialize)]
pub(super) struct LinkPayload {
    objective_pids: Vec<Uuid>,
}

/// `PUT /api/workforce-plans/{pid}/demand-lines/{line_pid}/objectives` —
/// replace the set of objectives a demand line serves.
#[debug_handler]
pub(super) async fn set_line_objectives(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, line_pid)): Path<(String, String)>,
    Json(payload): Json<LinkPayload>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    ensure_editable(&plan)?;
    let line = sqlx::query_scalar!(
        "SELECT pid FROM plan_demand_lines WHERE plan_pid = $1 AND pid = $2",
        plan.pid,
        records::parse_pid(&line_pid)?,
    )
    .fetch_optional(crate::db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let known: BTreeSet<Uuid> = sqlx::query_scalar!(
        "SELECT pid FROM plan_objectives WHERE plan_pid = $1",
        plan.pid
    )
    .fetch_all(crate::db::pool(&ctx))
    .await?
    .into_iter()
    .collect();
    let wanted: BTreeSet<Uuid> = payload.objective_pids.iter().copied().collect();
    if !wanted.is_subset(&known) {
        return Err(unprocessable("objective_pids must belong to this plan"));
    }
    let wanted: Vec<Uuid> = wanted.into_iter().collect();
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "DELETE FROM demand_line_objectives WHERE line_pid = $1",
        line
    )
    .execute(&mut *tx)
    .await?;
    // The new set in one statement, each link with its own pid.
    sqlx::query!(
        "INSERT INTO demand_line_objectives (pid, line_pid, objective_pid)
         SELECT gen_random_uuid(), $1, objective FROM unnest($2::uuid[]) AS objective",
        line,
        &wanted,
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "workforce_plan",
        plan.pid,
        "line_objectives_set",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}
