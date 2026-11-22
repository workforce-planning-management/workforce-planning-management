//! Row type — `engagement_extensions`. One dated extension of an engagement: the previous end, the new end and why (WPM-R79).

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
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub previous_end: SqlDate,
    pub new_end: SqlDate,
    pub reason: Option<String>,
    pub decided_by: Option<String>,
}
