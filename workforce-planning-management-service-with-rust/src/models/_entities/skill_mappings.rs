//! Row type — `skill_mappings`. A skill declared under one record counts as another, with its confidence and basis (WPM-R158).

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
    pub from_skill_pid: Uuid,
    pub to_skill_pid: Uuid,
    pub confidence: String,
    pub basis: String,
    pub source: String,
    pub recorded_by: Option<String>,
    pub recorded_on: SqlDate,
}
