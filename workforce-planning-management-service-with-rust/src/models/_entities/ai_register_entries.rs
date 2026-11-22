//! Row type — `ai_register_entries`. What a deployer recorded for one scoring feature under Article 26 of the AI
//! Act (WPM-R148): the category, the oversight role, when people were informed and the log retention. No personal data.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub feature_id: String,
    pub annex_category: Option<String>,
    pub overseer_role: Option<String>,
    pub representatives_informed_on: Option<SqlDate>,
    pub workers_informed_on: Option<SqlDate>,
    pub log_retention_months: Option<i32>,
    pub retention_note: Option<String>,
    pub recorded_by: Option<String>,
}
