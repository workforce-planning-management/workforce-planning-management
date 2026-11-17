//! Row type — `learning_requirements`. What must be learned, on what basis and for whom (WPM-R170, WPM-D117).

#![allow(missing_docs)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub organization_ref: String,
    pub subject: String,
    pub basis: String,
    pub source: Option<String>,
    pub refresher_months: Option<i32>,
    pub courses: Value,
    pub audience: Value,
    pub accepts_transferred: bool,
    pub retired_on: Option<SqlDate>,
    pub due_soon_days: Option<i32>,
    pub recorded_by: Option<String>,
}
