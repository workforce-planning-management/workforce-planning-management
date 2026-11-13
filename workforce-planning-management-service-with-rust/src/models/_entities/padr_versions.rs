//! Row type — `padr_versions`. One version of a personal development review: its content, its two signatures and, once signed, the objectives as they stood (WPM-R172, WPM-D165).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub review_pid: Uuid,
    pub worker_pid: Uuid,
    pub version: i32,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub values_summary: Option<String>,
    pub wellbeing_offered: Option<bool>,
    pub development_needs: Value,
    pub reviewer_signed_on: Option<NaiveDate>,
    pub reviewee_signed_on: Option<NaiveDate>,
    pub reviewee_comment: Option<String>,
    pub objectives: Option<Value>,
}
