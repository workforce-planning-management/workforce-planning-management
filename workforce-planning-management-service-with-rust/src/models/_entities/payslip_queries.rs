//! Row type — `payslip_queries`. A person's query on one of their payslips, seen and closed by payroll (WPM-R178).

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
    pub payslip_pid: Uuid,
    pub worker_pid: Uuid,
    pub comment: String,
    pub raised_on: SqlDate,
    pub closed_on: Option<SqlDate>,
    pub closed_by: Option<String>,
    pub deleted_at: Option<SqlTimestamp>,
}
