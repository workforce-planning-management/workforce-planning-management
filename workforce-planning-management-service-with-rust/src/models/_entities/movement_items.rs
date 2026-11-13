//! Row type — `movement_items`. One dated item of a joiner or leaver checklist.

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
    pub movement_pid: Uuid,
    pub position: i32,
    pub title: String,
    pub category: String,
    pub due_on: NaiveDate,
    pub assignee_pid: Option<Uuid>,
    pub done_on: Option<NaiveDate>,
    pub done_by: Option<String>,
    pub skipped_reason: Option<String>,
}
