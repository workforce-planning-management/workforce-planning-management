//! Row type — `worker_job_levels`. A worker's current job level on a reference ladder (WPM-R53).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub worker_pid: Uuid,
    pub framework: String,
    pub level_number: i32,
    pub effective_on: NaiveDate,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
}
