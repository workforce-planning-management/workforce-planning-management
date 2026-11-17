//! Row type — `role_qualification_requirements`. A qualification a role profile asks for (WPM-R171).

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub role_profile_pid: Uuid,
    pub title: String,
    pub kind: Option<String>,
    pub framework: Option<String>,
    pub min_level: Option<i32>,
    pub recorded_by: Option<String>,
}
