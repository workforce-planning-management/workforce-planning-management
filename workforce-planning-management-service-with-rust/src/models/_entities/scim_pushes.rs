//! Row type — `scim_pushes`. What was last sent to the identity provider for a worker (WPM-R150): the provider's id,
//! a fingerprint and the retry state. No name, no contact detail, no body.

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
    pub scim_id: Option<String>,
    pub body_hash: Option<String>,
    pub pushed_at: Option<DateTime<FixedOffset>>,
    pub attempts: i32,
    pub next_attempt_at: Option<DateTime<FixedOffset>>,
    pub last_error: Option<String>,
}
