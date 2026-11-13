//! Row type — `program_placements`. One person's placement on an early-career programme (WPM-R23).

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
    pub program_pid: Uuid,
    pub worker_pid: Uuid,
    pub supervisor_pid: Option<Uuid>,
    pub started_on: NaiveDate,
    pub ends_on: Option<NaiveDate>,
    pub status: String,
    pub off_the_job_hours: i32,
    pub outcome: String,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
