//! Row type — `coverage_evidence`. An organization's workers' compensation or employer's liability cover in one jurisdiction
//! (WPM-R220, WPM-D154): a record of the fact and its dates, with a reference to the certificate, never a copy.

#![allow(missing_docs)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub organization_ref: String,
    pub jurisdiction: String,
    pub kind: String,
    pub insurer: Option<String>,
    pub policy_reference: Option<String>,
    pub minimum_cover_minor: Option<i64>,
    pub currency: Option<String>,
    pub valid_from: Option<SqlDate>,
    pub valid_to: Option<SqlDate>,
    pub certificate_ref: Option<String>,
    pub recorded_by: Option<String>,
    pub retired_on: Option<SqlDate>,
}
