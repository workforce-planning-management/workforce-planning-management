//! Row type — `engagement_decisions`. A person's decision about an engagement that is ending: extend, convert or end (WPM-R81, WPM-D66).

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
    pub ends_on: NaiveDate,
    pub decision: String,
    pub decided_by: Option<String>,
    pub decided_on: NaiveDate,
}
