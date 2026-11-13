//! Row type — `rota_swap_requests`. A request that a colleague take the requester's on-call days in a window.

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
    pub rota_pid: Uuid,
    pub requester_pid: Uuid,
    pub taker_pid: Uuid,
    pub starts_on: NaiveDate,
    pub ends_on: NaiveDate,
    pub note: Option<String>,
    pub status: String,
    pub decided_at: Option<DateTime<FixedOffset>>,
    pub created_by: Option<String>,
}
