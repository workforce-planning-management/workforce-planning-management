//! Row type — `headcount_snapshots`. Append-only aggregate
//! history of employed headcount per organization × department × date.

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
    pub organization_ref: String,
    pub department: String,
    pub as_of: SqlDate,
    pub headcount: i32,
    pub fte_percent_total: i64,
    pub starters: Option<i32>,
    pub leavers: Option<i32>,
    /// Contractors employed on the date; `None` for a row recorded before the split existed (unknown, not zero).
    pub contractor_headcount: Option<i32>,
    pub contractor_fte_percent_total: Option<i64>,
    pub fixed_term_headcount: Option<i32>,
    pub fixed_term_fte_percent_total: Option<i64>,
}
