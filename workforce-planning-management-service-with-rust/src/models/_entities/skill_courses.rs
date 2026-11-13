//! Row type — `skill_courses`. A catalogue course that builds a skill: its hours, how many levels it typically adds, and a fee if one is recorded.

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
    pub skill_pid: Uuid,
    pub course_ref: String,
    pub title: String,
    pub hours: i32,
    pub levels: i32,
    pub deleted_at: Option<DateTime<FixedOffset>>,
    /// The fee in minor units; absent is unknown, never free (WPM-R227).
    pub fee_minor: Option<i64>,
    pub fee_currency: Option<String>,
}
