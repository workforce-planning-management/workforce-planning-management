//! Row type — `notifications`. One in-app notification (WPM-R31): reference-only (WPM-D23 — a kind, a neutral body, pids/names; never scores, comments, or masked-tier values), owned by one worker.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub kind: String,
    pub body: String,
    pub data: Value,
    pub read_at: Option<DateTime<FixedOffset>>,
}
