//! Row type — `benefit_enrollments`. One worker's enrolment in one plan (WPM-R9).

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
    pub plan_pid: Uuid,
    pub worker_pid: Uuid,
    pub starts_on: NaiveDate,
    pub ends_on: Option<NaiveDate>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
