//! Row type — `shift_assignments`. One worker assigned to one shift; unassign = soft delete (WPM-R6).

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
    pub shift_pid: Uuid,
    pub worker_pid: Uuid,
    pub deleted_at: Option<SqlTimestamp>,
}
