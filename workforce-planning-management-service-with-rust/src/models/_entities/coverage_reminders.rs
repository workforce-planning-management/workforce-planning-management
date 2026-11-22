//! Row type — `coverage_reminders`. A coverage-expiry reminder already sent (WPM-R220, WPM-T420), keyed by record, end date and kind.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub coverage_pid: Uuid,
    pub valid_to: SqlDate,
    pub kind: String,
    pub reminded_on: SqlDate,
}
