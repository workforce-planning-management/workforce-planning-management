//! Row type — `pulse_responses`. One anonymous pulse response (WPM-R28). **No author column exists, by design** (WPM-D20): survey, department, score, date — nothing that can come back to a person.

#![allow(missing_docs)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub survey_pid: Uuid,
    pub department: String,
    pub score: i32,
    pub submitted_on: SqlDate,
}
