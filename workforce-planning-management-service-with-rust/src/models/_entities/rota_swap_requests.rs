//! Row type — `rota_swap_requests`. A request that a colleague take the requester's on-call days in a window.

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
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub rota_pid: Uuid,
    pub requester_pid: Uuid,
    pub taker_pid: Uuid,
    pub starts_on: SqlDate,
    pub ends_on: SqlDate,
    pub note: Option<String>,
    pub status: String,
    pub decided_at: Option<SqlTimestamp>,
    pub created_by: Option<String>,
}
