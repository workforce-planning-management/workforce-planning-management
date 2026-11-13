//! Row type — `assets`. A piece of equipment (WPM-R155): kind, tag and owner role. No location, cost or value.

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
    pub kind: String,
    pub tag: String,
    pub owner_role: String,
    pub recorded_by: Option<String>,
}
