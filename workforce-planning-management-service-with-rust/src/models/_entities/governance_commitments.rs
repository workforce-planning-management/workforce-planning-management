//! Row type — `governance_commitments`. A recurring oversight commitment of a role: how often, how long, how many people; never a person (WPM-R64).

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
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
