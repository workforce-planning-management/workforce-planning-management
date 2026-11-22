//! Row type — `resignations`. A worker's logged intent to resign and its outcome (WPM-R127).

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
    pub logged_on: SqlDate,
    pub proposed_last_day: SqlDate,
    pub reason: Option<String>,
    pub status: String,
    pub agreed_last_day: Option<SqlDate>,
    pub movement_pid: Option<Uuid>,
    pub decided_by: Option<String>,
    pub decided_at: Option<SqlTimestamp>,
}
