//! Row type — `training_enrollments`. One enrolment against the family course registry (WPM-R11, WPM-D10).

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
    pub course_ref: String,
    pub status: String,
    pub completed_on: Option<SqlDate>,
    pub certificate_expires_on: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
}
