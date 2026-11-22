//! Row type — `mobility_interests`. An employee's expressed interest
//! in a role profile or an open requisition.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub role_profile_pid: Option<Uuid>,
    pub requisition_pid: Option<Uuid>,
    pub note: Option<String>,
    pub deleted_at: Option<SqlTimestamp>,
}
