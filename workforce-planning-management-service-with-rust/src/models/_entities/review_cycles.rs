//! Row type — `review_cycles`. One performance review cycle (WPM-R10).

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
    pub name: String,
    pub period_start: SqlDate,
    pub period_end: SqlDate,
    pub status: String,
    pub deleted_at: Option<SqlTimestamp>,
}
