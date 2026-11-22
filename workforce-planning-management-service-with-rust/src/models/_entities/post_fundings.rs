//! Row type — `post_fundings`. Where a worker's post is funded from and when that ends: a kind and a date, never an amount (WPM-R62, WPM-R98).

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
    pub worker_pid: Uuid,
    pub kind: String,
    pub programme_ref: Option<String>,
    pub ends_on: Option<SqlDate>,
    pub recorded_by: Option<String>,
}
