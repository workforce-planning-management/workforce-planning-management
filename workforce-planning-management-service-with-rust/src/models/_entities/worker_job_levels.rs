//! Row type — `worker_job_levels`. A worker's current job level on a reference ladder (WPM-R53).

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
    pub worker_pid: Uuid,
    pub framework: String,
    pub level_number: i32,
    pub effective_on: SqlDate,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
}
