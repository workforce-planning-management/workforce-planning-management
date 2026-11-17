//! Row type — `engagement_reminders`. That the manager and HR were told about one end date, once (WPM-R81).

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
    pub ends_on: SqlDate,
    pub kind: String,
    pub reminded_on: SqlDate,
}
