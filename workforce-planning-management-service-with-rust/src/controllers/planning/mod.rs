//! **Strategic workforce planning** (WPM-R36–R38, WPM-D26–D28): workforce
//! plans (draft worlds of aggregate hypothetical headcount), demand lines,
//! strategic objectives, the supply forecast and gap analysis, and the
//! alignment view. Pure rules in [`crate::rules::planning`].
//!
//! A plan never references a worker row (WPM-D26). The forecast is stated
//! assumptions plus arithmetic — with too little history it answers
//! `insufficient_history`, not an extrapolation (WPM-D27). Gaps are
//! aggregate and levers are suggestions (WPM-D28).

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use super::{ensure_valid, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{
    benchmarks, conversion_plans, demand_line_objectives, headcount_snapshots, plan_demand_lines,
    plan_objectives, role_profiles, role_skill_requirements, succession_candidates,
    succession_plans, worker_skills, workers, workforce_plans,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::rules::cost as cost_rules;
use crate::rules::metrics::is_employed_on;
use crate::rules::planning as rules;
use crate::rules::talent as talent_rules;
use crate::validation::Problems;

/// A `{pid}` reference response.
#[derive(Serialize)]
struct PidRef {
    pid: String,
}

fn ratio_json(terms: Option<(usize, usize, f64)>) -> serde_json::Value {
    terms.map_or(serde_json::Value::Null, |(numerator, denominator, value)| {
        serde_json::json!({ "numerator": numerator, "denominator": denominator, "value": value })
    })
}

mod alignment;
mod cost;
mod demand_lines;
mod forecast;
mod objectives;
mod plans;
// The sections share this module's imports and each other's handlers.
#[allow(clippy::wildcard_imports)]
use self::alignment::*;
#[allow(clippy::wildcard_imports)]
use self::cost::*;
#[allow(clippy::wildcard_imports)]
use self::demand_lines::*;
#[allow(clippy::wildcard_imports)]
use self::forecast::*;
#[allow(clippy::wildcard_imports)]
use self::objectives::*;
#[allow(clippy::wildcard_imports)]
use self::plans::*;

/// The workforce-planning routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workforce-plans", post(create_plan))
        .add("/workforce-plans", get(list_plans))
        .add("/workforce-plans/{pid}", get(get_plan))
        .add("/workforce-plans/{pid}/status", post(set_status))
        .add("/workforce-plans/{pid}/demand-lines", put(set_line))
        .add(
            "/workforce-plans/{pid}/demand-lines/{line_pid}",
            delete(remove_line),
        )
        .add(
            "/workforce-plans/{pid}/demand-lines/{line_pid}/objectives",
            put(set_line_objectives),
        )
        .add("/workforce-plans/{pid}/objectives", post(create_objective))
        .add("/workforce-plans/{pid}/forecast", get(forecast))
        .add("/workforce-plans/{pid}/cost", get(cost))
        .add("/workforce-plans/{pid}/alignment", get(alignment))
}
