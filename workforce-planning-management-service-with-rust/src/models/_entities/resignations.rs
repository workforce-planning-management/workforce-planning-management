//! Row type — `resignations`. A worker's logged intent to resign and its outcome (WPM-R127).

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
    pub worker_pid: Uuid,
    pub logged_on: NaiveDate,
    pub proposed_last_day: NaiveDate,
    pub reason: Option<String>,
    pub status: String,
    pub agreed_last_day: Option<NaiveDate>,
    pub movement_pid: Option<Uuid>,
    pub decided_by: Option<String>,
    pub decided_at: Option<DateTime<FixedOffset>>,
}
