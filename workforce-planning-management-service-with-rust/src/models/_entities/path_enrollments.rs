//! Row type — `path_enrollments`. Learning & development (declared/recorded data).

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
    pub path_pid: Uuid,
    pub worker_pid: Uuid,
    pub enrolled_on: NaiveDate,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
