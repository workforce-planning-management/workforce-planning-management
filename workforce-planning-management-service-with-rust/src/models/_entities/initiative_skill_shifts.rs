//! Row type — `initiative_skill_shifts`. A skill whose demand an initiative moves.

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
    pub initiative_pid: Uuid,
    pub skill_pid: Uuid,
    pub direction: String,
    pub note: Option<String>,
}
