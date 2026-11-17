//! Row type — `worker_goals`. One person's goal for a period: a record of intent and evidence, never a score (WPM-R224, WPM-D163).

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
    pub organization_ref: String,
    pub title: String,
    pub description: Option<String>,
    pub period_start: SqlDate,
    pub period_end: SqlDate,
    pub status: String,
    pub proposed_by: String,
    pub agreed_on: Option<SqlDate>,
    pub skill_pid: Option<Uuid>,
    pub skill_target_level: Option<i32>,
    pub plan_item_pid: Option<Uuid>,
    pub aligns_to: Option<String>,
    pub deleted_at: Option<SqlTimestamp>,
}
