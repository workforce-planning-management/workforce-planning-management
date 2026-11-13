//! Row type — `early_career_programs`. An apprenticeship, internship, or graduate scheme (WPM-R23).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub name: String,
    pub kind: String,
    pub level: Option<i32>,
    pub duration_months: i32,
    pub min_off_the_job_hours: Option<i32>,
    pub provider_ref: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
