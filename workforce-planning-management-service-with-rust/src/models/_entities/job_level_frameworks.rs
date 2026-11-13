//! Row type — `job_level_frameworks`. A job-level ladder held in the database.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub organization: String,
    pub track: String,
    pub source: String,
    pub version: Option<String>,
    pub redistribution: String,
}
