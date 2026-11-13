//! Row type — `oversight_readings`. One period's reading of a reported measure, with its evidence note and source document (WPM-R66).

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
    pub measure_pid: Uuid,
    pub period_on: NaiveDate,
    pub value: Option<i64>,
    pub evidence: Option<String>,
    pub source_document: Option<String>,
    pub recorded_by: Option<String>,
}
