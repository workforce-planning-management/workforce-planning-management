//! Row type — `time_entries`. One day's recorded time in whole minutes (WPM-R4).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub worked_on: NaiveDate,
    pub minutes: i32,
    pub kind: String,
    pub status: String,
    pub notes: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
