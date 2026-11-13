//! Row type — `facility_time_periods`. Minutes released for representative duties, per recognised body per period (WPM-R107). There is no
//! person here and no kind of time entry for it: a budget can be agreed without a list of representatives (WPM-D68).

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
    pub body_pid: Uuid,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub minutes: i32,
    pub budget_minutes: Option<i32>,
    pub recorded_by: Option<String>,
}
