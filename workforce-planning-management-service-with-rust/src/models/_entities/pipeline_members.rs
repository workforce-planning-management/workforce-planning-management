//! Row type — `pipeline_members`. One candidate or worker in a talent pipeline (WPM-R22).

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
    pub pipeline_pid: Uuid,
    pub subject_kind: String,
    pub subject_pid: Uuid,
    pub stage: String,
    pub readiness: Option<String>,
    pub added_on: SqlDate,
    pub deleted_at: Option<SqlTimestamp>,
}
