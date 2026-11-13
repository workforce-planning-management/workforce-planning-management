//! Row type — `start_decisions`. A recorded decision to start or defer a programme, with its reason (WPM-R60, WPM-D50).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub organization_ref: String,
    pub programme_ref: String,
    pub decision: String,
    pub reason: String,
    pub fit: String,
    pub earliest_shift_months: Option<i32>,
    pub wip_exceeds: Option<bool>,
    pub decided_by: Option<String>,
}
