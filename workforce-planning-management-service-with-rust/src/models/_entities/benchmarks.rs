//! Row type — `benchmarks`. One recorded market pay band for a job title (WPM-R14).

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
    pub job_title: String,
    pub currency: String,
    pub min_minor: i64,
    pub median_minor: i64,
    pub max_minor: i64,
    pub source: String,
    pub as_of: NaiveDate,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
