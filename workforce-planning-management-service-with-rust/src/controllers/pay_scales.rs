//! **Pay scales** (WPM-R51): `GET /api/pay-scales` and friends — the national
//! banded pay scale, from the government pay circular, and a stateless
//! "where does this salary sit?" lookup (see [`crate::rules::pay_scale`]).
//!
//! The scales are public reference data: no person is named and nothing is
//! stored. The lookup takes a salary in the query string and returns a position;
//! it is a `GET` and writes no audit row because it reads no record. (A caller who
//! asks about a *worker's* salary has already read it through the audited,
//! maskable `GET /api/workers/{pid}`.)

use loco_rs::prelude::*;
use serde::Deserialize;
use serde_json::json;

use crate::rules::pay_scale::{self as rules, PayScale};

/// A summary row: enough to choose a scale, without every band.
fn summary(scale: &PayScale) -> serde_json::Value {
    json!({
        "id": scale.id,
        "name": scale.name,
        "framework": scale.framework,
        "nation": scale.nation,
        "currency": scale.currency,
        "effective_from": scale.effective_from,
        "uplift_tenths_percent": scale.uplift_tenths_percent,
        "source": scale.source,
        "bands": scale.bands.iter().map(|b| b.code).collect::<Vec<_>>(),
    })
}

/// `GET /api/pay-scales` — every scale the service knows.
#[debug_handler]
async fn list() -> Result<Response> {
    let scales: Vec<_> = rules::all().iter().map(summary).collect();
    format::json(scales)
}

/// `GET /api/pay-scales/{id}` — one scale, with every band, step and allowance.
#[debug_handler]
async fn show(Path(id): Path<String>) -> Result<Response> {
    let scale = rules::find(&id).ok_or(Error::NotFound)?;
    format::json(scale)
}

/// Query for the lookup.
#[derive(Debug, Deserialize)]
pub struct PositionParams {
    band: Option<String>,
    salary_minor: Option<i64>,
    /// With `months_on_step`, asks whether a step up is due.
    step: Option<usize>,
    months_on_step: Option<u32>,
}

/// `GET /api/pay-scales/{id}/position?band=&salary_minor=[&step=&months_on_step=]`
/// — where a full-time-equivalent annual salary sits on a band: below the entry
/// step, on a step, between two, or above the top. With `step` and
/// `months_on_step`, also whether progression to the next step is due. Both
/// halves are optional but at least one is required, so the call never answers
/// a question nobody asked.
#[debug_handler]
async fn position(
    Path(id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<PositionParams>,
) -> Result<Response> {
    let scale = rules::find(&id).ok_or(Error::NotFound)?;
    let code = params
        .band
        .as_deref()
        .ok_or_else(|| super::unprocessable("band is required"))?;
    let band = scale
        .band(code)
        .ok_or_else(|| super::unprocessable(&format!("unknown band {code:?} on {}", scale.id)))?;
    if params.salary_minor.is_none() && params.step.is_none() {
        return Err(super::unprocessable(
            "give salary_minor, step with months_on_step, or both",
        ));
    }
    if params.salary_minor.is_some_and(|s| s < 0) {
        return Err(super::unprocessable("salary_minor must be non-negative"));
    }
    let located = params.salary_minor.map(|s| rules::locate(band, s));
    let progression = match (params.step, params.months_on_step) {
        (Some(step), Some(months)) => {
            Some(rules::progression(band, step, months).ok_or_else(|| {
                super::unprocessable(&format!(
                    "band {} has {} step(s); step is 1-based",
                    band.code,
                    band.steps.len()
                ))
            })?)
        }
        (None, None) => None,
        _ => {
            return Err(super::unprocessable(
                "step and months_on_step are given together",
            ));
        }
    };
    format::json(json!({
        "scale": scale.id,
        "band": band.code,
        "closed_to_new_entrants": band.closed,
        "currency": scale.currency,
        "basis": "annual, full-time (37.5 hours a week); normalise part-time pay first",
        "position": located,
        "progression": progression,
    }))
}

/// The pay-scale routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/pay-scales", get(list))
        .add("/pay-scales/{id}", get(show))
        .add("/pay-scales/{id}/position", get(position))
}
