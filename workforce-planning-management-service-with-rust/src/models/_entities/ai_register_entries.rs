//! Row type — `ai_register_entries`. What a deployer recorded for one scoring feature under Article 26 of the AI
//! Act (WPM-R148): the category, the oversight role, when people were informed and the log retention. No personal data.

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
    pub feature_id: String,
    pub annex_category: Option<String>,
    pub overseer_role: Option<String>,
    pub representatives_informed_on: Option<NaiveDate>,
    pub workers_informed_on: Option<NaiveDate>,
    pub log_retention_months: Option<i32>,
    pub retention_note: Option<String>,
    pub recorded_by: Option<String>,
}
