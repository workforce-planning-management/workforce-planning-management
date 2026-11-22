//! Row type — `organization_memberships`. One person's access
//! to one organization, with a role — covers both employment-linked
//! membership (`worker_pid` set) and staff/admin-only privilege grants
//! (`worker_pid` null) in one table.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub person_ref: String,
    pub organization_ref: String,
    pub worker_pid: Option<Uuid>,
    pub role: String,
    pub starts_on: SqlDate,
    pub ends_on: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
}
