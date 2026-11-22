//! Row type — `succession_candidates`. One ranked successor on a plan (WPM-R12).

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
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
    pub readiness: String,
    pub rank: i32,
    pub deleted_at: Option<SqlTimestamp>,
}
