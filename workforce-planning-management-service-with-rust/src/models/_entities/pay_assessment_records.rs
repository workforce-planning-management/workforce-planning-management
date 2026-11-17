//! Row type — `pay_assessment_records`. What a person recorded for a category of workers and a pay report: a
//! justification, a remedy, a joint assessment and its outcome (WPM-R146). No personal data.

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
    pub organization_ref: String,
    pub category: String,
    pub reported_on: SqlDate,
    pub justified: bool,
    pub justification: Option<String>,
    pub remedied_on: Option<SqlDate>,
    pub assessed_on: Option<SqlDate>,
    pub outcome: Option<String>,
    pub representatives: Option<String>,
    pub recorded_by: Option<String>,
}
