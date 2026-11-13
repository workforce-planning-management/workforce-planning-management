//! Row type — `qualifications`. A qualification a person holds: declared, or verified by HR with an evidence reference (WPM-R171).

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
    pub title: String,
    pub awarding_body: Option<String>,
    pub framework: Option<String>,
    pub level: Option<i32>,
    pub awarded_on: Option<NaiveDate>,
    pub expires_on: Option<NaiveDate>,
    pub verified_by: Option<String>,
    pub verified_on: Option<NaiveDate>,
    pub evidence_ref: Option<String>,
    pub recorded_by: Option<String>,
}
