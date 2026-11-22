//! Row type — `time_entries`. One day's recorded time in whole minutes (WPM-R4).

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub worked_on: SqlDate,
    pub minutes: i32,
    pub kind: String,
    pub status: String,
    pub notes: Option<String>,
    pub deleted_at: Option<SqlTimestamp>,
}
