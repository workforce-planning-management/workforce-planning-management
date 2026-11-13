//! Row type — `development_plan_items`. One skill step of a development plan (WPM-R21).

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
    pub plan_pid: Uuid,
    pub skill_pid: Uuid,
    pub current_level: i32,
    pub target_level: i32,
    pub method: String,
    pub course_ref: Option<String>,
    pub due_on: Option<NaiveDate>,
    pub status: String,
}
