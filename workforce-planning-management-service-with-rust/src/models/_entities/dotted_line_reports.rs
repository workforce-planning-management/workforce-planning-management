//! Row type — `dotted_line_reports`. A secondary (dotted-line) reporting relationship over an interval.

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub report_pid: Uuid,
    pub manager_pid: Uuid,
    pub note: Option<String>,
    pub started_at: SqlTimestamp,
    pub ended_at: Option<SqlTimestamp>,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
}
