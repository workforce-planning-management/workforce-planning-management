//! Row type — `payslips`. One worker's payslip in one run: gross, deduction lines (JSONB), net (WPM-R13).

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub run_pid: Uuid,
    pub worker_pid: Uuid,
    pub currency: String,
    pub gross_minor: i64,
    pub deductions: Value,
    pub net_minor: i64,
    pub statutory_overtime_minor: i64,
    pub employer_cost_minor: i64,
    pub employer_costs: Value,
    pub deleted_at: Option<SqlTimestamp>,
}
