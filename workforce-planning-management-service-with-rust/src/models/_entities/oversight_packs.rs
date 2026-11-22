//! Row type — `oversight_packs`. A monthly snapshot of the oversight position, criteria, measures and capacity view, frozen as of a date and never changed (WPM-R69). Aggregates only; no person appears.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub organization_ref: String,
    pub as_of: SqlDate,
    pub month: SqlDate,
    pub payload: Value,
}
