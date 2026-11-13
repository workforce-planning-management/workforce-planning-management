//! Row type — `pulse_responses`. One anonymous pulse response (WPM-R28). **No author column exists, by design** (WPM-D20): survey, department, score, date — nothing that can come back to a person.

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
    pub survey_pid: Uuid,
    pub department: String,
    pub score: i32,
    pub submitted_on: NaiveDate,
}
