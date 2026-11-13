//! Row type — `worker_goals`. One person's goal for a period: a record of intent and evidence, never a score (WPM-R224, WPM-D163).

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
    pub organization_ref: String,
    pub title: String,
    pub description: Option<String>,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub status: String,
    pub proposed_by: String,
    pub agreed_on: Option<NaiveDate>,
    pub skill_pid: Option<Uuid>,
    pub skill_target_level: Option<i32>,
    pub plan_item_pid: Option<Uuid>,
    pub aligns_to: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
