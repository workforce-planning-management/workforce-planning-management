//! Row type — `plan_org_nodes`. A role in a plan's target organization (WPM-R153): a department, a title, a headcount and a
//! parent role. Roles, never people.

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
    pub plan_pid: Uuid,
    pub node_key: String,
    pub parent_key: Option<String>,
    pub department: String,
    pub title: String,
    pub headcount: i32,
}
