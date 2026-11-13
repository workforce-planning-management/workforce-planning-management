//! Row type — `feedback_entries`. One feedback entry on a review (WPM-R10).

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
    pub review_pid: Uuid,
    pub author_ref: String,
    pub content: String,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
