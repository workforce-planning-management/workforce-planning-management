//! Row type — `role_skill_requirements`. A catalogue skill a role requires.

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub role_profile_pid: Uuid,
    pub skill_pid: Uuid,
    pub min_proficiency: i32,
    pub importance: String,
    pub note: Option<String>,
    pub source_framework: Option<String>,
    pub source_level: Option<i32>,
    pub source_scale_max: Option<i32>,
}
