//! Row type — `dotted_line_reports`. A secondary (dotted-line) reporting relationship over an interval.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub report_pid: Uuid,
    pub manager_pid: Uuid,
    pub note: Option<String>,
    pub started_at: DateTime<FixedOffset>,
    pub ended_at: Option<DateTime<FixedOffset>>,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
}
