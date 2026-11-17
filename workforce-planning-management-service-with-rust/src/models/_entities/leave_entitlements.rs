//! Row type — `leave_entitlements`. One worker's leave entitlement per kind and year, whole days (WPM-R5).

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub kind: String,
    pub year: i32,
    pub entitled_days: i32,
    pub used_days: i32,
    pub deleted_at: Option<SqlTimestamp>,
}
