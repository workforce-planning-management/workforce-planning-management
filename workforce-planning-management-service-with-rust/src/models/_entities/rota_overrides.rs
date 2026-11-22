//! Row type — `rota_overrides`. A swap: a worker on call for a date window regardless of the rotation.

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
    pub id: i32,
    pub pid: Uuid,
    pub rota_pid: Uuid,
    pub worker_pid: Uuid,
    pub starts_on: SqlDate,
    pub ends_on: SqlDate,
    pub note: Option<String>,
    pub created_by: Option<String>,
}
