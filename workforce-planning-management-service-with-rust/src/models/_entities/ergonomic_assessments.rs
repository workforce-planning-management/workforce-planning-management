//! Row type — `ergonomic_assessments`. One DSE workstation assessment (WPM-R32): the workstation, never the body (WPM-D24).

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
    pub workstation: String,
    pub status: String,
    pub assessed_on: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
}
