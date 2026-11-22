//! Row type — `shifts`. One planned shift for a department (WPM-R6).

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
    pub department: String,
    pub starts_at: SqlTimestamp,
    pub ends_at: SqlTimestamp,
    pub required_headcount: i32,
    pub deleted_at: Option<SqlTimestamp>,
}
