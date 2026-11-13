//! Row type — `oversight_positions`. A recorded position: the level, when it took effect, the deciding body (free text) and the published reason (WPM-R65).

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
    pub organization_ref: String,
    pub level_pid: Uuid,
    pub effective_on: NaiveDate,
    pub deciding_body: String,
    pub reason: String,
    pub recorded_by: Option<String>,
}
