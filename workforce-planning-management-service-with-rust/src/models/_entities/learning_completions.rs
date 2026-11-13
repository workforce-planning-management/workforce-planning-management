//! Row type — `learning_completions`. A completion HR recorded for a requirement: a date and where it was done, no certificate (WPM-R170).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub requirement_pid: Uuid,
    pub worker_pid: Uuid,
    pub completed_on: NaiveDate,
    pub source: Option<String>,
    pub source_organization: Option<String>,
    pub recorded_by: Option<String>,
}
