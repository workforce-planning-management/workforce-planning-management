//! **Job levels** (WPM-R52): `GET /api/job-levels` — published level ladders,
//! starting with Google's technical levels L3–L11 (see
//! [`crate::rules::job_levels`]). Public reference data: no person is named, no
//! pay is given, and nothing is stored.

use loco_rs::prelude::*;
use serde_json::json;

use crate::rules::job_levels as rules;

/// `GET /api/job-levels` — the frameworks, without their levels.
#[debug_handler]
async fn list() -> Result<Response> {
    let frameworks: Vec<_> = rules::all()
        .iter()
        .map(|f| {
            json!({
                "id": f.id,
                "name": f.name,
                "organization": f.organization,
                "track": f.track,
                "source": f.source,
                "levels": f.levels.iter().map(|l| l.code).collect::<Vec<_>>(),
            })
        })
        .collect();
    format::json(frameworks)
}

/// `GET /api/job-levels/{id}` — one framework with every level.
#[debug_handler]
async fn show(Path(id): Path<String>) -> Result<Response> {
    format::json(rules::find(&id).ok_or(Error::NotFound)?)
}

/// `GET /api/job-levels/{id}/levels/{code}` — one level and the next one up
/// (`null` at the top). `L5`, `l5` and `5` all name the same level.
#[debug_handler]
async fn level(Path((id, code)): Path<(String, String)>) -> Result<Response> {
    let framework = rules::find(&id).ok_or(Error::NotFound)?;
    let found = framework.level(&code).ok_or(Error::NotFound)?;
    format::json(json!({
        "framework": framework.id,
        "level": found,
        "next_level": framework.next_above(found.number),
    }))
}

/// The job-level routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/job-levels", get(list))
        .add("/job-levels/{id}", get(show))
        .add("/job-levels/{id}/levels/{code}", get(level))
}
