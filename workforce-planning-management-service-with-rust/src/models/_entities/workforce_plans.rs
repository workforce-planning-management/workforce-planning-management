//! Row type — `workforce_plans`. A workforce plan (scenario): hypothetical aggregate headcount.

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
    pub name: String,
    pub organization_ref: String,
    pub horizon_start: SqlDate,
    pub horizon_end: SqlDate,
    pub rationale: Option<String>,
    pub attrition_bp: Option<i32>,
    pub budget_minor: Option<i64>,
    pub budget_currency: Option<String>,
    pub on_cost_bp: Option<i32>,
    pub status: String,
    pub deleted_at: Option<SqlTimestamp>,
}
