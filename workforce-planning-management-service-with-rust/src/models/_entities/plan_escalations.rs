//! Row type — `plan_escalations`. A breach of a plan's tolerance taken to someone who can decide.

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
    pub plan_pid: Uuid,
    pub dimension: String,
    pub level: i32,
    pub status: String,
    pub raised_by: Option<String>,
    pub raised_on: SqlDate,
    pub level_raised_on: Option<SqlDate>,
    pub note: Option<String>,
    pub rag_at_raise: String,
    pub deviation_at_raise: Option<i64>,
    pub limit_at_raise: i64,
    pub decision: Option<String>,
    pub decided_by: Option<String>,
    pub decided_on: Option<SqlDate>,
    pub decision_note: Option<String>,
    pub rag_at_decision: Option<String>,
    pub deviation_at_decision: Option<i64>,
}
