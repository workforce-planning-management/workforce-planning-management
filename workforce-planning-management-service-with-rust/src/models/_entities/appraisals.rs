//! Row type — `appraisals`. One 360° appraisal (WPM-R29): a subject worker, the declared competencies, the lifecycle.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
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
    pub competencies: Value,
    pub status: String,
    pub shared_on: Option<NaiveDate>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
