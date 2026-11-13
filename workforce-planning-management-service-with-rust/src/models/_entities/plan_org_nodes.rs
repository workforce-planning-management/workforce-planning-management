//! Row type — `plan_org_nodes`. A role in a plan's target organization (WPM-R153): a department, a title, a headcount and a
//! parent role. Roles, never people.

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
    pub plan_pid: Uuid,
    pub node_key: String,
    pub parent_key: Option<String>,
    pub department: String,
    pub title: String,
    pub headcount: i32,
}
