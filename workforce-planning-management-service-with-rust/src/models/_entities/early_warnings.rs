//! Row type — `early_warnings`. A condition that became true (a pool over-committed, a measure in breach): when it was first seen, when it stopped, when a person acknowledged it (WPM-R68). Names a pool or a measure, never a person.

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
    pub kind: String,
    pub subject_pid: Uuid,
    pub subject_name: String,
    pub first_true_on: NaiveDate,
    pub cleared_on: Option<NaiveDate>,
    pub acknowledged_on: Option<NaiveDate>,
}
