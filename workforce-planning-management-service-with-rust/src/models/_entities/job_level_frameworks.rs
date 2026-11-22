//! Row type — `job_level_frameworks`. A job-level ladder held in the database.

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
    pub slug: String,
    pub name: String,
    pub organization: String,
    pub track: String,
    pub source: String,
    pub version: Option<String>,
    pub redistribution: String,
}
