//! Row type — `sfiaplus_resources`. A `SFIAplus` resource for a SFIA skill at a level (a pointer).

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
    pub kind: String,
    pub title: String,
    pub provider: Option<String>,
    pub url: Option<String>,
    pub retired_in: Option<String>,
}
