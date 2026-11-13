//! Row type — `worker_skills`. Learning & development (declared/recorded data).

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
    pub worker_pid: Uuid,
    pub skill_pid: Uuid,
    pub proficiency: i32,
    pub target: Option<i32>,
    pub assessed_on: NaiveDate,
    pub source_framework: Option<String>,
    pub source_level: Option<i32>,
    pub source_scale_max: Option<i32>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
