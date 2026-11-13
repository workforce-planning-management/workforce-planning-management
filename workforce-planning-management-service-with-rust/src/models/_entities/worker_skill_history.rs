//! Row type — `worker_skill_history`. One interval a worker held a skill at a level.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub skill_pid: Uuid,
    pub proficiency: i32,
    pub started_at: DateTime<FixedOffset>,
    pub ended_at: Option<DateTime<FixedOffset>>,
    pub source: String,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
}
