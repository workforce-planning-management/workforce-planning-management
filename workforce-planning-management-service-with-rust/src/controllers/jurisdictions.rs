//! Jurisdiction profiles (WPM-D144, WPM-T380): the codes an organization keeps and its dated, sourced parameter rows. HR of the
//! organization reads and writes them; rows are append-only (a change is a new row with a later `effective_from`, and the database
//! refuses an update or a delete). The audit names the code and the key, never a person or a figure about one.
//!
//! [`profiles_for`] loads what the rules resolve over: the organization's links and rows, for `rules::jurisdiction`.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use loco_rs::prelude::*;
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, forbidden, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::audit_logs::Model as Audit;
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

/// Load the profiles of `organizations` (missing ones are empty).
///
/// # Errors
///
/// A database error.
pub async fn profiles_for<C: ConnectionTrait>(
    db: &C,
    organizations: &[String],
) -> Result<BTreeMap<String, Profile>> {
    let mut out: BTreeMap<String, Profile> = organizations
        .iter()
        .map(|org| (org.clone(), Profile::default()))
        .collect();
    if organizations.is_empty() {
        return Ok(out);
    }
    let orgs: Vec<sea_orm::Value> = vec![organizations.to_vec().into()];
    for row in db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT organization_ref, code, layer, parent_code FROM jurisdictions WHERE organization_ref = ANY($1)",
            orgs.clone(),
        ))
        .await?
    {
        let org: String = row.try_get("", "organization_ref")?;
        let layer: String = row.try_get("", "layer")?;
        if let (Some(profile), Some(layer)) = (out.get_mut(&org), Layer::parse(&layer)) {
            profile.links.push(Link {
                code: row.try_get("", "code")?,
                layer,
                parent: row.try_get("", "parent_code")?,
            });
        }
    }
    for row in db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT organization_ref, code, key, value, effective_from, effective_to, source, status, id
               FROM jurisdiction_parameters WHERE organization_ref = ANY($1) ORDER BY id",
            orgs,
        ))
        .await?
    {
        let org: String = row.try_get("", "organization_ref")?;
        let key: String = row.try_get("", "key")?;
        let raw: serde_json::Value = row.try_get("", "value")?;
        let status: String = row.try_get("", "status")?;
        let (Some(profile), Ok(value), Some(status)) = (
            out.get_mut(&org),
            rules::validate(&key, &raw),
            Status::parse(&status),
        ) else {
            continue;
        };
        profile.rows.push(Row {
            code: row.try_get("", "code")?,
            key,
            value,
            effective_from: row.try_get("", "effective_from")?,
            effective_to: row.try_get("", "effective_to")?,
            source: row.try_get("", "source")?,
            status,
            seq: row.try_get("", "id")?,
        });
    }
    Ok(out)
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
    axum::extract::Query(q): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    hr_only(&ctx, &caller, &q.organization).await?;
    let org: Vec<sea_orm::Value> = vec![q.organization.clone().into()];
    let codes = ctx
        .db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT code, layer, parent_code, name FROM jurisdictions WHERE organization_ref = $1 ORDER BY code",
            org.clone(),
        ))
        .await?
        .into_iter()
        .map(|row| -> Result<serde_json::Value, sea_orm::DbErr> {
            Ok(json!({
                "code": row.try_get::<String>("", "code")?,
                "layer": row.try_get::<String>("", "layer")?,
                "parent_code": row.try_get::<Option<String>>("", "parent_code")?,
                "name": row.try_get::<Option<String>>("", "name")?,
            }))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let parameters = ctx
        .db
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT pid, code, key, value, effective_from, effective_to, source, checked_on, status, created_at
               FROM jurisdiction_parameters WHERE organization_ref = $1 ORDER BY code, key, effective_from, id",
            org,
        ))
        .await?
        .into_iter()
        .map(|row| -> Result<serde_json::Value, sea_orm::DbErr> {
            Ok(json!({
                "pid": row.try_get::<Uuid>("", "pid")?,
                "code": row.try_get::<String>("", "code")?,
                "key": row.try_get::<String>("", "key")?,
                "value": row.try_get::<serde_json::Value>("", "value")?,
                "effective_from": row.try_get::<NaiveDate>("", "effective_from")?,
                "effective_to": row.try_get::<Option<NaiveDate>>("", "effective_to")?,
                "source": row.try_get::<String>("", "source")?,
                "checked_on": row.try_get::<Option<NaiveDate>>("", "checked_on")?,
                "status": row.try_get::<String>("", "status")?,
                "created_at": row.try_get::<chrono::DateTime<chrono::FixedOffset>>("", "created_at")?,
            }))
        })
        .collect::<Result<Vec<_>, _>>()?;
    format::json(json!({
        "organization_ref": q.organization,
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
    let mut profile = profiles_for(&ctx.db, std::slice::from_ref(&p.organization_ref))
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
    let txn = ctx.db.begin().await?;
    txn.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO jurisdictions (organization_ref, code, layer, parent_code, name) VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (organization_ref, code)
         DO UPDATE SET layer = EXCLUDED.layer, parent_code = EXCLUDED.parent_code, name = EXCLUDED.name, updated_at = now()",
        [
            p.organization_ref.clone().into(),
            p.code.clone().into(),
            layer.as_str().into(),
            p.parent_code.clone().into(),
            name.map(str::to_string).into(),
        ],
    ))
    .await?;
    Audit::record(
        &txn,
        "jurisdiction",
        Uuid::new_v4(),
        "jurisdiction_recorded",
        caller.actor(),
        Some(json!({ "code": p.code, "layer": layer.as_str(), "parent_code": p.parent_code })),
    )
    .await?;
    txn.commit().await?;
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
    if let Err(message) = &value {
        problems.push(message.clone());
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
    let txn = ctx.db.begin().await?;
    txn.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO jurisdiction_parameters
             (pid, organization_ref, code, key, value, effective_from, effective_to, source, checked_on, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        [
            pid.into(),
            p.organization_ref.clone().into(),
            p.code.clone().into(),
            p.key.clone().into(),
            canonical.clone().into(),
            p.effective_from.into(),
            p.effective_to.into(),
            source.to_string().into(),
            p.checked_on.into(),
            p.status.clone().into(),
        ],
    ))
    .await?;
    Audit::record(
        &txn,
        "jurisdiction_parameter",
        pid,
        "jurisdiction_parameter_added",
        caller.actor(),
        Some(json!({ "code": p.code, "key": p.key, "effective_from": p.effective_from, "status": p.status })),
    )
    .await?;
    txn.commit().await?;
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

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/jurisdiction-profiles", get(list))
        .add("/jurisdictions", post(record_code))
        .add("/jurisdiction-parameters", post(add_parameter))
}
