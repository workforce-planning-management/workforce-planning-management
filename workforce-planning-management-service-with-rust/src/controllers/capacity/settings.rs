//! Settings (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

/// `PUT /api/capacity/settings` body.
#[derive(Debug, Deserialize)]
pub(super) struct SettingsPayload {
    organization_ref: String,
    /// The most programmes that may be active on the constraint pool; absent for no limit.
    #[serde(default)]
    wip_limit: Option<i32>,
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
    if payload.wip_limit.is_some_and(|n| n < 0) {
        return Err(unprocessable("wip_limit must not be negative"));
    }
    ensure_org_in_scope(&ctx, &caller, &payload.organization_ref).await?;
    let existing = capacity_settings::Entity::find()
        .filter(capacity_settings::Column::OrganizationRef.eq(&payload.organization_ref))
        .one(&ctx.db)
        .await?;
    let row = if let Some(row) = existing {
        let mut active = row.into_active_model();
        active.wip_limit = ActiveValue::set(payload.wip_limit);
        active.update(&ctx.db).await?
    } else {
        capacity_settings::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            organization_ref: ActiveValue::set(payload.organization_ref.clone()),
            wip_limit: ActiveValue::set(payload.wip_limit),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?
    };
    Audit::record(
        &ctx.db,
        "capacity_settings",
        row.pid,
        "set",
        caller.actor(),
        None,
    )
    .await?;
    format::json(json!({ "organization_ref": row.organization_ref, "wip_limit": row.wip_limit }))
}

pub(super) async fn wip_limit_of(ctx: &AppContext, org: &str) -> Result<Option<i32>> {
    Ok(capacity_settings::Entity::find()
        .filter(capacity_settings::Column::OrganizationRef.eq(org))
        .one(&ctx.db)
        .await?
        .and_then(|s| s.wip_limit))
}

/// `GET /api/capacity/settings?organization=`.
#[debug_handler]
pub(super) async fn get_settings(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<OrgQuery>,
) -> Result<Response> {
    let org = query
        .organization
        .ok_or_else(|| unprocessable("organization is required"))?;
    ensure_org_in_scope(&ctx, &caller, &org).await?;
    format::json(json!({ "organization_ref": org, "wip_limit": wip_limit_of(&ctx, &org).await? }))
}
