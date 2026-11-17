//! Row type — `initiative_role_impacts`. A role an initiative displaces, reshapes, or creates.

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
    pub initiative_pid: Uuid,
    pub role_profile_pid: Uuid,
    pub impact: String,
    pub timeframe: String,
    pub note: Option<String>,
}
