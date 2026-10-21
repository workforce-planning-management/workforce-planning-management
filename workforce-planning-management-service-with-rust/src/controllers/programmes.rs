//! **Programme roll-up** (WPM-R83, WPM-T141): per programme and month, the FTE funded, the
//! fixed-term and contractor share, and the engagements and funding that end. Pure rules in
//! [`crate::rules::programmes`]. Aggregate only; HR only and audited.

use loco_rs::prelude::*;
use serde_json::json;

use crate::auth::MaybeAuthUser;
use crate::models::_entities::post_fundings;
use crate::models::audit_logs::Model as Audit;
use crate::models::memberships;
use crate::rules::equality_monitoring::DEFAULT_FLOOR;
use crate::rules::programmes::{OnProgramme, rollup};

/// How far ahead the roll-up looks, in calendar months.
const MONTHS: u32 = 12;

/// `GET /api/workforce-intelligence/programmes` — per programme and month: FTE funded, the
/// contractor and fixed-term share of it, engagements ending and funding ending (WPM-R83). Names
/// no one. A programme with fewer people than the floor is left out, a figure for one to nine is
/// withheld, and no total across programmes is given. HR only.
#[debug_handler]
async fn programmes(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let today = chrono::Utc::now().date_naive();
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let funded = post_fundings::Entity::find()
        .filter(post_fundings::Column::ProgrammeRef.is_not_null())
        .all(&ctx.db)
        .await?;
    let funded_pids: Vec<uuid::Uuid> = funded
        .iter()
        .map(|f| f.worker_pid)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let staff = super::funding::employed_among(&ctx, scope.as_deref(), today, &funded_pids).await?;
    let rows: Vec<OnProgramme> = funded
        .into_iter()
        .filter_map(|f| {
            let w = staff.get(&f.worker_pid)?;
            Some(OnProgramme {
                programme: f.programme_ref?,
                fte_percent: w.fte_percent,
                basis: w.employment_type.clone(),
                funding_ends: f.ends_on,
                engagement_ends: w.engagement_ends_on,
            })
        })
        .collect();
    let floor = DEFAULT_FLOOR;
    let result = rollup(&rows, today, MONTHS, floor);
    Audit::record(
        &ctx.db,
        "contingent_workforce",
        uuid::Uuid::nil(),
        "programme_rollup_read",
        caller.actor(),
        None,
    )
    .await?;
    format::json(json!({
        "small_group_floor": floor,
        "withheld_programmes": result.withheld_programmes,
        "programmes": result.programmes.iter().map(|p| json!({
            "programme": p.programme,
            "months": p.months.iter().map(|m| json!({
                "month": m.month,
                "funded_fte": m.funded_fte,
                "contractor_percent_of_fte": m.contractor_percent,
                "fixed_term_percent_of_fte": m.fixed_term_percent,
                "engagements_ending": m.engagements_ending,
                "funding_ending": m.funding_ending,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "derivation": "Programmes are linked through the posts the money funds, not through time recorded \
                       (WPM-D59). A month counts a person's FTE as funded when their funding has no end or \
                       ends on or after the first of that month. A programme with fewer people than the floor \
                       is left out and counted in withheld_programmes; a figure for one to nine people is \
                       withheld, zero is shown; no total across programmes is given, so a withheld one cannot \
                       be worked out. A person with no funding recorded, or funding with no programme, is on \
                       no programme.",
    }))
}

/// The programme routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workforce-intelligence/programmes", get(programmes))
}
