//! Row type — `conversion_plans`. A recorded intent for a fixed-term or contractor engagement: convert, extend, end or undecided (WPM-R98, WPM-D66). No pay figure.

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
    pub intent: String,
    pub target_on: SqlDate,
    pub department: Option<String>,
    pub role_profile_ref: Option<String>,
    pub post_funding_kind: Option<String>,
    pub post_funding_ends_on: Option<SqlDate>,
    pub reason: Option<String>,
    pub proposed_by: Option<String>,
    pub approved_by: Option<String>,
    pub status: String,
    pub review_on: Option<SqlDate>,
    pub settled_on: Option<SqlDate>,
}
