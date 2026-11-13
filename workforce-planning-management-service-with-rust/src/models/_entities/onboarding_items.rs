//! Row type — `onboarding_items`. One onboarding checklist obligation gating activation (WPM-R3).

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
    pub worker_pid: Uuid,
    pub name: String,
    pub mandatory: bool,
    pub status: String,
    pub waived_reason: Option<String>,
    pub completed_at: Option<DateTime<FixedOffset>>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
