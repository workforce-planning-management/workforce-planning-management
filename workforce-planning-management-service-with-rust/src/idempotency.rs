//! **Idempotency keys** (WPM-R132, WPM-D80): the middleware that recognises a repeated request.
//!
//! A request that changes something and carries an `Idempotency-Key` is run once per caller and key.
//! The response of a success is kept for one calendar day and returned again, with
//! `Idempotent-Replayed: true`, for the same request repeated under the same key. The same key with a
//! different request is refused (`422`); a repeat that arrives while the first is still running is
//! told to wait (`409`). A failure is not kept, so the caller may fix the request and retry under the
//! same key. The key belongs to the caller: it is stored with the caller's identity, so no one can
//! replay another caller's response. A request with no key is untouched. The rules are pure, in
//! [`crate::rules::idempotency`].

use axum::{
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use loco_rs::app::AppContext;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, Value};

use crate::auth::MaybeAuthUser;
use crate::rules::idempotency::{self as rules, Decision, Existing};

/// The header a caller names an action with.
pub const HEADER: &str = "idempotency-key";
/// The header that says a response is a replay.
pub const REPLAYED: &str = "idempotent-replayed";
/// The largest response buffered to be stored; a larger one on a keyed request is an error.
const MAX_BUFFERED_RESPONSE_BYTES: usize = 16 * 1024 * 1024;

fn sql(text: &str, values: impl IntoIterator<Item = Value>) -> Statement {
    Statement::from_sql_and_values(DatabaseBackend::Postgres, text, values)
}

fn refuse(status: StatusCode, message: &str) -> Response {
    (status, message.to_string()).into_response()
}

/// Delete what is older than the keep period.
async fn purge(ctx: &AppContext) -> Result<(), sea_orm::DbErr> {
    let days = i32::try_from(rules::KEEP_CALENDAR_DAYS).unwrap_or(1);
    ctx.db
        .execute_raw(sql(
            "DELETE FROM idempotency_keys WHERE created_at < now() - make_interval(days => $1)",
            [days.into()],
        ))
        .await?;
    Ok(())
}

/// Try to claim the key; `true` when this request now owns it.
async fn claim(
    ctx: &AppContext,
    actor: &str,
    key: &str,
    fingerprint: &str,
) -> Result<bool, sea_orm::DbErr> {
    Ok(ctx
        .db
        .query_one_raw(sql(
            "INSERT INTO idempotency_keys (actor, idem_key, fingerprint) VALUES ($1, $2, $3)
             ON CONFLICT (actor, idem_key) DO NOTHING RETURNING id",
            [actor.into(), key.into(), fingerprint.into()],
        ))
        .await?
        .is_some())
}

/// What is held under the key, with the stored response if there is one.
struct Held {
    existing: Existing,
    status: Option<i32>,
    content_type: Option<String>,
    body: Option<Vec<u8>>,
}

async fn held(ctx: &AppContext, actor: &str, key: &str) -> Result<Option<Held>, sea_orm::DbErr> {
    let Some(row) = ctx
        .db
        .query_one_raw(sql(
            "SELECT fingerprint, status_code, content_type, response_body,
                    EXTRACT(EPOCH FROM (now() - created_at))::BIGINT AS age
               FROM idempotency_keys WHERE actor = $1 AND idem_key = $2",
            [actor.into(), key.into()],
        ))
        .await?
    else {
        return Ok(None);
    };
    let status: Option<i32> = row.try_get("", "status_code")?;
    Ok(Some(Held {
        existing: Existing {
            fingerprint: row.try_get("", "fingerprint")?,
            finished: status.is_some(),
            age_seconds: row.try_get("", "age")?,
        },
        status,
        content_type: row.try_get("", "content_type")?,
        body: row.try_get("", "response_body")?,
    }))
}

async fn release(ctx: &AppContext, actor: &str, key: &str) -> Result<(), sea_orm::DbErr> {
    ctx.db
        .execute_raw(sql(
            "DELETE FROM idempotency_keys WHERE actor = $1 AND idem_key = $2",
            [actor.into(), key.into()],
        ))
        .await?;
    Ok(())
}

async fn finish(
    ctx: &AppContext,
    actor: &str,
    key: &str,
    status: u16,
    content_type: Option<&str>,
    body: &[u8],
) -> Result<(), sea_orm::DbErr> {
    ctx.db
        .execute_raw(sql(
            "UPDATE idempotency_keys
                SET status_code = $3, content_type = $4, response_body = $5, updated_at = now()
              WHERE actor = $1 AND idem_key = $2",
            [
                actor.into(),
                key.into(),
                i32::from(status).into(),
                content_type.map(ToString::to_string).into(),
                body.to_vec().into(),
            ],
        ))
        .await?;
    Ok(())
}

fn replay(h: &Held) -> Response {
    let status = h
        .status
        .and_then(|s| u16::try_from(s).ok())
        .and_then(|s| StatusCode::from_u16(s).ok())
        .unwrap_or(StatusCode::OK);
    let mut response = Response::new(Body::from(h.body.clone().unwrap_or_default()));
    *response.status_mut() = status;
    if let Some(content_type) = h
        .content_type
        .as_deref()
        .and_then(|c| HeaderValue::from_str(c).ok())
    {
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, content_type);
    }
    response
        .headers_mut()
        .insert(REPLAYED, HeaderValue::from_static("true"));
    response
}

fn unavailable(error: &sea_orm::DbErr) -> Response {
    tracing::error!(%error, "idempotency: could not check the key, refusing");
    refuse(
        StatusCode::SERVICE_UNAVAILABLE,
        "could not check the Idempotency-Key",
    )
}

fn still_running() -> Response {
    let mut response = refuse(
        StatusCode::CONFLICT,
        "a request with this Idempotency-Key is still running",
    );
    response
        .headers_mut()
        .insert(header::RETRY_AFTER, HeaderValue::from_static("1"));
    response
}

/// Claim the key for this request. `Ok` means the request should run; `Err` is the response to give
/// instead (a replay, a clash, a wait, or an error).
async fn claim_or_answer(
    ctx: &AppContext,
    actor: &str,
    key: &str,
    fingerprint: &str,
) -> Result<(), Box<Response>> {
    purge(ctx).await.map_err(|e| Box::new(unavailable(&e)))?;
    // At most one take-over of an abandoned claim.
    for attempt in 0..2 {
        if claim(ctx, actor, key, fingerprint)
            .await
            .map_err(|e| Box::new(unavailable(&e)))?
        {
            return Ok(());
        }
        let Some(existing) = held(ctx, actor, key)
            .await
            .map_err(|e| Box::new(unavailable(&e)))?
        else {
            // Deleted between the claim and the read: try the claim again.
            continue;
        };
        match rules::decide(Some(&existing.existing), fingerprint) {
            Decision::Replay => return Err(Box::new(replay(&existing))),
            Decision::Clash => {
                return Err(Box::new(refuse(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "this Idempotency-Key was used for a different request",
                )));
            }
            Decision::TakeOver if attempt == 0 => {
                release(ctx, actor, key)
                    .await
                    .map_err(|e| Box::new(unavailable(&e)))?;
            }
            Decision::InProgress | Decision::TakeOver | Decision::Run => {
                return Err(Box::new(still_running()));
            }
        }
    }
    Err(Box::new(still_running()))
}

/// The middleware. It does nothing for a request that does not change anything, for one with no key,
/// and for anything outside `/api`.
pub async fn idempotency_mw(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    req: Request,
    next: Next,
) -> Response {
    if !rules::is_mutating(req.method().as_str()) || !req.uri().path().starts_with("/api/") {
        return next.run(req).await;
    }
    let Some(raw) = req.headers().get(HEADER) else {
        return next.run(req).await;
    };
    let key = match raw.to_str() {
        Ok(k) => k.to_string(),
        Err(_) => {
            return refuse(
                StatusCode::UNPROCESSABLE_ENTITY,
                "Idempotency-Key is not text",
            );
        }
    };
    if let Err(message) = rules::validate_key(&key) {
        return refuse(StatusCode::UNPROCESSABLE_ENTITY, &message);
    }
    let actor = caller.actor().unwrap_or("anonymous").to_string();
    let (parts, body) = req.into_parts();
    let Ok(bytes) = to_bytes(body, rules::MAX_REQUEST_BYTES).await else {
        return refuse(StatusCode::PAYLOAD_TOO_LARGE, "request body too large");
    };
    let path_and_query = parts
        .uri
        .path_and_query()
        .map_or_else(|| parts.uri.path().to_string(), ToString::to_string);
    let fingerprint = rules::fingerprint(parts.method.as_str(), &path_and_query, &bytes);
    if let Err(response) = claim_or_answer(&ctx, &actor, &key, &fingerprint).await {
        return *response;
    }

    let response = next
        .run(Request::from_parts(parts, Body::from(bytes)))
        .await;
    let (response_parts, response_body) = response.into_parts();
    let Ok(buffered) = to_bytes(response_body, MAX_BUFFERED_RESPONSE_BYTES).await else {
        let _ = release(&ctx, &actor, &key).await;
        return refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            "response too large to keep",
        );
    };
    let status = response_parts.status.as_u16();
    if rules::is_storable(status, Some(buffered.len())) {
        let content_type = response_parts
            .headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok());
        if let Err(error) = finish(&ctx, &actor, &key, status, content_type, &buffered).await {
            tracing::error!(%error, "idempotency: could not keep the response");
            let _ = release(&ctx, &actor, &key).await;
        }
    } else if let Err(error) = release(&ctx, &actor, &key).await {
        tracing::error!(%error, "idempotency: could not release the key");
    }
    Response::from_parts(response_parts, Body::from(buffered))
}
