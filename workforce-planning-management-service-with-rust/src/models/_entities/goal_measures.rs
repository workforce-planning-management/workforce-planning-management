//! Row type — `goal_measures`. A measure of a goal: a number with its start, target and current value, or a yes-or-no milestone. Every value is optional (WPM-R224).

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub goal_pid: Uuid,
    pub worker_pid: Uuid,
    pub name: String,
    pub kind: String,
    pub unit: Option<String>,
    pub start_value: Option<i64>,
    pub target_value: Option<i64>,
    pub current_value: Option<i64>,
    pub done: Option<bool>,
}
