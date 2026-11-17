//! Row type — `assessment_instruments`. The catalog of named tests: category + the scales each reports (WPM-R20).

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub name: String,
    pub category: String,
    pub provider: Option<String>,
    pub scales: Value,
    pub duration_minutes: Option<i32>,
    pub validity_months: Option<i32>,
    pub deleted_at: Option<SqlTimestamp>,
}
