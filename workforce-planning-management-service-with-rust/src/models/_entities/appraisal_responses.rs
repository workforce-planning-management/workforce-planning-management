//! Row type — `appraisal_responses`. One rater's 360° response (WPM-R29): per-competency scores + optional comment. Links to its nomination by design (WPM-D21 — procedural anonymity: the link enforces once-per-rater; the API never serves rater-level content).

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
    pub appraisal_pid: Uuid,
    pub nomination_pid: Uuid,
    pub rater_group: String,
    pub scores: Value,
    pub comment: Option<String>,
}
