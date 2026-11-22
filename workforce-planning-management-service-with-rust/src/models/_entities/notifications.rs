//! Row type — `notifications`. One in-app notification (WPM-R31): reference-only (WPM-D23 — a kind, a neutral body, pids/names; never scores, comments, or masked-tier values), owned by one worker.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub kind: String,
    pub body: String,
    pub data: Value,
    pub read_at: Option<SqlTimestamp>,
}
