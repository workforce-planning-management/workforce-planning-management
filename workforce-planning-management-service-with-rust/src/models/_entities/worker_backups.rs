//! Row type — `worker_backups`. A colleague who covers for a worker when they are out, ranked, optionally for a dated window.

#![allow(missing_docs)]

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
    pub backup_pid: Uuid,
    pub priority: i32,
    pub starts_on: Option<SqlDate>,
    pub ends_on: Option<SqlDate>,
    pub note: Option<String>,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
    pub deleted_at: Option<SqlTimestamp>,
}
