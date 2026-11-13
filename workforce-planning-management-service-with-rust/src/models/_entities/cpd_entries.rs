//! Row type — `cpd_entries`. One CPD activity a worker recorded.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub entry_date: NaiveDate,
    pub activity: String,
    pub category: String,
    pub unit: String,
    pub amount_hundredths: i64,
    pub evidence_note: Option<String>,
    pub evidence_url: Option<String>,
    pub source: String,
    pub external_ref: Option<String>,
    pub course_ref: Option<String>,
    pub verified_on: Option<NaiveDate>,
    pub verified_by: Option<String>,
    pub skill_pid: Option<Uuid>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
