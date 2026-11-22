//! Row type — `development_plan_benefits`. A stated yearly benefit of a development plan with its named source: an estimate, never derived from a skill level or a score (WPM-R230, WPM-D173).

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub plan_pid: Uuid,
    pub worker_pid: Uuid,
    pub year: i32,
    pub amount_minor: i64,
    pub currency: String,
    pub source: String,
    pub stated_by: Option<String>,
    pub stated_on: SqlDate,
    pub deleted_at: Option<SqlTimestamp>,
}
