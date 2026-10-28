//! **Job levels** (WPM-R52): `GET /api/job-levels` — published level ladders,
//! starting with Google's technical levels L3–L11 (see
//! [`crate::rules::job_levels`]). Public reference data: no person is named, no
//! pay is given. The coded ladders are fixed; an imported one (SFIA's levels of
//! responsibility, WPM-R110) is read from the database. Both need a signed-in
//! caller: the imported wording does not leave the organization (WPM-R113).

use loco_rs::prelude::*;
use sea_orm::{DatabaseConnection, QueryOrder};
use serde_json::json;

use crate::models::_entities::{job_level_frameworks, job_level_levels};
use crate::rules::job_levels::{self as rules, StoredFramework, StoredLevel};

/// Every ladder: the coded ones, then those imported into the database.
///
/// # Errors
/// A database error.
pub(crate) async fn all_frameworks(db: &DatabaseConnection) -> Result<Vec<StoredFramework>> {
    let mut out: Vec<StoredFramework> = rules::all().into_iter().map(Into::into).collect();
    let levels = job_level_levels::Entity::find()
        .order_by_asc(job_level_levels::Column::Number)
        .all(db)
        .await?;
    for f in job_level_frameworks::Entity::find()
        .order_by_asc(job_level_frameworks::Column::Name)
        .all(db)
        .await?
    {
        if out.iter().any(|c| c.id == f.slug) {
            continue; // a coded ladder wins over a stored one with the same id
        }
        out.push(StoredFramework {
            levels: levels
                .iter()
                .filter(|l| l.framework == f.slug)
                .filter_map(|l| {
                    Some(StoredLevel {
                        number: u8::try_from(l.number).ok()?,
                        code: l.code.clone(),
                        title: l.title.clone(),
                        summary: l.summary.clone(),
                        experience: None,
                        management_equivalent: None,
                        attributes: l
                            .attributes
                            .as_deref()
                            .and_then(|a| serde_json::from_str(a).ok())
                            .unwrap_or_default(),
                    })
                })
                .collect(),
            id: f.slug,
            name: f.name,
            organization: f.organization,
            track: f.track,
            source: f.source,
        });
    }
    Ok(out)
}

/// One ladder by id, coded or stored.
///
/// # Errors
/// A database error.
pub(crate) async fn find_framework(
    db: &DatabaseConnection,
    id: &str,
) -> Result<Option<StoredFramework>> {
    Ok(all_frameworks(db).await?.into_iter().find(|f| f.id == id))
}

/// `GET /api/job-levels` — the frameworks, without their levels.
#[debug_handler]
async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let frameworks: Vec<_> = all_frameworks(&ctx.db)
        .await?
        .iter()
        .map(|f| {
            json!({
                "id": f.id,
                "name": f.name,
                "organization": f.organization,
                "track": f.track,
                "source": f.source,
                "levels": f.levels.iter().map(|l| l.code.as_str()).collect::<Vec<_>>(),
            })
        })
        .collect();
    format::json(frameworks)
}

/// `GET /api/job-levels/{id}` — one framework with every level.
#[debug_handler]
async fn show(State(ctx): State<AppContext>, Path(id): Path<String>) -> Result<Response> {
    format::json(find_framework(&ctx.db, &id).await?.ok_or(Error::NotFound)?)
}

/// `GET /api/job-levels/{id}/levels/{code}` — one level and the next one up
/// (`null` at the top). `L5`, `l5` and `5` all name the same level.
#[debug_handler]
async fn level(
    State(ctx): State<AppContext>,
    Path((id, code)): Path<(String, String)>,
) -> Result<Response> {
    let framework = find_framework(&ctx.db, &id).await?.ok_or(Error::NotFound)?;
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
