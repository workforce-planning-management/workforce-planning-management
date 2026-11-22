//! Row type — `wellbeing_entitlements`. One configurable health-entitlement rule (WPM-R25): non-clinical predicates only (age band, departments, job titles) per WPM-D17.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub name: String,
    pub kind: String,
    pub benefit_plan_pid: Option<Uuid>,
    pub description: String,
    pub info_url: Option<String>,
    pub min_age: Option<i32>,
    pub max_age: Option<i32>,
    pub departments: Value,
    pub job_titles: Value,
    pub doses: i32,
    pub active_from: Option<SqlDate>,
    pub active_until: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
}
