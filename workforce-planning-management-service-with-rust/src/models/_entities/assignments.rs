//! Row type — `assignments`. A contract a worker holds (WPM-R161): job title, department, location, optional categories
//! (`None` is unknown), the FTE the contract says, and the days it runs. One primary is open at a time.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub job_title: String,
    pub department: String,
    pub location: Option<String>,
    pub site: Option<String>,
    pub area_of_work: Option<String>,
    pub job_role: Option<String>,
    pub staff_group: Option<String>,
    pub occupation_code: Option<String>,
    pub cost_centre: Option<String>,
    pub fte_percent: i32,
    pub is_primary: bool,
    pub starts_on: SqlDate,
    pub ends_on: Option<SqlDate>,
    pub recorded_by: Option<String>,
    pub grade_scale: Option<String>,
    pub grade_band: Option<String>,
    pub grade_pay_point: Option<i32>,
}
