//! Row type — `payroll_runs`. One payroll run per organization and period (WPM-R13, WPM-D5).

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
    pub organization_ref: String,
    pub period_start: SqlDate,
    pub period_end: SqlDate,
    pub status: String,
    pub approved_by: Option<String>,
    pub notes: Option<String>,
    pub deleted_at: Option<SqlTimestamp>,
}
