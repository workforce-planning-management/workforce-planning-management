//! Row type — `conversion_plans`. A recorded intent for a fixed-term or contractor engagement: convert, extend, end or undecided (WPM-R98, WPM-D66). No pay figure.

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
    pub intent: String,
    pub target_on: NaiveDate,
    pub department: Option<String>,
    pub role_profile_ref: Option<String>,
    pub post_funding_kind: Option<String>,
    pub post_funding_ends_on: Option<NaiveDate>,
    pub reason: Option<String>,
    pub proposed_by: Option<String>,
    pub approved_by: Option<String>,
    pub status: String,
    pub review_on: Option<NaiveDate>,
    pub settled_on: Option<NaiveDate>,
}
