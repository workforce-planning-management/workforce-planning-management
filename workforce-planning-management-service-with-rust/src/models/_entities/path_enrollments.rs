//! Row type — `path_enrollments`. Learning & development (declared/recorded data).

#![allow(missing_docs)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub path_pid: Uuid,
    pub worker_pid: Uuid,
    pub enrolled_on: SqlDate,
    pub deleted_at: Option<SqlTimestamp>,
}
