//! Row type — `pay_information_requests`. A worker's request for their own pay level and their category's
//! average pay by sex (WPM-R145). Dates only; the figures are not stored.

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
    pub asked_on: NaiveDate,
    pub due_on: NaiveDate,
    pub answered_on: Option<NaiveDate>,
    pub answered_by: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
