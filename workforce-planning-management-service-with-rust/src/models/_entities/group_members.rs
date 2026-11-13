//! Row type — `group_members`. One person in one group over an interval.

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
    pub group_pid: Uuid,
    pub worker_pid: Uuid,
    pub role: String,
    pub joined_at: DateTime<FixedOffset>,
    pub left_at: Option<DateTime<FixedOffset>>,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
}
