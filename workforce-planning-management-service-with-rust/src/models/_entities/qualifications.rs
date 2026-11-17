//! Row type — `qualifications`. A qualification a person holds: declared, or verified by HR with an evidence reference (WPM-R171).

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
    pub worker_pid: Uuid,
    pub kind: String,
    pub title: String,
    pub awarding_body: Option<String>,
    pub framework: Option<String>,
    pub level: Option<i32>,
    pub awarded_on: Option<SqlDate>,
    pub expires_on: Option<SqlDate>,
    pub verified_by: Option<String>,
    pub verified_on: Option<SqlDate>,
    pub evidence_ref: Option<String>,
    pub recorded_by: Option<String>,
}
