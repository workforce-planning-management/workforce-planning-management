//! Row type — `interviews`. One scheduled interview on an application (WPM-R2).

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
    pub application_pid: Uuid,
    pub scheduled_at: DateTime<FixedOffset>,
    pub interviewer_ref: String,
    pub outcome: String,
    pub notes: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
