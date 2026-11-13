//! Row type — `rota_overrides`. A swap: a worker on call for a date window regardless of the rotation.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub rota_pid: Uuid,
    pub worker_pid: Uuid,
    pub starts_on: NaiveDate,
    pub ends_on: NaiveDate,
    pub note: Option<String>,
    pub created_by: Option<String>,
}
