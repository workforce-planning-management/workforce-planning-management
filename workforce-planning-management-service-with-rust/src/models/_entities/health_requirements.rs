//! Row type — `health_requirements`. A requirement an area sets, such as a required immunization (WPM-R128). Not personal data.

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
    pub description: Option<String>,
    pub departments: Value,
    pub recheck_calendar_days: Option<i32>,
    pub reminder_calendar_days: i32,
    pub deleted_at: Option<SqlTimestamp>,
}
