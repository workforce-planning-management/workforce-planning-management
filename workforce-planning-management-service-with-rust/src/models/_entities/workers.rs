//! Row type — `workers`. One employment relationship: the single source of employment truth (WPM-R7). Identities are `EntityRef` URNs; salary is minor units (sensitive).

#![allow(missing_docs)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub person_ref: String,
    pub upstream_worker_ref: Option<String>,
    pub organization_ref: String,
    pub worker_number: String,
    pub display_name: String,
    pub status: String,
    pub employment_type: String,
    pub fte_percent: i32,
    pub department: String,
    pub job_title: String,
    pub location: Option<String>,
    pub work_jurisdiction: Option<String>,
    pub manager_pid: Option<Uuid>,
    pub salary_minor: Option<i64>,
    pub salary_currency: Option<String>,
    pub hired_on: SqlDate,
    pub terminated_on: Option<SqlDate>,
    pub engagement_ends_on: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
}
