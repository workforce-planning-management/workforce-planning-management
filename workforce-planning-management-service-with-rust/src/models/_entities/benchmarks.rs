//! Row type — `benchmarks`. One recorded market pay band for a job title (WPM-R14).

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
    pub job_title: String,
    pub currency: String,
    pub min_minor: i64,
    pub median_minor: i64,
    pub max_minor: i64,
    pub source: String,
    pub as_of: SqlDate,
    pub deleted_at: Option<SqlTimestamp>,
}
