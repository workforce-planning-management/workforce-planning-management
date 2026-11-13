//! Row type — `sfia_skill_levels`. A level a SFIA skill is defined at.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub code: String,
    pub level: i32,
    pub description: String,
    pub attributes: Option<String>,
    pub retired_in: Option<String>,
}
