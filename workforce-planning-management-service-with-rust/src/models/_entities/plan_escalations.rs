//! Row type — `plan_escalations`. A breach of a plan's tolerance taken to someone who can decide.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub plan_pid: Uuid,
    pub dimension: String,
    pub level: i32,
    pub status: String,
    pub raised_by: Option<String>,
    pub raised_on: NaiveDate,
    pub level_raised_on: Option<NaiveDate>,
    pub note: Option<String>,
    pub rag_at_raise: String,
    pub deviation_at_raise: Option<i64>,
    pub limit_at_raise: i64,
    pub decision: Option<String>,
    pub decided_by: Option<String>,
    pub decided_on: Option<NaiveDate>,
    pub decision_note: Option<String>,
    pub rag_at_decision: Option<String>,
    pub deviation_at_decision: Option<i64>,
}
