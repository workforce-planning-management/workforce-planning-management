//! Row type — `policy_versions`. One version of a policy: title, a reference to the document (no text), effective date,
//! audience and days to acknowledge (WPM-R156).

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
    pub policy_pid: Uuid,
    pub version: i32,
    pub title: String,
    pub document_ref: String,
    pub effective_on: NaiveDate,
    pub audience_kind: String,
    pub audience_value: Option<String>,
    pub acknowledge_within_days: i32,
    pub recorded_by: Option<String>,
}
