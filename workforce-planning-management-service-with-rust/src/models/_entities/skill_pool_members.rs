//! Row type — `skill_pool_members`. How a worker counts toward a pool: a role profile or a skill at a minimum proficiency (WPM-R56).

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub pool_pid: Uuid,
    pub role_profile_pid: Option<Uuid>,
    pub skill_pid: Option<Uuid>,
    pub min_proficiency: Option<i32>,
    pub deleted_at: Option<SqlTimestamp>,
}
