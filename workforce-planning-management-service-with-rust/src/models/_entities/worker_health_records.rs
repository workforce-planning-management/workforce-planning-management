//! Row type — `worker_health_records`. A worker's status against one requirement: a status token and two dates, never a reason (WPM-R128, WPM-D75). Health data.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub worker_pid: Uuid,
    pub requirement_pid: Uuid,
    pub status: String,
    pub recorded_on: NaiveDate,
    pub next_due: Option<NaiveDate>,
    pub recorded_by: Option<String>,
}
