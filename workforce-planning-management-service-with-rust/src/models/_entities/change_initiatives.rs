//! Row type — `change_initiatives`. An automation / AI / process change.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub kind: String,
    pub status: String,
    pub starts_on: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
}
