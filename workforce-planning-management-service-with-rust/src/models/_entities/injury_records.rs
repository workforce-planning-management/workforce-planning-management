//! Row type — `injury_records`. An incident: the proof that reports were owed and a pointer to the statutory record, with no description and no medical detail (WPM-R221, WPM-D179).

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
    pub jurisdiction: String,
    pub kind: String,
    pub worker_pid: Option<Uuid>,
    pub occurred_at: DateTime<FixedOffset>,
    pub learned_at: Option<DateTime<FixedOffset>>,
    pub class: String,
    pub absence_days: Option<i32>,
    pub people_affected: i32,
    pub statutory_record_ref: Option<String>,
    pub keep_until: Option<NaiveDate>,
    pub recorded_by: Option<String>,
}
