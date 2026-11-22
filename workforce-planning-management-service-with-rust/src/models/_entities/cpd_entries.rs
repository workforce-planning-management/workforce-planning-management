//! Row type — `cpd_entries`. One CPD activity a worker recorded.

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
    pub entry_date: SqlDate,
    pub activity: String,
    pub category: String,
    pub unit: String,
    pub amount_hundredths: i64,
    pub evidence_note: Option<String>,
    pub evidence_url: Option<String>,
    pub source: String,
    pub external_ref: Option<String>,
    pub course_ref: Option<String>,
    pub verified_on: Option<SqlDate>,
    pub verified_by: Option<String>,
    pub skill_pid: Option<Uuid>,
    pub deleted_at: Option<SqlTimestamp>,
}
