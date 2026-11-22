//! Row type — `credential_issuances`. That a credential was issued from a verified CPD entry (WPM-R157): not the credential.

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
    pub worker_pid: Uuid,
    pub kind: String,
    pub source_pid: Uuid,
    pub issued_at: SqlTimestamp,
    pub revoked_at: Option<SqlTimestamp>,
    pub revoked_by: Option<String>,
    pub sha256: String,
}
