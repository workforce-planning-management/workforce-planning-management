//! Row type — `cutover_roles`. A role that must be staffed with trained people for a cutover (WPM-R70).

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
    pub cutover_pid: Uuid,
    pub role_profile_pid: Uuid,
    pub course_ref: String,
    pub required_headcount: i32,
}
