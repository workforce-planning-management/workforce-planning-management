//! Row type — `worker_health_records`. A worker's status against one requirement: a status token and two dates, never a reason (WPM-R128, WPM-D75). Health data.

#![allow(missing_docs)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub worker_pid: Uuid,
    pub requirement_pid: Uuid,
    pub status: String,
    pub recorded_on: SqlDate,
    pub next_due: Option<SqlDate>,
    pub recorded_by: Option<String>,
}
