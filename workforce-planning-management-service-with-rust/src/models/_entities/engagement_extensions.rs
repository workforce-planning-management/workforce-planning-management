//! Row type — `engagement_extensions`. One dated extension of an engagement: the previous end, the new end and why (WPM-R79).

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
    pub previous_end: NaiveDate,
    pub new_end: NaiveDate,
    pub reason: Option<String>,
    pub decided_by: Option<String>,
}
