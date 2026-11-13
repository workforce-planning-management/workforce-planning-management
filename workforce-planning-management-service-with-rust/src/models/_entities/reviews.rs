//! Row type — `reviews`. One worker's review in one cycle; content is high-sensitivity (WPM-R10).

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
    /// `None` for a PADR, which belongs to no cycle (WPM-D176).
    pub cycle_pid: Option<Uuid>,
    /// `performance` or `padr` (WPM-R172).
    pub kind: String,
    pub worker_pid: Uuid,
    pub reviewer_ref: String,
    pub status: String,
    pub rating: Option<i32>,
    pub content: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
