//! Row type — `qualification_reminders`. That the person and their manager were told about one expiry date of one qualification, once (WPM-T435).

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
    pub qualification_pid: Uuid,
    pub expires_on: NaiveDate,
    pub kind: String,
    pub reminded_on: NaiveDate,
}
