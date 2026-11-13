//! Row type — `engagement_reminders`. That the manager and HR were told about one end date, once (WPM-R81).

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
    pub ends_on: NaiveDate,
    pub kind: String,
    pub reminded_on: NaiveDate,
}
