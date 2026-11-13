//! Row type — `applications`. One candidate's application to one requisition (WPM-R2).

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
    pub requisition_pid: Uuid,
    pub candidate_pid: Uuid,
    pub stage: String,
    pub notes: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
