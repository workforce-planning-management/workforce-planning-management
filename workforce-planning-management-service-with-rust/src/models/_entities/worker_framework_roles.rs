//! Row type — `worker_framework_roles`. The role a person says they
//! hold now in an external framework (a PCF role level or an ESCO occupation).

#![allow(missing_docs)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub framework_slug: String,
    pub role_profile_pid: Option<Uuid>,
    pub esco_occupation_uri: Option<String>,
    pub role_label: String,
    pub selected_on: SqlDate,
    pub started_at: SqlTimestamp,
    pub ended_at: Option<SqlTimestamp>,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
}
