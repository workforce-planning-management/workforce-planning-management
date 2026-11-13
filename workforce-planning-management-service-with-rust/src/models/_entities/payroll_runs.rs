//! Row type — `payroll_runs`. One payroll run per organization and period (WPM-R13, WPM-D5).

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
    pub organization_ref: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub status: String,
    pub approved_by: Option<String>,
    pub notes: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
