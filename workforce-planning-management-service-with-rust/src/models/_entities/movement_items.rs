//! Row type — `movement_items`. One dated item of a joiner or leaver checklist.

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
    pub movement_pid: Uuid,
    pub position: i32,
    pub title: String,
    pub category: String,
    pub due_on: SqlDate,
    pub assignee_pid: Option<Uuid>,
    pub done_on: Option<SqlDate>,
    pub done_by: Option<String>,
    pub skipped_reason: Option<String>,
}
