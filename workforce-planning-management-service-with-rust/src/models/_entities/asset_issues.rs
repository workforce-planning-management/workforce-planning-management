//! Row type — `asset_issues`. Who holds an asset, from when to when (WPM-R155). Closed by a return or a write-off.

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
    pub asset_pid: Uuid,
    pub worker_pid: Uuid,
    pub issued_on: NaiveDate,
    pub returned_on: Option<NaiveDate>,
    pub outcome: Option<String>,
    pub note: Option<String>,
    pub recorded_by: Option<String>,
}
