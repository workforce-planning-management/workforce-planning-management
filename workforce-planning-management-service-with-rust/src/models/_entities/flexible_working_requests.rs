//! Row type — `flexible_working_requests`. A worker's request for a different working arrangement and its decision (WPM-R126).

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
    pub kind: String,
    pub requested_on: NaiveDate,
    pub proposed_start: NaiveDate,
    pub proposed_fte_percent: Option<i32>,
    pub effect_on_team: Option<String>,
    pub how_to_manage: Option<String>,
    pub status: String,
    pub decide_by: NaiveDate,
    pub decided_on: Option<NaiveDate>,
    pub decided_by: Option<String>,
    pub decision_reason: Option<String>,
    pub decision_note: Option<String>,
    pub counter_note: Option<String>,
    pub counter_fte_percent: Option<i32>,
    pub trial_until: Option<NaiveDate>,
    pub appealed_on: Option<NaiveDate>,
    pub appeal_note: Option<String>,
    pub appeal_decided_by: Option<String>,
}
