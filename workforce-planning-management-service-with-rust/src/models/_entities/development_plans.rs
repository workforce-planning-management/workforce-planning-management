//! Row type — `development_plans`. One worker's upskilling or reskilling plan (WPM-R21).

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
    pub kind: String,
    pub target_job_title: Option<String>,
    pub target_department: Option<String>,
    pub rationale: Option<String>,
    pub status: String,
    pub started_on: Option<SqlDate>,
    pub target_on: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
}
