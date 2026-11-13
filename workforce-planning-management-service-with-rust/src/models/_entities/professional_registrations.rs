//! Row type — `professional_registrations`. A registration with an expiry.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub body: String,
    pub reference: Option<String>,
    pub registered_on: Option<NaiveDate>,
    pub expires_on: Option<NaiveDate>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
