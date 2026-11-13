//! Row type — `sfia_skills`. A skill from the deployer's licensed SFIA download.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub code: String,
    pub name: String,
    pub category: String,
    pub subcategory: String,
    pub description: String,
    pub retired_in: Option<String>,
}
