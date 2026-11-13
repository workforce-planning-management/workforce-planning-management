//! Row type — `organization_confederations`. One direct
//! parent→child organization edge; arbitrary-depth nesting is chained
//! edges, walked in Rust (`crate::rules::org_access::descendants_of`),
//! not recursive SQL.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub parent_organization_ref: String,
    pub child_organization_ref: String,
    pub starts_on: NaiveDate,
    pub ends_on: Option<NaiveDate>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
