//! Row type — `adjustment_requests`. One reasonable-adjustment request (WPM-R33): barrier, impact, change — **no diagnosis column exists** (WPM-D25).

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
    pub category: String,
    pub barrier: String,
    pub impact: String,
    pub adjustment: String,
    pub status: String,
    pub decision_note: Option<String>,
    pub decided_on: Option<NaiveDate>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
