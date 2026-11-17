//! Row type — `decision_gates`. A kind of change that needs consultation with a recognised body (WPM-R108).

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
    pub organization_ref: String,
    pub change_kind: String,
    pub body_pid: Uuid,
    pub note: Option<String>,
    pub recorded_by: Option<String>,
}
