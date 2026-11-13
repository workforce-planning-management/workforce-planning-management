//! Row type — `leave_requests`. One leave request with its decision trail (WPM-R5).

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
    pub kind: String,
    pub start_on: NaiveDate,
    pub end_on: NaiveDate,
    pub days: i32,
    pub status: String,
    pub negative_balance: bool,
    pub reason: Option<String>,
    pub decided_by: Option<String>,
    pub decided_at: Option<DateTime<FixedOffset>>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
