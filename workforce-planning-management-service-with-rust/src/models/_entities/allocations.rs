//! Row type — `allocations`. A share of one of a worker's assignments planned for a programme, product or service over a run of
//! months (WPM-R275): what a person is planned to do, never effort or performance (WPM-D211).

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
    pub assignment_pid: Uuid,
    pub organization_ref: String,
    pub source_ref: String,
    pub pool_pid: Option<Uuid>,
    pub from_month: SqlDate,
    pub to_month: SqlDate,
    pub fte_centi: i32,
    pub status: String,
    pub note: Option<String>,
    pub recorded_by: Option<String>,
    pub deleted_at: Option<SqlTimestamp>,
}
