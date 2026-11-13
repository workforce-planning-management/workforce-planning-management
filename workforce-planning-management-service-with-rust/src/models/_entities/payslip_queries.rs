//! Row type — `payslip_queries`. A person's query on one of their payslips, seen and closed by payroll (WPM-R178).

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
    pub payslip_pid: Uuid,
    pub worker_pid: Uuid,
    pub comment: String,
    pub raised_on: NaiveDate,
    pub closed_on: Option<NaiveDate>,
    pub closed_by: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
