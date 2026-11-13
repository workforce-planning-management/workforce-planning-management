//! Row type — `rotas`. A named on-call rotation in one organization.

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
    pub organization_ref: String,
    pub name: String,
    pub description: Option<String>,
    pub period_days: i32,
    pub starts_on: NaiveDate,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
