//! Row type — `development_plan_items`. One skill step of a development plan (WPM-R21).

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
    pub plan_pid: Uuid,
    pub skill_pid: Uuid,
    pub current_level: i32,
    pub target_level: i32,
    pub method: String,
    pub course_ref: Option<String>,
    pub due_on: Option<SqlDate>,
    pub status: String,
}
