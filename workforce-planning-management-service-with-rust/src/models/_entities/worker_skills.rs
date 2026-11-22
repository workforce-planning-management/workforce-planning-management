//! Row type — `worker_skills`. Learning & development (declared/recorded data).

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub skill_pid: Uuid,
    pub proficiency: i32,
    pub target: Option<i32>,
    pub assessed_on: SqlDate,
    pub source_framework: Option<String>,
    pub source_level: Option<i32>,
    pub source_scale_max: Option<i32>,
    pub deleted_at: Option<SqlTimestamp>,
}
