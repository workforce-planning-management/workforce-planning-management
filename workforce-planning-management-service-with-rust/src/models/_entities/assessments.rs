//! Row type — `assessments`. One administration of one instrument to one candidate or worker (WPM-R20).

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
    pub instrument_pid: Uuid,
    pub subject_kind: String,
    pub subject_pid: Uuid,
    pub application_pid: Option<Uuid>,
    pub status: String,
    pub scheduled_on: Option<NaiveDate>,
    pub completed_on: Option<NaiveDate>,
    pub expires_on: Option<NaiveDate>,
    pub administered_by: Option<String>,
    pub notes: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
