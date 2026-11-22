//! Row type — `benefit_enrollments`. One worker's enrolment in one plan (WPM-R9).

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
    pub plan_pid: Uuid,
    pub worker_pid: Uuid,
    pub starts_on: SqlDate,
    pub ends_on: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
}
