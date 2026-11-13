//! Row type — `post_fundings`. Where a worker's post is funded from and when that ends: a kind and a date, never an amount (WPM-R62, WPM-R98).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub worker_pid: Uuid,
    pub kind: String,
    pub programme_ref: Option<String>,
    pub ends_on: Option<NaiveDate>,
    pub recorded_by: Option<String>,
}
