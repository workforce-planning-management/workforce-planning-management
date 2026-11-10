//! `POST /api/workers/possible-duplicates` — before hiring or importing someone, which existing
//! workers might be them (WPM-R136, WPM-D87).
//!
//! Read-only: it writes nothing, merges nothing and creates no audit row. The candidates are the
//! workers the caller can already read (their organizations, and a per-record `Read` check), so
//! the answer names no one the directory would not. The comparison itself is
//! [`crate::rules::duplicates`].

use axa::extract::Json;
use axa::prelude::*;
use serde::Deserialize;

use super::{ensure_valid, forbidden};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
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

    let scope = memberships::scope_organization_refs(&ctx, caller.claims()).await?;
    if let Some(refs) = &scope
        && !refs.contains(&payload.organization_ref)
    {
        return Err(forbidden(
            "organization_ref is outside the organizations you can read",
        ));
    }
    let rows = sqlx::query!(
        "SELECT pid, display_name, organization_ref, worker_number, person_ref, department, status, manager_pid
           FROM workers
          WHERE deleted_at IS NULL AND ($1::text[] IS NULL OR organization_ref = ANY($1))",
        scope.as_deref(),
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let mut known = Vec::new();
    for worker in rows {
        // a worker the caller may not read is not a candidate: their existence is not disclosed
        let attrs = auth::record_attrs(&auth::RecordFacts {
            person_ref: &worker.person_ref,
            department: &worker.department,
            status: &worker.status,
            organization_ref: &worker.organization_ref,
            manager_pid: worker.manager_pid,
        });
        if auth::authorize_record(&caller, authentication_verifier::Action::Read, &attrs).is_err() {
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
