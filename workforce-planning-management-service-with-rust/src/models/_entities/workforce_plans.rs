//! Row type — `workforce_plans`. A workforce plan (scenario): hypothetical aggregate headcount.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub name: String,
    pub organization_ref: String,
    pub horizon_start: NaiveDate,
    pub horizon_end: NaiveDate,
    pub rationale: Option<String>,
    pub attrition_bp: Option<i32>,
    pub budget_minor: Option<i64>,
    pub budget_currency: Option<String>,
    pub on_cost_bp: Option<i32>,
    pub status: String,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
