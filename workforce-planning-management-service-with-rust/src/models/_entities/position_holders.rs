//! Row type — `position_holders`. One tenure of a position's holder (WPM-R290, WPM-T649): who, from when, until when. The worker is
//! cut (NULL) by an erasure and the dates stay.

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
    pub id: i32,
    pub position_pid: Uuid,
    pub organization_ref: String,
    pub worker_pid: Option<Uuid>,
    pub started_on: SqlDate,
    pub start_known: bool,
    pub ended_on: Option<SqlDate>,
    pub recorded_by: Option<String>,
}
