//! Row type — `mentorships`. Learning & development (declared/recorded data).

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
    pub mentor_pid: Uuid,
    pub mentee_pid: Uuid,
    pub focus: String,
    pub status: String,
    pub started_on: Option<SqlDate>,
    pub ended_on: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
}
