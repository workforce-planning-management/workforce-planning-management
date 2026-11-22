//! Row type — `job_level_levels`. A level on a stored job-level ladder.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub framework: String,
    pub number: i32,
    pub code: String,
    pub title: String,
    pub summary: String,
    pub attributes: Option<String>,
}
