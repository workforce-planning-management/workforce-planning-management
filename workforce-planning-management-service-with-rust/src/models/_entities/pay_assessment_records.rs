//! Row type — `pay_assessment_records`. What a person recorded for a category of workers and a pay report: a
//! justification, a remedy, a joint assessment and its outcome (WPM-R146). No personal data.

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
    pub organization_ref: String,
    pub category: String,
    pub reported_on: NaiveDate,
    pub justified: bool,
    pub justification: Option<String>,
    pub remedied_on: Option<NaiveDate>,
    pub assessed_on: Option<NaiveDate>,
    pub outcome: Option<String>,
    pub representatives: Option<String>,
    pub recorded_by: Option<String>,
}
