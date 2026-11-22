//! Row type — `policy_acknowledgements`. One worker saying they have read one version, on a date (WPM-R156).

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
    pub version_pid: Uuid,
    pub worker_pid: Uuid,
    pub acknowledged_on: SqlDate,
}
