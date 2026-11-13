//! Row type — `skill_pools`. A plannable pool of role profiles and skills with an operations reservation (WPM-R56).

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
    pub organization_ref: String,
    pub name: String,
    pub description: Option<String>,
    pub operations_reservation_bp: i32,
    pub peer_pool_ref: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
