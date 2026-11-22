//! Row type — `interviews`. One scheduled interview on an application (WPM-R2).

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub application_pid: Uuid,
    pub scheduled_at: SqlTimestamp,
    pub interviewer_ref: String,
    pub outcome: String,
    pub notes: Option<String>,
    pub deleted_at: Option<SqlTimestamp>,
}
