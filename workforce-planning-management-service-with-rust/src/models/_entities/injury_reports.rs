//! Row type — `injury_reports`. One report of an incident to one recipient: an owed report until it says when and how it went (WPM-R221, WPM-D179).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub record_pid: Uuid,
    pub recipient: String,
    pub due_at: Option<DateTime<FixedOffset>>,
    pub sent_at: Option<DateTime<FixedOffset>>,
    pub method: Option<String>,
    pub reference: Option<String>,
    pub recorded_by: Option<String>,
}
