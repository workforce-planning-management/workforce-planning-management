//! Row type — `expense_claims`. A worker's request to be repaid for money they spent (WPM-R55).

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
    pub worker_pid: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub currency: String,
    pub status: String,
    pub submitted_at: Option<DateTime<FixedOffset>>,
    pub decided_by: Option<String>,
    pub decided_at: Option<DateTime<FixedOffset>>,
    pub decision_note: Option<String>,
    pub reimbursed_on: Option<NaiveDate>,
    pub reimbursed_by: Option<String>,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
