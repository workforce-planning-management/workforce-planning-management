//! Row type — `flexible_working_requests`. A worker's request for a different working arrangement and its decision (WPM-R126).

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
    pub kind: String,
    pub requested_on: SqlDate,
    pub proposed_start: SqlDate,
    pub proposed_fte_percent: Option<i32>,
    pub effect_on_team: Option<String>,
    pub how_to_manage: Option<String>,
    pub status: String,
    pub decide_by: SqlDate,
    pub decided_on: Option<SqlDate>,
    pub decided_by: Option<String>,
    pub decision_reason: Option<String>,
    pub decision_note: Option<String>,
    pub counter_note: Option<String>,
    pub counter_fte_percent: Option<i32>,
    pub trial_until: Option<SqlDate>,
    pub appealed_on: Option<SqlDate>,
    pub appeal_note: Option<String>,
    pub appeal_decided_by: Option<String>,
}
