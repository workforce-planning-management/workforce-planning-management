//! Row type — `assets`. A piece of equipment (WPM-R155): kind, tag and owner role. No location, cost or value.

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
    pub kind: String,
    pub tag: String,
    pub owner_role: String,
    pub recorded_by: Option<String>,
}
