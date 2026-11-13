//! Row type — `partner_commitments`. A partner organization's aggregate capacity for a pool in a month (WPM-R58).

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
    pub partner_ref: String,
    pub pool_pid: Uuid,
    pub month: NaiveDate,
    pub fte_centi: i64,
    pub status: String,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
