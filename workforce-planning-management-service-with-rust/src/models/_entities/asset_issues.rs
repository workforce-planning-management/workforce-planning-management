//! Row type — `asset_issues`. Who holds an asset, from when to when (WPM-R155). Closed by a return or a write-off.

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
    pub asset_pid: Uuid,
    pub worker_pid: Uuid,
    pub issued_on: SqlDate,
    pub returned_on: Option<SqlDate>,
    pub outcome: Option<String>,
    pub note: Option<String>,
    pub recorded_by: Option<String>,
}
