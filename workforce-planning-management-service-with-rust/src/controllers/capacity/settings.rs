//! Settings (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
use super::{
    AppContext, Deserialize, MaybeAuthUser, OrgQuery, Problems, Response, Result, State, Uuid,
    debug_handler, ensure_org_in_scope, ensure_valid, format, json, unprocessable,
};
use axa::extract::Json;

/// `PUT /api/capacity/settings` body.
#[derive(Debug, Deserialize)]
pub(super) struct SettingsPayload {
    organization_ref: String,
    /// The most programmes that may be active on the constraint pool; absent for no limit.
    #[serde(default)]
    wip_limit: Option<i32>,
    /// Whether the organization records per-person allocation (WPM-R275); absent keeps what is set.
    #[serde(default)]
    allocation_tracked: Option<bool>,
}

/// `PUT /api/capacity/settings` — the organization's work-in-progress limit.
#[debug_handler]
pub(super) async fn set_settings(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<SettingsPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        &payload.organization_ref,
    );
    ensure_valid(&problems.into_vec())?;
    if payload.wip_limit.is_some_and(|n| n < 0_i32) {
        return Err(unprocessable("wip_limit must not be negative"));
    }
    ensure_org_in_scope(&ctx, &caller, &payload.organization_ref).await?;
    let mut tx = crate::db::pool(&ctx).begin().await?;
    // One row per organization: one upsert on its unique key rather than a read and a write. A row already there
    // keeps its pid.
    let row = sqlx::query!(
        "INSERT INTO capacity_settings (pid, organization_ref, wip_limit, allocation_tracked)
         VALUES ($1, $2, $3, COALESCE($4, FALSE))
         ON CONFLICT (organization_ref) DO UPDATE SET wip_limit = EXCLUDED.wip_limit,
                allocation_tracked = COALESCE($4, capacity_settings.allocation_tracked)
         RETURNING pid, organization_ref, wip_limit, allocation_tracked",
        Uuid::new_v4(),
        payload.organization_ref,
        payload.wip_limit,
        payload.allocation_tracked,
    )
    .fetch_one(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "capacity_settings",
        row.pid,
        "set",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({
        "organization_ref": row.organization_ref,
        "wip_limit": row.wip_limit,
        "allocation_tracked": row.allocation_tracked,
    }))
}

pub(super) async fn wip_limit_of(ctx: &AppContext, org: &str) -> Result<Option<i32>> {
    Ok(sqlx::query_scalar!(
        "SELECT wip_limit FROM capacity_settings WHERE organization_ref = $1",
        org
    )
    .fetch_optional(crate::db::pool(ctx))
    .await?
    .flatten())
}

/// `GET /api/capacity/settings?organization=`.
#[debug_handler]
pub(super) async fn get_settings(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(query): axa::extract::Query<OrgQuery>,
) -> Result<Response> {
    let org = query
        .organization
        .ok_or_else(|| unprocessable("organization is required"))?;
    ensure_org_in_scope(&ctx, &caller, &org).await?;
    let tracked = sqlx::query_scalar!(
        "SELECT allocation_tracked FROM capacity_settings WHERE organization_ref = $1",
        org
    )
    .fetch_optional(crate::db::pool(&ctx))
    .await?
    .unwrap_or(false);
    format::json(json!({
        "organization_ref": org,
        "wip_limit": wip_limit_of(&ctx, &org).await?,
        "allocation_tracked": tracked,
    }))
}
