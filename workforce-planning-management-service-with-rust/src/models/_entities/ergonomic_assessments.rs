//! Row type — `ergonomic_assessments`. One DSE workstation assessment (WPM-R32): the workstation, never the body (WPM-D24).

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
    pub workstation: String,
    pub status: String,
    pub assessed_on: Option<NaiveDate>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
