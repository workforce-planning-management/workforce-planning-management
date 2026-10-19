//! `POST /api/workers/possible-duplicates` — before hiring or importing someone, which existing
//! workers might be them (WPM-R136, WPM-D87).
//!
//! Read-only: it writes nothing, merges nothing and creates no audit row. The candidates are the
//! workers the caller can already read (their organizations, and a per-record `Read` check), so
//! the answer names no one the directory would not. The comparison itself is
//! [`crate::rules::duplicates`].

use axum::http::StatusCode;
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

use super::{ensure_valid, record_rejection};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::workers;
use crate::models::memberships;
use crate::rules::duplicates::{self as rules, Known, Query};
use crate::validation::Problems;

/// The person being checked.
#[derive(Debug, Deserialize)]
struct CheckPayload {
    display_name: String,
    organization_ref: String,
    #[serde(default)]
    worker_number: Option<String>,
}

/// `POST /api/workers/possible-duplicates`.
#[debug_handler]
async fn possible_duplicates(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<CheckPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("display_name", &payload.display_name);
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        &payload.organization_ref,
    );
    ensure_valid(&problems.into_vec())?;

    let mut query = workers::Entity::find().filter(workers::Column::DeletedAt.is_null());
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await? {
        if !refs.contains(&payload.organization_ref) {
            return Err(record_rejection((
                StatusCode::FORBIDDEN,
                "organization_ref is outside the organizations you can read".to_string(),
            )));
        }
        query = query.filter(workers::Column::OrganizationRef.is_in(refs));
    }
    let mut known = Vec::new();
    for worker in query.all(&ctx.db).await? {
        // a worker the caller may not read is not a candidate: their existence is not disclosed
        if auth::authorize_record(
            &caller,
            authentication_verifier::Action::Read,
            &auth::worker_resource_attrs(&worker),
        )
        .is_err()
        {
            continue;
        }
        known.push(Known {
            pid: worker.pid,
            display_name: worker.display_name,
            organization_ref: worker.organization_ref,
            worker_number: worker.worker_number,
        });
    }
    let found = rules::possible_duplicates(
        &Query {
            display_name: payload.display_name,
            organization_ref: payload.organization_ref,
            worker_number: payload.worker_number,
        },
        &known,
    );
    format::json(found)
}

/// The duplicate-check route.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/possible-duplicates", post(possible_duplicates))
}
