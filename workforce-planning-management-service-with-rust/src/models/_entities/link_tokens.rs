//! Row type — `link_tokens`. A single-use link token, stored only as its SHA-256 hash. There is deliberately no column that
//! could hold the token itself.

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
    pub token_hash: String,
    pub purpose: String,
    pub subject_pid: Option<Uuid>,
    pub expires_at: DateTime<FixedOffset>,
    pub used_at: Option<DateTime<FixedOffset>>,
}
