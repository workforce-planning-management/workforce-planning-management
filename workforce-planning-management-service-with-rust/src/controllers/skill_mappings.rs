//! The skills crosswalk (WPM-R158, WPM-D109): statements that a skill declared under one record counts as another, each with its
//! confidence and the basis it rests on. Reference data: any signed-in caller reads; HR writes. Gap analysis reads them
//! (`controllers::skill_gaps`). Spec: `spec/skills-and-training.md`.

use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::{ensure_valid, row_json, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db::{self, pool};
use crate::models::_entities::skill_mappings;
use crate::rules::crosswalk as rules;

/// `GET /api/skill-mappings` — every mapping, with the names of its two skills.
#[debug_handler]
async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let names: std::collections::HashMap<Uuid, String> =
        sqlx::query!("SELECT pid, name FROM skills")
            .fetch_all(pool(&ctx))
            .await?
            .into_iter()
            .map(|s| (s.pid, s.name))
            .collect();
    let rows = sqlx::query_as!(
        skill_mappings::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, from_skill_pid, to_skill_pid,
                  confidence, basis, source, recorded_by, recorded_on
           FROM skill_mappings ORDER BY id"#
    )
    .fetch_all(pool(&ctx))
    .await?;
    let mut out = Vec::new();
    for row in &rows {
        let mut v = row_json(row)?;
        if let Some(o) = v.as_object_mut() {
            o.insert("from_skill".into(), json!(names.get(&row.from_skill_pid)));
            o.insert("to_skill".into(), json!(names.get(&row.to_skill_pid)));
            o.insert("counts".into(), json!(rules::counts(&row.confidence)));
        }
        out.push(v);
    }
    format::json(json!({
        "note": "A mapping runs from the skill a person declares to the skill that is needed. Only exact and close count in gap analysis.",
        "mappings": out,
    }))
}

#[derive(Debug, Deserialize)]
struct Payload {
    from_skill_pid: Uuid,
    to_skill_pid: Uuid,
    confidence: String,
    basis: String,
    source: String,
}

async fn skill_exists(ctx: &AppContext, pid: Uuid) -> Result<bool> {
    Ok(sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM skills WHERE pid = $1 AND deleted_at IS NULL) AS "found!""#,
        pid,
    )
    .fetch_one(pool(ctx))
    .await?)
}

/// `POST /api/skill-mappings` — state a mapping. The pair is unique; both skills must exist.
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<Payload>,
) -> Result<Response> {
    ensure_valid(&rules::problems(
        payload.from_skill_pid,
        payload.to_skill_pid,
        &payload.confidence,
        &payload.basis,
        &payload.source,
    ))?;
    if !skill_exists(&ctx, payload.from_skill_pid).await?
        || !skill_exists(&ctx, payload.to_skill_pid).await?
    {
        return Err(unprocessable("both skills must exist"));
    }
    let clash = sqlx::query_scalar!(
        r#"SELECT EXISTS (
               SELECT 1 FROM skill_mappings WHERE from_skill_pid = $1 AND to_skill_pid = $2
           ) AS "clash!""#,
        payload.from_skill_pid,
        payload.to_skill_pid,
    )
    .fetch_one(pool(&ctx))
    .await?;
    if clash {
        return Err(unprocessable(
            "this pair is already mapped; remove the mapping to state it again",
        ));
    }
    let pid = Uuid::new_v4();
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO skill_mappings (pid, from_skill_pid, to_skill_pid, confidence, basis, source, recorded_by, recorded_on)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        pid,
        payload.from_skill_pid,
        payload.to_skill_pid,
        payload.confidence,
        payload.basis,
        payload.source.trim(),
        caller.actor(),
        Utc::now().date_naive(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "skill_mapping",
        pid,
        "skill_mapping_stated",
        caller.actor(),
        Some(json!({ "confidence": payload.confidence })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// `DELETE /api/skill-mappings/{pid}` — withdraw a mapping; gap analysis stops using it at once.
#[debug_handler]
async fn remove(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let mut tx = pool(&ctx).begin().await?;
    let removed = sqlx::query!("DELETE FROM skill_mappings WHERE pid = $1", pid)
        .execute(&mut *tx)
        .await?;
    if removed.rows_affected() == 0 {
        return Err(Error::NotFound);
    }
    db::audit(
        &mut tx,
        "skill_mapping",
        pid,
        "skill_mapping_withdrawn",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/skill-mappings", get(list))
        .add("/skill-mappings", post(create))
        .add("/skill-mappings/{pid}", delete(remove))
}
