//! Row type — `expense_claims`. A worker's request to be repaid for money they spent (WPM-R55).

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
    pub title: String,
    pub description: Option<String>,
    pub currency: String,
    pub status: String,
    pub submitted_at: Option<SqlTimestamp>,
    pub decided_by: Option<String>,
    pub decided_at: Option<SqlTimestamp>,
    pub decision_note: Option<String>,
    pub reimbursed_on: Option<SqlDate>,
    pub reimbursed_by: Option<String>,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
    pub deleted_at: Option<SqlTimestamp>,
}
