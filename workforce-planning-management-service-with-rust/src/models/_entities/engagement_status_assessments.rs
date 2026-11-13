//! Row type — `engagement_status_assessments`. An employment-status assessment for a contractor: outcome, date, reviewer (WPM-R80).

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
    pub outcome: String,
    pub assessed_on: NaiveDate,
    pub reviewed_by: Option<String>,
}
