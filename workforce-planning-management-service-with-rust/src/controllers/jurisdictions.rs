//! Jurisdiction profiles (WPM-D144, WPM-T380): the codes an organization keeps and its dated, sourced parameter rows. HR of the
//! organization reads and writes them; rows are append-only (a change is a new row with a later `effective_from`, and the database
//! refuses an update or a delete). The audit names the code and the key, never a person or a figure about one.
//!
//! [`profiles_for`] loads what the rules resolve over: the organization's links and rows, for `rules::jurisdiction`.
//!
//! **Whose chain** (WPM-D153): a reader about one worker resolves over that worker's `work_jurisdiction`; a reader about the
//! organization as a whole (no single worker) over the organization's **home jurisdiction**, the legal entity's registered
//! jurisdiction, which HR sets (`PUT /api/home-jurisdiction`). [`Scope`] is the one place a reader gets its chain.

use std::collections::BTreeMap;

use axa::extract::Json;
use axa::prelude::*;
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, forbidden, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::workers;
use crate::rules::jurisdiction::{self as rules, Layer, Link, Row, Status};

/// One organization's chain links and parameter rows.
#[derive(Clone, Debug, Default)]
pub struct Profile {
    /// The codes it keeps, with their layer and parent.
    pub links: Vec<Link>,
    /// Its parameter rows (validated; a row that no longer fits the registry is left out).
    pub rows: Vec<Row>,
}

impl Profile {
    /// Resolve `key` for work done in `work_jurisdiction` as of `as_of`. A chain that cannot be built (a malformed link) is
    /// treated as unknown rather than guessed.
    #[must_use]
    pub fn resolve(
        &self,
        key: &str,
        work_jurisdiction: Option<&str>,
        as_of: NaiveDate,
    ) -> rules::Resolution {
        let chain = work_jurisdiction.map(|code| rules::chain(code, &self.links));
        match chain {
            Some(Err(_)) => rules::Resolution::Unknown {
                reason: "chain_not_resolvable",
            },
            Some(Ok(steps)) => rules::resolve(key, Some(&steps), &self.rows, as_of).unwrap_or(
                rules::Resolution::Unknown {
                    reason: "not_configured",
                },
            ),
            None => rules::Resolution::Unknown {
                reason: "no_work_jurisdiction",
            },
        }
    }
}

/// Whose chain a reader resolves over (WPM-D153).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reader {
    /// A reader about one worker: the chain of where that worker's work is done.
    Worker,
    /// A reader about the organization as a whole: the chain of its home jurisdiction.
    Organization,
}

impl Reader {
    /// The reason an unrecorded start gives.
    const fn missing(self) -> &'static str {
        match self {
            Self::Worker => "no_work_jurisdiction",
            Self::Organization => "no_home_jurisdiction",
        }
    }
}

/// One reader's chain start and the organization's profile, loaded once per request.
#[derive(Clone, Debug)]
pub struct Scope {
    /// The organization whose profile it is.
    pub organization: String,
    /// Whose chain.
    pub reader: Reader,
    /// Where the chain starts: the worker's `work_jurisdiction` or the organization's home jurisdiction; `None` when not recorded.
    pub start: Option<String>,
    /// The organization's links and rows.
    pub profile: Profile,
}

impl Scope {
    /// The chain of where `worker`'s work is done, over their organization's profile.
    ///
    /// # Errors
    ///
    /// A database error.
    pub async fn for_worker(db: &PgPool, worker: &workers::Model) -> Result<Self> {
        let organization = worker.organization_ref.clone();
        let profile = profiles_for(db, std::slice::from_ref(&organization))
            .await?
            .remove(&organization)
            .unwrap_or_default();
        Ok(Self {
            organization,
            reader: Reader::Worker,
            start: worker.work_jurisdiction.clone(),
            profile,
        })
    }

    /// The chain of `organization`'s home jurisdiction, over its profile.
    ///
    /// # Errors
    ///
    /// A database error.
    pub async fn for_organization(db: &PgPool, organization: &str) -> Result<Self> {
        let profile = profiles_for(db, &[organization.to_string()])
            .await?
            .remove(organization)
            .unwrap_or_default();
        Ok(Self {
            organization: organization.to_string(),
            reader: Reader::Organization,
            start: home_jurisdiction(db, organization).await?,
            profile,
        })
    }

    /// Resolve `key` over this scope's chain as of `as_of`. An unrecorded start is unknown with the reader's reason
    /// (`no_work_jurisdiction` or `no_home_jurisdiction`), never the organization's country or a guess.
    #[must_use]
    pub fn resolve(&self, key: &str, as_of: NaiveDate) -> rules::Resolution {
        match self.profile.resolve(key, self.start.as_deref(), as_of) {
            rules::Resolution::Unknown {
                reason: "no_work_jurisdiction",
            } => rules::Resolution::Unknown {
                reason: self.reader.missing(),
            },
            other => other,
        }
    }
}

/// The home jurisdiction HR recorded for `organization` (WPM-D153), if any.
///
/// # Errors
///
/// A database error.
pub async fn home_jurisdiction(db: &PgPool, organization: &str) -> Result<Option<String>> {
    Ok(sqlx::query_scalar!(
        "SELECT home_jurisdiction FROM organization_settings WHERE organization_ref = $1",
        organization,
    )
    .fetch_optional(db)
    .await?
    .flatten())
}

/// Every organization with a home jurisdiction recorded, by reference.
///
/// # Errors
///
/// A database error.
pub async fn organizations_with_a_home(db: &PgPool) -> Result<Vec<String>> {
    Ok(sqlx::query_scalar!(
        "SELECT organization_ref FROM organization_settings WHERE home_jurisdiction IS NOT NULL ORDER BY organization_ref",
    )
    .fetch_all(db)
    .await?)
}

/// Load the profiles of `organizations` (missing ones are empty).
///
/// # Errors
///
/// A database error.
pub async fn profiles_for(
    db: &PgPool,
    organizations: &[String],
) -> Result<BTreeMap<String, Profile>> {
    let mut out: BTreeMap<String, Profile> = organizations
        .iter()
        .map(|org| (org.clone(), Profile::default()))
        .collect();
    if organizations.is_empty() {
        return Ok(out);
    }
    for row in sqlx::query!(
        "SELECT organization_ref, code, layer, parent_code FROM jurisdictions WHERE organization_ref = ANY($1)",
        organizations,
    )
    .fetch_all(db)
    .await?
    {
        if let (Some(profile), Some(layer)) = (out.get_mut(&row.organization_ref), Layer::parse(&row.layer)) {
            profile.links.push(Link {
                code: row.code,
                layer,
                parent: row.parent_code,
            });
        }
    }
    for row in sqlx::query!(
        "SELECT organization_ref, code, key, value, effective_from, effective_to, source, status, id
           FROM jurisdiction_parameters WHERE organization_ref = ANY($1) ORDER BY id",
        organizations,
    )
    .fetch_all(db)
    .await?
    {
        let (Some(profile), Ok(value), Some(status)) = (
            out.get_mut(&row.organization_ref),
            rules::validate(&row.key, &row.value),
            Status::parse(&row.status),
        ) else {
            continue;
        };
        profile.rows.push(Row {
            code: row.code,
            key: row.key,
            value,
            effective_from: row.effective_from,
            effective_to: row.effective_to,
            source: row.source,
            status,
            seq: row.id,
        });
    }
    Ok(out)
}

/// The country codes `organization` has added that ISO 3166-1 lacks (WPM-D152 question 3): its national jurisdiction codes of
/// two letters that are not ISO 3166-1's, such as `XK`, with the name it recorded. A contact address may use them.
///
/// # Errors
///
/// A database error.
pub async fn deployer_countries(
    db: &PgPool,
    organization: &str,
) -> Result<Vec<(String, Option<String>)>> {
    Ok(sqlx::query!(
        "SELECT code, name FROM jurisdictions
          WHERE organization_ref = $1 AND layer = 'national' AND code ~ '^[A-Z]{2}$' ORDER BY code",
        organization,
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .filter(|row| !crate::rules::contact_details::is_iso_country(&row.code))
    .map(|row| (row.code, row.name))
    .collect())
}

/// What a reader needs of a value beyond its registry shape (WPM-T384): refusal reasons are lowercase tokens, and reporting tiers
/// parse as the gap report reads them. A row the reader could not use is refused here rather than ignored later.
fn reader_check(key: &str, value: &rules::Value) -> Result<(), String> {
    let joined = || value.as_names().unwrap_or_default().join(",");
    match key {
        "flexible_working.refusal_reasons" => {
            crate::rules::flexible_working::parse_reasons(&joined())
                .map(|_| ())
                .map_err(|why| format!("{key}: {why}"))
        }
        "pay_transparency.reporting_tiers" => crate::rules::pay_gap::parse_tiers(&joined())
            .map(|_| ())
            .map_err(|why| format!("{key}: {why}")),
        "employer_contribution.lines" => {
            crate::rules::employer_contribution::parse_lines(&value.as_names().unwrap_or_default())
                .map(|_| ())
                .map_err(|why| format!("{key}: {why}"))
        }
        "recruitment.currency" => match value.as_names().as_deref() {
            Some([one]) if crate::rules::currency::is_known(one) => Ok(()),
            _ => Err(format!("{key}: one ISO-4217 code, e.g. GBP")),
        },
        "employer_contribution.class" => match value.as_names().as_deref() {
            Some([one]) if crate::rules::employer_contribution::is_token(one) => Ok(()),
            _ => Err(format!(
                "{key} is one lowercase token of 1 to 32 letters, digits or underscores"
            )),
        },
        _ => Ok(()),
    }
}

async fn hr_only(ctx: &AppContext, caller: &MaybeAuthUser, organization: &str) -> Result<()> {
    ensure_valid(&organization_problems(organization))?;
    ensure_org_in_scope(ctx, caller, organization).await?;
    if !super::policies::privileged(ctx, caller, organization).await? {
        return Err(forbidden("jurisdiction profiles are kept by HR"));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    organization: String,
}

/// `GET /api/jurisdiction-profiles?organization=` — the organization's codes, every parameter row (append-only, so the history
/// too), and the registry. HR.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(q): axa::extract::Query<ListQuery>,
) -> Result<Response> {
    hr_only(&ctx, &caller, &q.organization).await?;
    let db = crate::db::pool(&ctx);
    let codes: Vec<serde_json::Value> = sqlx::query!(
        "SELECT code, layer, parent_code, name FROM jurisdictions WHERE organization_ref = $1 ORDER BY code",
        q.organization,
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|row| json!({ "code": row.code, "layer": row.layer, "parent_code": row.parent_code, "name": row.name }))
    .collect();
    let parameters: Vec<serde_json::Value> = sqlx::query!(
        "SELECT pid, code, key, value, effective_from, effective_to, source, checked_on, status, created_at
           FROM jurisdiction_parameters WHERE organization_ref = $1 ORDER BY code, key, effective_from, id",
        q.organization,
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|row| {
        json!({
            "pid": row.pid,
            "code": row.code,
            "key": row.key,
            "value": row.value,
            "effective_from": row.effective_from,
            "effective_to": row.effective_to,
            "source": row.source,
            "checked_on": row.checked_on,
            "status": row.status,
            // As the answer gave it before: with a `+00:00` offset, not `Z`.
            "created_at": row.created_at.fixed_offset(),
        })
    })
    .collect();
    let home = home_jurisdiction(db, &q.organization).await?;
    format::json(json!({
        "organization_ref": q.organization,
        "home_jurisdiction": home,
        "jurisdictions": codes,
        "parameters": parameters,
        "registry": rules::REGISTRY,
        "note": "A profile is the deployer's statement with a source, not WPM's claim about any law. A parameter with no row is \
                 unknown (no limit configured), never a default.",
    }))
}

#[derive(Debug, Deserialize)]
struct CodePayload {
    organization_ref: String,
    code: String,
    layer: String,
    #[serde(default)]
    parent_code: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

/// `POST /api/jurisdictions` — record (or re-record) a code the organization keeps, its layer and its parent. Refused when the
/// chain through it would be malformed (a cycle, a parent narrower than its child). HR.
#[debug_handler]
async fn record_code(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<CodePayload>,
) -> Result<Response> {
    hr_only(&ctx, &caller, &p.organization_ref).await?;
    let mut problems = Vec::new();
    if !rules::valid_code(&p.code) {
        problems.push(
            "code must be 2 or 3 capital letters or digits, optionally - and 1 to 3 more"
                .to_string(),
        );
    }
    let layer = Layer::parse(&p.layer);
    match layer {
        None => problems.push("layer must be supranational, national or subnational".to_string()),
        Some(layer) if !rules::layer_fits(&p.code, layer) => problems.push(
            "a code with a subdivision part is subnational; one without is national or supranational".to_string(),
        ),
        Some(_) => {}
    }
    if let Some(parent) = &p.parent_code
        && (!rules::valid_code(parent) || *parent == p.code)
    {
        problems.push("parent_code must be another jurisdiction code".to_string());
    }
    let name = p.name.as_deref().map(str::trim).filter(|n| !n.is_empty());
    if name.is_some_and(|n| n.chars().count() > 100) {
        problems.push("name must be at most 100 characters".to_string());
    }
    ensure_valid(&problems)?;
    let layer = layer.unwrap_or(Layer::National);
    let mut profile = profiles_for(
        crate::db::pool(&ctx),
        std::slice::from_ref(&p.organization_ref),
    )
    .await?
    .remove(&p.organization_ref)
    .unwrap_or_default();
    profile.links.retain(|link| link.code != p.code);
    profile.links.push(Link {
        code: p.code.clone(),
        layer,
        parent: p.parent_code.clone(),
    });
    // Every chain that runs through the code must still be well formed.
    for link in &profile.links {
        if let Err(error) = rules::chain(&link.code, &profile.links) {
            return Err(unprocessable(&format!(
                "the chain from {} would be malformed: {error:?}",
                link.code
            )));
        }
    }
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO jurisdictions (organization_ref, code, layer, parent_code, name) VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (organization_ref, code)
         DO UPDATE SET layer = EXCLUDED.layer, parent_code = EXCLUDED.parent_code, name = EXCLUDED.name, updated_at = now()",
        p.organization_ref,
        p.code,
        layer.as_str(),
        p.parent_code,
        name,
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "jurisdiction",
        Uuid::new_v4(),
        "jurisdiction_recorded",
        caller.actor(),
        Some(json!({ "code": p.code, "layer": layer.as_str(), "parent_code": p.parent_code })),
    )
    .await?;
    tx.commit().await?;
    format::json(
        json!({ "code": p.code, "layer": layer.as_str(), "parent_code": p.parent_code, "name": name }),
    )
}

#[derive(Debug, Deserialize)]
struct ParameterPayload {
    organization_ref: String,
    code: String,
    key: String,
    value: serde_json::Value,
    effective_from: NaiveDate,
    #[serde(default)]
    effective_to: Option<NaiveDate>,
    source: String,
    #[serde(default)]
    checked_on: Option<NaiveDate>,
    status: String,
}

/// `POST /api/jurisdiction-parameters` — append one dated, sourced parameter row. A `read` row needs `checked_on`. HR.
#[debug_handler]
async fn add_parameter(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<ParameterPayload>,
) -> Result<Response> {
    hr_only(&ctx, &caller, &p.organization_ref).await?;
    let mut problems = Vec::new();
    if !rules::valid_code(&p.code) {
        problems.push("code must be a jurisdiction code".to_string());
    }
    let value = rules::validate(&p.key, &p.value);
    match &value {
        Err(message) => problems.push(message.clone()),
        Ok(value) => {
            if let Err(message) = reader_check(&p.key, value) {
                problems.push(message);
            }
        }
    }
    if p.effective_to.is_some_and(|to| to < p.effective_from) {
        problems.push("effective_to must not be before effective_from".to_string());
    }
    let source = p.source.trim();
    if source.is_empty() || source.chars().count() > 500 {
        problems.push("source must say what the value rests on (1 to 500 characters)".to_string());
    }
    match Status::parse(&p.status) {
        None => problems.push("status must be example, unverified or read".to_string()),
        Some(Status::Read) if p.checked_on.is_none() => {
            problems.push("a read row says when it was checked (checked_on)".to_string());
        }
        Some(_) => {}
    }
    ensure_valid(&problems)?;
    let canonical = serde_json::to_value(value.map_err(|m| unprocessable(&m))?)
        .map_err(|e| Error::Message(e.to_string()))?;
    let pid = Uuid::new_v4();
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO jurisdiction_parameters
             (pid, organization_ref, code, key, value, effective_from, effective_to, source, checked_on, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        pid,
        p.organization_ref,
        p.code,
        p.key,
        canonical,
        p.effective_from,
        p.effective_to,
        source,
        p.checked_on,
        p.status,
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "jurisdiction_parameter",
        pid,
        "jurisdiction_parameter_added",
        caller.actor(),
        Some(json!({ "code": p.code, "key": p.key, "effective_from": p.effective_from, "status": p.status })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({
        "pid": pid,
        "code": p.code,
        "key": p.key,
        "value": canonical,
        "effective_from": p.effective_from,
        "effective_to": p.effective_to,
        "status": p.status,
    }))
}

#[derive(Debug, Deserialize)]
struct HomePayload {
    organization_ref: String,
    /// A jurisdiction code, or `null` or blank to clear.
    #[serde(default)]
    home_jurisdiction: Option<String>,
}

/// `PUT /api/home-jurisdiction` — set (or clear, with `null` or blank) the organization's home jurisdiction: the legal entity's
/// registered jurisdiction, whose chain the organization-level readers resolve over (WPM-D153). Refused when the chain from it
/// would be malformed. HR of the organization; audited with the code, which names no person.
#[debug_handler]
async fn set_home(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<HomePayload>,
) -> Result<Response> {
    hr_only(&ctx, &caller, &p.organization_ref).await?;
    let code = p
        .home_jurisdiction
        .as_deref()
        .map(str::trim)
        .filter(|code| !code.is_empty())
        .map(str::to_string);
    if let Some(code) = &code {
        if !rules::valid_code(code) {
            return Err(unprocessable(
                "home_jurisdiction must be a jurisdiction code (2 or 3 capital letters or digits, optionally - and 1 to 3 more)",
            ));
        }
        let profile = profiles_for(
            crate::db::pool(&ctx),
            std::slice::from_ref(&p.organization_ref),
        )
        .await?
        .remove(&p.organization_ref)
        .unwrap_or_default();
        if let Err(error) = rules::chain(code, &profile.links) {
            return Err(unprocessable(&format!(
                "the chain from {code} is malformed: {error:?}"
            )));
        }
    }
    let previous = home_jurisdiction(crate::db::pool(&ctx), &p.organization_ref).await?;
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO organization_settings (organization_ref, home_jurisdiction) VALUES ($1, $2)
         ON CONFLICT (organization_ref) DO UPDATE SET home_jurisdiction = EXCLUDED.home_jurisdiction, updated_at = now()",
        p.organization_ref,
        code,
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "organization_settings",
        Uuid::new_v4(),
        "home_jurisdiction_set",
        caller.actor(),
        Some(json!({ "organization_ref": p.organization_ref, "from": previous, "to": code })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "organization_ref": p.organization_ref, "home_jurisdiction": code }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/jurisdiction-profiles", get(list))
        .add("/jurisdictions", post(record_code))
        .add("/jurisdiction-parameters", post(add_parameter))
        .add("/home-jurisdiction", put(set_home))
}
