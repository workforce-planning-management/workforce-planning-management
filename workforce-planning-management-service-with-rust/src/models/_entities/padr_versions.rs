//! Row type — `padr_versions`. One version of a personal development review: its content, its two signatures and, once signed, the objectives as they stood (WPM-R172, WPM-D165).

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub review_pid: Uuid,
    pub worker_pid: Uuid,
    pub version: i32,
    pub period_start: SqlDate,
    pub period_end: SqlDate,
    pub values_summary: Option<String>,
    pub wellbeing_offered: Option<bool>,
    pub development_needs: Value,
    pub reviewer_signed_on: Option<SqlDate>,
    pub reviewee_signed_on: Option<SqlDate>,
    pub reviewee_comment: Option<String>,
    pub objectives: Option<Value>,
}
