//! Row type — `early_warnings`. A condition that became true (a pool over-committed, a measure in breach): when it was first seen, when it stopped, when a person acknowledged it (WPM-R68). Names a pool or a measure, never a person.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
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
    pub subject_pid: Uuid,
    pub subject_name: String,
    pub first_true_on: SqlDate,
    pub cleared_on: Option<SqlDate>,
    pub acknowledged_on: Option<SqlDate>,
}
