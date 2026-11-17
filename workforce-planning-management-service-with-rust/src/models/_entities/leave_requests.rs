//! Row type — `leave_requests`. One leave request with its decision trail (WPM-R5).

#![allow(missing_docs)]

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
    pub kind: String,
    pub start_on: SqlDate,
    pub end_on: SqlDate,
    pub days: i32,
    pub status: String,
    pub negative_balance: bool,
    pub reason: Option<String>,
    pub decided_by: Option<String>,
    pub decided_at: Option<SqlTimestamp>,
    pub deleted_at: Option<SqlTimestamp>,
}
