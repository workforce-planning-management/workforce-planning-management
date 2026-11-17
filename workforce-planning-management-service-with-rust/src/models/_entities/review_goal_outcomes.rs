//! Row type — `review_goal_outcomes`. A goal in a review: the outcome the reviewer and the reviewee each record, and the snapshot frozen when the review is shared (WPM-R225, WPM-D165).

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub review_pid: Uuid,
    pub goal_pid: Uuid,
    pub worker_pid: Uuid,
    pub reviewer_outcome: Option<String>,
    pub reviewee_outcome: Option<String>,
    pub snapshot: Option<Value>,
}
