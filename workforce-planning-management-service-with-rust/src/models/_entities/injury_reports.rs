//! Row type — `injury_reports`. One report of an incident to one recipient: an owed report until it says when and how it went (WPM-R221, WPM-D179).

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
    pub record_pid: Uuid,
    pub recipient: String,
    pub due_at: Option<SqlTimestamp>,
    pub sent_at: Option<SqlTimestamp>,
    pub method: Option<String>,
    pub reference: Option<String>,
    pub recorded_by: Option<String>,
}
