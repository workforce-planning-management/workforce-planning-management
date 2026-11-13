//! Row type — `goal_check_ins`. A dated entry on a goal: how it stands and a short note (WPM-R224).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub goal_pid: Uuid,
    pub worker_pid: Uuid,
    pub checked_on: NaiveDate,
    pub status: String,
    pub note: Option<String>,
    pub recorded_by: Option<String>,
}
