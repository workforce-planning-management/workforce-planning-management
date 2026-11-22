//! Row type — `link_tokens`. A single-use link token, stored only as its SHA-256 hash. There is deliberately no column that
//! could hold the token itself.

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
    pub token_hash: String,
    pub purpose: String,
    pub subject_pid: Option<Uuid>,
    pub expires_at: SqlTimestamp,
    pub used_at: Option<SqlTimestamp>,
}
