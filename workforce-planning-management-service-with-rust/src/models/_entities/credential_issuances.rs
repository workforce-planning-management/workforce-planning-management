//! Row type — `credential_issuances`. That a credential was issued from a verified CPD entry (WPM-R157): not the credential.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub kind: String,
    pub source_pid: Uuid,
    pub issued_at: DateTime<FixedOffset>,
    pub revoked_at: Option<DateTime<FixedOffset>>,
    pub revoked_by: Option<String>,
    pub sha256: String,
}
