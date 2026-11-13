//! Row type — `oversight_levels`. An organization's own oversight level (WPM-R65). Nothing is pre-loaded; `synthetic` marks the demo scale.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub organization_ref: String,
    pub ordinal: i32,
    pub name: String,
    pub synthetic: bool,
}
