//! Verifiable credentials out (WPM-R157, WPM-D109): a verified CPD entry issued as an Open Badges 3.0 credential, signed as a
//! `vc+jwt` (`EdDSA`), with a `Bitstring Status List` a verifier can read. WPM keeps the **issuance record only**: the credential is
//! returned once and not stored, and it names no person. The signing key is held outside the database (`WPM_CREDENTIAL_SIGNING_KEY`).
//! The issuer's public key and the status list are public by necessity (a verifier has no account here) and hold no personal data.
//! Spec: `spec/skills-and-training.md`.

use chrono::{DateTime, SecondsFormat, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::me::me_worker;
use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{cpd_entries, credential_issuances};
use crate::models::audit_logs::Model as Audit;
use crate::rules::credential as rules;
use crate::rules::webhook::check_url;

/// How the issuer is set up: its URL, name and key. Read each time; the key is never stored or logged.
struct Issuer {
    id: String,
    name: String,
    seed: [u8; 32],
    kid: String,
    list_url: String,
}

fn issuer() -> Result<Issuer> {
    let off = || unprocessable("credential issuing is not set up on this deployment (WPM_CREDENTIAL_ISSUER and WPM_CREDENTIAL_SIGNING_KEY)");
    let id = crate::compat::env_var("WPM_CREDENTIAL_ISSUER").ok_or_else(off)?;
    let id = check_url(&id).map_err(|e| unprocessable(&format!("WPM_CREDENTIAL_ISSUER: {e}")))?;
    let id = id.trim_end_matches('/').to_string();
    let seed = crate::compat::env_var("WPM_CREDENTIAL_SIGNING_KEY").ok_or_else(off)?;
    let seed = rules::parse_seed(&seed).map_err(|e| unprocessable(&format!("WPM_CREDENTIAL_SIGNING_KEY: {e}")))?;
    let name = crate::compat::env_var("WPM_CREDENTIAL_ISSUER_NAME").unwrap_or_else(|| id.clone());
    Ok(Issuer { kid: format!("{id}#key-1"), list_url: format!("{id}/api/credentials/status-list"), id, name, seed })
}

fn stamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn row(i: &credential_issuances::Model) -> serde_json::Value {
    json!({
        "pid": i.pid, "kind": i.kind, "source_pid": i.source_pid, "issued_at": i.issued_at,
        "revoked_at": i.revoked_at, "status_index": i.id - 1,
    })
}

#[derive(Debug, Deserialize)]
pub(crate) struct IssuePayload {
    cpd_entry_pid: Uuid,
}

/// `POST /api/me/credentials` — issue a credential for one of the caller's own CPD entries. Only on the worker's request, only for an
/// entry that exists, is theirs and is verified, and once while it is live. The signed credential is returned here and nowhere else.
#[debug_handler]
pub(crate) async fn issue(State(ctx): State<AppContext>, caller: MaybeAuthUser, Json(payload): Json<IssuePayload>) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let issuer = issuer()?;
    let entry = cpd_entries::Entity::find()
        .filter(cpd_entries::Column::Pid.eq(payload.cpd_entry_pid))
        .filter(cpd_entries::Column::WorkerPid.eq(worker.pid))
        .filter(cpd_entries::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let Some(verified_on) = entry.verified_on else {
        return Err(unprocessable("only a verified CPD entry can be issued as a credential"));
    };
    let live = credential_issuances::Entity::find()
        .filter(credential_issuances::Column::Kind.eq("cpd_entry"))
        .filter(credential_issuances::Column::SourcePid.eq(entry.pid))
        .filter(credential_issuances::Column::RevokedAt.is_null())
        .count(&ctx.db)
        .await?;
    if live > 0 {
        return Err(unprocessable("a credential for this entry is already live; revoke it to issue another"));
    }
    let now = Utc::now();
    let pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    let created = credential_issuances::ActiveModel {
        pid: ActiveValue::set(pid),
        worker_pid: ActiveValue::set(worker.pid),
        kind: ActiveValue::set("cpd_entry".to_string()),
        source_pid: ActiveValue::set(entry.pid),
        issued_at: ActiveValue::set(now.into()),
        sha256: ActiveValue::set("0".repeat(64)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    let index = usize::try_from(created.id - 1).unwrap_or(usize::MAX);
    if index >= rules::STATUS_BITS {
        return Err(unprocessable("the revocation list is full; a new list is needed before more can be issued"));
    }
    let description = format!("A continuing professional development activity ({}).", entry.category);
    let criteria = format!("Recorded by the person and verified by the employer on {verified_on}.");
    let credential = rules::badge(&rules::Badge {
        issuer_id: &issuer.id,
        issuer_name: &issuer.name,
        issuance: pid,
        name: &entry.activity,
        description: &description,
        criteria: &criteria,
        valid_from: &stamp(now),
        status_list_url: &issuer.list_url,
        status_index: index,
        alignment: None,
    });
    let token = rules::sign(&issuer.seed, &issuer.kid, &credential);
    let mut active: credential_issuances::ActiveModel = created.into();
    active.sha256 = ActiveValue::set(rules::sha256_hex(&token));
    active.update(&txn).await?;
    Audit::record(&txn, "credential", pid, "credential_issued", caller.actor(), Some(json!({ "kind": "cpd_entry" }))).await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid, "status_index": index, "format": "vc+jwt", "credential": token }))
}

/// `GET /api/me/credentials` — the caller's issuances: that each was issued, and whether it is revoked. Not the credentials.
#[debug_handler]
pub(crate) async fn mine(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let rows = credential_issuances::Entity::find()
        .filter(credential_issuances::Column::WorkerPid.eq(worker.pid))
        .order_by_desc(credential_issuances::Column::Id)
        .all(&ctx.db)
        .await?;
    format::json(json!({ "credentials": rows.iter().map(row).collect::<Vec<_>>() }))
}

async fn revoke_row(ctx: &AppContext, caller: &MaybeAuthUser, found: credential_issuances::Model) -> Result<Response> {
    if found.revoked_at.is_some() {
        return Err(unprocessable("this credential is already revoked"));
    }
    let pid = found.pid;
    let txn = ctx.db.begin().await?;
    let mut active: credential_issuances::ActiveModel = found.into();
    active.revoked_at = ActiveValue::set(Some(Utc::now().into()));
    active.revoked_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    active.update(&txn).await?;
    Audit::record(&txn, "credential", pid, "credential_revoked", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid, "revoked": true }))
}

async fn find(ctx: &AppContext, pid: Uuid) -> Result<credential_issuances::Model> {
    credential_issuances::Entity::find()
        .filter(credential_issuances::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

/// `POST /api/me/credentials/{pid}/revoke` — the worker withdraws one of their own credentials.
#[debug_handler]
pub(crate) async fn revoke_mine(State(ctx): State<AppContext>, caller: MaybeAuthUser, Path(pid): Path<Uuid>) -> Result<Response> {
    let worker = me_worker(&ctx, &caller).await?;
    let found = find(&ctx, pid).await?;
    if found.worker_pid != worker.pid {
        return Err(Error::NotFound);
    }
    revoke_row(&ctx, &caller, found).await
}

/// `POST /api/credentials/{pid}/revoke` — HR (or an administrator) revokes a credential, for instance when its CPD entry is withdrawn.
#[debug_handler]
async fn revoke(State(ctx): State<AppContext>, caller: MaybeAuthUser, Path(pid): Path<Uuid>) -> Result<Response> {
    let found = find(&ctx, pid).await?;
    revoke_row(&ctx, &caller, found).await
}

/// `GET /api/credentials/issuer` — who issues, and the public key to check a credential against. Public; no personal data.
#[debug_handler]
async fn issuer_key() -> Result<Response> {
    let issuer = issuer()?;
    format::json(json!({
        "issuer": issuer.id, "name": issuer.name, "kid": issuer.kid, "jwk": rules::jwk(&issuer.seed, &issuer.kid),
        "status_list": issuer.list_url,
        "formats": ["vc+jwt"], "credential_kind": "OpenBadgeCredential",
        "specifications": { "open_badges": rules::OB_CONTEXT, "verifiable_credentials": rules::VC_CONTEXT, "status_list": "Bitstring Status List v1.0" },
    }))
}

/// `GET /api/credentials/status-list` — the revocation list, as a signed credential. Public; a list of positions and nothing else.
#[debug_handler]
async fn status_list(State(ctx): State<AppContext>) -> Result<Response> {
    let issuer = issuer()?;
    let revoked: Vec<usize> = credential_issuances::Entity::find()
        .filter(credential_issuances::Column::RevokedAt.is_not_null())
        .all(&ctx.db)
        .await?
        .iter()
        .filter_map(|i| usize::try_from(i.id - 1).ok())
        .collect();
    let encoded = rules::encoded_list(&revoked).map_err(|e| unprocessable(&e))?;
    let credential = rules::status_list(&issuer.id, &issuer.list_url, &stamp(Utc::now()), &encoded);
    let token = rules::sign(&issuer.seed, &issuer.kid, &credential);
    format::json(json!({ "credential": credential, "jwt": token }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/credentials/{pid}/revoke", post(revoke))
        .add("/credentials/issuer", get(issuer_key))
        .add("/credentials/status-list", get(status_list))
}
