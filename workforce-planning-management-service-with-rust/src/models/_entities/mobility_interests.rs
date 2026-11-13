//! Row type — `mobility_interests`. An employee's expressed interest
//! in a role profile or an open requisition.

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
    pub role_profile_pid: Option<Uuid>,
    pub requisition_pid: Option<Uuid>,
    pub note: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
