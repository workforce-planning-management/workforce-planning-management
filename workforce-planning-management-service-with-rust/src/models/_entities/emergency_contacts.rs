//! Row type — `emergency_contacts`. A person a worker names to be reached in an emergency (third-party personal data).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub name: String,
    pub relationship: String,
    pub phone: String,
    pub alt_phone: Option<String>,
    pub email: Option<String>,
    pub priority: i32,
    pub note: Option<String>,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
    pub deleted_at: Option<DateTime<FixedOffset>>,
    pub phone_digits: Option<String>,
    pub phone_e164: Option<String>,
    pub alt_phone_digits: Option<String>,
    pub alt_phone_e164: Option<String>,
}
