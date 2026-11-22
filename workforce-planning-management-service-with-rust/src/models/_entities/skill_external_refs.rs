//! Row type — `skill_external_refs`. How a catalogue skill is known in
//! an external framework (PCF name, ESCO concept URI).

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub skill_pid: Uuid,
    pub framework_slug: String,
    /// The `ref` column (a reserved word in Rust); a query selects it under this name.
    pub reference: String,
    pub label: Option<String>,
    pub version: Option<String>,
}
