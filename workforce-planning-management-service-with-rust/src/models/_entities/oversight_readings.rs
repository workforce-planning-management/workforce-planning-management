//! Row type — `oversight_readings`. One period's reading of a reported measure, with its evidence note and source document (WPM-R66).

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
    pub measure_pid: Uuid,
    pub period_on: SqlDate,
    pub value: Option<i64>,
    pub evidence: Option<String>,
    pub source_document: Option<String>,
    pub recorded_by: Option<String>,
}
