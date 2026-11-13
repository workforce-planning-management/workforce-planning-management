//! Row type — `cpd_requirements`. Hours or points required in a period.

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
    pub name: String,
    pub unit: String,
    pub required_hundredths: i64,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub job_title: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
