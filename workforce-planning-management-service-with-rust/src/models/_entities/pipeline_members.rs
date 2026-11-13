//! Row type — `pipeline_members`. One candidate or worker in a talent pipeline (WPM-R22).

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
    pub pipeline_pid: Uuid,
    pub subject_kind: String,
    pub subject_pid: Uuid,
    pub stage: String,
    pub readiness: Option<String>,
    pub added_on: NaiveDate,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
