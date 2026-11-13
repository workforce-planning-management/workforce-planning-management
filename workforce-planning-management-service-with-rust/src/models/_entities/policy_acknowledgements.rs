//! Row type — `policy_acknowledgements`. One worker saying they have read one version, on a date (WPM-R156).

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
    pub version_pid: Uuid,
    pub worker_pid: Uuid,
    pub acknowledged_on: NaiveDate,
}
