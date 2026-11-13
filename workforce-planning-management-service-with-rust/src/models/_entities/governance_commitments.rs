//! Row type — `governance_commitments`. A recurring oversight commitment of a role: how often, how long, how many people; never a person (WPM-R64).

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
    pub name: String,
    pub role: String,
    pub senior: bool,
    pub kind: String,
    pub frequency: String,
    pub minutes: i32,
    pub holders: i32,
    pub recorded_by: Option<String>,
}
