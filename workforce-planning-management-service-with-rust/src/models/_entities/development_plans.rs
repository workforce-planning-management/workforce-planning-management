//! Row type — `development_plans`. One worker's upskilling or reskilling plan (WPM-R21).

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
    pub worker_pid: Uuid,
    pub kind: String,
    pub target_job_title: Option<String>,
    pub target_department: Option<String>,
    pub rationale: Option<String>,
    pub status: String,
    pub started_on: Option<NaiveDate>,
    pub target_on: Option<NaiveDate>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
