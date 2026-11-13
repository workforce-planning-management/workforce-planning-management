//! Row type — `assignments`. A contract a worker holds (WPM-R161): job title, department, location, optional categories
//! (`None` is unknown), the FTE the contract says, and the days it runs. One primary is open at a time.

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
    pub starts_on: NaiveDate,
    pub ends_on: Option<NaiveDate>,
    pub recorded_by: Option<String>,
}
