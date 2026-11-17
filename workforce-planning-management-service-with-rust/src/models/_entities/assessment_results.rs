//! Row type — `assessment_results`. One scale's outcome within an assessment; scores are integers (WPM-R20).

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub assessment_pid: Uuid,
    pub scale: String,
    pub raw_score: Option<i32>,
    pub max_score: Option<i32>,
    pub percentile: Option<i32>,
    pub band: Option<String>,
    pub narrative: Option<String>,
}
