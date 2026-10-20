//! **Delivery capacity** (WPM-R56–R60, WPM-D45, WPM-D46, WPM-D50): skill pools,
//! programme demand, partner commitments, the capacity view and the start check.
//! The arithmetic is in [`crate::rules::capacity`].
//!
//! Nothing here names a worker. Supply is worked out from workers' FTE, dates, role
//! profiles and skills and leaves only as numbers. Programmes and partners are
//! `EntityRef` URNs owned by other services; WPM keeps no copy of them. A claim becomes
//! **active** only through a recorded start decision (WPM-D50), never by a plain write.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, IntoActiveModel, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    capacity_settings, leave_requests, partner_commitments, programme_demands, role_profiles,
    skill_pool_members, skill_pools, skills, start_decisions, worker_framework_roles,
    worker_skills, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::rules::capacity as rules;
use crate::validation::Problems;

/// The default number of months in the view.
const DEFAULT_MONTHS: u32 = 12;
/// The number of months a start check looks across, so a delay has room to be found.
const CHECK_MONTHS: u32 = 24;

fn today() -> NaiveDate {
    Utc::now().date_naive()
}

/// The organizations the caller may see: `None` when unrestricted.
async fn scope(ctx: &AppContext, caller: &MaybeAuthUser) -> Result<Option<Vec<String>>> {
    memberships::scope_organization_refs(&ctx.db, caller.claims()).await
}

/// `404` for an organization outside the caller's scope, as for every scoped read.
async fn ensure_org_in_scope(ctx: &AppContext, caller: &MaybeAuthUser, org: &str) -> Result<()> {
    if let Some(refs) = scope(ctx, caller).await?
        && !refs.iter().any(|r| r == org)
    {
        return Err(Error::NotFound);
    }
    Ok(())
}

async fn live_pool(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<skill_pools::Model> {
    let pool = skill_pools::Entity::find()
        .filter(skill_pools::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(skill_pools::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &pool.organization_ref).await?;
    Ok(pool)
}

fn pid_ref(pid: Uuid) -> serde_json::Value {
    json!({ "pid": pid.to_string() })
}

mod demand;
mod pools;
mod settings;
mod start_check;
mod view;
// The sections share this module's imports and each other's handlers.
#[allow(clippy::wildcard_imports)]
use self::demand::*;
#[allow(clippy::wildcard_imports)]
use self::pools::*;
#[allow(clippy::wildcard_imports)]
use self::settings::*;
#[allow(clippy::wildcard_imports)]
use self::start_check::*;
#[allow(clippy::wildcard_imports)]
use self::view::*;
pub(crate) use self::view::{load_org_capacity, pack_view};

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/skill-pools", post(create_pool))
        .add("/skill-pools", get(list_pools))
        .add("/skill-pools/{pid}", get(get_pool))
        .add("/skill-pools/{pid}", put(update_pool))
        .add("/skill-pools/{pid}", delete(delete_pool))
        .add("/skill-pools/{pid}/members", post(add_member))
        .add(
            "/skill-pools/{pid}/members/{member_pid}",
            delete(remove_member),
        )
        .add("/programme-demands", put(set_demand))
        .add("/programme-demands", get(list_demand))
        .add("/programme-demands/{pid}", delete(delete_demand))
        .add("/partner-commitments", put(set_commitment))
        .add("/partner-commitments", get(list_commitments))
        .add("/partner-commitments/{pid}", delete(delete_commitment))
        .add("/capacity", get(capacity_view))
        .add("/capacity/settings", put(set_settings))
        .add("/capacity/settings", get(get_settings))
        .add("/capacity/start-check", post(start_check))
        .add("/capacity/start-decisions", post(record_start_decision))
        .add("/capacity/start-decisions", get(list_start_decisions))
}
