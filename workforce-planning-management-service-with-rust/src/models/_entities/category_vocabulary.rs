//! Row type — `category_vocabulary`. One entry of an organization's assignment category vocabulary (WPM-R162): a code, a label,
//! the national standard it comes from where it does, and, for a job role, the staff group it belongs to.

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
    pub kind: String,
    pub code: String,
    pub label: String,
    pub standard: Option<String>,
    pub standard_code: Option<String>,
    pub staff_group: Option<String>,
    pub recorded_by: Option<String>,
}
