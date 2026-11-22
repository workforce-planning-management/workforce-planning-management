//! Row type — `equality_declarations`. A worker's own voluntary answer to one monitored category (WPM-R125). Special-category data: readable by the worker only.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub worker_pid: Uuid,
    pub category: String,
    pub value: String,
}
