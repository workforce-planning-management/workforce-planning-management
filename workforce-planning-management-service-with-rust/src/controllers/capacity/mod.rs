//! **Delivery capacity** (WPM-R56–R60, WPM-D45, WPM-D46, WPM-D50): skill pools,
//! programme demand, partner commitments, the capacity view and the start check.
//! The arithmetic is in [`crate::rules::capacity`].
//!
//! Nothing here names a worker. Supply is worked out from workers' FTE, dates, role
//! profiles and skills and leaves only as numbers. Programmes and partners are
//! `EntityRef` URNs owned by other services; WPM keeps no copy of them. A claim becomes
//! **active** only through a recorded start decision (WPM-D50), never by a plain write.

use super::ensure_org_in_scope;
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    partner_commitments, programme_demands, skill_pool_members, skill_pools,
};
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
    memberships::scope_organization_refs(ctx, caller.claims()).await
}

async fn live_pool(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<skill_pools::Model> {
    let pool = pool_by_pid(ctx, records::parse_pid(pid)?)
        .await?
        .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &pool.organization_ref).await?;
    Ok(pool)
}

// The shared reads, into the same models (WPM-T466). Each statement is written out once here.

/// A live pool by pid.
async fn pool_by_pid(ctx: &AppContext, pid: Uuid) -> Result<Option<skill_pools::Model>> {
    Ok(sqlx::query_as!(
        skill_pools::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name,
                  description, operations_reservation_bp, peer_pool_ref, deleted_at AS "deleted_at: _"
             FROM skill_pools WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(crate::db::pool(ctx))
    .await?)
}

/// The live pools of `org`, by name.
async fn pools_of(ctx: &AppContext, org: &str) -> Result<Vec<skill_pools::Model>> {
    Ok(sqlx::query_as!(
        skill_pools::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name,
                  description, operations_reservation_bp, peer_pool_ref, deleted_at AS "deleted_at: _"
             FROM skill_pools WHERE organization_ref = $1 AND deleted_at IS NULL ORDER BY name"#,
        org,
    )
    .fetch_all(crate::db::pool(ctx))
    .await?)
}

/// The live members of `pools`.
async fn members_in(ctx: &AppContext, pools: &[Uuid]) -> Result<Vec<skill_pool_members::Model>> {
    Ok(sqlx::query_as!(
        skill_pool_members::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, pool_pid, role_profile_pid,
                  skill_pid, min_proficiency, deleted_at AS "deleted_at: _"
             FROM skill_pool_members WHERE pool_pid = ANY($1) AND deleted_at IS NULL ORDER BY id"#,
        pools,
    )
    .fetch_all(crate::db::pool(ctx))
    .await?)
}

/// The live demand claims on `pools`.
async fn demands_on(ctx: &AppContext, pools: &[Uuid]) -> Result<Vec<programme_demands::Model>> {
    Ok(sqlx::query_as!(
        programme_demands::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, programme_ref, pool_pid, month,
                  fte_centi, status, source, peer_event_id, received_at AS "received_at: _",
                  peer_sent_at AS "peer_sent_at: _", peer_withdrawn_at AS "peer_withdrawn_at: _",
                  deleted_at AS "deleted_at: _"
             FROM programme_demands WHERE pool_pid = ANY($1) AND deleted_at IS NULL ORDER BY id"#,
        pools,
    )
    .fetch_all(crate::db::pool(ctx))
    .await?)
}

/// The live demand claims of `programme`: only `proposed` ones when `proposed_only`, and only on `pools` when given.
async fn demands_of_programme(
    ctx: &AppContext,
    programme: &str,
    proposed_only: bool,
    pools: Option<&[Uuid]>,
) -> Result<Vec<programme_demands::Model>> {
    Ok(sqlx::query_as!(
        programme_demands::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, programme_ref, pool_pid, month,
                  fte_centi, status, source, peer_event_id, received_at AS "received_at: _",
                  peer_sent_at AS "peer_sent_at: _", peer_withdrawn_at AS "peer_withdrawn_at: _",
                  deleted_at AS "deleted_at: _"
             FROM programme_demands
            WHERE programme_ref = $1 AND deleted_at IS NULL AND (NOT $2 OR status = 'proposed')
              AND ($3::uuid[] IS NULL OR pool_pid = ANY($3))
            ORDER BY id"#,
        programme,
        proposed_only,
        pools,
    )
    .fetch_all(crate::db::pool(ctx))
    .await?)
}

/// The live partner commitments to `pools`.
async fn commitments_to(
    ctx: &AppContext,
    pools: &[Uuid],
) -> Result<Vec<partner_commitments::Model>> {
    Ok(sqlx::query_as!(
        partner_commitments::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, partner_ref, pool_pid, month,
                  fte_centi, status, deleted_at AS "deleted_at: _"
             FROM partner_commitments WHERE pool_pid = ANY($1) AND deleted_at IS NULL ORDER BY id"#,
        pools,
    )
    .fetch_all(crate::db::pool(ctx))
    .await?)
}

fn pid_ref(pid: Uuid) -> serde_json::Value {
    json!({ "pid": pid.to_string() })
}

mod demand;
mod peer;
mod people_impact;
mod pools;
mod settings;
mod start_check;
mod view;
mod what_if;
// The sections share this module's imports and each other's handlers.
#[allow(clippy::wildcard_imports)]
use self::demand::*;
#[allow(clippy::wildcard_imports)]
use self::peer::*;
#[allow(clippy::wildcard_imports)]
use self::people_impact::*;
#[allow(clippy::wildcard_imports)]
use self::pools::*;
#[allow(clippy::wildcard_imports)]
use self::settings::*;
#[allow(clippy::wildcard_imports)]
use self::start_check::*;
#[allow(clippy::wildcard_imports)]
use self::view::*;
pub(crate) use self::view::{load_org_capacity, pack_view};
#[allow(clippy::wildcard_imports)]
use self::what_if::*;

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
        .add("/skill-pools/{pid}/peer-link", put(set_peer_link))
        .add("/capacity/peer-demand", post(receive_peer_demand))
        .add("/capacity/peer-supply", get(peer_supply_view))
        .add("/capacity/people-impact", get(people_impact))
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
        .add("/capacity/what-if", post(what_if))
        .add("/capacity/start-decisions", post(record_start_decision))
        .add("/capacity/start-decisions", get(list_start_decisions))
}
