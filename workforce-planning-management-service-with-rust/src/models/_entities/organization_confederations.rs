//! Row type — `organization_confederations`. One direct
//! parent→child organization edge; arbitrary-depth nesting is chained
//! edges, walked in Rust (`crate::rules::org_access::descendants_of`),
//! not recursive SQL.

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
    pub parent_organization_ref: String,
    pub child_organization_ref: String,
    pub starts_on: SqlDate,
    pub ends_on: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
}
