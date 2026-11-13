//! Row type — `skill_external_refs`. How a catalogue skill is known in
//! an external framework (PCF name, ESCO concept URI).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub skill_pid: Uuid,
    pub framework_slug: String,
    /// The `ref` column (a reserved word in Rust); a query selects it under this name.
    pub reference: String,
    pub label: Option<String>,
    pub version: Option<String>,
}
