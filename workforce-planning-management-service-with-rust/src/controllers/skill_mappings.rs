//! The skills crosswalk (WPM-R158, WPM-D109): statements that a skill declared under one record counts as another, each with its
//! confidence and the basis it rests on. Reference data: any signed-in caller reads; HR writes. Gap analysis reads them
//! (`controllers::skill_gaps`). Spec: `spec/skills-and-training.md`.

use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::{ensure_valid, row_json, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{skill_mappings, skills};
use crate::models::audit_logs::Model as Audit;
use crate::rules::crosswalk as rules;

/// `GET /api/skill-mappings` — every mapping, with the names of its two skills.
#[debug_handler]
async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let names: std::collections::HashMap<Uuid, String> = skills::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|s| (s.pid, s.name))
        .collect();
    let rows = skill_mappings::Entity::find()
        .order_by_asc(skill_mappings::Column::Id)
        .all(&ctx.db)
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
    Ok(skills::Entity::find()
        .filter(skills::Column::Pid.eq(pid))
        .filter(skills::Column::DeletedAt.is_null())
        .count(&ctx.db)
        .await?
        > 0)
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
    let clash = skill_mappings::Entity::find()
        .filter(skill_mappings::Column::FromSkillPid.eq(payload.from_skill_pid))
        .filter(skill_mappings::Column::ToSkillPid.eq(payload.to_skill_pid))
        .count(&ctx.db)
        .await?;
    if clash > 0 {
        return Err(unprocessable(
            "this pair is already mapped; remove the mapping to state it again",
        ));
    }
    let pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    skill_mappings::ActiveModel {
        pid: ActiveValue::set(pid),
        from_skill_pid: ActiveValue::set(payload.from_skill_pid),
        to_skill_pid: ActiveValue::set(payload.to_skill_pid),
        confidence: ActiveValue::set(payload.confidence.clone()),
        basis: ActiveValue::set(payload.basis),
        source: ActiveValue::set(payload.source.trim().to_string()),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        recorded_on: ActiveValue::set(Utc::now().date_naive()),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "skill_mapping",
        pid,
        "skill_mapping_stated",
        caller.actor(),
        Some(json!({ "confidence": payload.confidence })),
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// `DELETE /api/skill-mappings/{pid}` — withdraw a mapping; gap analysis stops using it at once.
#[debug_handler]
async fn remove(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let row = skill_mappings::Entity::find()
        .filter(skill_mappings::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let txn = ctx.db.begin().await?;
    skill_mappings::Entity::delete_by_id(row.id)
        .exec(&txn)
        .await?;
    Audit::record(
        &txn,
        "skill_mapping",
        pid,
        "skill_mapping_withdrawn",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
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
