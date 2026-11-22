//! Row type — `goal_check_ins`. A dated entry on a goal: how it stands and a short note (WPM-R224).

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
    pub goal_pid: Uuid,
    pub worker_pid: Uuid,
    pub checked_on: SqlDate,
    pub status: String,
    pub note: Option<String>,
    pub recorded_by: Option<String>,
}
