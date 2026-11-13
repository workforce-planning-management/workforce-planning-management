//! Row type — `coverage_reminders`. A coverage-expiry reminder already sent (WPM-R220, WPM-T420), keyed by record, end date and kind.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub coverage_pid: Uuid,
    pub valid_to: NaiveDate,
    pub kind: String,
    pub reminded_on: NaiveDate,
}
