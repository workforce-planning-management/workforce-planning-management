//! Row type — `movements`. Someone joining or leaving, on an effective day.

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
    pub kind: String,
    pub effective_on: SqlDate,
    pub reason: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<String>,
    pub completed_at: Option<SqlTimestamp>,
}
