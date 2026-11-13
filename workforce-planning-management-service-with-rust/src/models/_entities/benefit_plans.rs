//! Row type — `benefit_plans`. One benefit plan with minor-unit costs (WPM-R9).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub name: String,
    pub kind: String,
    pub provider: String,
    pub worker_cost_minor: i64,
    pub employer_cost_minor: i64,
    pub currency: String,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
