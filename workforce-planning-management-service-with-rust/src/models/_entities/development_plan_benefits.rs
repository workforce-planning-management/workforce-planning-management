//! Row type — `development_plan_benefits`. A stated yearly benefit of a development plan with its named source: an estimate, never derived from a skill level or a score (WPM-R230, WPM-D173).

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
    pub plan_pid: Uuid,
    pub worker_pid: Uuid,
    pub year: i32,
    pub amount_minor: i64,
    pub currency: String,
    pub source: String,
    pub stated_by: Option<String>,
    pub stated_on: NaiveDate,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
