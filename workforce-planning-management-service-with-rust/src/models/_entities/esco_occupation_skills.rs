//! Row type — `esco_occupation_skills`. A pinned ESCO occupation-to-skill relation.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub occupation_uri: String,
    pub skill_uri: String,
    pub relation: String,
}
