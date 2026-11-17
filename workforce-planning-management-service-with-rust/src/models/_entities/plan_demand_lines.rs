//! Row type — `plan_demand_lines`. Planned headcount for a department (and optionally a role) at a date.

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
    pub plan_pid: Uuid,
    pub department: String,
    pub role_profile_pid: Option<Uuid>,
    pub target_on: SqlDate,
    pub target_headcount: i32,
    pub note: Option<String>,
    /// The programme the demand serves (`thing:` URN), if any.
    pub programme_ref: Option<String>,
    /// FTE target in percent of one person across the headcount; `None` is unknown, not zero.
    pub target_fte_percent: Option<i32>,
    /// The engagement basis of the demand.
    pub basis: String,
    /// The last day the demanded engagements run, for a basis that ends.
    pub expected_ends_on: Option<SqlDate>,
}
