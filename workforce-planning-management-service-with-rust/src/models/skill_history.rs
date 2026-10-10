//! The **skill history** recorder: every change to a worker's declared level
//! in a skill closes the interval they held the old level and opens a new one,
//! so past skills and levels can be read back. One helper, called from each
//! place that declares or clears a skill, so the history cannot drift from
//! the declarations.

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, ConnectionTrait};
use uuid::Uuid;

use super::_entities::worker_skill_history;

/// Record that, as of now, `worker_pid` holds `skill_pid` at `level`
/// (`None` = no longer declared). A no-op when nothing changed.
///
/// `source` says how it was recorded (`declared`, `framework`); `on_behalf` is
/// whether someone other than the person made the change.
///
/// # Errors
/// A database error.
pub async fn record<C: ConnectionTrait>(
    db: &C,
    worker_pid: Uuid,
    skill_pid: Uuid,
    level: Option<i32>,
    source: &str,
    actor: Option<&str>,
    on_behalf: bool,
) -> Result<()> {
    let open = worker_skill_history::Entity::find()
        .filter(worker_skill_history::Column::WorkerPid.eq(worker_pid))
        .filter(worker_skill_history::Column::SkillPid.eq(skill_pid))
        .filter(worker_skill_history::Column::EndedAt.is_null())
        .one(db)
        .await?;
    let now: sea_orm::prelude::DateTimeWithTimeZone = chrono::Utc::now().into();
    if let (Some(open), Some(level)) = (&open, level)
        && open.proficiency == level
    {
        return Ok(());
    }
    if let Some(open) = open {
        let mut active: worker_skill_history::ActiveModel = open.into();
        active.ended_at = ActiveValue::set(Some(now));
        active.update(db).await?;
    }
    if let Some(level) = level {
        worker_skill_history::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            worker_pid: ActiveValue::set(worker_pid),
            skill_pid: ActiveValue::set(skill_pid),
            proficiency: ActiveValue::set(level),
            started_at: ActiveValue::set(now),
            ended_at: ActiveValue::set(None),
            source: ActiveValue::set(source.to_string()),
            recorded_by: ActiveValue::set(actor.map(ToString::to_string)),
            on_behalf: ActiveValue::set(on_behalf),
            ..Default::default()
        }
        .insert(db)
        .await?;
    }
    Ok(())
}
