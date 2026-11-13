//! Row type — `course_requests`. A person's request to take a course whose recorded fee is not zero (WPM-R236).

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
    pub course_ref: String,
    pub status: String,
    pub decided_by: Option<String>,
    pub decided_on: Option<NaiveDate>,
}
