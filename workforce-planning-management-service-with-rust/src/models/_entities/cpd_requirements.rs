//! Row type — `cpd_requirements`. Hours or points required in a period.

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
    pub name: String,
    pub unit: String,
    pub required_hundredths: i64,
    pub period_start: SqlDate,
    pub period_end: SqlDate,
    pub job_title: Option<String>,
    pub deleted_at: Option<SqlTimestamp>,
}
