//! Row type — `appraisals`. One 360° appraisal (WPM-R29): a subject worker, the declared competencies, the lifecycle.

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
    pub worker_pid: Uuid,
    pub competencies: Value,
    pub status: String,
    pub shared_on: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
}
