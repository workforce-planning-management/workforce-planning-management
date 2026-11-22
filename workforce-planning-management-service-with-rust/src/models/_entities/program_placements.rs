//! Row type — `program_placements`. One person's placement on an early-career programme (WPM-R23).

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
    pub program_pid: Uuid,
    pub worker_pid: Uuid,
    pub supervisor_pid: Option<Uuid>,
    pub started_on: SqlDate,
    pub ends_on: Option<SqlDate>,
    pub status: String,
    pub off_the_job_hours: i32,
    pub outcome: String,
    pub deleted_at: Option<SqlTimestamp>,
}
