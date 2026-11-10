//! Single-use, hashed link tokens (see `rules::link_token`). A token is made here, handed out once, and consumed here.
//!
//! [`issue`] returns the token **once**; only its hash is stored. [`consume`] marks it used in a single `UPDATE ... WHERE` that
//! also checks purpose, expiry and that it is unused, so two people presenting the same link at once cannot both succeed. A caller is
//! told only whether it worked; the reason for a refusal goes to the audit trail, and the token never does.

use chrono::{DateTime, Utc};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::db;
use crate::rules::link_token as rules;
use axa::prelude::*;

/// What a successful consume gives back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consumed {
    /// The record the link was made for, if it named one.
    pub subject_pid: Option<Uuid>,
}

/// 32 random bytes from the operating system's generator, by way of two version 4 UUIDs.
fn random_bytes() -> [u8; rules::TOKEN_BYTES] {
    let mut out = [0u8; rules::TOKEN_BYTES];
    out[..16].copy_from_slice(Uuid::new_v4().as_bytes());
    out[16..].copy_from_slice(Uuid::new_v4().as_bytes());
    out
}

/// Make a token for `purpose` that works once until `ttl_minutes` from `now`. The returned string is the only copy: put it in the
/// link and forget it.
///
/// # Errors
///
/// A refusal for a bad purpose or life, or a database error.
pub async fn issue(
    connection: &mut PgConnection,
    purpose: &str,
    subject_pid: Option<Uuid>,
    ttl_minutes: i64,
    now: DateTime<Utc>,
) -> Result<String> {
    let mut problems = rules::purpose_problems(purpose);
    let expires = match rules::expires_at(now, ttl_minutes) {
        Ok(at) => Some(at),
        Err(m) => {
            problems.push(m);
            None
        }
    };
    let Some(expires) = expires.filter(|_| problems.is_empty()) else {
        return Err(Error::string(&problems.join("; ")));
    };
    let token = rules::encode(random_bytes());
    sqlx::query!(
        "INSERT INTO link_tokens (pid, token_hash, purpose, subject_pid, expires_at) VALUES ($1, $2, $3, $4, $5)",
        Uuid::new_v4(),
        rules::hash(&token),
        purpose,
        subject_pid,
        expires,
    )
    .execute(connection)
    .await?;
    Ok(token)
}

/// Use a token. `Ok(Some(..))` only the first time, for its purpose, before it expires. Every other case is `Ok(None)`: the caller
/// cannot tell an unknown token from a used or expired one. The reason is written to the audit trail (never the token).
///
/// # Errors
///
/// A database error.
pub async fn consume(
    connection: &mut PgConnection,
    purpose: &str,
    token: &str,
    now: DateTime<Utc>,
) -> Result<Option<Consumed>> {
    if !rules::well_formed(token) {
        db::audit(
            connection,
            "link_token",
            Uuid::new_v4(),
            "link_token_refused",
            None,
            Some(serde_json::json!({ "reason": "malformed" })),
        )
        .await?;
        return Ok(None);
    }
    let hash = rules::hash(token);
    let used = sqlx::query!(
        "UPDATE link_tokens SET used_at = $3, updated_at = $3
          WHERE token_hash = $1 AND purpose = $2 AND used_at IS NULL AND expires_at > $3
          RETURNING pid, subject_pid",
        hash,
        purpose,
        now,
    )
    .fetch_optional(&mut *connection)
    .await?;
    if let Some(used) = used {
        db::audit(
            connection,
            "link_token",
            used.pid,
            "link_token_consumed",
            None,
            Some(serde_json::json!({ "purpose": purpose })),
        )
        .await?;
        return Ok(Some(Consumed {
            subject_pid: used.subject_pid,
        }));
    }
    let stored = sqlx::query!(
        "SELECT pid, purpose, expires_at, used_at FROM link_tokens WHERE token_hash = $1",
        hash,
    )
    .fetch_optional(&mut *connection)
    .await?;
    let reason = rules::check(
        stored
            .as_ref()
            .map(|r| rules::Stored {
                purpose: r.purpose.clone(),
                expires_at: r.expires_at,
                used_at: r.used_at,
            })
            .as_ref(),
        purpose,
        now,
    )
    .err()
    .map_or("unknown", rules::Refusal::token);
    db::audit(
        connection,
        "link_token",
        stored.map_or_else(Uuid::new_v4, |r| r.pid),
        "link_token_refused",
        None,
        Some(serde_json::json!({ "reason": reason, "purpose": purpose })),
    )
    .await?;
    Ok(None)
}

/// Remove tokens that expired before `before`, used or not. Returns how many were removed.
///
/// # Errors
///
/// A database error.
pub async fn purge_expired(connection: &mut PgConnection, before: DateTime<Utc>) -> Result<u64> {
    let done = sqlx::query!("DELETE FROM link_tokens WHERE expires_at < $1", before)
        .execute(connection)
        .await?;
    Ok(done.rows_affected())
}
