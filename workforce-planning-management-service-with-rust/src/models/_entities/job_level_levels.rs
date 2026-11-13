//! Row type — `job_level_levels`. A level on a stored job-level ladder.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub framework: String,
    pub number: i32,
    pub code: String,
    pub title: String,
    pub summary: String,
    pub attributes: Option<String>,
}
