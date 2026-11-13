//! Row type — `pay_categories`. A category of workers for pay reporting: roles the employer treats as work of
//! equal value, with the basis stated (WPM-R144). Holds no personal data.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub organization_ref: String,
    pub name: String,
    pub basis: String,
    pub job_titles: Value,
    pub set_by: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
