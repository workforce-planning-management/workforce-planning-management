//! Row type — `learning_completions`. A completion HR recorded for a requirement: a date and where it was done, no certificate (WPM-R170).

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
    pub requirement_pid: Uuid,
    pub worker_pid: Uuid,
    pub completed_on: SqlDate,
    pub source: Option<String>,
    pub source_organization: Option<String>,
    pub recorded_by: Option<String>,
}
